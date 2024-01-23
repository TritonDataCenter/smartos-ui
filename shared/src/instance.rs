/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::fmt::{Display, Error, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub zonename: String,
    pub autoboot: bool,
    pub brand: Brand,
    pub limit_priv: String,
    pub v: u64,
    pub create_timestamp: String, // OffsetDataTime
    pub image_uuid: String,
    pub cpu_shares: u64,
    pub max_lwps: u64,
    pub max_msg_ids: u64,
    pub max_sem_ids: u64,
    pub max_shm_ids: u64,
    pub max_shm_memory: u64,
    pub zfs_io_priority: u64,
    pub max_physical_memory: u64,
    pub max_locked_memory: u64,
    pub max_swap: u64,
    pub billing_id: String,
    pub owner_uuid: String,
    pub tmpfs: u64,
    pub dns_domain: String,
    pub resolvers: Vec<String>,
    pub alias: String,
    pub nics: Vec<Nic>,
    pub datasets: Option<Vec<String>>,
    pub uuid: String,
    pub zone_state: String, // Enum
    pub zonepath: String,
    pub hvm: bool,
    pub zoneid: u64,
    pub zonedid: u64,
    pub last_modified: String, // OffsetDateTime
    pub firewall_enabled: bool,
    pub server_uuid: String,
    pub platform_buildstamp: String,
    pub state: String,
    pub boot_timestamp: String, // OffsetDateTime
    pub init_restarts: u64,
    pub pid: u64,
    // pub customer_metadata: Struct1,
    // pub internal_metadata: Struct1,
    // pub routes: Struct1,
    // pub tags: Struct1,
    pub quota: u64,
    pub zfs_root_recsize: u64,
    pub zfs_filesystem: String,
    pub zpool: String,
    pub zfs_data_recsize: Option<u64>,
    //pub snapshots: Vec<_>,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub enum Brand {
    #[serde(rename = "joyent")]
    Joyent,
    #[serde(rename = "joyent-minimal")]
    JoyentMinimal,
    #[serde(rename = "bhyve")]
    Bhyve,
    #[serde(rename = "kvm")]
    KVM,
}

impl Display for Brand {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            Brand::Joyent => write!(fmt, "joyent"),
            Brand::JoyentMinimal => write!(fmt, "joyent-minimal"),
            Brand::Bhyve => write!(fmt, "bhyve"),
            Brand::KVM => write!(fmt, "kvm"),
        }
    }
}
