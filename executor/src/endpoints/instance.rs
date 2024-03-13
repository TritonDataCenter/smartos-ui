/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::process::Stdio;

use crate::endpoints::{exec, Context, PathParams};
use smartos_shared::instance::{
    InstancePayload, InstanceValidateResponse, PayloadContainer,
};

use dropshot::{
    endpoint, HttpError, HttpResponseOk, Path, RequestContext, TypedBody,
};
use hyper::{Body, Response, StatusCode};
use slog::{debug, error, info};
use smartos_shared::http_server::{
    empty_ok, to_bad_request, to_internal_error,
};

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[endpoint {
method = POST,
path = "/provision",
}]
pub async fn post_provision_index(
    ctx: RequestContext<Context>,
    request_body: TypedBody<InstancePayload>,
) -> Result<Response<Body>, HttpError> {
    let req = request_body.into_inner();

    let PayloadContainer { uuid } =
        serde_json::from_str(&req.payload).map_err(to_bad_request)?;

    let args = ["create"];
    debug!(ctx.log, "Executing vmadm {:?}", &args);

    let mut process = Command::new("vmadm")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(to_internal_error)?;

    if let Some(mut stdin) = process.stdin.take() {
        stdin
            .write_all(req.payload.as_bytes())
            .await
            .map_err(to_internal_error)?;
        drop(stdin);
    } else {
        return Err(to_internal_error("Failed to acquire stdin pipe"));
    }

    let out = process.wait_with_output().await.map_err(to_internal_error)?;

    if out.status.success() {
        let stdout =
            String::from_utf8(out.stdout).map_err(to_internal_error)?;
        info!(ctx.log, "Instance {} create success: {}", uuid, stdout);
        return empty_ok();
    }

    let stderr = String::from_utf8(out.stderr).map_err(to_internal_error)?;
    error!(ctx.log, "Instance {} create failed: {}", uuid, stderr);

    Err(to_bad_request(stderr))
}

#[endpoint {
method = POST,
path = "/validate/create",
}]
pub async fn post_validate_create(
    _: RequestContext<Context>,
    request_body: TypedBody<InstancePayload>,
) -> Result<HttpResponseOk<InstanceValidateResponse>, HttpError> {
    let InstancePayload { payload } = request_body.into_inner();

    let mut process = Command::new("vmadm")
        .args(["validate", "create"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(to_internal_error)?;

    if let Some(mut stdin) = process.stdin.take() {
        stdin.write_all(payload.as_bytes()).await.map_err(to_internal_error)?;

        // When dropped, the underlying file handle will be closed.
        drop(stdin);

        let out =
            process.wait_with_output().await.map_err(to_internal_error)?;
        let stderr = String::from_utf8(out.stderr).unwrap_or_default();

        let response = InstanceValidateResponse {
            message: stderr,
            success: out.status.success(),
        };

        return Ok(HttpResponseOk(response));
    }

    Err(to_internal_error("Failed opening stdin of process"))
}

#[endpoint {
method = DELETE,
path = "/instance/{id}",
}]
pub async fn delete_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let req = path_params.into_inner();
    let (_, stderr) =
        exec(&ctx, "vmadm", ["delete", &req.id.to_string()]).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(stderr.into())
        .map_err(to_internal_error)
}

#[endpoint {
method = POST,
path = "/instance/{id}/stop",
}]
pub async fn stop_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let req = path_params.into_inner();
    let (_, stderr) =
        exec(&ctx, "vmadm", ["stop", &req.id.to_string()]).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(stderr.into())
        .map_err(to_internal_error)
}

#[endpoint {
method = POST,
path = "/instance/{id}/start",
}]
pub async fn start_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let req = path_params.into_inner();
    let (_, stderr) =
        exec(&ctx, "vmadm", ["start", &req.id.to_string()]).await?;
    Response::builder()
        .status(StatusCode::OK)
        .body(stderr.into())
        .map_err(to_internal_error)
}

#[endpoint {
method = GET,
path = "/info/{id}",
}]
pub async fn info_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let req = path_params.into_inner();

    let out = Command::new("vmadm")
        .args(["info", &req.id.to_string(), "vnc"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(to_internal_error)?
        .wait_with_output()
        .await
        .map_err(to_internal_error)?;

    if !out.status.success() {
        let stderr =
            String::from_utf8(out.stderr).map_err(to_internal_error)?;

        error!(
            ctx.log,
            "Exec failed vmadm {} info: {}",
            &req.id.to_string(),
            stderr
        );

        return Err(HttpError::for_bad_request(None, stderr));
    }

    let stdout = String::from_utf8(out.stdout).map_err(to_internal_error)?;

    Response::builder()
        .status(StatusCode::OK)
        .body(stdout.into())
        .map_err(to_internal_error)
}
