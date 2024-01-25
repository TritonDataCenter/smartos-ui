/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::fs;
use std::path::PathBuf;
use smartos_shared::{
    image::Image, instance::CreatePayload, instance::Instance, nictag::NicTag,
    sysinfo::Sysinfo,
};

use reqwest::{Client as HTTPClient, RequestBuilder, Url};
use smartos_shared::image::{ImageImportParams, Manifest, Source};
use uuid::Uuid;
use crate::endpoints::to_internal_error;

pub struct Client {
    exec: ExecClient,
    vminfo: VminfodClient,
}

impl Client {
    #[must_use]
    pub fn new(exec_address: String, vminfo_address: String) -> Self {
        Self {
            exec: ExecClient::new(exec_address),
            vminfo: VminfodClient::new(vminfo_address),
        }
    }
    pub async fn get_instances(&self) -> Result<Vec<Instance>, reqwest::Error> {
        self.vminfo.get_instances().await
    }

    pub async fn get_instance(
        &self,
        id: &Uuid,
    ) -> Result<Instance, reqwest::Error> {
        self.vminfo.get_instance(id).await
    }
    pub async fn get_sysinfo(&self) -> Result<Sysinfo, reqwest::Error> {
        self.exec.get_sysinfo().await
    }

    pub async fn get_images(&self) -> Result<Vec<Image>, reqwest::Error> {
        self.exec.get_images().await
    }
    pub async fn get_image(&self, id: &Uuid) -> Result<Image, reqwest::Error> {
        self.exec.get_image(id).await
    }

    pub async fn get_sources(&self) -> Result<Vec<Source>, reqwest::Error> {
        self.exec.get_sources().await
    }

    pub async fn delete_image(&self, id: &Uuid) -> Result<(), reqwest::Error> {
        self.exec.delete_image(id).await
    }

    pub async fn import_image(
        &self,
        id: &Uuid,
        params: &ImageImportParams,
    ) -> Result<(), reqwest::Error> {
        self.exec.import_image(id, params).await
    }

    pub async fn create_instance(
        &self,
        payload: CreatePayload,
    ) -> Result<(), reqwest::Error> {
        self.exec.create_instance(payload).await
    }

    pub async fn delete_instance(
        &self,
        id: &Uuid,
    ) -> Result<(), reqwest::Error> {
        self.exec.delete_instance(id).await
    }

    pub async fn get_nictags(&self) -> Result<Vec<NicTag>, reqwest::Error> {
        self.exec.get_nictags().await
    }
}

pub struct VminfodClient {
    http: HTTPClient,
    url: String,
}

impl VminfodClient {
    #[must_use]
    pub fn new(address: String) -> Self {
        Self {
            http: HTTPClient::new(),
            url: format!("http://{}", address),
        }
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.http.post(format!("{}/{path}", self.url))
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.http.get(format!("{}/{path}", self.url))
    }

    pub async fn get_instances(&self) -> Result<Vec<Instance>, reqwest::Error> {
        self.get("vms")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_instance(
        &self,
        id: &Uuid,
    ) -> Result<Instance, reqwest::Error> {
        self.get(format!("vms/{}", &id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

pub struct ExecClient {
    http: HTTPClient,
    url: String,
}

impl ExecClient {
    #[must_use]
    pub fn new(address: String) -> Self {
        Self {
            http: HTTPClient::new(),
            url: format!("http://{}", address),
        }
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.http.post(format!("{}/{path}", self.url))
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.http.get(format!("{}/{path}", self.url))
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.http.delete(format!("{}/{path}", self.url))
    }

    pub async fn get_sysinfo(&self) -> Result<Sysinfo, reqwest::Error> {
        self.get("sysinfo")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_images(&self) -> Result<Vec<Image>, reqwest::Error> {
        self.get("image")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_image(&self, id: &Uuid) -> Result<Image, reqwest::Error> {
        self.get(format!("image/{}", &id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn delete_image(&self, id: &Uuid) -> Result<(), reqwest::Error> {
        self.delete(format!("image/{}", id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn import_image(
        &self,
        id: &Uuid,
        params: &ImageImportParams,
    ) -> Result<(), reqwest::Error> {
        let req = serde_json::to_string(&params).expect("failed");
        self.post(format!("import/{}", id.as_hyphenated()).as_str())
            .body(req)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_sources(&self) -> Result<Vec<Source>, reqwest::Error> {
        self.get("source")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn create_instance(
        &self,
        payload: CreatePayload,
    ) -> Result<(), reqwest::Error> {
        let req = serde_json::to_string(&payload).expect("failed");
        self.post("instance")
            .body(req)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn delete_instance(
        &self,
        id: &Uuid,
    ) -> Result<(), reqwest::Error> {
        self.delete(format!("instance/{}", id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_nictags(&self) -> Result<Vec<NicTag>, reqwest::Error> {
        self.get("nictag")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

pub struct ImageApiClient {
    http: HTTPClient,
    url: String,
    cache_dir: String
}

impl ImageApiClient {
    #[must_use]
    pub fn new(address: String, cache_dir: String) -> Self {
        Self {
            http: HTTPClient::new(),
            url: address,
            cache_dir
        }
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.http.post(format!("{}/{path}", self.url))
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.http.get(format!("{}/{path}", self.url))
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.http.delete(format!("{}/{path}", self.url))
    }

    pub async fn get_images(&self) -> Result<Vec<Manifest>, reqwest::Error>  {
        let url = Url::parse(&self.url).unwrap();
        let mut cache_file = PathBuf::from(&self.cache_dir);
        cache_file.push(format!("{}.json", url.domain().unwrap_or("domain.tld")));
        println!("cache file: {}", cache_file.to_str().unwrap());
        if let Ok(metadata) = fs::metadata(&cache_file) {
            if let Ok(time) = metadata.modified() {
                println!("last modified: {:?}", time.elapsed());
            } else {
                println!("Not supported on this platform");
            }
        }

        let response = self
            .get("images")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let manifests: Vec<Manifest> = serde_json::from_str(&response).unwrap();
        fs::write(&cache_file, response).expect("Unable to write file");
        Ok(manifests)
    }
}