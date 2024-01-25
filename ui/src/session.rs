/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::Context;
use dropshot::{
    http_response_temporary_redirect, HttpError, HttpResponseTemporaryRedirect,
    RequestContext,
};
use http::{HeaderValue, Response, StatusCode};
use hyper::Body;
use nanoid::nanoid;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct UserSession {
    pub login: String,
    // TODO: Check expiration on access
    // pub expires: OffsetDateTime,

    // TODO: Generate and check on POST, PUT, DELETE
    // pub csrf_token: String,
}

pub struct Session {}

impl Session {
    fn get_id(ctx: &RequestContext<Context>) -> Option<&str> {
        let mut id = None;
        if let Some(cookie) = ctx.request.headers().get("Cookie") {
            if let Ok(session_id) = cookie.to_str() {
                id = Some(session_id);
            }
        }
        id
    }

    pub fn is_valid(ctx: &RequestContext<Context>) -> bool {
        if let Some(id) = Session::get_id(ctx) {
            if let Ok(sessions) = ctx.context().sessions.clone().lock() {
                // Look up the UserSession using the session id
                if sessions.get(id).is_some() {
                    // TODO: check the session expiration
                    return true;
                }
            }
        }
        false
    }

    pub fn create(
        ctx: &RequestContext<Context>,
        username: String,
    ) -> Result<Response<Body>, HttpError> {
        let response = Response::builder();
        if let Ok(mut sessions) = ctx.context().sessions.clone().lock() {
            let session_id = nanoid!(64);
            sessions.insert(
                format!("sid={}", &session_id),
                UserSession {
                    login: username.clone(),
                },
            );
            let response = Response::builder();
            return Ok(response
                .status(StatusCode::SEE_OTHER)
                .header("Set-Cookie", format!("sid={}; HttpOnly", &session_id))
                .header("Location", "/dashboard")
                .body(Body::empty())?);
        }
        Ok(response
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Location", "/login")
            .body(Body::empty())?)
    }

    pub fn destroy(
        ctx: &RequestContext<Context>,
    ) -> Result<HttpResponseTemporaryRedirect, HttpError> {
        let login_path = String::from("/login");
        let mut response = http_response_temporary_redirect(login_path)?;
        if let Some(session_id) = Session::get_id(ctx) {
            if let Ok(mut sessions) = ctx.context().sessions.clone().lock() {
                if sessions.remove(session_id).is_some() {
                    let headers = response.headers_mut();
                    let cookie = HeaderValue::from_str(&format!(
                        "{}; Max-Age=0; HttpOnly",
                        session_id
                    ))
                    .map_err(|e| {
                        HttpError::for_bad_request(None, e.to_string())
                    })?;
                    headers.insert("Set-Cookie", cookie);
                }
            }
        }
        Ok(response)
    }

    pub fn get_login(ctx: &RequestContext<Context>) -> Option<String> {
        if let Some(session_id) = Session::get_id(ctx) {
            if let Ok(sessions) = ctx.context().sessions.clone().lock() {
                if let Some(session) = sessions.get(session_id) {
                    return Some(session.login.clone());
                }
            }
        }
        None
    }
}
