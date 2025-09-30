use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::timeout;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use hyper_rustls::HttpsConnectorBuilder;
use crate::config::Config;
use crate::router::Router;
use crate::error::ProxyError;

pub struct ProxyServer {
    config: Config,
    router: Arc<Router>,
    client: Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl ProxyServer {
    pub fn new(config: Config) -> Result<Self, ProxyError> {
        let router = Arc::new(Router::new(&config)?);
        
        // 创建HTTPS客户端，不验证证书
        let https = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();
        let client = Client::builder()
            .build::<_, hyper::Body>(https);
        
        Ok(ProxyServer {
            config,
            router,
            client,
        })
    }
    
    pub async fn run(self, listener: TcpListener) -> Result<(), ProxyError> {
        let server = self;
        
        // 创建服务
        let make_svc = make_service_fn(move |_conn| {
            let server = server.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let server = server.clone();
                    async move {
                        server.handle_request(req).await
                    }
                }))
            }
        });
        
        // 启动服务器
        let addr = listener.local_addr()?;
        let server = Server::from_tcp(listener.into_std()?)?
            .serve(make_svc);
        
        println!("Proxy server running on http://{}", addr);
        
        if let Err(e) = server.await {
            println!("Server error: {}", e);
        }
        
        Ok(())
    }
    
    async fn handle_request(&self, mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let method = req.method().clone();
        let uri = req.uri().clone();
        let path = uri.path();
        
        // 记录访问日志
        if self.config.logging.access_log {
            println!("Request: {} {}", method, uri);
        }
        
        if self.config.logging.debug {
            println!("Handling request: {} {}", method, path);
        }
        
        // 查找路由并执行token校验
        let (_target_base, target_url) = match self.router.find_route_and_validate(path, &mut req) {
            Ok((base, url)) => (base, url),
            Err(e) => {
                println!("Route not found or validation failed for {}: {}", path, e);
                return Ok(e.into());
            }
        };
        
        // 代理请求
        match self.proxy_request(req, &target_url).await {
            Ok(response) => Ok(response),
            Err(e) => {
                println!("Proxy error for {}: {}", target_url, e);
                Ok(e.into())
            }
        }
    }
    
    async fn proxy_request(&self, mut req: Request<Body>, target_url: &str) -> Result<Response<Body>, ProxyError> {
        if self.config.logging.debug {
            println!("Proxying request to: {}", target_url);
        }
        
        // 解析目标URL
        let target_uri: Uri = target_url.parse()
            .map_err(|e| ProxyError::InternalError(format!("URI parse error: {}", e)))?;
        
        // 复制查询参数
        let mut final_url = target_url.to_string();
        if let Some(query) = req.uri().query() {
            final_url.push('?');
            final_url.push_str(query);
        }
        
        let final_uri: Uri = final_url.parse()
            .map_err(|e| ProxyError::InternalError(format!("URI parse error: {}", e)))?;
        
        // 更新请求URI
        *req.uri_mut() = final_uri;
        
        // 移除可能影响代理的头部
        req.headers_mut().remove("host");
        req.headers_mut().remove("connection");
        req.headers_mut().remove("proxy-connection");
        
        // 添加目标主机头部
        if let Some(host) = target_uri.host() {
            let host_value = if let Some(port) = target_uri.port() {
                format!("{}:{}", host, port)
            } else {
                host.to_string()
            };
            req.headers_mut().insert("host", host_value.parse().unwrap());
        }
        
        // 执行代理请求
        let response = timeout(
            self.config.get_timeout(),
            self.client.request(req)
        ).await
        .map_err(|_| ProxyError::TimeoutError)?
        .map_err(|e| ProxyError::HyperError(e))?;
        
        if self.config.logging.debug {
            println!("Response status: {}", response.status());
        }
        
        Ok(response)
    }
}

impl Clone for ProxyServer {
    fn clone(&self) -> Self {
        ProxyServer {
            config: self.config.clone(),
            router: Arc::clone(&self.router),
            client: self.client.clone(),
        }
    }
}