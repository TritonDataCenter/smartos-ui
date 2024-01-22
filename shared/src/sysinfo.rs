/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Sysinfo {
    #[serde(rename = "Live Image")]
    pub live_image: String,
    #[serde(rename = "CPU Count")]
    pub cpu_count: i64,
    #[serde(rename = "MiB of Memory")]
    pub mib_of_memory: String,
    #[serde(rename = "Zpool Size in GiB")]
    pub zpool_size_in_gib: i64,
}
