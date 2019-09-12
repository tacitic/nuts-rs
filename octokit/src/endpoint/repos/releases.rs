use crate::{util, Config};
use reqwest::{Client, Method, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    url: String,
    assets_url: String,
    upload_url: String,
    html_url: String,
    id: u32,
    node_id: String,
    author: User,
    tag_name: String,
    target_commitish: String,
    name: String,
    draft: bool,
    prerelease: bool,
    created_at: String,
    published_at: String,
    assets: Vec<Asset>,
}

// TODO: move User to more general place.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    login: String,
    id: u32,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    site_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    url: String,
    id: u32,
    node_id: String,
    name: String,
    label: String,
    uploader: User,
    content_type: String,
    state: String,
    size: u32,
    download_count: u32,
    created_at: String,
    updated_at: String,
    browser_download_url: String,
}

pub fn list_releases(cfg: &Config, repo: &str) -> Result<Vec<Release>, String> {
    let b = util::get_request_builder(&cfg, Method::GET, format!("/repos/{}/releases", repo));
    let responses = util::paginate(&b, 1, 5).unwrap();

    let json: Vec<Vec<Release>> = responses
        .into_iter()
        .map(|mut r| r.json().unwrap())
        .collect();

    Ok(json.into_iter().flatten().collect())
}
