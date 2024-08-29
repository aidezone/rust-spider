use reqwest::blocking::{Client, RequestBuilder};
use reqwest::Proxy;
use reqwest::header::USER_AGENT;

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
        let body = response.text()?;
        Ok(body)
    }
}
