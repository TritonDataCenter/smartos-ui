/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use dropshot::HttpError;
use hyper::{Body, Response, StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GenericResponse {
    pub request_id: String,

    #[serde(default)]
    pub message: String,

    #[serde(default)]
    pub detail: String,
}

pub fn to_internal_error<T: std::fmt::Display>(e: T) -> HttpError {
    HttpError::for_internal_error(e.to_string())
}

pub fn to_bad_request<T: std::fmt::Display>(e: T) -> HttpError {
    HttpError::for_bad_request(None, e.to_string())
}

pub fn empty_ok() -> Result<Response<Body>, HttpError> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .map_err(to_internal_error)
}
