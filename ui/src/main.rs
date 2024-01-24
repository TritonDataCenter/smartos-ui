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
use smartos_ui::privilege::drop_privileges;
use smartos_ui::{endpoints, endpoints::Context};

use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, HttpServerStarter,
};

#[tokio::main]
async fn main() -> Result<(), String> {
    let name = option_env!("CARGO_PKG_NAME").unwrap_or("?");
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("v?");
    let config = Config::new(name);

    let request_body_max_bytes = config.request_body_max_bytes;
    let bind_address = config
        .ui_bind_address
        .parse()
        .expect("Failed to parse BIND_ADDRESS");
    let chroot = config.chroot.clone();

    let config_logging = ConfigLogging::File {
        level: dropshot::ConfigLoggingLevel::Debug,
        path: config.log_file.clone().into(),
        if_exists: dropshot::ConfigLoggingIfExists::Append,
    };

    let log = config_logging
        .to_logger(String::from("smartos_ui"))
        .map_err(|error| format!("Failed to create logger: {}", error))?;

    // Must occur before chroot
    let ctx = Context::new(config);

    drop_privileges(&log, &chroot);

    let mut api = ApiDescription::new();

    // /
    api.register(endpoints::get_index)?;

    // /login
    api.register(endpoints::login::get_index)?;
    api.register(endpoints::login::post_index)?;
    api.register(endpoints::login::get_logout)?;

    // /dashboard
    api.register(endpoints::dashboard::get_index)?;

    // /favicon.ico
    api.register(endpoints::assets::get_favicon)?;

    // /js/htmx.js
    api.register(endpoints::assets::get_js_main)?;

    // /css/main.css
    api.register(endpoints::assets::get_css_main)?;

    // /instances
    api.register(endpoints::instances::get_index)?;
    api.register(endpoints::instances::get_by_id)?;
    api.register(endpoints::instances::get_create)?;
    api.register(endpoints::instances::post_create)?;
    api.register(endpoints::instances::delete_by_id)?;

    // /images
    api.register(endpoints::images::get_index)?;
    //api.register(endpoints::images::get_by_id)?;

    info!(log, "{} v{}", name, version);

    let server = HttpServerStarter::new(
        &ConfigDropshot {
            bind_address,
            request_body_max_bytes,
            tls: None,
        },
        api,
        ctx,
        &log,
    )
    .map_err(|error| format!("failed to start server: {}", error))?
    .start();

    server.await
}
