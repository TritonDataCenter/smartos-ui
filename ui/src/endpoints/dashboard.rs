/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::filters;
use crate::endpoints::{
    htmx_response, redirect_login, to_internal_error, Context,
};
use crate::session::Session;

use smartos_shared::sysinfo::Sysinfo;

use askama::Template;
use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response};

#[derive(Template)]
#[template(path = "dashboard.j2")]
pub struct DashboardTemplate<'a> {
    title: &'a str,
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
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let sysinfo =
        ctx.context().client.get_sysinfo().await.map_err(to_internal_error)?;

    let template = DashboardTemplate { title: "Dashboard", sysinfo };
    let result = template.render().map_err(to_internal_error)?;
    htmx_response(response, "/dashboard", result.into())
}
