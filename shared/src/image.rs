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
use serde_json::Value;
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct ImageImportParams {
    pub url: Url,
    pub name: String,
    pub version: String,
    pub r#type: String,
    pub os: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    //pub v: u64, // put this back later as an example to handle malformed requests.
    pub uuid: Uuid,
    pub name: String,
    pub version: String,

    #[serde(default)]
    pub state: String, // Create enum

    #[serde(
        deserialize_with = "time::serde::iso8601::deserialize",
        serialize_with = "time::serde::iso8601::serialize"
    )]
    pub published_at: OffsetDateTime,
    pub r#type: String,
    pub os: String,
    pub description: String,
    pub homepage: Option<String>,
    pub requirements: Option<Value>,
    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ImportStatus {
    Importing,
    Failed(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub manifest: Manifest,

    pub source: Url,

    // field for internal use
    pub import_status: Option<ImportStatus>,
}

impl Image {
    pub fn is_for_hvm(&self) -> bool {
        self.manifest.r#type == "zvol"
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub url: Url,
    pub r#type: String,
}
