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
pub mod images;
pub mod instances;
pub mod login;

use std::collections::HashMap;
use std::fmt;
use std::fs::read_to_string;
use std::sync::{Arc, Mutex};

use crate::exec::Client;
use crate::session::{Session, UserSession};
use smartos_shared::config::Config;

use dropshot::{endpoint, HttpError, RequestContext};
use http::response::Builder;
use hyper::{Body, Response, StatusCode};
use pwhash::unix;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    // How are these supposed to be structured? Docs don't say, guessing a
    // Option<Vec<(String; 2)>> ?
    // /// Values to submit with the request
    // pub values: Option<String>,
    // /// Headers to submit with the request
    // pub headers: Option<String>,
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
            select: None,
        }
    }

    pub fn common(
        response: Builder,
        path: &str,
    ) -> http::Result<Response<Body>> {
        let location = Self {
            path: String::from(path),
            source: None,
            event: None,
            handler: None,
            target: Some(String::from("#main")),
            swap: None,
            select: Some(String::from("#content")),
        };
        response
            .header("HX-Location", location.to_string())
            .status(StatusCode::OK)
            .body(Body::empty())
    }
}

impl fmt::Display for HXLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
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
    pub root_password_hash: Option<String>,
}

impl Context {
    #[must_use]
    pub fn new(config: Config) -> Self {
        let map: HashMap<String, UserSession> = HashMap::new();
        let exec_bind_address = config.exec_bind_address.clone();
        let vminfo_bind_address = config.vminfo_bind_address.clone();
        let root_password_hash =
            Self::get_root_password_hash(&config.shadow_path);
        Self {
            config,
            client: Client::new(exec_bind_address, vminfo_bind_address),
            sessions: Arc::new(Mutex::new(map)),
            root_password_hash,
        }
    }

    /// This runs before chroot while /etc/shadow is still accessible
    pub fn get_root_password_hash(shadow_path: &String) -> Option<String> {
        if let Ok(shadow_contents) = read_to_string(shadow_path) {
            for line in shadow_contents.lines() {
                let v: Vec<&str> = line.split(':').collect();
                if v.len() < 2 {
                    continue;
                }
                if v[0] == "root" {
                    return Some(String::from(v[1]));
                }
            }
        }
        None
    }

    pub fn validate_password(&self, password: String) -> bool {
        if let Some(root_password_hash) = &self.root_password_hash {
            return unix::verify(password, root_password_hash);
        }
        false
    }
}

pub fn get_header(
    ctx: &RequestContext<Context>,
    header: &str,
) -> Option<String> {
    ctx.request
        .headers()
        .get(header)
        .map(|value| String::from(value.to_str().unwrap_or("")))
}

pub fn redirect_login(
    response: Builder,
    ctx: &RequestContext<Context>,
) -> http::Result<Response<Body>> {
    let is_htmx = get_header(ctx, "HX-Request").is_some();

    if is_htmx {
        return response
            .status(StatusCode::OK)
            .header("HX-Refresh", "true")
            .body(Body::empty());
    }

    response
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/login")
        .body(Body::empty())
}

/// Redirect / to either /dashboard (if user has a valid session) or /login
#[endpoint {
method = GET,
path = "/"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    if Session::is_valid(&ctx) {
        return Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header("Location", "/dashboard")
            .body(Body::empty())
            .unwrap());
    }
    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", "/login")
        .body(Body::empty())
        .unwrap())
}
