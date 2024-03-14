/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::fmt;

use smartos_shared::{
    image::Image, image::ImageImportParams, image::Source, instance::Info,
    instance::Instance, instance::InstancePayload,
    instance::InstanceValidateResponse, instance::InstanceView, nictag::NicTag,
    sysinfo::Sysinfo,
};

use reqwest::{Client as HTTPClient, RequestBuilder, Response};
use serde_json::to_string as stringify;
use uuid::Uuid;

#[derive(Debug)]
pub enum RequestError {
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
}

impl From<reqwest::Error> for RequestError {
    fn from(e: reqwest::Error) -> Self {
        RequestError::ReqwestError(e)
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(e: serde_json::Error) -> Self {
        RequestError::JsonError(e)
    }
}

// TODO: format these errors with more details
impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestError::ReqwestError(_) => write!(f, "HTTP Request Error"),
            RequestError::JsonError(_) => write!(f, "JSON parsing Error"),
        }
    }
}

pub struct VMInfodClient {
    http: HTTPClient,
    url: String,
}

impl VMInfodClient {
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

    pub async fn get_instances(
        &self,
    ) -> Result<Vec<InstanceView>, reqwest::Error> {
        let instances: Vec<Instance> =
            self.get("vms").send().await?.error_for_status()?.json().await?;
        let mut views: Vec<InstanceView> = Vec::default();
        for i in instances {
            views.push(i.try_into().expect("failed"))
        }
        Ok(views)
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

    pub async fn get_instance_view(
        &self,
        id: &Uuid,
    ) -> Result<InstanceView, reqwest::Error> {
        Ok(self.get_instance(id).await?.try_into().expect("failed"))
    }

    pub async fn get_instance_json(
        &self,
        id: &Uuid,
    ) -> Result<String, reqwest::Error> {
        self.get(format!("vms/{}", &id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn ping(&self) -> Result<bool, reqwest::Error> {
        Ok(self
            .get("ping")
            .send()
            .await?
            .error_for_status()?
            .status()
            .is_success())
    }
}

pub struct ExecutorClient {
    http: HTTPClient,
    url: String,
}

impl ExecutorClient {
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
        let response =
            self.get("image").send().await?.error_for_status()?.text().await?;
        Ok(Image::deserialize_list(response.as_str()))
    }

    pub async fn get_image(&self, id: &Uuid) -> Result<Image, reqwest::Error> {
        self.get(format!("image/{}", &id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_image_json(
        &self,
        id: &Uuid,
    ) -> Result<String, reqwest::Error> {
        self.get(format!("image/{}", &id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn get_available_images(
        &self,
    ) -> Result<Vec<Image>, reqwest::Error> {
        let response =
            self.get("avail").send().await?.error_for_status()?.text().await?;
        Ok(Image::deserialize_list(response.as_str()))
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
    ) -> Result<Response, RequestError> {
        let req = stringify(&params)?;
        Ok(self
            .post(format!("import/{}", id.as_hyphenated()).as_str())
            .body(req)
            .send()
            .await?)
    }

    pub async fn get_sources(&self) -> Result<Vec<Source>, reqwest::Error> {
        self.get("source").send().await?.error_for_status()?.json().await
    }

    pub async fn provision(
        &self,
        payload: InstancePayload,
    ) -> Result<Response, reqwest::Error> {
        let req = stringify(&payload).expect("failed");
        self.post("provision").body(req).send().await
    }

    pub async fn validate_create(
        &self,
        payload: InstancePayload,
    ) -> Result<InstanceValidateResponse, reqwest::Error> {
        let req = stringify(&payload).expect("failed");
        self.post("validate/create")
            .body(req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn delete_instance(
        &self,
        id: &Uuid,
    ) -> Result<String, reqwest::Error> {
        self.delete(format!("instance/{}", id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn start_instance(
        &self,
        id: &Uuid,
    ) -> Result<String, reqwest::Error> {
        self.post(format!("instance/{}/start", id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn stop_instance(
        &self,
        id: &Uuid,
    ) -> Result<String, reqwest::Error> {
        self.post(format!("instance/{}/stop", id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn get_nictags(&self) -> Result<Vec<NicTag>, reqwest::Error> {
        self.get("nictag").send().await?.error_for_status()?.json().await
    }

    pub async fn get_pwhash(&self) -> Result<String, reqwest::Error> {
        self.get("pwhash").send().await?.error_for_status()?.text().await
    }

    pub async fn ping(&self) -> Result<bool, reqwest::Error> {
        Ok(self
            .get("ping")
            .send()
            .await?
            .error_for_status()?
            .status()
            .is_success())
    }

    pub async fn info(&self, id: &Uuid) -> Result<Info, reqwest::Error> {
        self.get(format!("info/{}", id.as_hyphenated()).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_gz_config(
        &self,
    ) -> Result<Vec<(String, String)>, reqwest::Error> {
        self.get("config/gz").send().await?.error_for_status()?.json().await
    }
}
