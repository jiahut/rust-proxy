# Rust HTTP/HTTPS 代理服务器

一个基于Rust的高性能HTTP/HTTPS代理服务器，支持多路由规则配置，以低延迟为核心目标。

## 🚀 特性

- **高性能**: 基于Tokio异步运行时和Hyper HTTP库
- **多路由支持**: 支持多个路由规则配置，按顺序匹配
- **协议支持**: 支持HTTP→HTTP、HTTP→HTTPS转发
- **路径处理**: 自动去掉匹配前缀进行转发
- **流式传输**: 支持流式数据传输
- **灵活配置**: 支持YAML配置文件和命令行参数
- **完整日志**: 访问日志和调试日志支持

## 📦 安装

### 从源码构建

```bash
# 克隆项目
git clone <repository-url>
cd rust-proxy

# 构建项目
cargo build --release

# 运行
./target/release/proxy
```

### 预编译二进制文件

下载适合您系统的预编译二进制文件，放置在PATH中即可使用。

## 🔧 配置

### 配置文件 (config.yaml)

```yaml
server:
  port: 8888          # 监听端口
  bind: "0.0.0.0"     # 绑定地址
  timeout: 10         # 超时时间（秒）

logging:
  access_log: true    # 启用访问日志
  debug: false        # 启用调试日志

routes:
  - path: "/api/*"                           # 路由路径（支持通配符）
    target: "https://backend.example.com"    # 目标服务器
  - path: "/static/*"
    target: "https://cdn.example.com"
  - path: "/health"                          # 精确匹配
    target: "https://health.example.com"
```

### 命令行参数

```bash
# 使用自定义配置文件
proxy --config my-config.yaml

# 覆盖配置选项
proxy --port 9000 --debug

# 显示帮助
proxy --help
```

## 🎯 使用示例

### 基本用法

```bash
# 使用默认配置启动
proxy

# 使用自定义端口
proxy --port 8080

# 启用调试模式
proxy --debug

# 使用自定义配置文件
proxy --config /path/to/config.yaml
```

### 路由示例

基于示例配置，以下请求将被代理：

```bash
# 代理API请求
curl http://localhost:8888/api/users/123
# 转发到: https://backend.example.com/users/123

# 代理静态资源
curl http://localhost:8888/static/css/style.css
# 转发到: https://cdn.example.com/css/style.css

# 精确匹配
curl http://localhost:8888/health
# 转发到: https://health.example.com
```

## 🛠 开发

### 项目结构

```
rust-proxy/
├── src/
│   ├── main.rs          # 主程序入口
│   ├── config.rs        # 配置管理
│   ├── proxy.rs         # 代理核心逻辑
│   ├── router.rs        # 路由匹配
│   └── error.rs         # 错误处理
├── config.yaml          # 示例配置文件
├── Cargo.toml           # 依赖配置
└── README.md           # 文档
```

### 运行测试

```bash
cargo test
```

### 构建发布版本

```bash
cargo build --release
```

## 🔍 日志

### 访问日志

```
2024-01-01T12:00:00Z INFO Request: GET /api/users/123
2024-01-01T12:00:00Z INFO Request: POST /api/users
```

### 调试日志

启用调试模式后，将显示详细的代理信息：

```bash
proxy --debug
```

## ⚡ 性能

- **低延迟**: 针对低延迟优化，最小化请求处理时间
- **高并发**: 支持高并发连接处理
- **内存效率**: 优化内存使用，避免不必要的数据复制
- **流式传输**: 支持大文件的流式传输

## 🚦 错误处理

代理服务器会处理以下错误情况：

- **路由不匹配**: 返回404 Not Found
- **网络超时**: 返回504 Gateway Timeout
- **网络错误**: 返回502 Bad Gateway
- **内部错误**: 返回500 Internal Server Error

所有错误都会记录到日志中，包含详细的错误信息。

## 📋 系统要求

- **操作系统**: Windows, Linux, macOS
- **内存**: 最低64MB
- **网络**: 支持出站HTTPS连接

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建Pull Request

## 📄 许可证

本项目采用MIT许可证 - 详见 [LICENSE](LICENSE) 文件。

## 🔗 相关链接

- [Rust官网](https://www.rust-lang.org/)
- [Tokio异步运行时](https://tokio.rs/)
- [Hyper HTTP库](https://hyper.rs/)