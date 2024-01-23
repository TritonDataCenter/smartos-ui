/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::process::Stdio;

use crate::endpoints::{Context, PathParams};
use smartos_shared::instance::CreatePayload;

use dropshot::{endpoint, HttpError, Path, RequestContext, TypedBody};
use hyper::{Body, Response, StatusCode};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[endpoint {
method = POST,
path = "/instance",
}]
pub async fn post_index(
    _: RequestContext<Context>,
    request_body: TypedBody<CreatePayload>,
) -> Result<Response<Body>, HttpError> {
    let req = request_body.into_inner();
    println!("req: {:#?}", req);

    let payload = serde_json::to_string(&req).expect("failed");

    println!("payload string: {}", payload);

    let mut process = Command::new("vmadm")
        .args(["create"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("vmadm command failed to start");

    println!("spawned");

    let mut stdin = process.stdin.take().expect("Failed to open stdin");

    println!("stdin.take");

    stdin
        .write_all(payload.as_bytes())
        .await
        .expect("Failed to write to stdin");
    println!("write all bytes");

    drop(stdin);

    let out = process
        .wait_with_output()
        .await
        .expect("Failed to read stdout");

    let stdout = String::from_utf8(out.stdout).unwrap_or_default();
    println!("stdout: {}", stdout);

    let stderr = String::from_utf8(out.stderr).unwrap_or_default();
    println!("stderr: {}", stderr);

    println!("status: {}", out.status);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap())
}

#[endpoint {
method = DELETE,
path = "/instance/{id}",
}]
pub async fn delete_by_id(
    _: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let req = path_params.into_inner();

    let _ = Command::new("vmadm")
        .args(["delete", &req.id.to_string()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("vmadm command failed to start");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}
