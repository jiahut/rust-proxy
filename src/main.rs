use std::net::SocketAddr;
use tokio::net::TcpListener;

mod config;
mod proxy;
mod router;
mod error;

use config::Config;
use proxy::ProxyServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化简单日志
    println!("Starting Rust HTTP/HTTPS Proxy Server...");

    // 解析配置
    let config = Config::load().await?;
    
    println!("Starting proxy server on {}:{}", config.server.bind, config.server.port);
    
    // 创建监听地址
    let addr: SocketAddr = format!("{}:{}", config.server.bind, config.server.port)
        .parse()
        .expect("Invalid bind address");
    
    // 创建TCP监听器
    let listener = TcpListener::bind(addr).await?;
    println!("Proxy server listening on {}", addr);
    
    // 创建代理服务器
    let server = ProxyServer::new(config);
    
    // 启动服务器
    server.run(listener).await?;
    
    Ok(())
}