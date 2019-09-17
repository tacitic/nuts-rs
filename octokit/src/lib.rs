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





mod config;
pub mod endpoint;
pub(crate) mod error;
pub(crate) mod util;

pub use config::Config;
