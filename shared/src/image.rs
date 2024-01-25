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
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct ImageImportParams {
    pub url: String, // sanitize
                     // pub channel: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub v: i64,
    pub uuid: Uuid,
    pub name: String,
    pub version: String,
    pub state: String,
    pub disabled: bool,
    pub public: bool,
    #[serde(deserialize_with = "time::serde::iso8601::deserialize")]
    pub published_at: OffsetDateTime,
    pub r#type: String,
    pub os: String,
    pub description: String,
    pub homepage: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub manifest: Manifest,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub url: String,
    pub r#type: String,
}
