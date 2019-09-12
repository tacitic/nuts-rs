#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::env;

use rocket::request::FromParam;
use rocket::response::content::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

use nuts::backend::github::{self, Github};
use nuts::backend::{Backend, Release};
use nuts::{ApiToken, Config, Platform, Version};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResponse {
    url: String,
}

impl UpdateResponse {
    pub fn dummy() -> Self {
        UpdateResponse {
            url: "http://localhost:4000/flux/download/version/0.3.0-alpha.16/osx_64?filetype=zip"
                .to_string(),
        }
    }
}

fn main() {
    let cfg = Config {
        jwt_secret: env::var("JWT_SECRET").unwrap_or_default(),
        github_repository: env::var("GITHUB_REPOSITORY").unwrap_or_default(),
        github_access_token: env::var("GITHUB_TOKEN").unwrap_or_default(),
    };

    let backend = Github::new(github::Config {
        repository: cfg.github_repository.clone(),
        access_token: cfg.github_access_token.clone(),
    });

    rocket::ignite()
        .manage(backend)
        .mount("/", routes![index, update])
        .launch();
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

/// TODO: backend: State<Box<dyn Backend + Sync + Send>>,
#[get("/update/<platform>/<version>")]
fn update(
    platform: Platform,
    version: Version,
    api_token: ApiToken,
    backend: State<Github>,
) -> Json<String> {
    let release = backend.resolve_release(platform, version).unwrap();

    Json(
        serde_json::to_string(&UpdateResponse {
            url: generate_download_url(release),
        })
        .unwrap(),
    )
}

#[get("/download/<platform>/<version>")]
fn download(platform: Platform, version: Version) -> &'static str {
    "Download"
}

fn generate_download_url(release: Box<dyn Release>) -> String {
    format!(
        "{scheme}://{host}/download/{platform}/{version}?file_type={file_type}",
        scheme = "http",
        host = "localhost:8000",
        platform = release.get_platform().to_string(),
        version = release.get_version().inner_version().to_string(),
        file_type = release.get_file_type().unwrap().to_string_lossy()
    )
}
