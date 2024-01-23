/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::env;

pub struct Config {
    pub log_file: String,
    pub ui_bind_address: String,
    pub exec_bind_address: String,
    pub vminfo_bind_address: String,
    pub request_body_max_bytes: usize,
    pub chroot: String,
}

impl Config {
    #[must_use]
    pub fn new(log_file: &str) -> Self {
        Self {
            log_file: env::var("LOG_FILE")
                .unwrap_or(format!("/var/log/{}.log", log_file)),
            ui_bind_address: env::var("UI_BIND_ADDRESS")
                .unwrap_or_else(|_| String::from("127.0.0.1:8080")),
            exec_bind_address: env::var("EXEC_BIND_ADDRESS")
                .unwrap_or_else(|_| String::from("127.0.0.1:8081")),
            vminfo_bind_address: env::var("VMINFO_BIND_ADDRESS")
                .unwrap_or_else(|_| String::from("127.0.0.1:9090")),
            chroot: env::var("CHROOT")
                .unwrap_or_else(|_| String::from("/opt/smartos_ui")),
            request_body_max_bytes: 1024 * 1024 * 8,
        }
    }
}
