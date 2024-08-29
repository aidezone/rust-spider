use std::fs;
use toml::Value;

pub struct Config {
    pub start_urls: Vec<String>,
    pub max_depth: usize,
    pub proxy: Option<String>,
    pub login: Option<LoginConfig>,
}

pub struct LoginConfig {
    pub username: String,
    pub password: String,
    pub login_url: String,
}

impl Config {
    pub fn new(config_file: &str) -> Result<Self, &'static str> {
        let config_content = fs::read_to_string(config_file).map_err(|_| "Error reading config file")?;
        let config: Value = toml::from_str(&config_content).map_err(|_| "Invalid config format")?;

        Ok(Config {
            start_urls: config["start_urls"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect(),
            max_depth: config["max_depth"].as_integer().unwrap() as usize,
            proxy: config.get("proxy").and_then(|p| p.as_str()).map(|s| s.to_string()),
            login: config.get("login").map(|l| LoginConfig {
                username: l["username"].as_str().unwrap().to_string(),
                password: l["password"].as_str().unwrap().to_string(),
                login_url: l["login_url"].as_str().unwrap().to_string(),
            }),
        })
    }
}
