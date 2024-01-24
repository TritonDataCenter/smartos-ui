use std::process::Stdio;

use crate::endpoints::{CacheEntry, Context, PathParams};

use dropshot::{endpoint, HttpError, Path, RequestContext};
use hyper::{Body, Response, StatusCode};
use time::{Duration, OffsetDateTime};
use tokio::process::Command;

#[endpoint {
method = GET,
path = "/image",
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    if let Ok(cache) = ctx.context().cache.clone().lock() {
        if let Some(entry) = &cache.images {
            if entry.expiry > OffsetDateTime::now_utc() {
                println!("Cache hit: expiry: {}", entry.expiry);
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(entry.content.clone().into())?);
            }
        }
        drop(cache); // unlock mutex
    }

    let out = Command::new("imgadm")
        .args(["list", "-j"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("imgadm command failed to start")
        .wait_with_output()
        .await
        .expect("imgadm command failed to run");

    let stdout = String::from_utf8(out.stdout).unwrap();

    if let Ok(mut cache) = ctx.context().cache.clone().lock() {
        let expiry = OffsetDateTime::now_utc() + Duration::new(60 * 3, 0); // 3 mins
        cache.images = Some(CacheEntry {
            expiry,
            content: stdout.clone(),
        });
        drop(cache); // unlock mutex
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(stdout.into())
        .unwrap())
}

#[endpoint {
method = GET,
path = "/image/{id}",
}]
pub async fn get_by_id(
    _: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let path = path_params.into_inner();
    let out = Command::new("imgadm")
        .args(["get", &path.id.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("imgadm get failed to start")
        .wait_with_output()
        .await
        .expect("imgadm get failed to run");

    let stdout = String::from_utf8(out.stdout).unwrap_or_default();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(stdout.into())
        .unwrap())
}
