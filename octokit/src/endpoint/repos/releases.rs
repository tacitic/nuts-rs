
use crate::{util, Config};
use failure::Error;
use reqwest::{Client, Method, RequestBuilder, Response};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: u32,
    pub node_id: String,
    pub author: User,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub assets: Vec<Asset>,
}

// TODO: move User to more general place.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    pub site_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub url: String,
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub label: String,
    pub uploader: User,
    pub content_type: String,
    pub state: String,
    pub size: u32,
    pub download_count: u32,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: String,
}

pub fn list_releases(cfg: &Config, repo: &str) -> Result<Vec<Release>, Error> {
    let b = util::get_request_builder(&cfg, Method::GET, format!("/repos/{}/releases", repo));
    let responses = util::paginate(&b, 1, 30)?;
    let json: Result<Vec<Vec<Release>>, _> = responses.into_iter().map(map_release).collect();
    match json {
        Ok(x) => Ok(x.into_iter().flatten().collect()),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn download_asset(cfg: &Config, repo: &str, asset_id: u32) -> Result<Response, Error> {
    let b = util::get_request_builder(
        &cfg,
        Method::GET,
        format!("/repos/{}/releases/assets/{}", repo, asset_id),
    );

    let x = b.header("Accept", "application/octet-stream").send()?;
    Ok(x)
}

fn map_release(mut r: Response) -> Result<Vec<Release>, Error> {
    match r.json() {
        Ok(x) => Ok(x),
        Err(e) => Err(Error::from(e)),
    }
}
