/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::fs::read_to_string;

use crate::endpoints::Context;

use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};
use smartos_shared::http_server::to_internal_error;

#[endpoint {
method = GET,
path = "/pwhash",
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let shadow_path = &ctx.context().config.shadow_path;
    let mut pwhash = String::new();
    if let Ok(shadow_contents) = read_to_string(shadow_path) {
        for line in shadow_contents.lines() {
            let v: Vec<&str> = line.split(':').collect();
            if v.len() < 2 {
                continue;
            }
            if v[0] == "root" {
                pwhash = String::from(v[1]);
                break;
            }
        }
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(pwhash.into())
            .map_err(to_internal_error);
    }
    Err(to_internal_error("Failed to read password hash"))
}
