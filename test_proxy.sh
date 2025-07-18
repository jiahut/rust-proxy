#!/bin/bash

echo "Testing Rust HTTP/HTTPS Proxy..."

# 启动代理服务器（在后台运行）
echo "Starting proxy server..."
./target/release/proxy &
PROXY_PID=$!

# 等待服务器启动
sleep 3

# 测试API路由
echo "Testing API route..."
curl -s http://localhost:8888/api/posts/1 | head -5

# 测试健康检查
echo "Testing health check..."
curl -s http://localhost:8888/health

# 测试echo路由
echo "Testing echo route..."
curl -s http://localhost:8888/echo/get | head -5

# 测试不存在的路由
echo "Testing non-existent route..."
curl -s http://localhost:8888/nonexistent

# 关闭代理服务器
kill $PROXY_PID

echo "Tests completed!"