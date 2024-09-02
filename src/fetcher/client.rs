use reqwest::blocking::{Client, RequestBuilder};
use reqwest::Proxy;
use reqwest::header::USER_AGENT;
// 引入宏
use crate::info;
use crate::warn;
use crate::error;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new(proxy_url: Option<&str>) -> Self {
        let client = if let Some(proxy_url) = proxy_url {
            let proxy = Proxy::all(proxy_url).unwrap();
            Client::builder()
                .proxy(proxy)
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .build()
                .unwrap()
        } else {
            Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .build()
                .unwrap()
        };

        HttpClient { client }
    }

    pub fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        let response = self.client.get(url).send()?;
        // 打印最终的 URL 和状态码
        info!("Final URL: {}", response.url());
        info!("Status: {}", response.status());

        // 检查响应头中的 Content-Encoding
        let content_encoding = response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        info!("Content-Encoding: {}", content_encoding);

        let body = response.text_with_charset("UTF-8")?;
        Ok(body)
    }
}
