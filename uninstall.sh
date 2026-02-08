#!/bin/bash

set -e

sudo rm -f /usr/local/bin/summit
cd stdlib && make uninstall-global && make clean
cd ..
cargo clean

echo "Uninstall complete. Restart your shell."