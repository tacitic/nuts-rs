use crate::Config;
use reqwest::{Client, Error, Method, RequestBuilder, Response};

/// Returns a 'reqwest' request-builder for a given method and path
/// And configures security headers according to configuration.
pub fn get_request_builder(cfg: &Config, method: Method, path: String) -> RequestBuilder {
    let client = get_client().unwrap();
    let url = cfg.base_url.join(path.as_str()).unwrap();
    let mut request_builder = client.request(method, url);

    if let Some(a) = &cfg.auth {
        request_builder = request_builder.bearer_auth(a);
    }

    return request_builder;
}

// TODO: docs
pub fn get_client() -> Result<Client, Error> {
    reqwest::Client::builder().build()
}
