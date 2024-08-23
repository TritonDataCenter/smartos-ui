/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::serde_helpers::deserialize_into_u64;

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
    pub image_uuid: Option<Uuid>,
    #[serde(default)]
    pub image_size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nic {
    pub nic_tag: Option<String>,
    pub ips: Option<Vec<String>>,
    pub gateways: Option<Vec<String>>,
    pub model: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Generic {
    pub v: u8,
    pub uuid: Uuid,
    pub alias: Option<String>,
    pub state: String,
    pub hvm: bool,
    #[serde(deserialize_with = "deserialize_into_u64")]
    pub quota: u64,
    #[serde(default)]
    pub max_physical_memory: u64,
    pub resolvers: Option<Vec<String>>,
    #[serde(default)]
    pub cpu_shares: u64,
    pub firewall_enabled: bool,
    pub autoboot: bool,
    pub billing_id: String,
    pub owner_uuid: Uuid,
    pub dns_domain: Option<String>,
    pub limit_priv: String,
    #[serde(default)]
    pub max_lwps: u64,
    pub max_shm_memory: Option<u64>,
    #[serde(default)]
    pub max_swap: u64,
    pub zfs_filesystem: String,
    #[serde(default)]
    pub zfs_io_priority: u64,
    pub zonepath: String,
    pub create_timestamp: String,
    pub last_modified: String,
    pub platform_buildstamp: String,
    pub nics: Vec<Nic>,
    #[serde(default)]
    pub cpu_cap: u64,

    // if started
    pub boot_timestamp: Option<String>,
    pub init_restarts: Option<u64>,
    pub pid: Option<u64>,
    pub zoneid: Option<u64>,

    // if stopped
    pub exit_status: Option<i64>,
    pub exit_timestamp: Option<String>,
}

impl Generic {
    pub fn primary_ip(&self) -> Option<String> {
        let mut first_address: Option<String> = None;
        for nic in self.nics.iter() {
            if let Some(primary) = nic.primary {
                if !primary {
                    continue;
                }
            }
            if let Some(ips) = &nic.ips {
                if let Some(ip) = ips.iter().next() {
                    first_address = Some(ip.clone());
                }
            }
            if first_address.is_some() {
                break;
            }
        }
        first_address
    }

    /// Return CPU Cap / 100 or 0
    pub fn get_cpus(&self) -> f32 {
        if self.cpu_cap > 0 {
            self.cpu_cap as f32 / 100.0
        } else {
            0.0
        }
    }
    pub fn alias(&self) -> String {
        if let Some(alias) = &self.alias {
            alias.clone()
        } else {
            self.uuid.to_string().split('-').nth(0).unwrap_or("-").to_string()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HVM {
    #[serde(default)]
    pub ram: u64,
    pub disks: Vec<Disk>,
    #[serde(default)]
    pub vcpus: u64,
}

impl HVM {
    pub fn get_disk_usage(&self) -> u64 {
        self.disks.iter().fold(0, |acc, d| d.image_size + acc)
    }
    pub fn get_boot_image_uuid(&self) -> Uuid {
        if let Some(boot_disk) = self.disks.iter().find(|d| d.boot.is_some()) {
            boot_disk.image_uuid.unwrap_or(UuidBuilder::nil().into_uuid())
        } else {
            UuidBuilder::nil().into_uuid()
        }
    }

    /// Return CPU Cap / 100 or vCPUs, whichever is larger
    pub fn get_cpus(&self, cpu_cap: u64) -> f32 {
        let cap = if cpu_cap > 0 { cpu_cap as f32 / 100.0 } else { 0.0 };
        if cap > self.vcpus as f32 {
            cap
        } else {
            self.vcpus as f32
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Native {
    pub image_uuid: Uuid,
    #[serde(default)]
    pub tmpfs: u64,
    pub datasets: Option<Vec<String>>,
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
    pub alias: String,
    pub brand: Brand,
    pub state: String,
    #[serde(default)]
    pub ram: u64,
    #[serde(default)]
    pub disk_usage: u64,
    pub hvm: bool,
    pub image_uuid: Uuid,
    #[serde(default)]
    pub cpu: f32,
    pub primary_ip: Option<String>,
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

impl Instance {
    pub fn is_hvm(&self) -> bool {
        matches!(self, Instance::Bhyve(_) | Instance::KVM(_))
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            Instance::Joyent(i) => i.generic.uuid,
            Instance::JoyentMinimal(i) => i.generic.uuid,
            Instance::Bhyve(i) => i.generic.uuid,
            Instance::KVM(i) => i.generic.uuid,
            Instance::LX(i) => i.generic.uuid,
        }
    }

    pub fn state(&self) -> &str {
        match self {
            Instance::Joyent(i) => &i.generic.state,
            Instance::JoyentMinimal(i) => &i.generic.state,
            Instance::Bhyve(i) => &i.generic.state,
            Instance::KVM(i) => &i.generic.state,
            Instance::LX(i) => &i.generic.state,
        }
    }

    pub fn alias(&self) -> String {
        match self {
            Instance::Joyent(i) => i.generic.alias(),
            Instance::JoyentMinimal(i) => i.generic.alias(),
            Instance::Bhyve(i) => i.generic.alias(),
            Instance::KVM(i) => i.generic.alias(),
            Instance::LX(i) => i.generic.alias(),
        }
    }

    pub fn image_uuid(&self) -> Uuid {
        match self {
            Instance::Joyent(i) => i.native.image_uuid,
            Instance::JoyentMinimal(i) => i.native.image_uuid,
            Instance::LX(i) => i.native.image_uuid,
            Instance::Bhyve(i) => i.hvm.get_boot_image_uuid(),
            Instance::KVM(i) => i.hvm.get_boot_image_uuid(),
        }
    }
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
        let primary_ip = value.generic.primary_ip();

        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias(),
            brand: Brand::KVM,
            ram: value.hvm.ram,
            hvm: true,
            state: value.generic.state,
            disk_usage: value.hvm.get_disk_usage(),
            image_uuid: value.hvm.get_boot_image_uuid(),
            cpu: value.hvm.get_cpus(value.generic.cpu_cap),
            primary_ip,
        })
    }
}

impl TryFrom<Bhyve> for InstanceView {
    type Error = ();

    fn try_from(value: Bhyve) -> Result<Self, Self::Error> {
        let primary_ip = value.generic.primary_ip();
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias(),
            brand: Brand::Bhyve,
            ram: value.hvm.ram,
            hvm: true,
            state: value.generic.state,
            disk_usage: value.hvm.get_disk_usage(),
            image_uuid: value.hvm.get_boot_image_uuid(),
            cpu: value.hvm.get_cpus(value.generic.cpu_cap),
            primary_ip,
        })
    }
}

impl TryFrom<JoyentMinimal> for InstanceView {
    type Error = ();

    fn try_from(value: JoyentMinimal) -> Result<Self, Self::Error> {
        let primary_ip = value.generic.primary_ip();
        let cpu = value.generic.get_cpus();
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias(),
            brand: Brand::JoyentMinimal,
            ram: value.generic.max_physical_memory,
            hvm: false,
            state: value.generic.state,
            disk_usage: value.generic.quota * 1024,
            image_uuid: value.native.image_uuid,
            cpu,
            primary_ip,
        })
    }
}

