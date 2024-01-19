/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

#[macro_use]
extern crate slog;

use smartos_shared::config::Config;
use smartos_ui::{endpoints, endpoints::Context};

use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, HttpServerStarter,
};

use privdrop::PrivDrop;

use libc::{getuid, uid_t};
pub fn get_current_uid() -> uid_t {
    unsafe { getuid() }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let name = option_env!("CARGO_PKG_NAME").unwrap_or("?");
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("v?");
    let config = Config::new(name);

    let config_logging = ConfigLogging::File {
        level: dropshot::ConfigLoggingLevel::Debug,
        path: config.log_file.clone().into(),
        if_exists: dropshot::ConfigLoggingIfExists::Append,
    };

    let log = config_logging
        .to_logger(String::from("smartos_ui"))
        .map_err(|error| format!("Failed to create logger: {}", error))?;

    // TODO: we'll likely just do this in SMF instead of here
    let uid = get_current_uid();
    if uid == 0 {
        info!(log, "Running as root uid: {}, dropping privileges", uid);
        PrivDrop::default()
            .chroot(&config.chroot)
            .user("nobody")
            .apply()
            .unwrap_or_else(|e| panic!("Failed to drop privileges: {}", e));
        info!(log, "New uid: {}", get_current_uid());
    } else {
        info!(
            log,
            "Running as non-root uid: {}, not dropping privileges", uid
        );
    }

    let mut api = ApiDescription::new();
    api.register(endpoints::get_index)?;
    api.register(endpoints::login::get_index)?;
    api.register(endpoints::login::post_index)?;
    api.register(endpoints::login::get_logout)?;
    api.register(endpoints::dashboard::get_index)?;
    api.register(endpoints::assets::get_favicon)?;

    info!(log, "{} v{}", name, version);

    let server = HttpServerStarter::new(
        &ConfigDropshot {
            bind_address: config
                .ui_bind_address
                .parse()
                .expect("Failed to parse BIND_ADDRESS"),
            request_body_max_bytes: config.request_body_max_bytes,
            tls: None,
        },
        api,
        Context::new(config),
        &log,
    )
    .map_err(|error| format!("failed to start server: {}", error))?
    .start();

    server.await
}
