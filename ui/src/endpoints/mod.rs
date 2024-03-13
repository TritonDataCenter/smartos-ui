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

use crate::exec::{Client, PingResponse};
use crate::session::{Session, UserSession};

use smartos_shared::{config::Config, http_server::to_internal_error};

use askama::Template;
use dropshot::{
    endpoint, http_response_see_other, HttpError, HttpResponseOk,
    HttpResponseSeeOther, RequestContext,
};
use http::response::Builder;
use hyper::{Body, Response, StatusCode};
use pwhash::unix;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Default)]
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

    /// Parsed by: https://htmx.org/api/#parseInterval
    /// None == no timeout, must be closed manually
    timeout: Option<String>,

    /// If Some, will load the specified path after showing the notification
    redirect: Option<String>,

    /// The path where the notification was created.
    created_at: String,

    /// Arbitrary string for the front-end and e2e tests to use, usually a UUI
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
    pub sessions: Arc<Mutex<HashMap<String, UserSession>>>,
    pub client: Client,
}

impl Context {
    #[must_use]
    pub fn new(config: Config) -> Self {
        let map: HashMap<String, UserSession> = HashMap::new();
        let exec_bind_address = config.exec_bind_address.clone();
        let vminfo_bind_address = config.vminfo_bind_address.clone();
        Self {
            config,
            client: Client::new(exec_bind_address, vminfo_bind_address),
            sessions: Arc::new(Mutex::new(map)),
        }
    }

    pub async fn validate_password(
        &self,
        password: String,
    ) -> Result<bool, reqwest::Error> {
        let hash = self.client.get_pwhash().await?;
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

/// Redirect / to either /dashboard (if user has a valid session) or /login
#[endpoint {
method = GET,
path = "/"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<HttpResponseSeeOther, HttpError> {
    let location =
        if Session::is_valid(&ctx) { "/dashboard" } else { "/login" };
    http_response_see_other(location.to_string())
}

#[endpoint {
method = GET,
path = "/ping"
}]
pub async fn get_ping(
    ctx: RequestContext<Context>,
) -> Result<HttpResponseOk<PingResponse>, HttpError> {
    let response =
        ctx.context().client.ping().await.map_err(to_internal_error)?;
    Ok(HttpResponseOk(response))
}
