/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::instance::Brand;
use crate::serde_helpers::{
    deserialize_into_bool, deserialize_into_option_bool,
    deserialize_into_option_u64, deserialize_into_u64,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slog::{warn, Logger};
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
    #[serde(deserialize_with = "deserialize_into_u64")]
    pub v: u64,

    /// The unique identifier for a UUID. This is set by the IMGAPI server.
    pub uuid: Uuid,

    /// The UUID of the owner of this image (the account that created it).
    #[serde(default)]
    pub owner: Uuid,

    /// A short name for this image. Max 512 characters (though practical usage should be much shorter). No uniqueness guarantee.
    pub name: String,

    /// A version string for this image. Max 128 characters. No uniqueness guarantee.
    pub version: String,

    /// A short description of the image.
    pub description: Option<String>,

    /// Homepage URL where users can find more information about the image.
    pub homepage: Option<String>,

    /// URL of the End User License Agreement (EULA) for the image.
    pub eula: Option<String>,

    /// Indicates if the image has an icon file. If not present, then no icon is present.
    pub icon: Option<bool>,

    #[serde(default)]
    pub state: String,

    /// An object with details on image creation failure. It only exists when state=='failed'.
    pub error: Option<Value>,

    /// Indicates if this image is available for provisioning.
    #[serde(default)]
    pub disabled: bool,

    /// Indicates if this image is publicly available.
    #[serde(default, deserialize_with = "deserialize_into_bool")]
    pub public: bool,

    /// The date at which the image is activated. Set by the IMGAPI server.
    pub published_at: Option<String>,

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
    #[serde(default, deserialize_with = "deserialize_into_option_bool")]
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
    #[serde(default, deserialize_with = "deserialize_into_option_u64")]
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
            published_at: None,
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
    #[serde(default)]
    pub description: String,
}

/// A set of named requirements for provisioning a VM with this image
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Requirements {
    /// Defines the minimum number of network interfaces required by this image.
    pub networks: Option<Vec<Network>>,

    /// Defines the brand that is required to provision with this image.
    pub brand: Option<Brand>,

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

    /// Field for internal use by the Executor
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

    /// Helper to get kernel_version from an LX image
    pub fn kernel_version(&self) -> String {
        if self.manifest.r#type == Type::LXDataset {
            if let Some(tags) = &self.manifest.tags {
                if let Some(obj) = tags.as_object() {
                    if let Some(version) = obj.get("kernel_version") {
                        return String::from(
                            version.as_str().unwrap_or("0.0.0"),
                        );
                    }
                }
            }
        }
        String::from("0.0.0")
    }

    /// Helper for determining if we should offer the option of setting a
    /// root_pw for an image
    pub fn has_root_user(&self) -> bool {
        return match &self.manifest.r#type {
            Type::ZVol => match &self.manifest.users {
                Some(users) => users.iter().any(|user| user.name == "root"),
                None => true,
            },
            _ => false,
        };
    }

    /// Helper for determining if we should set the bootrom to uefi
    pub fn uses_uefi_bootrom(&self, brand: &Brand) -> bool {
        // If the image specifies a bootrom uefi requirement
        if let Some(requirements) = &self.manifest.requirements {
            if let Some(bootrom) = &requirements.bootrom {
                if bootrom.as_str() == "uefi" {
                    return true;
                }
            }
        }

        // If it's a Bhyve brand
        if brand == &Brand::Bhyve {
            // ...and an Official image
            if let Some(source) = &self.source {
                if source.as_str() == "https://images.smartos.org/" {
                    //  ...and published 2023 later
                    if let Some(published) = &self.manifest.published_at {
                        if published.as_str() >= "2023-01-01T00:00:00Z" {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Helper to choose a default brand for an image
    pub fn default_brand(&self) -> Brand {
        // If the image specifies a brand requirement
        if let Some(requirements) = &self.manifest.requirements {
            if let Some(brand) = &requirements.brand {
                return brand.clone();
            }
        }
        match self.manifest.r#type {
            Type::ZVol => Brand::Bhyve,
            Type::LXDataset => Brand::LX,
            Type::LXD => Brand::LXD,
            Type::ZoneDataset => Brand::Joyent,
            Type::Other => Brand::Joyent,
        }
    }

    /// Helper to verify if a brand is valid for the given image
    pub fn valid_brand(&self, brand: &Brand) -> bool {
        // If the image specifies a brand requirement
        if let Some(requirements) = &self.manifest.requirements {
            if let Some(image_brand) = &requirements.brand {
                if image_brand == brand {
                    return true;
                }
            }
        }
        match &self.manifest.r#type {
            Type::ZVol => {
                if brand == &Brand::Bhyve || brand == &Brand::KVM {
                    return true;
                }
            }
            Type::LXDataset => {
                if brand == &Brand::LX {
                    return true;
                }
            }
            Type::LXD => {
                if brand == &Brand::LXD {
                    return true;
                }
            }
            Type::ZoneDataset => {
                if brand == &Brand::Joyent || brand == &Brand::JoyentMinimal {
                    return true;
                }
            }
            Type::Other => return true,
        }

        false
    }

    /// Deserializes each image in a list of images individually so that
    /// images from external sources, which don't strictly follow the IMGAPI
    /// format won't cause issues elsewhere.
    pub fn deserialize_list(list: &str, log: &Logger) -> Vec<Image> {
        let mut images: Vec<Image> = Vec::new();
        let mut values: Vec<Value> =
            serde_json::from_str(list).unwrap_or(Vec::new());
        for value in values.drain(..) {
            match serde_json::from_value(value) {
                Ok(image) => images.push(image),
                Err(err) => {
                    warn!(log, "Failed to serialize manifest: {:?}", err);
                }
            }
        }
        images
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
