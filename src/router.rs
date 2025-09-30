use url::Url;
use std::collections::HashMap;
use hyper::{Request, Body};
use crate::config::{Config, RouteConfig, TokenValidationConfig};
use crate::error::ProxyError;
use crate::token_validator::{TokenValidator, BearerTokenValidator};

pub struct Router {
    routes: Vec<RouteConfig>,
    token_validators: HashMap<String, Box<dyn TokenValidator>>,
}

impl Router {
    pub fn new(config: &Config) -> Result<Self, ProxyError> {
        let mut token_validators: HashMap<String, Box<dyn TokenValidator>> = HashMap::new();
        
        // 为每个配置了token_validation的路由创建验证器
        for route in &config.routes {
            if let Some(token_config) = &route.token_validation {
                let validator = BearerTokenValidator::new(token_config, route.path.clone())?;
                token_validators.insert(route.path.clone(), Box::new(validator));
            }
        }
        
        Ok(Router {
            routes: config.routes.clone(),
            token_validators,
        })
    }
    
    pub fn find_route_and_validate(&self, path: &str, request: &mut Request<Body>) -> Result<(String, String), ProxyError> {
        println!("Finding route for path: {}", path);
        
        for route in &self.routes {
            if let Some(remaining_path) = self.matches_route(&route.path, path) {
                println!("Route matched: {} -> {}", route.path, route.target);
                
                // 执行token校验（如果配置了）
                if let Some(validator) = self.token_validators.get(&route.path) {
                    validator.validate(request)?;
                }
                
                // 构建目标URL
                let target_url = self.build_target_url(&route.target, &remaining_path)?;
                println!("Target URL: {}", target_url);
                
                return Ok((route.target.clone(), target_url));
            }
        }
        
        Err(ProxyError::RouteError(format!("No route found for path: {}", path)))
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
                    token_validation: None,
                },
                RouteConfig {
                    path: "/static/*".to_string(),
                    target: "https://cdn.example.com".to_string(),
                    token_validation: None,
                },
                RouteConfig {
                    path: "/health".to_string(),
                    target: "https://health.example.com".to_string(),
                    token_validation: None,
                },
            ],
        }
    }
    
    #[test]
    fn test_wildcard_route_matching() {
        let config = create_test_config();
        let router = Router::new(&config).unwrap();
        
        let (_, target_url) = router.find_route("/api/users/123").unwrap();
        assert_eq!(target_url, "https://backend.example.com/users/123");
        
        let (_, target_url) = router.find_route("/static/images/logo.png").unwrap();
        assert_eq!(target_url, "https://cdn.example.com/images/logo.png");
    }
    
    #[test]
    fn test_exact_route_matching() {
        let config = create_test_config();
        let router = Router::new(&config).unwrap();
        
        let (_, target_url) = router.find_route("/health").unwrap();
        assert_eq!(target_url, "https://health.example.com");
    }
    
    #[test]
    fn test_no_route_found() {
        let config = create_test_config();
        let router = Router::new(&config).unwrap();
        
        let result = router.find_route("/unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_validation_success() {
        use crate::config::{ServerConfig, LoggingConfig};
        
        let config = Config {
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
                    token_validation: Some(TokenValidationConfig {
                        verified_token: vec!["token01".to_string(), "token02".to_string()],
                        qualified_token: vec!["token03".to_string()],
                    }),
                },
            ],
        };
        
        let router = Router::new(&config).unwrap();
        
        let mut request = Request::builder()
            .uri("/api/users")
            .header("authorization", "Bearer token01")
            .body(Body::empty())
            .unwrap();
        
        let result = router.find_route_and_validate("/api/users", &mut request);
        assert!(result.is_ok());
        
        // 验证token是否被替换
        let auth_header = request.headers().get("authorization").unwrap();
        assert_eq!(auth_header.to_str().unwrap(), "Bearer token03");
    }

    #[test]
    fn test_token_validation_failure() {
        use crate::config::{ServerConfig, LoggingConfig};
        
        let config = Config {
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
                    token_validation: Some(TokenValidationConfig {
                        verified_token: vec!["token01".to_string()],
                        qualified_token: vec!["token03".to_string()],
                    }),
                },
            ],
        };
        
        let router = Router::new(&config).unwrap();
        
        let mut request = Request::builder()
            .uri("/api/users")
            .header("authorization", "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();
        
        let result = router.find_route_and_validate("/api/users", &mut request);
        assert!(result.is_err());
        
        // 确保是Unauthorized错误
        match result.unwrap_err() {
            ProxyError::Unauthorized(_) => {}, // 预期的错误类型
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_no_token_validation_for_routes_without_config() {
        use crate::config::{ServerConfig, LoggingConfig};
        
        let config = Config {
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
                    path: "/public/*".to_string(),
                    target: "https://backend.example.com".to_string(),
                    token_validation: None,
                },
            ],
        };
        
        let router = Router::new(&config).unwrap();
        
        let mut request = Request::builder()
            .uri("/public/info")
            .body(Body::empty())
            .unwrap();
        
        // 没有Authorization header也应该成功
        let result = router.find_route_and_validate("/public/info", &mut request);
        assert!(result.is_ok());
    }
}