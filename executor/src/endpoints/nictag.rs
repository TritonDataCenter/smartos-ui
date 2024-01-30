/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{exec, Context};

use smartos_shared::nictag::NicTag;

use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};
use smartos_shared::http_server::to_internal_error;

#[endpoint {
method = GET,
path = "/nictag",
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let stdout = exec(&ctx, "nictagadm", ["list", "-p", "-d", ","]).await?;

    let mut tags: Vec<NicTag> = Vec::new();

    for line in stdout.lines() {
        let v: Vec<&str> = line.split(',').collect();
        if v.len() < 4 {
            return Err(to_internal_error("nictagadm sent unexpected output"));
        }
        tags.push(NicTag {
            name: String::from(v[0]),
            mac_address: String::from(v[1]),
            link: String::from(v[2]),
            r#type: String::from(v[3]),
        });
    }

    let body = serde_json::to_string(&tags).map_err(to_internal_error)?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body.into())
        .map_err(to_internal_error)
}
