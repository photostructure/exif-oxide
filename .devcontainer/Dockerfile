# Use the official Rust image as base
FROM rust:1.83-bookworm

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    pkg-config \
    libssl-dev \
    # Perl and dependencies
    perl \
    libperl-dev \
    cpanminus \
    # Development tools
    git \
    curl \
    vim \
    tmux \
    # LLDB for debugging
    lldb \
    # Firewall dependencies
    iptables \
    ipset \
    dnsutils \
    jq \
    # Clean up
    && rm -rf /var/lib/apt/lists/*

# Install Rust components
RUN rustup component add rustfmt clippy rust-analyzer

# Install Node.js 20
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Claude Code will be available via bind mount from ~/.claude
# No need to install it in the container

# Create non-root user
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && apt-get update \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME \
    && rm -rf /var/lib/apt/lists/*

# Fix cargo permissions for non-root user
RUN chown -R $USERNAME:$USERNAME /usr/local/cargo \
    && chmod -R g+w /usr/local/cargo

# Install additional Rust tools as the vscode user
USER $USERNAME
RUN cargo install cargo-audit --locked
USER root

# Set up Perl environment for the user
USER $USERNAME
WORKDIR /home/$USERNAME

# Install Perl::LanguageServer for VS Code Perl extension
RUN cpanm --local-lib=~/perl5 local::lib && eval $(perl -I ~/perl5/lib/perl5 -Mlocal::lib) \
    && cpanm Perl::LanguageServer

# Add local Perl library to path
RUN echo 'eval "$(perl -I$HOME/perl5/lib/perl5 -Mlocal::lib)"' >> ~/.bashrc \
    && echo 'eval "$(perl -I$HOME/perl5/lib/perl5 -Mlocal::lib)"' >> ~/.zshrc

# Set the default shell to zsh (will be configured by devcontainer features)
SHELL ["/bin/bash", "-c"]

# Switch back to root for final setup
USER root

# Copy firewall initialization script
COPY init-firewall.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/init-firewall.sh

# Set working directory
WORKDIR /workspace

# Keep container running
CMD ["sleep", "infinity"]