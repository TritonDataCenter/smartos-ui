/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::collections::HashMap;
use std::ffi::OsStr;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use smartos_shared::config::Config;
use time::{Duration, OffsetDateTime};

use dropshot::{HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};
use schemars::JsonSchema;
use serde::Deserialize;
use slog::{debug, error};
use smartos_shared::http_server::to_internal_error;
use smartos_shared::image::Image;
use tokio::process::Command;
use uuid::Uuid;

pub mod image;
pub mod instance;
pub mod nictag;
pub mod pwhash;
pub mod sysinfo;

#[derive(Debug)]
pub struct CacheEntry {
    pub expiry: OffsetDateTime,
    pub content: String,
}

pub struct Context {
    pub config: Config,
    pub cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    pub import_queue: Arc<Mutex<HashMap<Uuid, Image>>>,
}

impl Context {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            cache: Arc::new(Mutex::new(HashMap::<String, CacheEntry>::new())),
            import_queue: Arc::new(Mutex::new(HashMap::<Uuid, Image>::new())),
        }
    }

    pub fn get_cache<S: Into<String>>(&self, key: S) -> Option<String> {
        if let Ok(cache) = self.cache.clone().lock() {
            if let Some(entry) = cache.get(&key.into()) {
                if entry.expiry > OffsetDateTime::now_utc() {
                    return Some(entry.content.clone());
                }
            }
        }
        None
    }

    pub fn set_cache<S: Into<String>>(
        &self,
        key: S,
        content: String,
    ) -> Option<CacheEntry> {
        if let Ok(mut cache) = self.cache.clone().lock() {
            let cache_duration = self.config.exec_cache_seconds;
            let expiry =
                OffsetDateTime::now_utc() + Duration::new(cache_duration, 0);
            return cache.insert(key.into(), CacheEntry { expiry, content });
        }
        None
    }

    pub fn remove_cache<S: Into<String>>(&self, key: S) -> Option<CacheEntry> {
        if let Ok(mut cache) = self.cache.clone().lock() {
            return cache.remove(&key.into());
        }
        None
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathParams {
    id: Uuid,
}

pub async fn exec<I, S>(
    ctx: &RequestContext<Context>,
    cmd: S,
    args: I,
) -> Result<String, HttpError>
where
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr> + std::fmt::Display,
{
    let out = Command::new(&cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(to_internal_error)?
        .wait_with_output()
        .await
        .map_err(to_internal_error)?;

    if !out.status.success() {
        let stderr =
            String::from_utf8(out.stderr).map_err(to_internal_error)?;

        error!(ctx.log, "Exec failed {}: {}", cmd, stderr);

        return Err(HttpError::for_internal_error(stderr));
    }

    String::from_utf8(out.stdout).map_err(to_internal_error)
}

pub async fn exec_and_cache<I, S>(
    ctx: RequestContext<Context>,
    cmd: S,
    args: I,
) -> Result<Response<Body>, HttpError>
where
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr> + std::fmt::Display,
{
    // TODO: Build key without using Debug ({:?})
    let key = format!("{} {:?}", &cmd, &args);

    let response =
        Response::builder().header("Content-Type", "application/json");

    if let Some(cache) = ctx.context().get_cache(&key) {
        debug!(ctx.log, "Cache hit for \"{key}\"");
        return response
            .status(StatusCode::OK)
            .body(cache.into())
            .map_err(to_internal_error);
    }

    let out = Command::new(&cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(to_internal_error)?
        .wait_with_output()
        .await
        .map_err(to_internal_error)?;

    if !out.status.success() {
        let stderr =
            String::from_utf8(out.stderr).map_err(to_internal_error)?;

        error!(ctx.log, "Exec failed for \"{key}\": {stderr}");

        return response
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .map_err(to_internal_error);
    }

    let stdout = String::from_utf8(out.stdout).map_err(to_internal_error)?;

    ctx.context().set_cache(key, stdout.clone());

    response
        .status(StatusCode::OK)
        .body(stdout.into())
        .map_err(to_internal_error)
}
