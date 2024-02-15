/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::{Builder as UuidBuilder, Uuid};

/// Used for sending the instance json for `vmadm validate` and `vmadm create`
#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct InstancePayload {
    /// String containing instance JSON
    pub payload: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PayloadContainer {
    pub uuid: Uuid,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct InstanceValidateResponse {
    pub message: String,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Disk {
    pub boot: Option<bool>,
    pub image_uuid: Uuid,
    pub image_size: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Generic {
    pub v: u8,
    pub uuid: Uuid,
    pub alias: Option<String>,
    pub state: String,
    pub hvm: bool,
    pub quota: u64,
    pub max_physical_memory: u64,
    pub resolvers: Option<Vec<String>>,
    pub cpu_shares: u64,
    pub firewall_enabled: bool,
    pub autoboot: bool,
    pub billing_id: String,
    pub owner_uuid: Uuid,
    pub dns_domain: Option<String>,
    pub limit_priv: String,
    pub max_locked_memory: u64,
    pub max_lwps: u64,
    pub max_msg_ids: u64,
    pub max_sem_ids: u64,
    pub max_shm_ids: u64,
    pub max_shm_memory: u64,
    pub max_swap: u64,
    pub server_uuid: Uuid,
    pub zfs_filesystem: String,
    pub zfs_io_priority: u64,
    pub zfs_root_recsize: u64,
    pub zone_state: String,
    pub zonename: Uuid,
    pub zonepath: String,
    pub zpool: String,
    pub create_timestamp: String,
    pub last_modified: String,
    pub platform_buildstamp: String,
    // snapshots
    // tags
    // routes
    // nics
    // internal_metadata
    // customer_metadata

    // if started
    pub boot_timestamp: Option<String>,
    pub init_restarts: Option<u64>,
    pub pid: Option<u64>,
    pub zoneid: Option<u64>,

    // if stopped
    pub exit_status: Option<u64>,
    pub exit_timestamp: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HVM {
    pub ram: u64,
    pub disks: Vec<Disk>,
    pub vcpus: u64,
    // device
}

impl HVM {
    pub fn get_disk_usage(&self) -> u64 {
        self.disks.iter().fold(0, |acc, d| d.image_size + acc)
    }
    pub fn get_boot_image_uuid(&self) -> Uuid {
        if let Some(boot_disk) = self.disks.iter().find(|d| d.boot.is_some()) {
            boot_disk.image_uuid
        } else {
            UuidBuilder::nil().into_uuid()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Native {
    pub image_uuid: Uuid,
    pub tmpfs: u64,
    pub datasets: Option<Vec<String>>,
    // if dataset
    pub zfs_data_recsize: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bhyve {
    #[serde(flatten)]
    pub generic: Generic,
    #[serde(flatten)]
    pub hvm: HVM,
    pub com1: String,
    pub com2: String,
    pub zlog_mode: String,
    pub zlog_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KVM {
    #[serde(flatten)]
    pub generic: Generic,
    #[serde(flatten)]
    pub hvm: HVM,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Joyent {
    #[serde(flatten)]
    pub generic: Generic,
    #[serde(flatten)]
    pub native: Native,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoyentMinimal {
    #[serde(flatten)]
    pub generic: Generic,
    #[serde(flatten)]
    pub native: Native,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LX {
    pub kernel_version: String,
    #[serde(flatten)]
    pub generic: Generic,
    #[serde(flatten)]
    pub native: Native,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceView {
    pub uuid: Uuid,
    pub alias: Option<String>,
    pub brand: Brand,
    pub state: String,
    pub ram: u64,
    pub disk_usage: u64,
    pub hvm: bool,
    pub image_uuid: Uuid,
    pub cpu: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "brand")]
pub enum Instance {
    #[serde(rename = "joyent")]
    Joyent(Joyent),
    #[serde(rename = "joyent-minimal")]
    JoyentMinimal(JoyentMinimal),
    #[serde(rename = "bhyve")]
    Bhyve(Bhyve),
    #[serde(rename = "kvm")]
    KVM(KVM),
    #[serde(rename = "lx")]
    LX(LX),
}

impl TryFrom<Instance> for InstanceView {
    type Error = ();

    fn try_from(value: Instance) -> Result<Self, Self::Error> {
        match value {
            Instance::Joyent(i) => i.try_into(),
            Instance::JoyentMinimal(i) => i.try_into(),
            Instance::Bhyve(i) => i.try_into(),
            Instance::KVM(i) => i.try_into(),
            Instance::LX(i) => i.try_into(),
        }
    }
}

impl TryFrom<KVM> for InstanceView {
    type Error = ();

    fn try_from(value: KVM) -> Result<Self, Self::Error> {
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias,
            brand: Brand::KVM,
            ram: value.hvm.ram,
            hvm: true,
            state: value.generic.state,
            disk_usage: value.hvm.get_disk_usage(),
            image_uuid: value.hvm.get_boot_image_uuid(),
            cpu: value.hvm.vcpus as f32,
        })
    }
}

impl TryFrom<Bhyve> for InstanceView {
    type Error = ();

    fn try_from(value: Bhyve) -> Result<Self, Self::Error> {
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias,
            brand: Brand::Bhyve,
            ram: value.hvm.ram,
            hvm: true,
            state: value.generic.state,
            disk_usage: value.hvm.get_disk_usage(),
            image_uuid: value.hvm.get_boot_image_uuid(),
            cpu: value.hvm.vcpus as f32,
        })
    }
}

impl TryFrom<JoyentMinimal> for InstanceView {
    type Error = ();

    fn try_from(value: JoyentMinimal) -> Result<Self, Self::Error> {
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias,
            brand: Brand::JoyentMinimal,
            ram: value.generic.max_physical_memory,
            hvm: false,
            state: value.generic.state,
            disk_usage: value.generic.quota * 1024,
            image_uuid: value.native.image_uuid,
            cpu: value.generic.cpu_shares as f32 / 100.0,
        })
    }
}

impl TryFrom<Joyent> for InstanceView {
    type Error = ();

    fn try_from(value: Joyent) -> Result<Self, Self::Error> {
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias,
            brand: Brand::Joyent,
            ram: value.generic.max_physical_memory,
            hvm: false,
            state: value.generic.state,
            disk_usage: value.generic.quota * 1024,
            image_uuid: value.native.image_uuid,
            cpu: value.generic.cpu_shares as f32 / 100.0,
        })
    }
}

impl TryFrom<LX> for InstanceView {
    type Error = ();

    fn try_from(value: LX) -> Result<Self, Self::Error> {
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias,
            brand: Brand::LX,
            ram: value.generic.max_physical_memory,
            hvm: false,
            state: value.generic.state,
            disk_usage: value.generic.quota * 1024,
            image_uuid: value.native.image_uuid,
            cpu: value.generic.cpu_shares as f32 / 100.0,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Eq, PartialEq)]
pub enum Brand {
    #[serde(rename = "joyent")]
    Joyent,
    #[serde(rename = "joyent-minimal")]
    JoyentMinimal,
    #[serde(rename = "bhyve")]
    Bhyve,
    #[serde(rename = "kvm")]
    KVM,
    #[serde(rename = "lx")]
    LX,
    #[serde(rename = "lxd")]
    LXD,
    #[serde(rename = "other")]
    Other,
}

impl Brand {
    pub fn for_image_type(&self, image_type: &str) -> bool {
        matches!(
            (self, image_type),
            (Brand::Joyent, "zone-dataset")
                | (Brand::JoyentMinimal, "zone-dataset")
                | (Brand::Bhyve, "zvol")
                | (Brand::KVM, "zvol")
                | (Brand::LX, "lx-dataset")
                | (Brand::LXD, "lxd")
                | (Brand::Other, _)
        )
    }

    pub fn allows_delegate_dataset(&self) -> bool {
        matches!(self, Brand::Joyent | Brand::JoyentMinimal | Brand::LX)
    }

    pub fn is_hvm(&self) -> bool {
        match self {
            Brand::Joyent => false,
            Brand::JoyentMinimal => false,
            Brand::Bhyve => true,
            Brand::KVM => true,
            Brand::LX => false,
            Brand::LXD => false,
            Brand::Other => false,
        }
    }
}

impl Display for Brand {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            Brand::Joyent => write!(fmt, "joyent"),
            Brand::JoyentMinimal => write!(fmt, "joyent-minimal"),
            Brand::Bhyve => write!(fmt, "bhyve"),
            Brand::KVM => write!(fmt, "kvm"),
            Brand::LX => write!(fmt, "lx"),
            Brand::LXD => write!(fmt, "lxd"),
            Brand::Other => write!(fmt, "other"),
        }
    }
}

impl Default for Brand {
    fn default() -> Self {
        Self::Other
    }
}

#[derive(Debug)]
pub enum BrandError {
    UnknownBrand,
}

impl std::error::Error for BrandError {}

impl fmt::Display for BrandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BrandError::UnknownBrand => write!(f, "Unknown Brand"),
        }
    }
}

impl FromStr for Brand {
    type Err = BrandError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "joyent" => Ok(Brand::Joyent),
            "joyent-minimal" => Ok(Brand::JoyentMinimal),
            "bhyve" => Ok(Brand::Bhyve),
            "kvm" => Ok(Brand::KVM),
            "lx" => Ok(Brand::LX),
            "lxd" => Ok(Brand::LXD),
            "other" => Ok(Brand::LXD),
            _ => Err(BrandError::UnknownBrand),
        }
    }
}
