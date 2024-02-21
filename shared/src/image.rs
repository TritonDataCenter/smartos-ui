/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct ImageImportParams {
    pub url: Url,
    pub name: String,
    pub version: String,
    pub r#type: String,
    pub os: String,
}

/// Assumes the version 2 Manifest format created 2013-Jan-31
/// <https://images.tritondatacenter.com/docs/>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    /// Version of the manifest format/spec. The current value is 2.
    pub v: u64,

    /// The unique identifier for a UUID. This is set by the IMGAPI server.
    pub uuid: Uuid,

    /// The UUID of the owner of this image (the account that created it).
    pub owner: Uuid,

    /// A short name for this image. Max 512 characters (though practical usage should be much shorter). No uniqueness guarantee.
    pub name: String,

    /// A version string for this image. Max 128 characters. No uniqueness guarantee.
    pub version: String,

    /// A short description of the image.
    pub description: Option<String>,

    /// Homepage URL where users can find more information about the image.
    pub homepage: Option<String>, // TODO: URL type?

    /// URL of the End User License Agreement (EULA) for the image.
    pub eula: Option<String>, // TODO: URL type?

    /// Indicates if the image has an icon file. If not present, then no icon is present.
    pub icon: Option<bool>,

    #[serde(default)]
    pub state: String, // TODO: use State

    /// An object with details on image creation failure. It only exists when state=='failed'.
    pub error: Option<Value>,

    /// Indicates if this image is available for provisioning.
    pub disabled: bool,

    /// Indicates if this image is publicly available.
    pub public: bool,

    /// The date at which the image is activated. Set by the IMGAPI server.
    // #[serde(
    //     deserialize_with = "time::serde::iso8601::deserialize"
    // )]
    pub published_at: Option<String>, // deserialize_with = "time::serde::iso8601::deserialize" can't be use with Option, add deserializer

    /// The image type. One of "zone-dataset" for a ZFS dataset used to create
    /// a new SmartOS zone, "lx-dataset" for a Lx-brand image, "lxd" for a
    /// LXD image, "zvol" for a virtual machine image or "other" for image
    /// types that serve any other specific purpose.
    pub r#type: Type,

    /// The OS family this image provides. One of "smartos", "windows",
    /// "linux", "bsd", "illumos" or "other".
    pub os: String,

    /// A set of named requirements for provisioning a VM with this image
    pub requirements: Option<Requirements>,

    /// A list of users for which passwords should be generated for
    /// provisioning. This may only make sense for some images
    /// Example: `[{"name": "root"}, {"name": "admin"}]`
    pub users: Option<Vec<User>>,

    /// A list of tags that can be used by operators for additional billing
    /// processing.
    pub billing_tags: Option<Value>,

    /// An object that defines a collection of properties that is used by other APIs to evaluate where should customer VMs be placed.
    pub traits: Option<Value>,

    /// An object that defines a collection of properties that is used by other
    /// APIs to evaluate where should customer VMs be placed.
    pub tags: Option<Value>,

    /// A boolean indicating whether to generate passwords for the users in the
    /// "users" field. If not present, the default value is true.
    pub generate_passwords: Option<bool>,

    /// A list of inherited directories (other than the defaults for the brand).
    pub inherited_directories: Option<Vec<String>>,

    /// NIC driver used by this VM image. (if type==="zvol")
    pub nic_driver: Option<String>,

    /// Disk driver used by this VM image. (if type==="zvol")
    pub disk_driver: Option<String>,

    /// The QEMU CPU model to use for this VM image. (if type==="zvol")
    pub cpu_type: Option<String>,

    /// The size (in MiB) of this VM image's disk. (if type==="zvol")
    pub image_size: Option<u64>,

    /// Array of channel names to which this image belongs.
    /// (if server uses channels)
    pub channels: Option<Vec<String>>,
}

