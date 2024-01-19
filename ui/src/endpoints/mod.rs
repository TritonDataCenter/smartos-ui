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
pub mod login;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::exec::ExecClient;
use crate::session::{Session, UserSession};
use smartos_shared::config::Config;

use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};

/// Available to in each Dropshot endpoint, contains global config, and the
/// user sessions
pub struct Context {
    pub config: Config,
    pub sessions: Arc<Mutex<HashMap<String, UserSession>>>,
    pub client: ExecClient,
}

impl Context {
    #[must_use]
    pub fn new(config: Config) -> Self {
        let map: HashMap<String, UserSession> = HashMap::new();
        let exec_bind_address = config.exec_bind_address.clone();
        Self {
            config,
            client: ExecClient::new(exec_bind_address),
            sessions: Arc::new(Mutex::new(map)),
        }
    }
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
