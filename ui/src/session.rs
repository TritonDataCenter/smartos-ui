/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::Context;

use dropshot::RequestContext;
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
    ) -> Option<String> {
        // Acquire sessions hashmap from context
        if let Ok(mut sessions) = ctx.context().sessions.clone().lock() {
            // Generate random Session ID
            let id = nanoid!(64);
            sessions.insert(
                format!("sid={}", &id),
                UserSession {
                    login: username.clone(),
                },
            );
            return Some(id);
        }
        None
    }

    pub fn destroy(ctx: &RequestContext<Context>) -> Option<String> {
        let mut id = None;
        if let Some(session_id) = Session::get_id(ctx) {
            id = Some(String::from(session_id));
            if let Ok(mut sessions) = ctx.context().sessions.clone().lock() {
                if sessions.remove(session_id).is_some() {
                    return id;
                }
            }
        }
        id
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
