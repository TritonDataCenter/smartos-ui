/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Sysinfo {
    #[serde(rename = "Live Image", default)]
    pub live_image: String,
    #[serde(rename = "CPU Count", default)]
    pub cpu_count: u64,
    #[serde(
        rename = "MiB of Memory",
        deserialize_with = "crate::serde_helpers::deserialize_into_u64",
        default
    )]
    pub mib_of_memory: u64,
    #[serde(rename = "Zpool Size in GiB", default)]
    pub zpool_size_in_gib: u64,
    #[serde(rename = "Boot Parameters")]
    pub boot_parameters: BootParameters,
}

#[derive(Deserialize)]
pub struct BootParameters {
    #[serde(default)]
    pub root_shadow: String,
}
