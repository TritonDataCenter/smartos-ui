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

use std::env;
use std::fs;

use smartos_shared::config::Config;

use smartos_ui::{
    endpoints, endpoints::Context, privilege::drop_privileges,
    GIT_COMMIT_SHORT, VERSION,
};

use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, ConfigTls, HandlerTaskMode,
    HttpServerStarter,
};
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), String> {
    let name = option_env!("CARGO_PKG_NAME").unwrap_or("?");
    let full_version = format!("{}-{}", VERSION, GIT_COMMIT_SHORT);

    // If provided with a single argument of "version", print version and exit.
    let mut args = env::args();
    if args.len() > 1 && args.nth(1).is_some_and(|arg| arg == "version") {
        println!("{}", full_version);
        return Ok(());
    }

    let config = Config::new(name);

    let request_body_max_bytes = config.request_body_max_bytes;
    let bind_https_address = config
        .ui_bind_https_address
        .parse()
        .expect("Failed to parse UI_BIND_HTTPS_ADDRESS");
    let bind_http_address = config
        .ui_bind_http_address
        .parse()
        .expect("Failed to parse UI_BIND_HTTP_ADDRESS");
    let chroot = config.chroot.clone();
    let skip_privilege_drop = config.skip_privilege_drop;

    let config_tls = Some(ConfigTls::AsBytes {
        certs: fs::read(&config.cert_file).unwrap_or_else(|_| {
            panic!("Failed reading TLS certificate at {}", config.key_file)
        }),
        key: fs::read(&config.key_file).unwrap_or_else(|_| {
            panic!("Failed reading TLS key at {}", config.key_file)
        }),
    });

    let config_logging = ConfigLogging::File {
        level: dropshot::ConfigLoggingLevel::Debug,
        path: config.log_file.clone().into(),
        if_exists: dropshot::ConfigLoggingIfExists::Append,
    };

    let log = config_logging
        .to_logger(name)
        .map_err(|error| format!("Failed to create logger: {}", error))?;

    debug!(log, "{} CONFIG: {:#?}", name, &config);

    let ctx = Context::new(config);

    if skip_privilege_drop {
        info!(log, "SKIP_PRIVILEGE_DROP set, not dropping privileges")
    } else {
        drop_privileges(&log, &chroot);
    }

    let mut api = ApiDescription::new();

    // /
    api.register(endpoints::get_index)?;

    // /login
    api.register(endpoints::login::get_index)?;
    api.register(endpoints::login::post_index)?;
    api.register(endpoints::login::get_logout)?;

    // /ping
    api.register(endpoints::get_ping)?;

    // /dashboard
    api.register(endpoints::dashboard::get_index)?;

    // /favicon.ico
    api.register(endpoints::assets::get_favicon)?;

    // /js/main.js
    api.register(endpoints::assets::get_js_main)?;

    // /css/main.css
    api.register(endpoints::assets::get_css_main)?;

    // /instances
    api.register(endpoints::instances::get_index)?;
    api.register(endpoints::instances::get_by_id)?;
    api.register(endpoints::instances::delete_by_id)?;
    api.register(endpoints::instances::stop_by_id)?;
    api.register(endpoints::instances::start_by_id)?;

    // /provision
    api.register(endpoints::instances::get_provision)?;
    api.register(endpoints::instances::post_provision)?;
    api.register(endpoints::instances::post_provision_validate)?;

    // /images
    api.register(endpoints::images::get_index)?;
    api.register(endpoints::images::get_by_id)?;
    api.register(endpoints::images::delete_by_id)?;

    // / import
    api.register(endpoints::images::get_import_index)?;
    api.register(endpoints::images::post_import_index)?;

    // /config
    api.register(endpoints::config::get_gz_index)?;

    info!(log, "{} v{}", name, full_version);

    let https_server = HttpServerStarter::new_with_tls(
        &ConfigDropshot {
            bind_address: bind_https_address,
            request_body_max_bytes,
            default_handler_task_mode: HandlerTaskMode::CancelOnDisconnect,
        },
        api,
        ctx,
        &log,
        config_tls,
    )
    .map_err(|error| format!("failed to start https server: {}", error))?
    .start();

    let mut redir = ApiDescription::new();

    // redirect / and /login requests on http to https
    redir.register(endpoints::get_tls_index)?;
    redir.register(endpoints::get_tls_login_index)?;

    let http_server = HttpServerStarter::new(
        &ConfigDropshot {
            bind_address: bind_http_address,
            request_body_max_bytes,
            default_handler_task_mode: HandlerTaskMode::CancelOnDisconnect,
        },
        redir,
        format!("https://{}", &bind_https_address),
        &log,
    )
    .map_err(|error| format!("failed to start http server: {}", error))?
    .start();

    try_join!(https_server, http_server)?;
    Ok(())
}
