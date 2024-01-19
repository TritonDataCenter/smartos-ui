use std::process::Stdio;

use crate::endpoints::{Context, PathParams};

use dropshot::{endpoint, HttpError, Path, RequestContext};
use hyper::{Body, Response, StatusCode};
use tokio::process::Command;

#[endpoint {
method = GET,
path = "/image",
}]
pub async fn get_index(
    _: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
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
        .args(["get", &path.id])
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
