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

use smartos_executor::{endpoints, endpoints::Context};
use smartos_shared::config::Config;

use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, HttpServerStarter,
};

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
        .to_logger(String::from("smartos_executor"))
        .map_err(|error| format!("Failed to create logger: {}", error))?;

    let mut api = ApiDescription::new();
    api.register(endpoints::sysinfo::get_index)?;
    api.register(endpoints::pwhash::get_index)?;
    api.register(endpoints::image::get_index)?;
    api.register(endpoints::image::get_by_id)?;
    api.register(endpoints::image::delete_by_id)?;
    api.register(endpoints::image::get_source_index)?;
    api.register(endpoints::image::post_import_by_id)?;
    api.register(endpoints::image::get_avail)?;
    api.register(endpoints::instance::post_provision_index)?;
    api.register(endpoints::instance::post_validate_create)?;
    api.register(endpoints::instance::delete_by_id)?;
    api.register(endpoints::nictag::get_index)?;
    api.register(endpoints::get_ping)?;

    info!(log, "{} v{}", name, version);

    let server = HttpServerStarter::new(
        &ConfigDropshot {
            bind_address: config
                .exec_bind_address
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
