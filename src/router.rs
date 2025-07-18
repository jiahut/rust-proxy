use url::Url;
use crate::config::{Config, RouteConfig};
use crate::error::ProxyError;

pub struct Router {
    routes: Vec<RouteConfig>,
}

impl Router {
    pub fn new(config: &Config) -> Self {
        Router {
            routes: config.routes.clone(),
        }
    }
    
    pub fn find_route(&self, path: &str) -> Result<(String, String), ProxyError> {
        println!("Finding route for path: {}", path);
        
        for route in &self.routes {
            if let Some(remaining_path) = self.matches_route(&route.path, path) {
                println!("Route matched: {} -> {}", route.path, route.target);
                
                // 构建目标URL
                let target_url = self.build_target_url(&route.target, &remaining_path)?;
                println!("Target URL: {}", target_url);
                
                return Ok((route.target.clone(), target_url));
            }
        }
        
        Err(ProxyError::RouteError(format!("No route found for path: {}", path)))
    }
    
    fn matches_route(&self, route_pattern: &str, request_path: &str) -> Option<String> {
        // 处理通配符路由
        if route_pattern.ends_with("/*") {
            let prefix = &route_pattern[..route_pattern.len() - 2];
            if request_path.starts_with(prefix) {
                // 返回去掉前缀后的路径
                let remaining = &request_path[prefix.len()..];
                return Some(remaining.to_string());
            }
        }
        // 精确匹配
        else if route_pattern == request_path {
            return Some("".to_string());
        }
        
        None
    }
    
    fn build_target_url(&self, target_base: &str, remaining_path: &str) -> Result<String, ProxyError> {
        // 确保目标URL有正确的格式
        let mut target_url = target_base.to_string();
        
        // 移除末尾的斜杠
        if target_url.ends_with('/') {
            target_url.pop();
        }
        
        // 添加剩余路径
        if !remaining_path.is_empty() {
            if !remaining_path.starts_with('/') {
                target_url.push('/');
            }
            target_url.push_str(remaining_path);
        }
        
        // 验证URL格式
        Url::parse(&target_url)?;
        
        Ok(target_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ServerConfig, LoggingConfig};
    
    fn create_test_config() -> Config {
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
            routes: vec![
                RouteConfig {
                    path: "/api/*".to_string(),
                    target: "https://backend.example.com".to_string(),
                },
                RouteConfig {
                    path: "/static/*".to_string(),
                    target: "https://cdn.example.com".to_string(),
                },
                RouteConfig {
                    path: "/health".to_string(),
                    target: "https://health.example.com".to_string(),
                },
            ],
        }
    }
    
    #[test]
    fn test_wildcard_route_matching() {
        let config = create_test_config();
        let router = Router::new(&config);
        
        let (_, target_url) = router.find_route("/api/users/123").unwrap();
        assert_eq!(target_url, "https://backend.example.com/users/123");
        
        let (_, target_url) = router.find_route("/static/images/logo.png").unwrap();
        assert_eq!(target_url, "https://cdn.example.com/images/logo.png");
    }
    
    #[test]
    fn test_exact_route_matching() {
        let config = create_test_config();
        let router = Router::new(&config);
        
        let (_, target_url) = router.find_route("/health").unwrap();
        assert_eq!(target_url, "https://health.example.com");
    }
    
    #[test]
    fn test_no_route_found() {
        let config = create_test_config();
        let router = Router::new(&config);
        
        let result = router.find_route("/unknown");
        assert!(result.is_err());
    }
}