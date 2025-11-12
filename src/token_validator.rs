use crate::config::TokenValidationConfig;
use crate::error::ProxyError;
use hyper::{header::HeaderValue, Body, Request};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

pub trait TokenValidator: Send + Sync {
    fn validate(&self, request: &mut Request<Body>) -> Result<(), ProxyError>;
    fn get_success_count(&self) -> u64;
}

pub struct BearerTokenValidator {
    verified_tokens: HashSet<String>,
    qualified_tokens: Vec<String>,
    success_count: AtomicU64,
    route_path: String,
}

impl BearerTokenValidator {
    pub fn new(config: &TokenValidationConfig, route_path: String) -> Result<Self, ProxyError> {
        if config.qualified_token.is_empty() {
            return Err(ProxyError::ConfigError(
                "qualified_token cannot be empty".to_string(),
            ));
        }

        if config.verified_token.is_empty() {
            return Err(ProxyError::ConfigError(
                "verified_token cannot be empty".to_string(),
            ));
        }

        Ok(BearerTokenValidator {
            verified_tokens: config.verified_token.iter().cloned().collect(),
            qualified_tokens: config.qualified_token.clone(),
            success_count: AtomicU64::new(0),
            route_path,
        })
    }
}

impl TokenValidator for BearerTokenValidator {
    fn validate(&self, request: &mut Request<Body>) -> Result<(), ProxyError> {
        // 提取Authorization header
        let auth_header = request.headers().get("authorization").ok_or_else(|| {
            println!(
                "[ERROR] [{}] Token validation error: path={}, reason=missing_authorization_header",
                std::thread::current().name().unwrap_or("Unknown"),
                self.route_path
            );
            ProxyError::Unauthorized("Missing Authorization header".to_string())
        })?;

        let auth_str = auth_header.to_str().map_err(|_| {
            println!(
                "[ERROR] [{}] Token validation error: path={}, reason=invalid_authorization_header",
                std::thread::current().name().unwrap_or("Unknown"),
                self.route_path
            );
            ProxyError::Unauthorized("Invalid Authorization header".to_string())
        })?;

        // 解析Bearer token
        if !auth_str.starts_with("Bearer ") {
            println!(
                "[ERROR] [{}] Token validation error: path={}, reason=invalid_bearer_format",
                std::thread::current().name().unwrap_or("Unknown"),
                self.route_path
            );
            return Err(ProxyError::Unauthorized(
                "Invalid Bearer token format".to_string(),
            ));
        }

        let token = auth_str[7..].to_string(); // 去掉 "Bearer " 前缀，转换为owned string

        // 校验token是否在verified_tokens中
        if !self.verified_tokens.contains(&token) {
            // 对token进行脱敏处理（只显示前3位和后3位）
            let masked_token = if token.len() > 6 {
                format!("{}***{}", &token[..3], &token[token.len() - 3..])
            } else {
                "***".to_string()
            };

            println!(
                "[WARN] [{}] Token validation failed: path={}, token={}, reason=not_in_whitelist",
                std::thread::current().name().unwrap_or("Unknown"),
                self.route_path,
                masked_token
            );
            return Err(ProxyError::Unauthorized("Invalid token".to_string()));
        }

        // 创建脱敏版本用于日志
        let masked_original = if token.len() > 6 {
            format!("{}***{}", &token[..3], &token[token.len() - 3..])
        } else {
            "***".to_string()
        };

        // 替换为qualified_token
        let new_token = &self.qualified_tokens[0];
        let new_auth_header = format!("Bearer {}", new_token);
        let new_header_value = HeaderValue::from_str(&new_auth_header).map_err(|_| {
            ProxyError::InternalError("Failed to create new auth header".to_string())
        })?;

        request
            .headers_mut()
            .insert("authorization", new_header_value);

        // 更新计数器
        let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;

        let masked_new = if new_token.len() > 6 {
            format!(
                "{}***{}",
                &new_token[..3],
                &new_token[new_token.len() - 3..]
            )
        } else {
            "***".to_string()
        };

        println!(
            "[INFO] [{}] Token validation success: path={}, token={}⟶{}, count={}",
            std::thread::current().name().unwrap_or("Unknown"),
            self.route_path,
            masked_original,
            masked_new,
            count
        );

        Ok(())
    }

    fn get_success_count(&self) -> u64 {
        self.success_count.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TokenValidationConfig;
    use hyper::{Body, Request};

    #[test]
    fn test_bearer_token_validator_success() {
        let config = TokenValidationConfig {
            verified_token: vec!["token01".to_string(), "token02".to_string()],
            qualified_token: vec!["token03".to_string()],
        };

        let validator = BearerTokenValidator::new(&config, "/api/test".to_string()).unwrap();

        let mut request = Request::builder()
            .header("authorization", "Bearer token01")
            .body(Body::empty())
            .unwrap();

        let result = validator.validate(&mut request);
        assert!(result.is_ok());

        // 检查token是否被替换
        let auth_header = request.headers().get("authorization").unwrap();
        assert_eq!(auth_header.to_str().unwrap(), "Bearer token03");

        // 检查计数器
        assert_eq!(validator.get_success_count(), 1);
    }

    #[test]
    fn test_bearer_token_validator_invalid_token() {
        let config = TokenValidationConfig {
            verified_token: vec!["token01".to_string()],
            qualified_token: vec!["token03".to_string()],
        };

        let validator = BearerTokenValidator::new(&config, "/api/test".to_string()).unwrap();

        let mut request = Request::builder()
            .header("authorization", "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();

        let result = validator.validate(&mut request);
        assert!(result.is_err());
        assert_eq!(validator.get_success_count(), 0);
    }

    #[test]
    fn test_bearer_token_validator_missing_header() {
        let config = TokenValidationConfig {
            verified_token: vec!["token01".to_string()],
            qualified_token: vec!["token03".to_string()],
        };

        let validator = BearerTokenValidator::new(&config, "/api/test".to_string()).unwrap();

        let mut request = Request::builder().body(Body::empty()).unwrap();

        let result = validator.validate(&mut request);
        assert!(result.is_err());
        assert_eq!(validator.get_success_count(), 0);
    }

    #[test]
    fn test_bearer_token_validator_invalid_format() {
        let config = TokenValidationConfig {
            verified_token: vec!["token01".to_string()],
            qualified_token: vec!["token03".to_string()],
        };

        let validator = BearerTokenValidator::new(&config, "/api/test".to_string()).unwrap();

        let mut request = Request::builder()
            .header("authorization", "Basic dGVzdDp0ZXN0")
            .body(Body::empty())
            .unwrap();

        let result = validator.validate(&mut request);
        assert!(result.is_err());
        assert_eq!(validator.get_success_count(), 0);
    }
}
