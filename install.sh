#!/bin/bash

set -e

echo "Building Summit..."
cargo build --release

echo "Installing Summit binary..."
sudo cp target/release/summit /usr/local/bin/summit
sudo chmod +x /usr/local/bin/summit

echo "Building and installing standard library..."
cd stdlib
make install-global
cd ..

echo ""
echo "Installation complete!"
echo "You can now use 'summit'."
echo "Please restart your shell or run:"
if [ -f ~/.zshrc ]; then
    echo "  source ~/.zshrc"
else
    echo "  source ~/.bashrc"
fi