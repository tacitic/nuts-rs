#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::{env, io};

use rocket::request::FromParam;
use rocket::response::content::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

use nuts::backend::github::{self, Github, GithubRelease};
use nuts::backend::{Backend, Release};
use nuts::{ApiToken, Config, Platform, Version};
use reqwest::Response;
use rocket::response::Stream;

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
        repo: cfg.github_repository,
        token: Some(cfg.github_access_token),
    });

    rocket::ignite()
        .manage(backend)
        .mount("/", routes![index, update, download])
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

#[get("/download/<filename>")]
fn download(filename: String, backend: State<Github>) -> io::Result<Stream<Response>> {
    let response = backend.download(filename).unwrap();
    Ok(Stream::chunked(response, 10))
}

fn generate_download_url(release: Box<dyn Release>) -> String {
    format!(
        "{scheme}://{host}/download/{filename}",
        scheme = "http",
        host = "localhost:8000",
        filename = release.get_filename().to_str().unwrap()
    )
}
