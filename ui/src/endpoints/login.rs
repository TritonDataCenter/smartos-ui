/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::{endpoints::Context, session};

use smartos_shared::http_server::to_internal_error;

use askama::Template;
use dropshot::{
    endpoint, HttpError, HttpResponseTemporaryRedirect, RequestContext,
    TypedBody,
};
use hyper::{Body, Response, StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::try_join;

#[derive(Template)]
#[template(path = "login.j2")]
struct LoginTemplate<'a> {
    message: Option<&'a str>,
    executor: bool,
    vminfod: bool,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct LoginRequestBody {
    pub user: String,
    pub password: String,
}

#[endpoint {
method = GET,
path = "/logout"
}]
pub async fn get_logout(
    ctx: RequestContext<Context>,
) -> Result<HttpResponseTemporaryRedirect, HttpError> {
    session::destroy(&ctx)
}

/// Authenticates POSTed user/pass, created a session and redirects to /dashboard
#[endpoint {
method = POST,
path = "/login",
content_type = "application/x-www-form-urlencoded"
}]
pub async fn post_index(
    ctx: RequestContext<Context>,
    body_param: TypedBody<LoginRequestBody>,
) -> Result<Response<Body>, HttpError> {
    let LoginRequestBody { user, password } = body_param.into_inner();
    let authed = ctx
        .context()
        .validate_password(password)
        .await
        .map_err(to_internal_error)?;
    if user == ctx.context().config.login_user && authed {
        try_join!(
            ctx.context().vminfod.get_instances(),
            ctx.context().executor.get_images(&ctx.log),
        )
        .map_err(to_internal_error)?;
        return session::create(&ctx, user);
    }
    let login = LoginTemplate {
        message: Some("Invalid username or password"),
        executor: true,
        vminfod: true,
    };
    let result = login.render().map_err(to_internal_error)?;
    Ok(Response::builder().status(StatusCode::FORBIDDEN).body(result.into())?)
}

/// Presents user with a login form
#[endpoint {
method = GET,
path = "/login"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let (vminfod, executor) =
        try_join!(ctx.context().vminfod.ping(), ctx.context().executor.ping())
            .map_err(to_internal_error)?;

    let login = LoginTemplate { message: None, executor, vminfod };

    let result = login.render().map_err(to_internal_error)?;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(result.into())
        .map_err(to_internal_error)
}
