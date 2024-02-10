/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct Sysinfo {
    #[serde(rename = "Live Image")]
    pub live_image: String,
    #[serde(rename = "CPU Count")]
    pub cpu_count: u64,
    #[serde(rename = "MiB of Memory", deserialize_with = "string_to_u64")]
    pub mib_of_memory: u64,
    #[serde(rename = "Zpool Size in GiB")]
    pub zpool_size_in_gib: u64,
    #[serde(rename = "Boot Parameters")]
    pub boot_parameters: BootParameters,
}

#[derive(Deserialize)]
pub struct BootParameters {
    pub root_shadow: String,
}

/// A Serde Deserializer to convert a string to u64
fn string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let result: u64 = String::deserialize(deserializer)?
        .as_str()
        .parse()
        .map_err(serde::de::Error::custom)?;
    Ok(result)
}
