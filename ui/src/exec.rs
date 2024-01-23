/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use smartos_shared::{
    image::Image, instance::CreatePayload, instance::Instance, sysinfo::Sysinfo,
};

use reqwest::{Client as HTTPClient, RequestBuilder};
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
        let response: Vec<Instance> = self
            .get("vms")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn get_instance(
        &self,
        id: &Uuid,
    ) -> Result<Instance, reqwest::Error> {
        let response: Instance = self
            .get(format!("vms/{}", &id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
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
        let response: Sysinfo = self
            .get("sysinfo")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    // TODO: Needs to be cached
    pub async fn get_images(&self) -> Result<Vec<Image>, reqwest::Error> {
        let response: Vec<Image> = self
            .get("image")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
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
}
