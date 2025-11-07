# Use the official Rust image as base
FROM rust:1.83-bookworm

# Install system dependencies in optimal layer order
# (least frequently changed packages first for better caching)
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    pkg-config \
    libssl-dev \
    # Perl and dependencies
    perl \
    libperl-dev \
    cpanminus \
    # Python for analysis scripts
    python3 \
    python3-pip \
    python3-venv \
    # Development tools
    git \
    curl \
    wget \
    vim \
    tmux \
    zsh \
    # Code formatting and linting tools
    ripgrep \
    # LLDB for debugging
    lldb \
    gdb \
    # Firewall dependencies
    iptables \
    ipset \
    dnsutils \
    # JSON/YAML processing
    jq \
    # ExifTool for validation and comparison
    libimage-exiftool-perl \
    # Additional utilities
    sudo \
    less \
    man-db \
    # Clean up
    && rm -rf /var/lib/apt/lists/*

# Install Python tools globally (for all users)
RUN pip3 install --no-cache-dir --break-system-packages \
    yamllint \
    jsonschema \
    ruff

# Install Rust components
RUN rustup component add rustfmt clippy rust-analyzer

# Install Go (for shfmt - shell formatter)
ARG GO_VERSION=1.23.5
RUN curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz" | tar -C /usr/local -xzf -
ENV PATH="/usr/local/go/bin:${PATH}"

# Install Go-based tools
RUN go install mvdan.cc/sh/v3/cmd/shfmt@latest && \
    mv /root/go/bin/shfmt /usr/local/bin/ && \
    rm -rf /root/go

# Install Node.js 20 (for potential future needs)
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
# These are development tools needed by the project
USER $USERNAME
RUN cargo install --locked \
    cargo-audit \
    cargo-edit \
    sd
USER root

# Set up Perl environment for the user
USER $USERNAME
WORKDIR /home/$USERNAME

# Install Perl modules for development
# - local::lib: Enables local Perl library installation
# - Perl::LanguageServer: Language server for VS Code
# - PPI: Perl parsing used by codegen (installed via cpanfile later)
RUN cpanm --local-lib=~/perl5 local::lib && \
    eval "$(perl -I ~/perl5/lib/perl5 -Mlocal::lib)" && \
    cpanm --quiet --notest \
        Perl::LanguageServer \
        Perl::Critic \
        Perl::Tidy

# Add local Perl library to shell initialization
# This ensures Perl can find locally installed modules
RUN echo 'eval "$(perl -I$HOME/perl5/lib/perl5 -Mlocal::lib)"' >> ~/.bashrc && \
    echo 'eval "$(perl -I$HOME/perl5/lib/perl5 -Mlocal::lib)"' >> ~/.zshrc && \
    echo 'export PATH="$HOME/perl5/bin:$PATH"' >> ~/.bashrc && \
    echo 'export PATH="$HOME/perl5/bin:$PATH"' >> ~/.zshrc

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