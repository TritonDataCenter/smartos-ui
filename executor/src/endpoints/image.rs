use std::process::Stdio;

use crate::endpoints::{CacheEntry, Context, PathParams};

use dropshot::{endpoint, HttpError, Path, RequestContext, TypedBody};
use hyper::{Body, Response, StatusCode};
use slog::info;
use smartos_shared::image::ImageImportParams;
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
                info!(ctx.log, "Cache hit: expiry: {}", entry.expiry);
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(entry.content.clone().into())?);
            } else {
                info!(ctx.log, "Cache miss");
            }
        } else {
            info!(ctx.log, "No cache");
        }

        drop(cache);
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

    info!(ctx.log, "imgadm list: {}", stdout);

    if let Ok(mut cache) = ctx.context().cache.clone().lock() {
        let cache_duration = ctx.context().config.exec_cache_seconds;
        let expiry =
            OffsetDateTime::now_utc() + Duration::new(cache_duration, 0);
        info!(ctx.log, "Updating cache, expiry: {}", expiry);
        cache.images = Some(CacheEntry {
            expiry,
            content: stdout.clone(),
        });
        drop(cache);
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
                info!(
                    ctx.log,
                    "/image/{} Cache hit, expires: {}", id, entry.expiry
                );
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(entry.content.clone().into())?);
            } else {
                info!(ctx.log, "Cache miss for: /image/{}", id);
            }
        } else {
            info!(ctx.log, "No cache for: /image/{}", id);
        }
        drop(cache);
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
        info!(
            ctx.log,
            "Cache entry added for /image/{}, expires: {}", id, expiry
        );
        cache.image.insert(
            id,
            CacheEntry {
                expiry,
                content: stdout.clone(),
            },
        );
        drop(cache);
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
        info!(ctx.log, "Purging cache due to image delete: {}", id);
        drop(cache);
    }

    // This is a hack. Sometimes after deleting an image, execing `imgadm list`
    // immediately after will return the deleted image (even after purging
    // the cache or disabling it entirely.)
    // Oddly, the same thing does not occur when adding an image, it's always
    // shown in the list. I've also not been able to reproduce with
    // `imgadm delete $ID && imgadm list -j`
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

#[endpoint {
method = GET,
path = "/source",
}]
pub async fn get_source_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    if let Ok(cache) = ctx.context().cache.clone().lock() {
        if let Some(entry) = &cache.sources {
            if entry.expiry > OffsetDateTime::now_utc() {
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(entry.content.clone().into())?);
            } else {
                info!(ctx.log, "Image source cache miss");
            }
        } else {
            info!(ctx.log, "No cache for image source");
        }
        drop(cache);
    }

    let out = Command::new("imgadm")
        .args(["sources", "-j"])
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
        info!(
            ctx.log,
            "Cache entry added for image sources, expires: {}", expiry
        );
        cache.sources = Some(CacheEntry {
            expiry,
            content: stdout.clone(),
        });
        drop(cache);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(stdout.into())
        .unwrap())
}

#[endpoint {
method = POST,
path = "/import/{id}",
}]
pub async fn post_import_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
    request_body: TypedBody<ImageImportParams>,
) -> Result<Response<Body>, HttpError> {
    let id = path_params.into_inner().id;
    let url = request_body.into_inner().url;
    let cache = ctx.context().cache.clone();
    info!(ctx.log, "ID: {}, URL: {}", id, url);

    // We need to spawn tokio tasks for long-running processes
    // https://github.com/oxidecomputer/dropshot/issues/695
    match tokio::spawn(async move {
        let out = Command::new("imgadm")
            .args(["import", "-q", "-S", &url, &id.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("imgadm get failed to start")
            .wait_with_output()
            .await
            .expect("imgadm get failed to run");

        info!(ctx.log, "Waiting for import");

        let stdout = String::from_utf8(out.stdout).unwrap_or_default();

        if let Ok(mut cache) = cache.lock() {
            info!(ctx.log, "Purging cache due to image import: {}", id);
            cache.images = None;
            cache.image.remove(&id);
            info!(ctx.log, "Purging cache");
            drop(cache);
        }

        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(stdout.into()).unwrap()
    }).await {
        Ok(result) => Ok(result),
        Err(e) => {
            return Err(HttpError::for_internal_error(format!(
                "unexpected failure awaiting \"services_ensure\": {:#}",
                e
            )));
        }
    }
}
