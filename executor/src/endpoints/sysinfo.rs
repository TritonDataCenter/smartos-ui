use std::process::Stdio;

use crate::endpoints::Context;

use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};
use tokio::process::Command;

#[endpoint {
method = GET,
path = "/sysinfo",
}]
pub async fn get_index(
    _: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let out = Command::new("sysinfo")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("sysinfo command failed to start")
        .wait_with_output()
        .await
        .expect("sysinfo command failed to run");

    let stdout = String::from_utf8(out.stdout).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(stdout.into())
        .unwrap())
}
