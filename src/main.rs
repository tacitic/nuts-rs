#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::{env, process};

use rocket::response::content;

use rocket::State;
use serde::{Deserialize, Serialize};

use nuts_rs::backend::github::{self, Github};
use nuts_rs::backend::{Backend, Release};
use nuts_rs::{ApiToken, Config, Platform, Version};
use reqwest::RedirectPolicy;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResponse {
    url: String,
}

const GITHUB_REPOSITORY: &str = "GITHUB_REPOSITORY";
const GITHUB_TOKEN: &str = "GITHUB_TOKEN";
const JWT_SECRET: &str = "JWT_SECRET";

fn main() {
    let cfg = Config {
        github_repository: env::var(GITHUB_REPOSITORY)
            // XXX(@czyk): I would rather not fall back to default but exit immediately.
            .unwrap_or_default(),

        github_access_token: env::var(GITHUB_TOKEN).unwrap_or_default(),
        jwt_secret: env::var(JWT_SECRET).unwrap_or_default(),
    };

    if cfg.github_repository.len() == 0 {
        println!("Missing env variable {}", GITHUB_REPOSITORY);
        process::exit(1);
    }

    let backend = Github::new(github::Config {
        repository: cfg.github_repository.clone(),
        access_token: cfg.github_access_token.clone(),
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
) -> content::Json<String> {
    let release = backend.resolve_release(platform, version).unwrap();
    println!("Updating {release}", release = log_release(&release));

    content::Json(
        serde_json::to_string(&UpdateResponse {
            url: generate_download_url(&release),
        })
        .unwrap(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct DownloadResponse {
    url: String,
}

#[get("/download/<platform>/<version>")]
fn download(
    platform: Platform,
    version: Version,
    backend: State<Github>,
) -> Result<String, String> {
    let release = backend.get_release(platform, version).unwrap();
    println!("Downloading {release}", release = log_release(&release));
    println!("{:?}", &release.get_download_url());

    let client = reqwest::Client::builder()
        .redirect(RedirectPolicy::none())
        .build()
        .unwrap();

    let mut response = client
        .get(release.get_download_url().clone()) // FIXME(@czyk): I don't like the .clone here...
        .header("User-Agent", "nutrs-rs")
        // @see: https://developer.github.com/v3/repos/releases/#get-a-single-release-asset
        .header("Accept", "application/octet-stream");

    // FIXME(@czyk): Use from config in Main
    let token: String = env::var(GITHUB_TOKEN).unwrap_or_default();
    if &token.len() > &0 {
        println!("Adding bearer auth {}", &token);
        response = response.bearer_auth(&token);
    }

    // XXX(@czyk): It would be awesome if we can just relay/proxy the request here with even bypassing
    //  downloading the asset through the release-server.

    match response.send() {
        Ok(mut res) => {
            if res.status().is_redirection() {
                let url = res
                    .headers()
                    .get("location")
                    .unwrap()
                    .to_str()
                    .unwrap_or_default();

                // XXX(@czyk): We do have the signed download url here which points directly to the
                //  amazon bucket form Github... Why don't we expose this url directly from the
                //  /update endpoint en remove the /download endpoint completely from our server?
                println!("Redirect {:?}", &url);
                return Ok(format!("Request for redirect... to {}", url).to_string());
            }
            println!("successful {:?}", &res);
            Ok("Okay, something else...".to_string())
        }
        Err(e) => Err(e.to_string()),
    }

    // TODO(@czyk): get output and stream to the body
    //  @see: https://rocket.rs/v0.4/guide/responses/#custom-responders
    //  @see: https://rocket.rs/v0.4/guide/responses/#streaming
    //  @see: https://api.rocket.rs/v0.4/rocket/response/struct.Stream.html
    //  @see: https://docs.rs/reqwest/0.9.20/reqwest/struct.Response.html#example

    // Code from nuts-server library in js.
    //   res.header('Content-Length', asset.size);
    //   res.attachment(asset.filename);
}

fn generate_download_url(release: &Box<dyn Release>) -> String {
    format!(
        "{scheme}://{host}/download/{platform}/{version}?file_type={file_type}",
        // TODO(@czyk): Make dynamic
        scheme = "http",
        // TODO(@czyk): Make dynamic
        host = "localhost:8000",
        platform = release.get_platform().to_string(),
        version = release.get_version().inner_version().to_string(),
        file_type = release.get_file_type().unwrap().to_string_lossy()
    )
}

fn log_release(release: &Box<dyn Release>) -> String {
    format!(
        "version: {:?} for platform: {:?} with filetype: {:?}",
        release.get_version().inner_version().to_string(),
        release.get_platform().to_string(),
        release.get_file_type().unwrap().to_string_lossy()
    )
}
