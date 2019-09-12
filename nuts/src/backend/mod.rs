use crate::{Platform, Version};
use std::ffi::OsStr;
use std::fmt::Display;

pub mod github;

pub trait Backend {
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
}
