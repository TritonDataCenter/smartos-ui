/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{htmx_response, redirect_login, Context};
use crate::session::Session;

use smartos_shared::http_server::to_internal_error;

use askama::Template;
use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response};

#[derive(Template)]
#[template(path = "gz_config.j2")]
pub struct GZConfigTemplate<'a> {
    title: &'a str,
    config: Vec<(String, String)>,
}

#[endpoint {
method = GET,
path = "/config/gz"
}]
pub async fn get_gz_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let config = ctx
        .context()
        .executor
        .get_gz_config()
        .await
        .map_err(to_internal_error)?;

    let template = GZConfigTemplate { title: "Global Zone Config", config };
    let result = template.render().map_err(to_internal_error)?;
    htmx_response(response, "/config/gz", result.into())
}
