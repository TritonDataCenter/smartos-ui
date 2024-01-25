use std::process::Stdio;

use crate::endpoints::{CacheEntry, Context, PathParams};

use dropshot::{endpoint, HttpError, Path, RequestContext};
use hyper::{Body, Response, StatusCode};
use slog::debug;
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
        let cache_duration = ctx.context().config.exec_cache_seconds;
        let expiry =
            OffsetDateTime::now_utc() + Duration::new(cache_duration, 0);
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
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let id = path_params.into_inner().id;

    if let Ok(cache) = ctx.context().cache.clone().lock() {
        if let Some(entry) = cache.image.get(&id) {
            if entry.expiry > OffsetDateTime::now_utc() {
                debug!(
                    ctx.log,
                    "/image/{} Cache hit, expires: {}", id, entry.expiry
                );
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(entry.content.clone().into())?);
            }
        }
        drop(cache); // unlock mutex
    } else {
        debug!(ctx.log, "/image/{id} Cache miss");
    }

    let out = Command::new("imgadm")
        .args(["get", &id.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("imgadm get failed to start")
        .wait_with_output()
        .await
        .expect("imgadm get failed to run");

    let stdout = String::from_utf8(out.stdout).unwrap_or_default();

    if let Ok(mut cache) = ctx.context().cache.clone().lock() {
        let cache_duration = ctx.context().config.exec_cache_seconds;
        let expiry =
            OffsetDateTime::now_utc() + Duration::new(cache_duration, 0);
        debug!(
            ctx.log,
            "/image/{} Cache entry added, expires: {}", id, expiry
        );
        cache.image.insert(
            id,
            CacheEntry {
                expiry,
                content: stdout.clone(),
            },
        );
        drop(cache); // unlock mutex
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(stdout.into())
        .unwrap())
}

#[endpoint {
method = DELETE,
path = "/image/{id}",
}]
pub async fn delete_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let id = path_params.into_inner().id;

    let _ = Command::new("imgadm")
        .args(["delete", &id.to_string()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("imgadm command failed to start");

    if let Ok(mut cache) = ctx.context().cache.clone().lock() {
        cache.images = None;
        cache.image.remove(&id);
        drop(cache);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}
