/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Instance {
    pub uuid: String,
    pub image_uuid: String,
    pub r#type: String,
    pub brand: String,
    pub ram: String,
    pub state: String,
    pub alias: String,
}
