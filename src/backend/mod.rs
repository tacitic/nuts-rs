use crate::{Platform, Version};
use reqwest::Url;
use std::ffi::OsStr;
use std::fmt::Display;

pub mod github;

pub trait Backend {
    // Filter the releases by these criteria
    fn get_release(&self, platform: Platform, version: Version)
        -> Result<Box<dyn Release>, String>;

    // Resolve a release, by filtering then taking the first result
    fn resolve_release(
        &self,
        platform: Platform,
        version: Version,
    ) -> Result<Box<dyn Release>, String>;
}

pub trait Release {
    fn get_platform(&self) -> &Platform;
    fn get_version(&self) -> &Version;
    fn get_file_type(&self) -> Option<&OsStr>;
    fn get_download_url(&self) -> &Url;
}
