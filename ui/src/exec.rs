/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use smartos_shared::{image::Image, instance::Instance, sysinfo::Sysinfo};

use reqwest::{Client as HTTPClient, RequestBuilder};

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
        id: &String,
    ) -> Result<Instance, reqwest::Error> {
        self.vminfo.get_instance(id).await
    }
    pub async fn get_sysinfo(&self) -> Result<Sysinfo, reqwest::Error> {
        self.exec.get_sysinfo().await
    }

    pub async fn get_images(&self) -> Result<Vec<Image>, reqwest::Error> {
        self.exec.get_images().await
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
        id: &String,
    ) -> Result<Instance, reqwest::Error> {
        let response: Instance = self
            .get(format!("vms/{}", &id).as_str())
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
}
