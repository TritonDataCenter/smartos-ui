/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{exec_and_cache, Context};

use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response};

#[endpoint {
method = GET,
path = "/sysinfo",
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    exec_and_cache(ctx, "sysinfo", []).await
}
