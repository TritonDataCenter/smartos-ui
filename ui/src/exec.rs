/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use smartos_shared::instance::Instance;

use reqwest::{Client, RequestBuilder};

pub struct ExecClient {
    http: Client,
    url: String,
}

impl ExecClient {
    #[must_use]
    pub fn new(exec_bind_address: String) -> Self {
        Self {
            http: Client::new(),
            url: format!("http://{}", exec_bind_address),
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
            .get("instance")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }
}
