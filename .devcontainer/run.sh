#!/bin/bash
# Helper script to run exif-oxide devcontainer manually (without VS Code)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGE_NAME="exif-oxide-dev"
CONTAINER_NAME="exif-oxide-dev"

usage() {
  echo "Usage: $0 [command]"
  echo ""
  echo "Commands:"
  echo "  build      - Build the devcontainer image"
  echo "  run        - Run interactive container (default)"
  echo "  start      - Start persistent container in background"
  echo "  exec       - Execute shell in running container"
  echo "  stop       - Stop persistent container"
  echo "  clean      - Remove container and image"
  echo "  help       - Show this help"
  echo ""
  echo "Examples:"
  echo "  $0 build          # Build image"
  echo "  $0 run            # Run interactive container"
  echo "  $0 start          # Start persistent container"
  echo "  $0 exec           # Connect to persistent container"
  exit 1
}

build() {
  echo "🔨 Building devcontainer image..."
  docker build -t "$IMAGE_NAME" "$SCRIPT_DIR/"
  echo "✅ Image built: $IMAGE_NAME"
}

run_interactive() {
  echo "🚀 Starting interactive devcontainer..."
  docker run -it --rm \
    --name "$CONTAINER_NAME" \
    --cap-add=SYS_PTRACE \
    --cap-add=NET_ADMIN \
    --security-opt seccomp=unconfined \
    --privileged \
    -v "$PROJECT_DIR:/workspace" \
    -v "$HOME/.claude:/home/vscode/.claude" \
    -v "$HOME/.cargo/registry:/usr/local/cargo/registry" \
    -v "$HOME/.cargo/git:/usr/local/cargo/git" \
    -e RUST_BACKTRACE=1 \
    -e CARGO_INCREMENTAL=1 \
    -e CARGO_HOME=/usr/local/cargo \
    -e RUSTUP_HOME=/usr/local/rustup \
    -w /workspace \
    --user vscode \
    "$IMAGE_NAME" \
    bash -c 'eval "$(perl -I$HOME/perl5/lib/perl5 -Mlocal::lib)" && exec bash'
}

start_persistent() {
  if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "⚠️  Container $CONTAINER_NAME already exists"
    echo "   Run: $0 stop (to remove) or $0 exec (to connect)"
    exit 1
  fi

  echo "🚀 Starting persistent devcontainer..."
  docker run -d \
    --name "$CONTAINER_NAME" \
    --cap-add=SYS_PTRACE \
    --cap-add=NET_ADMIN \
    --security-opt seccomp=unconfined \
    --privileged \
    -v "$PROJECT_DIR:/workspace" \
    -v "$HOME/.claude:/home/vscode/.claude" \
    -v "$HOME/.cargo/registry:/usr/local/cargo/registry" \
    -v "$HOME/.cargo/git:/usr/local/cargo/git" \
    -e RUST_BACKTRACE=1 \
    -e CARGO_INCREMENTAL=1 \
    -e CARGO_HOME=/usr/local/cargo \
    -e RUSTUP_HOME=/usr/local/rustup \
    -w /workspace \
    --user vscode \
    "$IMAGE_NAME" \
    sleep infinity

  echo "✅ Container started: $CONTAINER_NAME"
  echo "   Connect with: $0 exec"
  echo "   Stop with: $0 stop"
}

exec_shell() {
  if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "❌ Container $CONTAINER_NAME is not running"
    echo "   Start with: $0 start"
    exit 1
  fi

  echo "🔗 Connecting to devcontainer..."
  docker exec -it "$CONTAINER_NAME" bash -c 'eval "$(perl -I$HOME/perl5/lib/perl5 -Mlocal::lib)" && exec bash'
}

stop_container() {
  if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "🛑 Stopping and removing container..."
    docker stop "$CONTAINER_NAME" 2>/dev/null || true
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
    echo "✅ Container removed"
  else
    echo "ℹ️  Container $CONTAINER_NAME does not exist"
  fi
}

clean() {
  stop_container
  if docker images --format '{{.Repository}}' | grep -q "^${IMAGE_NAME}$"; then
    echo "🗑️  Removing image..."
    docker rmi "$IMAGE_NAME"
    echo "✅ Image removed"
  else
    echo "ℹ️  Image $IMAGE_NAME does not exist"
  fi
}

# Parse command
COMMAND="${1:-run}"

case "$COMMAND" in
  build)
    build
    ;;
  run)
    build
    run_interactive
    ;;
  start)
    build
    start_persistent
    ;;
  exec)
    exec_shell
    ;;
  stop)
    stop_container
    ;;
  clean)
    clean
    ;;
  help|--help|-h)
    usage
    ;;
  *)
    echo "❌ Unknown command: $COMMAND"
    echo ""
    usage
    ;;
esac
