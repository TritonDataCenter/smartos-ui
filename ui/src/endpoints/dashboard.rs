/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{redirect_login, Context};
use crate::session::Session;
use smartos_shared::sysinfo::Sysinfo;

use askama::Template;
use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};

#[derive(Template)]
#[template(path = "dashboard.j2")]
pub struct DashboardTemplate {
    title: String,
    sysinfo: Sysinfo,
}

#[endpoint {
method = GET,
path = "/dashboard"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if Session::get_login(&ctx).is_some() {
        let title = String::from("Dashboard");
        let sysinfo = ctx.context().client.get_sysinfo().await.unwrap();

        let template = DashboardTemplate { title, sysinfo };
        let result = template.render().unwrap();

        return Ok(response
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .header("HX-Push-Url", String::from("/dashboard"))
            .body(result.into())?);
    }

    Ok(redirect_login(response, &ctx)?)
}
