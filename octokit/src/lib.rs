/// Octokit
///
/// ## Usage
/// ```
/// fn main() {
///     let cfg = octokit::Config::new_authenticated("secret_access_token");
/// }
/// ```
#[macro_use]
extern crate nom;

use reqwest::header::HeaderMap;
use reqwest::Url;
use std::str::FromStr;

mod config;
mod endpoint;
pub(crate) mod util;

pub use config::Config;
