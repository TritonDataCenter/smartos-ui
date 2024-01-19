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
use smartos_shared::instance::Instance;

use dropshot::{endpoint, HttpError, Path, RequestContext};
use hyper::{Body, Response, StatusCode};
use tokio::process::Command;

#[endpoint {
method = GET,
path = "/instance",
}]
pub async fn get_index(
    _: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let out = Command::new("vmadm")
        .args([
            "list",
            "-p",
            "-o",
            "uuid,image_uuid,type,brand,ram,state,alias",
            "-s",
            "alias,image_uuid",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("vmadm command failed to start")
        .wait_with_output()
        .await
        .expect("vmadm command failed to run");

    let mut instances: Vec<Instance> = Vec::new();

    let stdout = String::from_utf8(out.stdout).unwrap();

    for line in stdout.lines() {
        let v: Vec<&str> = line.split(':').collect();
        instances.push(Instance {
            uuid: String::from(v[0]),
            image_uuid: String::from(v[1]),
            r#type: String::from(v[2]),
            brand: String::from(v[3]),
            ram: String::from(v[4]),
            state: String::from(v[5]),
            alias: String::from(v[6]),
        });
    }
    let body = serde_json::to_string(&instances).unwrap_or_default();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body.into())
        .unwrap())
}

#[endpoint {
method = GET,
path = "/instance/{id}",
}]
pub async fn get_by_id(
    _: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let path = path_params.into_inner();
    let out = Command::new("vmadm")
        .args(["get", &path.id])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("vmadm get failed to start")
        .wait_with_output()
        .await
        .expect("vmadm get failed to run");

    let stdout = String::from_utf8(out.stdout).unwrap_or_default();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(stdout.into())
        .unwrap())
}
