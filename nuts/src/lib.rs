use failure::{AsFail, Error};
use rocket::http::{RawStr, Status};
use rocket::request::{self, FromParam, FromRequest};
use rocket::{Outcome, Request, State};

pub mod backend;
pub(crate) mod error;
pub(crate) use error::ErrorKind;
use serde_json::to_string;
use signed_urls::validate;

#[macro_use]
extern crate failure;

/// Represents the platform where updates are available for.
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

#[derive(Debug)]
pub struct Config {
    /// Used to control access to the /update endpoint
    pub secret_token: Option<String>,

    pub url_signature_secret: Option<String>,

    /// <username>/<repo>
    pub github_repository: String,

    /// Will be used to access private repositories.
    pub github_access_token: String,
}

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

#[derive(Debug)]
pub struct Signature();

impl FromRequest<'_, '_> for Signature {
    type Error = String;

    fn from_request(request: &Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config = request.guard::<State<Config>>().unwrap();
        let host = get_host(&request).expect("host not found");
        let scheme = get_scheme(&request);

        let url = format!(
            "{scheme}://{host}{uri}",
            scheme = scheme,
            host = host,
            uri = request.uri().to_string()
        );

        if let Some(secret) = &config.url_signature_secret {
            return match validate(&secret, &url) {
                Ok(_) => Outcome::Success(Signature()),
                Err(_) => Outcome::Failure((Status::Unauthorized, "Invalid signature".to_string())),
            };
        }

        Outcome::Success(Signature())
    }
}

fn get_host(req: &Request) -> Option<String> {
    if let Some(h) = req.headers().get_one("X-Forwarded-Host") {
        return Some(h.to_string());
    }

    req.headers().get_one("Host").map(|x| x.to_string())
}

fn get_scheme(req: &Request) -> String {
    if let Some(s) = req.headers().get_one("X-Forwarded-Proto") {
        return s.to_string();
    }

    req.headers()
        .get_one("Scheme")
        .unwrap_or("http")
        .to_string()
}

// TODO(rharink): Implement better token authentication
fn is_valid_auth_token(header: &str, secret: &str) -> bool {
    if header.starts_with("Bearer") {
        let bearer_token = &header[7..];
        return bearer_token == secret;
    }

    return false;
}

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
    use semver;

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
}
