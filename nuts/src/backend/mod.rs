use crate::{Platform, Version};
use failure::Error;
use reqwest::Response;
use std::path::PathBuf;

pub mod github;

pub trait Backend {
    fn resolve_release(
        &self,
        platform: Platform,
        version: Version,
    ) -> Result<Box<dyn Release>, Error>;

    fn get_release_by_filename(&self, filename: String) -> Result<Box<dyn Release>, Error>;

    fn download(&self, filename: &str) -> Result<Response, Error>;
}

pub trait Release {
    fn get_platform(&self) -> &Platform;
    fn get_version(&self) -> &Version;
    fn get_filename(&self) -> &PathBuf;
}
