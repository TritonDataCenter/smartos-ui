/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{to_internal_error, Context};
use crate::session::Session;

use askama::Template;
use dropshot::{
    endpoint, HttpError, HttpResponseTemporaryRedirect, RequestContext,
    TypedBody,
};
use hyper::{Body, Response, StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "login.j2")]
struct LoginTemplate<'a> {
    message: Option<&'a str>,
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
    Session::destroy(&ctx)
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
        ctx.context().client.warm_cache().await.map_err(to_internal_error)?;
        return Session::create(&ctx, user);
    }
    let login = LoginTemplate { message: Some("Invalid username or password") };
    let result = login.render().map_err(to_internal_error)?;
    Ok(Response::builder().status(StatusCode::FORBIDDEN).body(result.into())?)
}

/// Presents user with a login form
#[endpoint {
method = GET,
path = "/login"
}]
pub async fn get_index(
    _: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let login = LoginTemplate { message: None };
    let result = login.render().map_err(to_internal_error)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(result.into())?)
}