impl Manifest {
    /// Return just enough of a Manifest so that we can insert this instance
    /// into the image import queue when importing an image.
    pub fn new_for_import(
        uuid: Uuid,
        name: String,
        version: String,
        r#type: String,
        os: String,
    ) -> Self {
        Self {
            v: 0,
            uuid,
            owner: Default::default(),
            name,
            version,
            r#type: Type::from_str(r#type.as_str()).unwrap_or_default(),
            os,
            state: "importing".to_string(),
            published_at: None, //OffsetDateTime::now_utc(),
            description: None,
            homepage: None,
            eula: None,
            requirements: None,
            users: None,
            billing_tags: None,
            traits: None,
            tags: None,
            generate_passwords: None,
            inherited_directories: None,
            nic_driver: None,
            disk_driver: None,
            cpu_type: None,
            image_size: None,
            disabled: false,
            icon: None,
            error: None,
            public: false,
            channels: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Network {
    pub name: String,
    pub description: String,
}

/// A set of named requirements for provisioning a VM with this image
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Requirements {
    /// Defines the minimum number of network interfaces required by this image.
    pub networks: Option<Vec<Network>>,

    /// Defines the brand that is required to provision with this image.
    pub brand: Option<String>,

    /// Indicates that provisioning with this image requires that an SSH public key be provided.
    pub ssh_key: Option<bool>,

    /// Minimum RAM (in MiB) required to provision this image.
    pub min_ram: Option<u64>,

    /// Maximum RAM (in MiB) this image may be provisioned with.
    pub max_ram: Option<u64>,

    /// Minimum platform requirement for provisioning with this image.
    pub min_platform: Option<Value>,

    /// Maximum platform requirement for provisioning with this image.
    pub max_platform: Option<Value>,

    /// Bootrom image to use with this image.
    pub bootrom: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ImportStatus {
    Importing,
    Failed(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub manifest: Manifest,
    pub source: Option<Url>,

    // field for internal use
    pub import_status: Option<ImportStatus>,
}

impl Image {
    pub fn is_for_hvm(&self) -> bool {
        self.manifest.r#type == Type::ZVol
    }
    pub fn group_name(&self) -> String {
        match self.manifest.r#type {
            Type::ZVol => String::from("Hardware Virtual Machine"),
            Type::LXDataset | Type::LXD => {
                String::from("Container-native Linux")
            }
            Type::ZoneDataset => String::from("SmartOS Zone (Container)"),
            Type::Other => String::from("Other"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub url: Url,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Eq, PartialEq)]
pub enum State {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "unactivated")]
    Unactivated,
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "creating")]
    Creating,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "unknown")]
    Unknown,
}

impl Display for State {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            State::Active => write!(fmt, "active"),
            State::Unactivated => write!(fmt, "unactivated"),
            State::Disabled => write!(fmt, "disabled"),
            State::Creating => write!(fmt, "creating"),
            State::Failed => write!(fmt, "failed"),
            State::Unknown => write!(fmt, "unknown"),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Eq, PartialEq, Clone)]
pub enum Type {
    #[serde(rename = "zone-dataset")]
    ZoneDataset,
    #[serde(rename = "lx-dataset")]
    LXDataset,
    #[serde(rename = "lxd")]
    LXD,
    #[serde(rename = "zvol")]
    ZVol,
    #[serde(rename = "other")]
    Other,
}

impl Display for crate::image::Type {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            Type::ZoneDataset => write!(fmt, "zone-dataset"),
            Type::LXDataset => write!(fmt, "lx-dataset"),
            Type::LXD => write!(fmt, "lxd"),
            Type::ZVol => write!(fmt, "zvol"),
            Type::Other => write!(fmt, "other"),
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::Other
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "zone-dataset" => Ok(Type::ZoneDataset),
            "lx-dataset" => Ok(Type::LXDataset),
            "lxd" => Ok(Type::LXD),
            "zvol" => Ok(Type::ZVol),
            "other" => Ok(Type::Other),
            _ => Err(format!("Unknown type provided: {}", s)),
        }
    }
}
