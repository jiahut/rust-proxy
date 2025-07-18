#[derive(Debug)]
pub enum ProxyError {
    ConfigError(String),
    NetworkError(String),
    HttpError(String),
    RouteError(String),
    IoError(std::io::Error),
    HyperError(hyper::Error),
    UrlParseError(url::ParseError),
    TimeoutError,
    InternalError(String),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            ProxyError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ProxyError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            ProxyError::RouteError(msg) => write!(f, "Route error: {}", msg),
            ProxyError::IoError(e) => write!(f, "IO error: {}", e),
            ProxyError::HyperError(e) => write!(f, "Hyper error: {}", e),
            ProxyError::UrlParseError(e) => write!(f, "URL parse error: {}", e),
            ProxyError::TimeoutError => write!(f, "Timeout error"),
            ProxyError::InternalError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl std::error::Error for ProxyError {}

impl From<std::io::Error> for ProxyError {
    fn from(e: std::io::Error) -> Self {
        ProxyError::IoError(e)
    }
}

impl From<hyper::Error> for ProxyError {
    fn from(e: hyper::Error) -> Self {
        ProxyError::HyperError(e)
    }
}

impl From<url::ParseError> for ProxyError {
    fn from(e: url::ParseError) -> Self {
        ProxyError::UrlParseError(e)
    }
}

impl From<ProxyError> for hyper::Response<hyper::Body> {
    fn from(err: ProxyError) -> Self {
        use hyper::{Response, Body, StatusCode};
        
        let (status, message) = match err {
            ProxyError::RouteError(_) => (StatusCode::NOT_FOUND, "Route not found"),
            ProxyError::TimeoutError => (StatusCode::GATEWAY_TIMEOUT, "Request timeout"),
            ProxyError::NetworkError(_) => (StatusCode::BAD_GATEWAY, "Network error"),
            ProxyError::HttpError(_) => (StatusCode::BAD_REQUEST, "HTTP error"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };
        
        Response::builder()
            .status(status)
            .body(Body::from(message))
            .unwrap()
    }
}