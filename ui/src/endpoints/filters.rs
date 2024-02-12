/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use askama::Result;
use std::fmt::Display;

/// Askama filter for properly capitalizing and formatting some names
pub fn format_name<S: Display>(s: S) -> Result<String> {
    let name = format!("{}", s);
    let found = match name.as_str() {
        "linux" => String::from("Linux"),
        "smartos" => String::from("SmartOS"),
        "zvol" => String::from("ZFS Volume"),
        "lx-dataset" => String::from("LX Dataset"),
        "zone-dataset" => String::from("Zone Dataset"),
        _ => String::new(),
    };
    if found.is_empty() {
        return Ok(name);
    }
    Ok(found)
}

/// Convert MiB to GiB
pub fn mib_to_gib(mib: &u64) -> Result<String> {
    let value = *mib as f64 / 1024.0;
    if value.round() == value {
        return Ok(format!("{}", value));
    }
    Ok(format!("{:.2}", value))
}
