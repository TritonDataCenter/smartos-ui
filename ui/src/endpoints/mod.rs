/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

pub mod assets;
pub mod dashboard;
pub mod filters;
pub mod images;
pub mod instances;
pub mod login;

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::exec::{Client, PingResponse};
use crate::session::{Session, UserSession};
use smartos_shared::config::Config;

use dropshot::{
    endpoint, http_response_see_other, HttpError, HttpResponseOk,
    HttpResponseSeeOther, RequestContext,
};
use http::response::Builder;
use hyper::{Body, Response, StatusCode};
use pwhash::unix;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub fn to_internal_error<T: fmt::Display>(e: T) -> HttpError {
    HttpError::for_internal_error(e.to_string())
}

/// <https://htmx.org/headers/hx-location>
#[derive(Debug, Deserialize, Serialize)]
pub struct HXLocation {
    /// url to load the response from
    pub path: String,

    /// The source element of the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// An event that “triggered” the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,

    /// A callback that will handle the response HTML
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler: Option<String>,

    /// The target to swap the response into
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// How the response will be swapped in relative to the target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<String>,

    /// Values to submit with the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Value>,

    // /// Headers to submit with the request
    // pub headers: Option<String>,
    //
    /// Allows you to select the content you want swapped from a response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<String>,
}

impl HXLocation {
    pub fn new(path: &str) -> HXLocation {
        Self {
            path: String::from(path),
            source: None,
            event: None,
            handler: None,
            target: None,
            swap: None,
            values: None,
            select: None,
        }
    }

    pub fn new_with_common(path: &str) -> Self {
        Self {
            path: String::from(path),
            source: None,
            event: None,
            handler: None,
            target: Some(String::from("#main")),
            swap: None,
            values: None,
            select: Some(String::from("#content")),
        }
    }

    pub fn serve(
        &self,
        response: Builder,
    ) -> Result<Response<Body>, HttpError> {
        let header = serde_json::to_string(&self).map_err(to_internal_error)?;
        response
            .header("HX-Location", header)
            .status(StatusCode::OK)
            .body(Body::empty())
            .map_err(to_internal_error)
    }

    pub fn common(
        response: Builder,
        path: &str,
    ) -> Result<Response<Body>, HttpError> {
        let location = Self::new_with_common(path);
        response
            .header("HX-Location", location.to_string())
            .status(StatusCode::OK)
            .body(Body::empty())
            .map_err(to_internal_error)
    }
}

impl fmt::Display for HXLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap_or_default())
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathParams {
    id: Uuid,
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
