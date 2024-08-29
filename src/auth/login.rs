use crate::config::LoginConfig;
use reqwest::blocking::Client;

pub struct Authenticator {
    login_config: LoginConfig,
}

impl Authenticator {
    pub fn new(login_config: &LoginConfig) -> Self {
        Authenticator {
            login_config: login_config.clone(),
        }
    }

    pub fn login(&self) {
        let client = Client::new();
        let _res = client.post(&self.login_config.login_url)
            .form(&[("username", &self.login_config.username), ("password", &self.login_config.password)])
            .send();
        println!("Logged in as {}", self.login_config.username);
    }
}
