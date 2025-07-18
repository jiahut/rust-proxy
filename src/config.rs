use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::fs;
use clap::Parser;
use crate::error::ProxyError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub bind: String,
    pub timeout: u64, // 超时时间（秒）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub access_log: bool,
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub path: String,
    pub target: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.yaml")]
    pub config: String,
    
    /// 服务器端口
    #[arg(short, long)]
    pub port: Option<u16>,
    
    /// 绑定地址
    #[arg(short, long)]
    pub bind: Option<String>,
    
    /// 启用调试模式
    #[arg(long)]
    pub debug: bool,
    
    /// 超时时间（秒）
    #[arg(short, long)]
    pub timeout: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                port: 8888,
                bind: "0.0.0.0".to_string(),
                timeout: 10,
            },
            logging: LoggingConfig {
                access_log: true,
                debug: false,
            },
            routes: vec![],
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self, ProxyError> {
        let args = Args::parse();
        
        // 读取配置文件
        let mut config = if std::path::Path::new(&args.config).exists() {
            println!("Loading configuration from: {}", args.config);
            let content = fs::read_to_string(&args.config)
                .map_err(|e| ProxyError::ConfigError(format!("Failed to read config file: {}", e)))?;
            
            serde_yaml::from_str::<Config>(&content)
                .map_err(|e| ProxyError::ConfigError(format!("Failed to parse config file: {}", e)))?
        } else {
            println!("Using default configuration");
            Config::default()
        };
        
        // 命令行参数覆盖配置文件
        if let Some(port) = args.port {
            config.server.port = port;
        }
        
        if let Some(bind) = args.bind {
            config.server.bind = bind;
        }
        
        if let Some(timeout) = args.timeout {
            config.server.timeout = timeout;
        }
        
        if args.debug {
            config.logging.debug = true;
        }
        
        println!("Final configuration: {:?}", config);
        
        Ok(config)
    }
    
    pub fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.server.timeout)
    }
}