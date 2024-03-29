#![feature(proc_macro_hygiene, decl_macro)]

use rocket::http::{RawStr, Status};
use rocket::request::{self, FromParam, FromRequest};
use rocket::{Outcome, Request, State};

pub mod backend;
pub(crate) mod error;
pub(crate) use error::ErrorKind;
use signed_urls::validate;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate rocket;

/// Represents a platform
#[derive(Debug, PartialEq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
}

impl Platform {
    /// Detects a platform from a given filename
    pub fn detect_from_filename(name: &str) -> Result<Self, ErrorKind> {
        if name.contains("mac")
            || name.contains("osx")
            || name.contains("darwin")
            || name.ends_with(".dmg")
            || name.ends_with(".dmg.blockmap")
        {
            return Ok(Self::MacOS);
        }

        if name.contains("linux")
            || name.contains("ubuntu")
            || name.ends_with(".deb")
            || name.ends_with(".rpm")
            || name.ends_with(".tgz")
            || name.ends_with(".tar.gz")
            || name.ends_with(".AppImage")
        {
            return Ok(Self::Linux);
        }

        if name.contains("win") || name.ends_with(".exe") {
            return Ok(Self::Windows);
        }

        Err(ErrorKind::UnknownPlatform(name.to_string()))
    }
}

impl ToString for Platform {
    fn to_string(&self) -> String {
        match &self {
            Platform::MacOS => "osx".to_string(),
            Platform::Windows => "win".to_string(),
            Platform::Linux => "linux".to_string(),
        }
    }
}

/// Configuation for Nuts
#[derive(Debug)]
pub struct Config {
    /// Used to control access to the /update endpoint, only enforced when set.
    pub secret_token: Option<String>,

    /// Used to control access to the /download endpoint, ony enforced when set.
    pub url_signature_secret: Option<String>,

    /// A Github repository in the form of '<username>/<repo>'
    pub github_repository: String,

    /// Will be used to access private Github repositories.
    pub github_access_token: String,

    /// Will be used to generate the download urls, if not set hostname and scheme is used.
    pub base_url: Option<String>,
}

/// ApiToken is a rocket guard  that is used in combination with the 'secret_token' config parameter.
#[derive(Debug)]
pub struct ApiToken();

impl FromRequest<'_, '_> for ApiToken {
    type Error = String;

    fn from_request(request: &Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config = request.guard::<State<Config>>().unwrap();
        if let Some(secret) = &config.secret_token {
            let keys: Vec<_> = request.headers().get("Authorization").collect();
            return match keys.len() {
                0 => Outcome::Failure((Status::BadRequest, "Missing bearer token".to_string())),
                1 if is_valid_auth_token(keys[0], secret) => Outcome::Success(ApiToken()),
                1 => Outcome::Failure((Status::Unauthorized, "Unauthorized".to_string())),
                _ => Outcome::Failure((Status::Unauthorized, "Unauthorized".to_string())),
            };
        }

        Outcome::Success(ApiToken())
    }
}

/// Signature is a rocket guard that is used in combination with the 'url_signature_secret'
/// configuration option.
#[derive(Debug)]
pub struct Signature();

impl FromRequest<'_, '_> for Signature {
    type Error = String;

    fn from_request(request: &Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config = request.guard::<State<Config>>().unwrap();
        let base_url = request.guard::<BaseUrl>().unwrap();

        // TODO(rharink): Find a better way to ignore/replace multiple consecutive slashes in a url
        let url = str::replace(
            format!(
                "{base_url}{uri}",
                base_url = base_url.to_string(),
                uri = request.uri().to_string(),
            )
            .as_str(),
            "//",
            "/",
        );
        println!("{}", url);

        if let Some(secret) = &config.url_signature_secret {
            return match validate(&secret, &url) {
                Ok(_) => Outcome::Success(Signature()),
                Err(_) => Outcome::Failure((Status::Unauthorized, "Invalid signature".to_string())),
            };
        }

        Outcome::Success(Signature())
    }
}

/// Returns the baseurl, this can be overwritten in the configuration.
pub struct BaseUrl(String);

impl ToString for BaseUrl {
    fn to_string(&self) -> String {
        if self.0.ends_with("/") {
            return self.0[..self.0.len() - 1].to_string();
        }
        self.0.clone()
    }
}

