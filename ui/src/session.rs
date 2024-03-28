/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

//! Sessions are stored in-memory in a
//! [std::collections::HashMap<String, Session>] where [String] is the Session
//! ID and [Session] is a struct with details about the Session's owner and
//! expiration.
//!
//! The Session ID is a randomly generated and sent to the user agent as a
//! Cookie.

use crate::endpoints::Context;

use smartos_shared::http_server::to_internal_error;

use dropshot::{
    http_response_temporary_redirect, HttpError, HttpResponseTemporaryRedirect,
    RequestContext,
};
use http::{HeaderValue, Response, StatusCode};
use hyper::Body;
use nanoid::nanoid;
use serde::Serialize;
use time::{format_description::well_known::Rfc2822, Duration, OffsetDateTime};

/// Name for cookie. Using the "__Host-" provides some additional assurance that
/// the cookie has been set with the values provided in COOKIE_ATTRS
/// https://datatracker.ietf.org/doc/html/draft-ietf-httpbis-cookie-prefixes-00#section-3.2
const COOKIE_NAME: &str = "__Host-ID";

/// Number of random characters to use for generating a Session ID
const SESSION_ID_LENGTH: usize = 64;

/// Static attributes for the session cookie:
/// HttpOnly - Do not allow access to the Cookie from Javascript
/// SameSite=Strict - Do not send cookie to a different domain or URI scheme
/// Secure - Only allow from https://
/// Path=/ - Required for using __Host- cookie prefix
const COOKIE_ATTRS: &str = "HttpOnly; SameSite=Strict; Secure; Path=/";

/// How many hours a session/cookie is valid
const SESSION_HOURS: u8 = 1;

#[derive(Serialize)]
/// Holds active Session information that can be access in the [RequestContext]
pub struct Session {
    /// Name of logged in user
    pub login: String,
    /// DateTime Session is no longer valid
    pub expires: OffsetDateTime,
}

/// Extract the Session ID from the `Cookie` header (if any)
fn get_id(ctx: &RequestContext<Context>) -> Option<&str> {
    let mut id = None;
    if let Some(cookie) = ctx.request.headers().get("Cookie") {
        if let Ok(session_id) = cookie.to_str() {
            id = Some(session_id);
        }
    }
    id
}

/// Confirms the validity of a [Session] using the Session ID provided in
/// the `Cookie` header of the current request.
pub fn is_valid(ctx: &RequestContext<Context>) -> bool {
    if let Some(id) = get_id(ctx) {
        if let Ok(mut sessions) = ctx.context().sessions.clone().lock() {
            if let Some(session) = sessions.get(id) {
                if OffsetDateTime::now_utc() < session.expires {
                    return true;
                } else {
                    sessions.remove(id);
                }
            }
        }
    }
    false
}

/// Create a new [Session] and send back the appropriate `Set-Cookie` header
/// in the response.
pub fn create(
    ctx: &RequestContext<Context>,
    username: String,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if let Ok(mut sessions) = ctx.context().sessions.clone().lock() {
        let session_id = nanoid!(SESSION_ID_LENGTH);
        let expires =
            OffsetDateTime::now_utc() + Duration::HOUR * SESSION_HOURS;

        sessions.insert(
            format!("{}={}", COOKIE_NAME, &session_id),
            Session { login: username.clone(), expires },
        );
        let expires_formatted =
            expires.format(&Rfc2822).map_err(to_internal_error)?;

        return Ok(response
            .status(StatusCode::SEE_OTHER)
            .header(
                "Set-Cookie",
                format!(
                    "{}={}; {}; Expires={}",
                    COOKIE_NAME,
                    &session_id,
                    COOKIE_ATTRS,
                    expires_formatted
                ),
            )
            .header("Location", "/dashboard")
            .body(Body::empty())?);
    }
    Ok(response
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Location", "/login")
        .body(Body::empty())?)
}

/// Remove the [Session] from the in-memory [std::collections::HashMap] of
/// Sessions and send a `Set-Cookie` header to the client expiring the cookie.
pub fn destroy(
    ctx: &RequestContext<Context>,
) -> Result<HttpResponseTemporaryRedirect, HttpError> {
    let login_path = String::from("/login");
    let mut response = http_response_temporary_redirect(login_path)?;
    if let Some(session_id) = get_id(ctx) {
        if let Ok(mut sessions) = ctx.context().sessions.clone().lock() {
            if sessions.remove(session_id).is_some() {
                let headers = response.headers_mut();
                let cookie = HeaderValue::from_str(&format!(
                    "{}; {}; Max-Age=0;",
                    session_id, COOKIE_ATTRS
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
