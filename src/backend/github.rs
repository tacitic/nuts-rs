use crate::backend::{Backend, Release};
use crate::{Platform, Version};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;

pub struct Config {
    pub repository: String,
    pub access_token: String,
}

pub struct Github {
    base_url: Url,
    config: Config,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReleaseResponse {
    id: u32,
    url: String,
    tag_name: String,
    name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<AssetResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetResponse {
    url: String,
    name: String,
    content_type: String,
    size: u32,
    state: String,
}

impl Github {
    pub fn new(cfg: Config) -> Self {
        Github {
            base_url: Url::parse("https://api.github.com").unwrap(),
            config: cfg,
        }
    }

    // TODO: handle paging
    // TODO: error handling
    fn get_releases(&self) -> Result<Vec<Box<dyn Release>>, String> {
        let client = reqwest::Client::new();
        let mut response = client.get(
            self.base_url
                .join(format!("/repos/{repo}/releases", repo = self.config.repository).as_str())
                .unwrap(),
        );

        if &self.config.access_token.len() > &0 {
            println!("Adding bearer auth {}", &self.config.access_token);
            response = response.bearer_auth(&self.config.access_token);
        }

        match response.send() {
            Ok(mut res) => {
                if !res.status().is_success() {
                    println!("{:?}", res);
                    return Err("unsuccessful response from github".to_string());
                }
                let releases: Vec<ReleaseResponse> = res.json().unwrap();

                let mut out: Vec<Box<dyn Release>> = vec![];
                for gh_release in releases {
                    for gh_asset in gh_release.assets {
                        out.push(Box::new(GithubRelease {
                            platform: Platform::detect_from_filename(&gh_asset.name),
                            version: Version::from(&gh_release.tag_name).unwrap(),
                            file: PathBuf::from(gh_asset.name),
                            url: Url::parse(gh_asset.url.as_str()).unwrap(),
                        }));
                    }
                }
                Ok(out)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Backend for Github {
    fn get_release(
        &self,
        platform: Platform,
        version: Version,
    ) -> Result<Box<dyn Release>, String> {
        self.get_releases()
            .unwrap()
            .into_iter()
            .filter(|x| {
                *x.get_version().inner_version() == *version.inner_version()
                    && *x.get_platform() == platform
                    && x.get_file_type().unwrap().to_string_lossy() == "zip"
            })
            .nth(0)
            .ok_or("no compatible release found".to_string())
    }

    fn resolve_release(
        &self,
        platform: Platform,
        version: Version,
    ) -> Result<Box<dyn Release>, String> {
        self.get_releases()
            .unwrap()
            .into_iter()
            .filter(|x| {
                *x.get_version().inner_version() > *version.inner_version()
                    && *x.get_platform() == platform
                    && x.get_file_type().unwrap().to_string_lossy() == "zip"
            })
            .nth(0)
            .ok_or("no compatible release found".to_string())
    }
}

#[derive(Debug)]
pub struct GithubRelease {
    platform: Platform,
    version: Version,
    file: PathBuf,
    url: Url,
}

impl Release for GithubRelease {
    fn get_platform(&self) -> &Platform {
        &self.platform
    }

    fn get_version(&self) -> &Version {
        &self.version
    }

    fn get_file_type(&self) -> Option<&OsStr> {
        self.file.extension()
    }

    fn get_download_url(&self) -> &Url {
        &self.url
    }
}
