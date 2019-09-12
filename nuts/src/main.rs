#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::prelude::*;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::{env, fs, io};

use reqwest::Response;
use rocket::request::FromParam;
use rocket::response::content::Json;
use rocket::response::{NamedFile, Stream};
use rocket::State;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use nuts::backend::github::{self, Github, GithubRelease};
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
        repo: cfg.github_repository.clone(),
        token: Some(cfg.github_access_token.clone()),
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
    _api_token: ApiToken,
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
fn download(filename: String, backend: State<Github>) -> io::Result<NamedFile> {
    let mut cache_path = std::env::temp_dir();
    cache_path.push(filename.as_str());

    if fs::metadata(&cache_path).is_err() {
        let mut tmp_file = NamedTempFile::new()?;
        backend
            .download(filename.as_str())
            .unwrap()
            .copy_to(&mut tmp_file)
            .unwrap();

        std::fs::rename(tmp_file.path(), &cache_path)?;
    }

    NamedFile::open(&cache_path)
}

fn generate_download_url(release: Box<dyn Release>) -> String {
    format!(
        "{scheme}://{host}/download/{filename}",
        scheme = "http",
        host = "localhost:8000",
        filename = release.get_filename().to_str().unwrap()
    )
}
