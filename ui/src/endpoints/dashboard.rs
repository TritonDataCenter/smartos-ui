/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::Context;
use crate::session::Session;
use smartos_shared::instance::Instance;

use askama::Template;
use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};

#[derive(Template)]
#[template(path = "dashboard.j2")]
pub struct DashboardTemplate {
    login: String,
    instances: Vec<Instance>,
}

#[endpoint {
method = GET,
path = "/dashboard"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    if let Some(login) = Session::get_login(&ctx) {
        let instances = ctx.context().client.get_instances().await.unwrap();
        let dashboard = DashboardTemplate { login, instances };
        let result = dashboard.render().unwrap(); // XXX
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(result.into())
            .unwrap());
    }
    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", "/login")
        .body(Body::empty())
        .unwrap())
}
