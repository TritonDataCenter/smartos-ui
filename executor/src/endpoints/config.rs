/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::Context;

use smartos_shared::http_server::to_internal_error;

use std::fs::read_to_string;

use dropshot::{endpoint, HttpError, HttpResponseOk, RequestContext};

type ConfigEntries = Vec<(String, String)>;

#[endpoint {
method = GET,
path = "/config/gz",
}]
pub async fn get_gz_index(
    ctx: RequestContext<Context>,
) -> Result<HttpResponseOk<ConfigEntries>, HttpError> {
    let gz_config_path = &ctx.context().config.gz_config_path;
    let config = read_to_string(gz_config_path).map_err(to_internal_error)?;
    let mut entries: ConfigEntries = Vec::new();

    for line in config.lines() {
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(' ')
            || !line.contains('=')
        {
            continue;
        }

        let v: Vec<&str> = line.split('=').collect();
        if v.len() != 2 {
            continue;
        }
        entries.push((v[0].to_string(), v[1].to_string()));
    }
    Ok(HttpResponseOk(entries))
}
