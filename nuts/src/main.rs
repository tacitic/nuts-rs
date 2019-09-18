#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::{env, fs, io, time};

use failure::Error;

use rocket::response::content::Json;
use rocket::response::NamedFile;
use rocket::State;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use nuts::backend::github::{self, Github};
use nuts::backend::{Backend, Release};
use nuts::{ApiToken, BaseUrl, Config, Platform, Signature, Version};
use rocket::config::Environment;
use signed_urls::sign_url;

/// Returned by a request to /update
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResponse {
    url: String,
}

fn main() {
    let cfg = Config {
        secret_token: env::var("NUTS_SECRET_TOKEN").ok(),
        url_signature_secret: env::var("NUTS_URL_SIGNATURE_SECRET").ok(),
        github_repository: env::var("NUTS_GITHUB_REPOSITORY").unwrap_or_default(),
        github_access_token: env::var("NUTS_GITHUB_TOKEN").unwrap_or_default(),
        base_url: env::var("NUTS_BASE_URL").ok(),
    };

    let backend = Github::new(github::Config {
        repo: cfg.github_repository.clone(),
        token: Some(cfg.github_access_token.clone()),
    });

    println!("config: {:?}", cfg);

    // TODO: make configurable
    let rocket_config = rocket::Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(8000)
        .finalize()
        .unwrap();

    rocket::custom(rocket_config)
        .manage(backend)
        .manage(cfg)
        .mount("/", routes![update, download])
        .launch();
}

/// TODO: backend: State<Box<dyn Backend + Sync + Send>>,
#[get("/update/<platform>/<version>")]
fn update(
    platform: Platform,
    version: Version,
    base_url: BaseUrl,
    config: State<Config>,
    backend: State<Github>,
    _api_token: ApiToken,
) -> Json<String> {
    let release = backend.resolve_release(platform, version).unwrap();

    Json(
        serde_json::to_string(&UpdateResponse {
            url: generate_download_url(&config, &base_url, release).unwrap(),
        })
        .unwrap(),
    )
}

#[get("/download/<filename>")]
fn download(
    filename: String,
    backend: State<Github>,
    _signature: Signature,
) -> io::Result<NamedFile> {
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

fn generate_download_url(
    config: &Config,
    base_url: &BaseUrl,
    release: Box<dyn Release>,
) -> Result<String, Error> {
    let url = format!(
        "{base_url}/download/{filename}",
        base_url = base_url.to_string(),
        filename = release.get_filename().to_str().unwrap()
    );

    println!(
        "generate_download_url: {} {} {}",
        base_url.to_string(),
        release.get_filename().to_str().unwrap(),
        url
    );

    if let Some(secret) = &config.url_signature_secret {
        let exp = time::SystemTime::now() + time::Duration::from_secs(60);
        let url = sign_url(secret, url.as_str(), exp)?;
        return Ok(url);
    }

    Ok(url.to_string())
}
