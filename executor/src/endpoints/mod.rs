/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use smartos_shared::config::Config;
use time::OffsetDateTime;

use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

pub mod image;
pub mod instance;
pub mod nictag;
pub mod sysinfo;

#[derive(Debug)]
pub struct CacheEntry {
    pub expiry: OffsetDateTime,
    pub content: String,
}
#[derive(Debug)]
pub struct ExecCache {
    pub images: Option<CacheEntry>,
    pub image: HashMap<Uuid, CacheEntry>,
}

pub struct Context {
    pub config: Config,
    pub cache: Arc<Mutex<ExecCache>>,
}

impl Context {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            cache: Arc::new(Mutex::new(ExecCache {
                images: None,
                image: HashMap::new(),
            })),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathParams {
    id: Uuid,
}
