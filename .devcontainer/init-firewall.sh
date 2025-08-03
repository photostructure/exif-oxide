#!/bin/bash
set -euo pipefail

echo "Initializing firewall..."

# Flush existing rules
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X
iptables -t mangle -F
iptables -t mangle -X
iptables -P INPUT ACCEPT
iptables -P FORWARD ACCEPT
iptables -P OUTPUT ACCEPT

# Allow essential services
iptables -A OUTPUT -p udp --dport 53 -j ACCEPT # DNS
iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT # DNS over TCP
iptables -A OUTPUT -p tcp --dport 22 -j ACCEPT # SSH
iptables -A OUTPUT -d 127.0.0.1/8 -j ACCEPT    # Localhost
# Skip IPv6 localhost if not supported

# Create ipset for allowed domains
ipset create allowed_domains hash:net family inet hashsize 1024 maxelem 65536 -exist

# Function to add IP ranges to ipset
add_to_ipset() {
  local ip_range=$1
  if [[ $ip_range =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+(/[0-9]+)?$ ]]; then
    ipset add allowed_domains "$ip_range" -exist
  fi
}

# Fetch and add GitHub IP ranges
echo "Fetching GitHub IP ranges..."
if github_meta=$(curl -s https://api.github.com/meta); then
  # Parse all GitHub IP ranges
  for key in web api git pages importer packages actions dependabot; do
    if echo "$github_meta" | jq -e ".$key" >/dev/null 2>&1; then
      while IFS= read -r ip; do
        ip_clean=$(echo "$ip" | tr -d '"' | tr -d ' ')
        add_to_ipset "$ip_clean"
      done < <(echo "$github_meta" | jq -r ".${key}[]" 2>/dev/null | grep -v ':')
    fi
  done
fi

# Add IPs for specific domains
declare -a domains=(
  "api.anthropic.com"
  "claude.ai"
  "*.claude.ai"
  "crates.io"
  "index.crates.io"
  "static.crates.io"
  "*.pkg.dev"
  "registry.npmjs.org"
  "nodejs.org"
  "deb.debian.org"
  "security.debian.org"
  "archive.ubuntu.com"
  "security.ubuntu.com"
  "packages.microsoft.com"
  "vscode.download.prss.microsoft.com"
)

for domain in "${domains[@]}"; do
  # Handle wildcards by resolving without the asterisk
  resolve_domain="${domain#\*.}"

  echo "Resolving $resolve_domain..."
  if ips=$(dig +short "$resolve_domain" A 2>/dev/null | grep -E '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$'); then
    while IFS= read -r ip; do
      add_to_ipset "$ip"
    done <<<"$ips"
  fi
done

# Detect and allow host network
if [[ -f /.dockerenv ]]; then
  # Get default gateway
  if default_route=$(ip route show default 2>/dev/null | head -n1); then
    if [[ $default_route =~ via[[:space:]]([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+) ]]; then
      gateway="${BASH_REMATCH[1]}"
      # Extract network (assumes /16 for container networks)
      if [[ $gateway =~ ^([0-9]+\.[0-9]+)\. ]]; then
        network_prefix="${BASH_REMATCH[1]}"
        add_to_ipset "${network_prefix}.0.0/16"
      fi
    fi
  fi
fi

# Set default policies
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT DROP

# Allow established connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow all traffic to allowed domains
iptables -A OUTPUT -m set --match-set allowed_domains dst -j ACCEPT

# Log dropped packets (optional, can be verbose)
# iptables -A OUTPUT -j LOG --log-prefix "DROPPED: " --log-level 4

echo "Firewall initialization complete."

# Verify connectivity
echo "Verifying connectivity..."
if curl -s -o /dev/null -w "%{http_code}" https://api.anthropic.com/health >/dev/null; then
  echo "✓ Anthropic API accessible"
else
  echo "✗ Cannot reach Anthropic API"
fi

if curl -s -o /dev/null -w "%{http_code}" https://github.com >/dev/null; then
  echo "✓ GitHub accessible"
else
  echo "✗ Cannot reach GitHub"
fi

echo "Firewall setup complete!"
