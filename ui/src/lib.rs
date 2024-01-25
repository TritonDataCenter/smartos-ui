/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod endpoints;
pub mod exec;
pub mod privilege;
pub mod session;
