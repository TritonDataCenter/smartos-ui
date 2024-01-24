use std::process::Stdio;

use crate::endpoints::Context;

use smartos_shared::nictag::NicTag;

use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};
use tokio::process::Command;

#[endpoint {
method = GET,
path = "/nictag",
}]
pub async fn get_index(
    _: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let out = Command::new("nictagadm")
        .args(["list", "-p", "-d", ","])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("nictagadm command failed to start")
        .wait_with_output()
        .await
        .expect("nictagadm command failed to run");

    let mut tags: Vec<NicTag> = Vec::new();

    let stdout = String::from_utf8(out.stdout).unwrap();

    for line in stdout.lines() {
        let v: Vec<&str> = line.split(',').collect();
        tags.push(NicTag {
            name: String::from(v[0]),
            mac_address: String::from(v[1]),
            link: String::from(v[2]),
            r#type: String::from(v[3]),
        });
    }
    let body = serde_json::to_string(&tags).unwrap_or_default();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body.into())
        .unwrap())
}
