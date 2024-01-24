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

use askama::Template;
use dropshot::{endpoint, HttpError, RequestContext, TypedBody};
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

// Destroy the in-memory session (if exists) and set the Max-Age of the
// cookie (if any) to 0
#[endpoint {
method = GET,
path = "/logout"
}]
pub async fn get_logout(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    if let Some(id) = Session::destroy(&ctx) {
        return Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header("Location", "/login")
            .header("Set-Cookie", format!("{}; Max-Age=0; HttpOnly", id))
            .body(Body::empty())
            .unwrap());
    }
    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", "/login")
        .body(Body::empty())
        .unwrap())
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
    let req = body_param.into_inner();
    let response = Response::builder();
    let authed = ctx.context().validate_password(req.password);
    if req.user == "root" && authed {
        if let Some(id) = Session::create(&ctx, req.user) {
            return Ok(response
                .status(StatusCode::SEE_OTHER)
                .header("Set-Cookie", format!("sid={}; HttpOnly", &id))
                .header("Location", "/dashboard")
                .body(Body::empty())?);
        }
    }

    let login = LoginTemplate {
        message: Some("Invalid username or password"),
    };
    let result = login.render().unwrap(); // XXX
    Ok(response.status(StatusCode::FORBIDDEN).body(result.into())?)
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
    let result = login.render().unwrap(); // XXX
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(result.into())?)
}
