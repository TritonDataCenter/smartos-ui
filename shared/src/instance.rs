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
pub struct ListInstance {
    pub uuid: String,
    pub image_uuid: String,
    pub r#type: String,
    pub brand: String,
    pub ram: u32,
    pub quota: u32,
    pub state: String,
    pub alias: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    pub zonename: String,
    pub autoboot: bool,
    pub brand: String,
    pub limit_priv: String,
    pub v: u64,
    pub create_timestamp: String,
    pub image_uuid: String,
    pub cpu_shares: u64,
    pub max_lwps: u64,
    pub zfs_io_priority: u64,
    pub max_physical_memory: u64,
    pub max_swap: u64,
    pub billing_id: String,
    pub owner_uuid: String,
    pub tmpfs: u64,
    pub dns_domain: String,
    pub resolvers: Vec<String>,
    pub alias: String,
    pub nics: Vec<Nic>,
    pub uuid: String,
    pub zone_state: String,
    pub zonepath: String,
    pub hvm: bool,
    pub last_modified: String,
    pub firewall_enabled: bool,
    pub state: String,
    pub boot_timestamp: String,
    pub quota: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nic {
    pub interface: String,
    pub mac: String,
    pub nic_tag: String,
    pub gateway: String,
    pub gateways: Vec<String>,
    pub netmask: String,
    pub ip: String,
    pub ips: Vec<String>,
    pub primary: bool,
}
