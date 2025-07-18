#!/bin/bash

echo "Building Rust HTTP/HTTPS Proxy..."

# 检查Rust是否安装
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust compiler not found. Please install Rust first."
    exit 1
fi

# 检查cargo是否可用
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found. Please install Cargo first."
    exit 1
fi

# 构建项目
echo "Checking dependencies..."
cargo check

echo "Building release version..."
cargo build --release

echo "Running tests..."
cargo test

echo "Build completed successfully!"
echo "Binary location: ./target/release/proxy"