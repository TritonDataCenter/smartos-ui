/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub v: i64,
    pub uuid: String,
    pub name: String,
    pub version: String,
    pub state: String,
    pub disabled: bool,
    pub public: bool,
    pub published_at: String,
    pub r#type: String,
    pub os: String,
    pub description: String,
    pub homepage: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub manifest: Manifest,
}