impl TryFrom<Joyent> for InstanceView {
    type Error = ();

    fn try_from(value: Joyent) -> Result<Self, Self::Error> {
        let primary_ip = value.generic.primary_ip();
        let cpu = value.generic.get_cpus();
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias(),
            brand: Brand::Joyent,
            ram: value.generic.max_physical_memory,
            hvm: false,
            state: value.generic.state,
            disk_usage: value.generic.quota * 1024,
            image_uuid: value.native.image_uuid,
            cpu,
            primary_ip,
        })
    }
}

impl TryFrom<LX> for InstanceView {
    type Error = ();

    fn try_from(value: LX) -> Result<Self, Self::Error> {
        let primary_ip = value.generic.primary_ip();
        let cpu = value.generic.get_cpus();
        Ok(InstanceView {
            uuid: value.generic.uuid,
            alias: value.generic.alias(),
            brand: Brand::LX,
            ram: value.generic.max_physical_memory,
            hvm: false,
            state: value.generic.state,
            disk_usage: value.generic.quota * 1024,
            image_uuid: value.native.image_uuid,
            cpu,
            primary_ip,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Eq, PartialEq, Clone)]
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

#[derive(Deserialize, Debug, JsonSchema)]
pub struct Vnc {
    pub host: Option<String>,
    pub port: Option<u64>,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct Info {
    pub vnc: Option<Vnc>,
}
