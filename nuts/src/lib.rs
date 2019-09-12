use crate::backend::Release;
use rocket::http::{RawStr, Status};
use rocket::request::{self, FromParam, FromRequest};
use rocket::{Outcome, Request};

pub mod backend;

#[derive(Debug, PartialEq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
    Unknown(String),
}

impl Platform {
    pub fn detect_from_filename(name: &str) -> Self {
        if name.contains("mac")
            || name.contains("osx")
            || name.contains("darwin")
            || name.ends_with(".dmg")
            || name.ends_with(".dmg.blockmap")
        {
            return Self::MacOS;
        }

        if name.contains("linux")
            || name.contains("ubuntu")
            || name.ends_with(".deb")
            || name.ends_with(".rpm")
            || name.ends_with(".tgz")
            || name.ends_with(".tar.gz")
            || name.ends_with(".AppImage")
        {
            return Self::Linux;
        }

        if name.contains("win") || name.ends_with(".exe") {
            return Self::Windows;
        }

        println!("cannot detect platform for {}", name);

        return Self::Unknown(name.to_string());
    }
}

impl ToString for Platform {
    fn to_string(&self) -> String {
        match &self {
            Platform::MacOS => "osx".to_string(),
            Platform::Windows => "win".to_string(),
            Platform::Linux => "linux".to_string(),
            Platform::Unknown(_) => "unknown".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    /// Used to control access to the /update endpoint
    pub jwt_secret: String,

    /// <username>/<repo>
    pub github_repository: String,

    /// Will be used to access private repositories.
    pub github_access_token: String,
}

#[derive(Debug)]
pub struct ApiToken(String);

impl FromRequest<'_, '_> for ApiToken {
    type Error = String;

    fn from_request(request: &Request<'_>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, "Missing bearer token".to_string())),
            1 if is_valid_auth_token(keys[0]) => Outcome::Success(ApiToken(keys[0].to_string())),
            1 => Outcome::Failure((Status::Unauthorized, "Unauthorized".to_string())),
            _ => Outcome::Failure((Status::Unauthorized, "Unauthorized".to_string())),
        }
    }
}

// TODO(rharink): Implement token authentication
fn is_valid_auth_token(token: &str) -> bool {
    true
}

// TODO(rharink): Should we use it like this? we might have to relay all of semver::Version's methods
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

// TODO(rharink): Error handling
// TODO(rharink): Handle errors, do not use unwrap in lib code!
impl<'a> FromParam<'a> for Platform {
    type Error = String;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        let platform = param.percent_decode().unwrap().to_lowercase();

        if platform.contains("darwin") || platform.contains("mac") || platform.contains("osx") {
            return Ok(Platform::MacOS);
        }

        return Err("supports only macOS, for now...".to_string());
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
            assert_eq!(Platform::detect_from_filename(filename), expect);
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
