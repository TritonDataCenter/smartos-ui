/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use smartos_shared::config::Config;

use schemars::JsonSchema;
use serde::Deserialize;

pub mod image;
pub mod instance;
pub mod sysinfo;

pub struct Context {
    pub config: Config,
}

impl Context {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathParams {
    id: String,
}