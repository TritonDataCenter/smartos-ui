/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{to_internal_error, Context};

use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{body::Bytes, Body, Response};

#[endpoint {
method = GET,
path = "/favicon.ico"
}]
pub async fn get_favicon(
    _: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let bytes = Bytes::from_static(include_bytes!("../../assets/favicon.ico"));
    Response::builder()
        .header("Content-Type", "image/vnd.microsoft.icon")
        .body(Body::from(bytes))
        .map_err(to_internal_error)
}

#[endpoint {
method = GET,
path = "/js/main.js"
}]
pub async fn get_js_main(
    _: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let bytes = Bytes::from_static(include_bytes!("../../assets/main.js.gz"));
    Response::builder()
        .header("Content-Encoding", "gzip")
        .header("Content-Type", "application/javascript")
        .body(Body::from(bytes))
        .map_err(to_internal_error)
}

#[endpoint {
method = GET,
path = "/css/main.css"
}]
pub async fn get_css_main(
    _: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let bytes = Bytes::from_static(include_bytes!("../../assets/main.css.gz"));
    Response::builder()
        .header("Content-Encoding", "gzip")
        .header("Content-Type", "text/css")
        .body(Body::from(bytes))
        .map_err(to_internal_error)
}
