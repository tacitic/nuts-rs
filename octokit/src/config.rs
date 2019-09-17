use reqwest::{Url};

pub struct Config {
    pub base_url: Url,
    pub auth: Option<String>,
    pub user_agent: String,
}

impl Config {
    pub fn new_authenticated(auth: &str) -> Self {
        Config {
            auth: Some(auth.to_string()),
            ..Config::default()
        }
    }
}

// TODO: How to handle the unwrap() here?
impl Default for Config {
    fn default() -> Self {
        Config {
            base_url: Url::parse("https://api.github.com").unwrap(),
            auth: None,
            user_agent: "octokit-rs".to_string(),
        }
    }
}
