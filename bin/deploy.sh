#!/bin/bash
set -e

# Configuration
LIB_DIR="$HOME/.local/lib/porty"
BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/porty"
BINARY_NAME="porty"

lib_file="$LIB_DIR/$BINARY_NAME"
bin_file="$BIN_DIR/$BINARY_NAME"

# Resolve repository root from bin/
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DEPLOYABLE="${BINARY_NAME}"

# Extract version from Cargo.toml at repo root
VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)

# Display deployment ceremony
echo "╔════════════════════════════════════════════════╗"
echo "║            PORTY DEPLOYMENT CEREMONY           ║"
echo "╠════════════════════════════════════════════════╣"
echo "║ Package: $BINARY_NAME                          ║"
echo "║ Version: v$VERSION                             ║"
echo "║ Target:  $bin_file                             ║"
echo "╚════════════════════════════════════════════════╝"
echo ""

echo "🔨 Building porty v$VERSION..."
cd "$ROOT_DIR"
if ! cargo build --release; then
    echo "❌ Build failed!"
    exit 1
fi

# Check if binary was created (at repo root)
if [ ! -f "$ROOT_DIR/target/release/${DEPLOYABLE}" ]; then
    echo "❌ Binary not found at target/release/${DEPLOYABLE}"
    exit 1
fi

echo "📦 Installing porty to $BIN_DIR..."
mkdir -p "$BIN_DIR" "$LIB_DIR" "$CONFIG_DIR"

if [ -f "$bin_file" ]; then
	echo "📦 Removing previous porty installation"
	rm -f "$lib_file"
	rm -f "$bin_file"
fi

if ! cp "$ROOT_DIR/target/release/${DEPLOYABLE}" "$lib_file"; then
    echo "❌ Failed to copy binary to $lib_file"
    exit 1
fi

if ! chmod +x "$lib_file"; then
	echo "❌ Failed to make binary executable"
	exit 1
fi

if ! ln -s "$lib_file" "$bin_file"; then
	echo "❌ Failed to create symlink"
	exit 1
fi

# Verify deployment
if [ ! -x "$bin_file" ]; then
    echo "❌ Binary is not executable at $bin_file"
    exit 1
fi

# Test the binary
echo "🧪 Testing binary..."
if ! "$bin_file" --help > /dev/null 2>&1; then
    echo "❌ Binary test failed!"
    exit 1
fi

# Deploy default config if it doesn't exist
echo "📋 Setting up configuration..."
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    echo "📋 Creating default configuration..."
    if "$bin_file" --generate-config > "$CONFIG_DIR/config.toml" 2>/dev/null; then
        echo "✅ Default config created at $CONFIG_DIR/config.toml"
    else
        echo "ℹ️  Config generation not available - use local config.toml"
    fi
else
    echo "✅ Existing configuration preserved"
fi

echo ""
echo "╔════════════════════════════════════════════════╗"
echo "║          DEPLOYMENT SUCCESSFUL!                ║"
echo "╚════════════════════════════════════════════════╝"
echo "  Deployed: porty v$VERSION"
echo "  Location: $bin_file"
echo ""
echo "🚀 Quick Start:"
echo "   $BINARY_NAME                   # Run with config.toml"
echo "   $BINARY_NAME --help            # View all options"
echo "   $BINARY_NAME --generate-config # Create example config"
echo ""
echo "📖 Example Usage:"
echo "   # TCP forwarding"
echo "   $BINARY_NAME --listen-port 8080 --target-port 3000"
echo "   "
echo "   # HTTP dynamic routing"
echo "   curl 'http://localhost:9090/api?porty_host=api.example.com&porty_port=80'"
echo ""
echo "📁 Configuration: $CONFIG_DIR/config.toml"
