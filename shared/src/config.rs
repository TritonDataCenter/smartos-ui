/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::env;

const REQ_MAX_BYTES: usize = 1024 * 1024 * 8;

#[derive(Debug)]
pub struct Config {
    pub log_file: String,
    pub ui_bind_address: String,
    pub exec_bind_address: String,
    pub vminfo_bind_address: String,
    pub request_body_max_bytes: usize,
    pub chroot: String,
    pub shadow_path: String,
    pub login_user: String,
    pub exec_cache_seconds: i64,
    pub skip_privilege_drop: bool,
    pub cache_dir: String,
}

impl Config {
    #[must_use]
    pub fn new(log_file: &str) -> Self {
        let skip_privilege_drop =
            if let Ok(priv_drop) = env::var("SKIP_PRIVILEGE_DROP") {
                println!(
                    "SKIP_PRIVILEGE_DROP: {} ({})",
                    priv_drop,
                    priv_drop.len()
                );
                !priv_drop.is_empty()
            } else {
                false
            };

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
            shadow_path: env::var("SHADOW_PATH")
                .unwrap_or_else(|_| String::from("/etc/shadow")),
            login_user: env::var("LOGIN_USER")
                .unwrap_or_else(|_| String::from("root")),
            request_body_max_bytes: env::var("REQ_MAX_BYTES")
                .unwrap_or_else(|_| REQ_MAX_BYTES.to_string())
                .parse()
                .unwrap_or(REQ_MAX_BYTES),
            exec_cache_seconds: env::var("EXEC_CACHE")
                .unwrap_or_else(|_| String::from("300"))
                .parse()
                .unwrap_or(300),
            skip_privilege_drop,
            cache_dir: env::var("CACHE_DIR")
                .unwrap_or_else(|_| String::from("/tmp/smartos_ui")),
        }
    }
}
