/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_COMMIT_SHORT: &str = env!("GIT_COMMIT_SHORT");

#[cfg(debug_assertions)]
pub const DEBUG: bool = true;

#[cfg(not(debug_assertions))]
pub const DEBUG: bool = false;

pub mod clients;
pub mod endpoints;
pub mod privilege;
pub mod session;
