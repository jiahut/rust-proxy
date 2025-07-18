#!/usr/bin/env python3
"""
验证Rust代理服务器功能的Python脚本
"""

import os
import sys
import subprocess
import time
import requests
import threading
import signal

def check_dependencies():
    """检查依赖项"""
    print("检查依赖项...")
    
    # 检查Rust
    try:
        result = subprocess.run(['rustc', '--version'], capture_output=True, text=True)
        print(f"Rust版本: {result.stdout.strip()}")
    except FileNotFoundError:
        print("错误: 未找到Rust编译器")
        return False
    
    # 检查Python requests库
    try:
        import requests
        print("Python requests库已安装")
    except ImportError:
        print("错误: 未安装requests库，请运行: pip install requests")
        return False
    
    return True

def build_project():
    """构建项目"""
    print("构建项目...")
    
    # 检查项目结构
    if not os.path.exists("Cargo.toml"):
        print("错误: 未找到Cargo.toml文件")
        return False
    
    # 构建项目
    try:
        result = subprocess.run(['cargo', 'build', '--release'], 
                              capture_output=True, text=True, cwd='.')
        if result.returncode != 0:
            print(f"构建失败: {result.stderr}")
            return False
        print("构建成功")
        return True
    except Exception as e:
        print(f"构建异常: {e}")
        return False

def test_router_logic():
    """测试路由逻辑"""
    print("测试路由逻辑...")
    
    # 简单的路由测试
    test_cases = [
        ("/api/users/123", "/api/*", "https://backend.com", "https://backend.com/users/123"),
        ("/static/css/style.css", "/static/*", "https://cdn.com", "https://cdn.com/css/style.css"),
        ("/health", "/health", "https://health.com", "https://health.com"),
    ]
    
    for request_path, route_pattern, target_base, expected_url in test_cases:
        # 简单的路由匹配逻辑验证
        if route_pattern.endswith("/*"):
            prefix = route_pattern[:-2]
            if request_path.startswith(prefix):
                remaining = request_path[len(prefix):]
                result_url = target_base + remaining
                if result_url == expected_url:
                    print(f"✓ 路由测试通过: {request_path} -> {result_url}")
                else:
                    print(f"✗ 路由测试失败: {request_path} -> {result_url} (期望: {expected_url})")
        elif route_pattern == request_path:
            if target_base == expected_url:
                print(f"✓ 精确匹配测试通过: {request_path} -> {target_base}")
            else:
                print(f"✗ 精确匹配测试失败: {request_path} -> {target_base} (期望: {expected_url})")

def validate_config():
    """验证配置文件"""
    print("验证配置文件...")
    
    if not os.path.exists("config.yaml"):
        print("警告: 未找到config.yaml文件")
        return False
    
    try:
        import yaml
        with open("config.yaml", 'r') as f:
            config = yaml.safe_load(f)
        
        # 验证配置结构
        required_keys = ['server', 'logging', 'routes']
        for key in required_keys:
            if key not in config:
                print(f"错误: 配置文件缺少必需的键: {key}")
                return False
        
        print("配置文件验证通过")
        return True
    except ImportError:
        print("警告: 未安装PyYAML库，跳过配置验证")
        return True
    except Exception as e:
        print(f"配置文件验证失败: {e}")
        return False

def main():
    """主函数"""
    print("=== Rust HTTP/HTTPS 代理服务器验证工具 ===\n")
    
    # 检查依赖项
    if not check_dependencies():
        sys.exit(1)
    
    # 验证配置
    validate_config()
    
    # 测试路由逻辑
    test_router_logic()
    
    # 构建项目
    if build_project():
        print("\n✓ 项目构建成功")
        if os.path.exists("target/release/proxy"):
            print("✓ 可执行文件已生成: target/release/proxy")
        else:
            print("✗ 未找到可执行文件")
    else:
        print("\n✗ 项目构建失败")
        sys.exit(1)
    
    print("\n=== 验证完成 ===")
    print("使用方法:")
    print("1. 运行代理服务器: ./target/release/proxy")
    print("2. 使用自定义配置: ./target/release/proxy --config config.yaml")
    print("3. 启用调试模式: ./target/release/proxy --debug")

if __name__ == "__main__":
    main()