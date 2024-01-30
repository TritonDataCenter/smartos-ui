/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use smartos_shared::{
    image::Image, instance::CreatePayload, instance::Instance, nictag::NicTag,
    sysinfo::Sysinfo,
};

use reqwest::{Client as HTTPClient, RequestBuilder};
use smartos_shared::image::{ImageImportParams, Source};
use tokio::try_join;
use uuid::Uuid;

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

    // Send requests to some of the slower endpoints to warm up their
    // cache and provide a smoother first-time experience.
    pub async fn warm_cache(&self) -> Result<(), reqwest::Error> {
        try_join!(self.vminfo.get_instances(), self.exec.get_images(),)?;
        Ok(())
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

    pub async fn get_available_images(
        &self,
    ) -> Result<Vec<Image>, reqwest::Error> {
        self.exec.get_available_images().await
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

    pub async fn get_pwhash(&self) -> Result<String, reqwest::Error> {
        self.exec.get_pwhash().await
    }
}

pub struct VminfodClient {
    http: HTTPClient,
    url: String,
}

impl VminfodClient {
    #[must_use]
    pub fn new(address: String) -> Self {
        Self { http: HTTPClient::new(), url: format!("http://{}", address) }
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.http.post(format!("{}/{path}", self.url))
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.http.get(format!("{}/{path}", self.url))
    }

    pub async fn get_instances(&self) -> Result<Vec<Instance>, reqwest::Error> {
        self.get("vms").send().await?.error_for_status()?.json().await
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
        Self { http: HTTPClient::new(), url: format!("http://{}", address) }
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
        self.get("sysinfo").send().await?.error_for_status()?.json().await
    }

    pub async fn get_images(&self) -> Result<Vec<Image>, reqwest::Error> {
        self.get("image").send().await?.error_for_status()?.json().await
    }

    pub async fn get_image(&self, id: &Uuid) -> Result<Image, reqwest::Error> {
        self.get(format!("image/{}", &id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_available_images(
        &self,
    ) -> Result<Vec<Image>, reqwest::Error> {
        self.get("avail").send().await?.error_for_status()?.json().await
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
        self.get("source").send().await?.error_for_status()?.json().await
    }

    pub async fn create_instance(
        &self,
        payload: CreatePayload,
    ) -> Result<(), reqwest::Error> {
        let req = serde_json::to_string(&payload).expect("failed");
        self.post("instance").body(req).send().await?.error_for_status()?;
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
        self.get("nictag").send().await?.error_for_status()?.json().await
    }

    pub async fn get_pwhash(&self) -> Result<String, reqwest::Error> {
        self.get("pwhash").send().await?.error_for_status()?.text().await
    }
}
