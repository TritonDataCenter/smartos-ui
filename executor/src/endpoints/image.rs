/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::process::Stdio;

use crate::endpoints::{exec, exec_and_cache, Context, PathParams};

use smartos_shared::http_server::{
    empty_ok, to_bad_request, to_internal_error, GenericResponse,
};
use smartos_shared::image::{Image, ImageImportParams, ImportStatus, Manifest};

use dropshot::{endpoint, HttpError, Path, RequestContext, TypedBody};
use hyper::{Body, Response, StatusCode};
use slog::{debug, error, info};
use tokio::process::Command;

#[endpoint {
method = GET,
path = "/image",
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json");
    let key = "imgadm list -j";

    let (stdout, cached) = match ctx.context().get_cache(key) {
        Some(cache) => {
            debug!(ctx.log, "Cache hit for \"{key}\"");
            (cache, true)
        }
        None => {
            debug!(ctx.log, "Cache miss for \"{key}\"");
            let (stdout, _) = exec(&ctx, "imgadm", ["list", "-j"]).await?;
            (stdout, false)
        }
    };

    let import_queue = ctx.context().import_queue.clone();
    if let Ok(mut queue) = import_queue.lock() {
        // If there are images in the import queue? Parse the imgadm output
        // and append any queued images to the list.
        if !queue.is_empty() {
            let mut images: Vec<Image> =
                serde_json::from_str(&stdout).map_err(to_internal_error)?;
            let mut found = Vec::new();
            for (id, image) in queue.iter() {
                if !images.iter().any(|i| i.manifest.uuid == *id) {
                    images.push(image.clone());
                } else {
                    found.push(*id);
                }
            }
            for k in found {
                queue.remove(&k);
            }
            let response_body =
                serde_json::to_string(&images).map_err(to_internal_error)?;

            if !cached {
                debug!(ctx.log, "Caching output");
                ctx.context().set_cache(key, stdout);
            }

            return response
                .body(response_body.into())
                .map_err(to_internal_error);
        }

        if !cached {
            debug!(ctx.log, "Caching output");
            ctx.context().set_cache("imgadm list -j", stdout.clone());
        }

        // happy path, just return the raw output from imgadm
        return response
            .status(StatusCode::OK)
            .body(stdout.into())
            .map_err(to_internal_error);
    }

    // failed to get lock on queue
    Err(to_internal_error("Failed to lock image import queue"))
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
    exec_and_cache(ctx, "imgadm", ["get", &id.to_string()]).await
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

    exec(&ctx, "imgadm", ["delete", &id.to_string()]).await?;

    ctx.context().remove_cache(format!("imgadm get {}", id));

    ctx.context().remove_cache("imgadm list -j");

    // TODO: Sometimes after invoking a delete, the image will still show up
    // in imgadm list, this very slight delay appears to resolve it but is
    // a hack.
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    empty_ok()
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
    let req = request_body.into_inner();
    let image_import_queue = ctx.context().import_queue.clone();

    // Check the queue to ensure this image isn't already being imported
    if let Ok(mut queue) = image_import_queue.lock() {
        if let Some(entry) = queue.get(&id) {
            match &entry.import_status {
                Some(ImportStatus::Failed(msg)) => {
                    info!(
                        ctx.log,
                        "Attempting to re-import failed image: {} {}", id, msg
                    );
                }
                Some(ImportStatus::Importing) => {
                    return Err(to_bad_request(format!(
                        "Image is still being imported: {}",
                        id
                    )));
                }
                None => {
                    return Err(to_internal_error(format!(
                        "Encountered image in queue with no status: {}",
                        id
                    )));
                }
            }
        }

        debug!(ctx.log, "Inserting {} into import queue", id);
        queue.insert(
            id,
            Image {
                manifest: Manifest::new_for_import(
                    id,
                    req.name,
                    req.version,
                    req.r#type,
                    req.os,
                ),
                source: req.url.clone(),
                import_status: Some(ImportStatus::Importing),
            },
        );
        drop(queue);
    } else {
        return Err(to_internal_error("Unable to get image import queue"));
    }

    // We need to spawn tokio tasks for long-running processes
    // https://github.com/oxidecomputer/dropshot/issues/695
    tokio::spawn(async move {
        info!(ctx.log, "Starting import task for {}", id);
        let args = ["import", "-q", "-S", &req.url.as_ref(), &id.to_string()];
        debug!(ctx.log, "Executing imgadm {:?}", &args);
        let out = Command::new("imgadm")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(to_internal_error)?
            .wait_with_output()
            .await
            .map_err(to_internal_error)?;

        if let Ok(mut queue) = image_import_queue.lock() {
            if out.status.success() {
                let stdout =
                    String::from_utf8(out.stdout).map_err(to_internal_error)?;
                info!(ctx.log, "Image {} import success: {}", id, stdout);
                queue.remove(&id);
                ctx.context().remove_cache("imgadm list -j");
                drop(queue);
                let resp = GenericResponse {
                    request_id: ctx.request_id,
                    message: format!(
                        "Image {} has been imported and is ready to use.",
                        id
                    ),
                    detail: stdout,
                };
                return Response::builder()
                    .status(StatusCode::OK)
                    .body(
                        serde_json::to_string(&resp)
                            .map_err(to_internal_error)?
                            .into(),
                    )
                    .map_err(to_internal_error);
            }

            let stderr =
                String::from_utf8(out.stderr).map_err(to_internal_error)?;

            if let Some(mut image) = queue.remove(&id) {
                error!(ctx.log, "Image {} import failed: {}", id, stderr);
                image.import_status =
                    Some(ImportStatus::Failed(stderr.clone()));
                queue.insert(id, image);
            } else {
                error!(ctx.log, "Not queue entry found for {}", id);
            }

            drop(queue);
            return Err(to_bad_request(format!(
                "Import failed for {}: {}",
                id, stderr
            )));
        }

        Err(to_internal_error("Failed to get image import queue"))
    })
    .await
    .unwrap_or_else(|e| {
        Err(HttpError::for_internal_error(format!(
            "Failed awaiting \"post_import_by_id\": {:#}",
            e
        )))
    })
}

#[endpoint {
method = GET,
path = "/source",
}]
pub async fn get_source_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    exec_and_cache(ctx, "imgadm", ["sources", "-j"]).await
}

#[endpoint {
method = GET,
path = "/avail",
}]
pub async fn get_avail(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    exec_and_cache(ctx, "imgadm", ["avail", "-j"]).await
}
