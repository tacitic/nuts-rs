use crate::{Platform, Version};
use reqwest::Response;
use std::ffi::OsStr;
use std::fmt::Display;
use std::path::PathBuf;

pub mod github;

pub trait Backend {
    fn resolve_release(
        &self,
        platform: Platform,
        version: Version,
    ) -> Result<Box<dyn Release>, String>;

    fn get_release_by_filename(&self, filename: String) -> Result<Box<dyn Release>, String>;

    fn download(&self, filename: String) -> Result<Response, String>;
}

pub trait Release {
    fn get_platform(&self) -> &Platform;
    fn get_version(&self) -> &Version;
    fn get_filename(&self) -> &PathBuf;
}
