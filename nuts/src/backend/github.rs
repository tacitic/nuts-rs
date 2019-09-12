use crate::backend::{Backend, Release};
use crate::{Platform, Version};
use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub struct Config {
    pub repo: String,
    pub token: Option<String>,
}

pub struct Github {
    repo: String,
    config: octokit::Config,
}

impl Github {
    pub fn new(cfg: Config) -> Self {
        Github {
            repo: cfg.repo,
            config: octokit::Config {
                auth: cfg.token,
                ..octokit::Config::default()
            },
        }
    }

    // TODO: error handling
    // TODO: this should be sorted on semver.
    fn get_releases(&self) -> Result<Vec<GithubRelease>, String> {
        let releases = octokit::endpoint::repos::list_releases(&self.config, &self.repo).unwrap();
        let mut out = vec![];
        for gh_release in releases {
            for gh_asset in gh_release.assets {
                out.push(GithubRelease {
                    platform: Platform::detect_from_filename(&gh_asset.name),
                    version: Version::from(&gh_release.tag_name).unwrap(),
                    filename: PathBuf::from(gh_asset.name),
                    download_url: gh_asset.url,
                    asset_id: gh_asset.id,
                });
            }
        }

        Ok(out)
    }

    fn get_release_by_predicate(
        &self,
        f: &dyn Fn(&GithubRelease) -> bool,
    ) -> Result<GithubRelease, String> {
        self.get_releases()
            .unwrap()
            .into_iter()
            .filter(f)
            .nth(0)
            .ok_or("no compatible release found".to_string())
    }
}

impl Backend for Github {
    fn resolve_release(
        &self,
        platform: Platform,
        version: Version,
    ) -> Result<Box<dyn Release>, String> {
        self.get_release_by_predicate(&|x: &GithubRelease| {
            *x.get_platform() == platform
                && *x.get_version().inner_version() > *version.inner_version()
        })
        .map(|x| Box::new(x) as Box<dyn Release>)
    }

    fn get_release_by_filename(&self, filename: String) -> Result<Box<dyn Release>, String> {
        self.get_release_by_predicate(&|x: &GithubRelease| {
            *x.get_filename() == PathBuf::from(filename.as_str())
        })
        .map(|x| Box::new(x) as Box<dyn Release>)
    }

    fn download(&self, filename: String) -> Result<Response, String> {
        let release = self
            .get_release_by_predicate(&|x: &GithubRelease| {
                *x.get_filename() == PathBuf::from(filename.as_str())
            })
            .unwrap();

        Ok(
            octokit::endpoint::repos::download_asset(&self.config, &self.repo, release.asset_id)
                .unwrap(),
        )
    }
}

#[derive(Debug)]
pub struct GithubRelease {
    platform: Platform,
    version: Version,
    filename: PathBuf,
    download_url: String,
    asset_id: u32,
}

impl Release for GithubRelease {
    fn get_platform(&self) -> &Platform {
        &self.platform
    }

    fn get_version(&self) -> &Version {
        &self.version
    }

    fn get_filename(&self) -> &PathBuf {
        &self.filename
    }
}
