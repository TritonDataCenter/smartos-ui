/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NicTag {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub mac_address: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub r#type: String,
}
