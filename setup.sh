#!/bin/bash

# Set up Rust environment
rustup default stable
rustup target add bpfel-unknown-none

# Install Anchor
sh -c "$(curl -sSfL https://raw.githubusercontent.com/coral-xyz/anchor/v0.31.1/scripts/install.sh)"


