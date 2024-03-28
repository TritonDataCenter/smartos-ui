/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

pub mod assets;
pub mod config;
pub mod dashboard;
pub mod filters;
pub mod images;
pub mod instances;
pub mod login;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::clients::{ExecutorClient, VMInfodClient};
use crate::session::{self, Session};

use smartos_shared::{config::Config, http_server::to_internal_error};

use askama::Template;
use dropshot::{
    endpoint, http_response_see_other, http_response_temporary_redirect,
    HttpError, HttpResponseOk, HttpResponseSeeOther,
    HttpResponseTemporaryRedirect, RequestContext,
};
use http::response::Builder;
use hyper::{Body, Response, StatusCode};
use pwhash::unix;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::try_join;
use uuid::Uuid;

#[derive(Default, PartialEq)]
enum NotificationKind {
    Ok,
    #[default]
    Error,
}

impl TryFrom<&reqwest::Response> for NotificationKind {
    type Error = ();

    fn try_from(response: &reqwest::Response) -> Result<Self, ()> {
        if response.status().is_success() {
            Ok(Self::Ok)
        } else {
            Ok(Self::Error)
        }
    }
}

#[derive(Template)]
#[template(path = "notification.j2")]
pub struct NotificationTemplate {
    /// ID for the notification, must be unique within the DOM, request ID
    /// is usually the best choice
    id: String,
    kind: NotificationKind,
    subject: String,
    message: String,

    /// Parsed by: <https://htmx.org/api/#parseInterval>
    /// None == no timeout, notification must be closed manually
    timeout: Option<String>,

    /// If [Some], will load the specified path after showing the notification
    redirect: Option<String>,

    /// The path where the notification was created
    created_at: String,

    /// Arbitrary string for the front-end and e2e tests to use, usually a UUID
    /// but not necessarily
    entity_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathParams {
    id: Uuid,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct AsJson {
    #[serde(default)]
    pub json: Option<bool>,
}

/// Available to in each Dropshot endpoint, contains global config, and the
/// user sessions
pub struct Context {
    pub config: Config,
    pub sessions: Arc<Mutex<HashMap<String, Session>>>,
    pub executor: ExecutorClient,
    pub vminfod: VMInfodClient,
}

impl Context {
    #[must_use]
    pub fn new(config: Config) -> Self {
        let map: HashMap<String, Session> = HashMap::new();
        let exec_bind_address = config.exec_bind_address.clone();
        let vminfo_bind_address = config.vminfo_bind_address.clone();
        Self {
            config,
            executor: ExecutorClient::new(exec_bind_address),
            vminfod: VMInfodClient::new(vminfo_bind_address),
            sessions: Arc::new(Mutex::new(map)),
        }
    }

    pub async fn validate_password(
        &self,
        password: String,
    ) -> Result<bool, reqwest::Error> {
        let hash = self.executor.get_pwhash().await?;
        Ok(unix::verify(password, &hash))
    }
}

pub fn get_header(
    ctx: &RequestContext<Context>,
    header: &str,
) -> Option<String> {
    ctx.request
        .headers()
        .get(header)
        .map(|value| String::from(value.to_str().unwrap_or_default()))
}

pub fn redirect_login(
    response: Builder,
    ctx: &RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let is_htmx = get_header(ctx, "HX-Request").is_some();

    if is_htmx {
        return response
            .status(StatusCode::OK)
            .header("HX-Refresh", "true")
            .body(Body::empty())
            .map_err(to_internal_error);
    }

    response
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/login")
        .body(Body::empty())
        .map_err(to_internal_error)
}

pub fn htmx_response(
    response: Builder,
    location: &str,
    body: Body,
) -> Result<Response<Body>, HttpError> {
    response
        .status(StatusCode::OK)
        .header("HX-Push-Url", location)
        .header("Content-Type", "text/html")
        .body(body)
        .map_err(to_internal_error)
}

/// Redirect to HTTPS port if loaded from non-HTTPS port
#[endpoint {
method = GET,
path = "/"
}]
pub async fn get_tls_index(
    ctx: RequestContext<String>,
) -> Result<HttpResponseTemporaryRedirect, HttpError> {
    http_response_temporary_redirect(ctx.context().to_string())
}

#[endpoint {
method = GET,
path = "/login"
}]
pub async fn get_tls_login_index(
    ctx: RequestContext<String>,
) -> Result<HttpResponseTemporaryRedirect, HttpError> {
    http_response_temporary_redirect(ctx.context().to_string())
}

/// Redirect a request to "/" to either "/dashboard" (if user has a valid
/// session) or "/login" (if user doesn't have a valid session.)
#[endpoint {
method = GET,
path = "/"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<HttpResponseSeeOther, HttpError> {
    let location =
        if session::is_valid(&ctx) { "/dashboard" } else { "/login" };
    http_response_see_other(location.to_string())
}

#[derive(Serialize, JsonSchema)]
pub struct PingResponse {
    /// Result of pinging the executor HTTP server, [true] if a 200 response was
    /// received
    pub executor: bool,
    /// Result of pinging the vminfod HTTP server, [true] if a 200 response was
    /// received
    pub vminfod: bool,
}

#[endpoint {
method = GET,
path = "/ping"
}]
pub async fn get_ping(
    ctx: RequestContext<Context>,
) -> Result<HttpResponseOk<PingResponse>, HttpError> {
    let (vminfod, executor) =
        try_join!(ctx.context().vminfod.ping(), ctx.context().executor.ping())
            .map_err(to_internal_error)?;
    Ok(HttpResponseOk(PingResponse { executor, vminfod }))
}