impl FromRequest<'_, '_> for BaseUrl {
    type Error = String;

    fn from_request(request: &Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config = request.guard::<State<Config>>().unwrap();
        if let Some(base_url) = &config.base_url {
            println!("returning base-url from config {}", base_url);
            return Outcome::Success(BaseUrl(base_url.clone()));
        }

        let scheme = get_scheme(&request);

        let host = match get_host(&request) {
            Some(host) => host,
            None => {
                return Outcome::Failure((
                    Status::BadRequest,
                    "cannot determine hostname".to_string(),
                ))
            }
        };

        let url = format!("{}://{}", scheme.to_string(), host);
        println!("returning base-url from request {}", url);

        Outcome::Success(BaseUrl(url))
    }
}

/// Scheme is a rocket guard that represents a scheme
pub enum Scheme {
    Http,
    Https,
}

impl ToString for Scheme {
    fn to_string(&self) -> String {
        match &self {
            Scheme::Http => "http".to_string(),
            Scheme::Https => "https".to_string(),
        }
    }
}

/// Returns the host of the request.
fn get_host(req: &Request) -> Option<String> {
    if let Some(h) = req.headers().get_one("X-Forwarded-Host") {
        return Some(h.to_string());
    }

    req.headers().get_one("Host").map(|x| x.to_string())
}

/// Returns the scheme of the request, respectful to the X-Forwarded-Proto header
fn get_scheme(req: &Request) -> Scheme {
    let raw = match req.headers().get_one("X-Forwarded-Proto") {
        Some(h) => h.to_string(),
        None => req
            .headers()
            .get_one("Scheme")
            .unwrap_or("http")
            .to_string(),
    };

    match raw.as_str() {
        "https" => Scheme::Https,
        _ => Scheme::Http,
    }
}

// TODO(rharink): Implement better token authentication
fn is_valid_auth_token(header: &str, secret: &str) -> bool {
    if header.starts_with("Bearer") {
        let bearer_token = &header[7..];
        return bearer_token == secret;
    }

    return false;
}

// TODO: docs
#[derive(Debug)]
pub struct Version(semver::Version);

impl Version {
    fn from(s: &str) -> Result<Self, semver::SemVerError> {
        let mut name = s.clone();
        if name.starts_with("v") {
            name = &name[1..];
        }

        let x = semver::Version::parse(name)?;
        Ok(Self(x))
    }

    pub fn inner_version(&self) -> &semver::Version {
        &self.0
    }

    pub fn channel(&self) -> Option<String> {
        self.inner_version().pre.first().map(|x| match x {
            semver::Identifier::AlphaNumeric(x) => x.clone(),
            semver::Identifier::Numeric(n) => n.to_string(),
        })
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<'a> FromParam<'a> for Version {
    type Error = semver::SemVerError;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        Version::from(param)
    }
}

impl<'a> FromParam<'a> for Platform {
    type Error = failure::Error;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        let platform = param.percent_decode()?.to_lowercase();

        if platform.contains("darwin") || platform.contains("mac") || platform.contains("osx") {
            return Ok(Platform::MacOS);
        }

        bail!("Unsupported platform")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_param() {
        Platform::from_param(RawStr::from_str("darwin")).unwrap();
    }

    #[test]
    fn test_parse_platform_from_filename() {
        let tests = vec![
            ("mac.zip", Platform::MacOS),
            ("some-cool-version-on-osx", Platform::MacOS),
            ("linux.tar", Platform::Linux),
            ("some-file.deb", Platform::Linux),
            ("some-file.rpm", Platform::Linux),
            ("winnie-the-pooh", Platform::Windows),
            ("awseome-app.exe", Platform::Windows),
        ];

        for (filename, expect) in tests {
            assert_eq!(Platform::detect_from_filename(filename).unwrap(), expect);
        }
    }

    #[test]
    fn test_version_from() {
        Version::from("0.1.0").unwrap();
        Version::from("0.1.0-alpha").unwrap();
        Version::from("0.1.0-alpha.0").unwrap();
        Version::from("v0.1.0-alpha.0").unwrap();
        Version::from("v2.0.0-beta.1").unwrap();
    }

    #[test]
    fn test_base_url() {
        let bu = BaseUrl("https://release.something.inc/product/".to_string());
        assert_eq!(
            bu.to_string(),
            "https://release.something.inc/product".to_string()
        );
    }
}
