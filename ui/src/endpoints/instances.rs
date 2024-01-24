/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::collections::HashMap;

use crate::endpoints::{redirect_login, Context, HXLocation, PathParams};
use crate::session::Session;

use smartos_shared::instance::{CreatePayload, Instance, Nic};
use smartos_shared::nictag::NicTag;

use askama::Template;
use dropshot::{endpoint, HttpError, Path, RequestContext, TypedBody};
use hyper::{Body, Response, StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "instance.j2")]
pub struct InstanceTemplate {
    title: String,
    login: String,
    instance: Instance,
}

#[endpoint {
method = GET,
path = "/instances/{id}",
}]
pub async fn get_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let builder = Response::builder();
    if let Some(login) = Session::get_login(&ctx) {
        let path = path_params.into_inner();
        let instance =
            ctx.context().client.get_instance(&path.id).await.unwrap();

        let title = String::from("Instance");
        let template = InstanceTemplate {
            title,
            login,
            instance,
        };
        let result = template.render().unwrap();

        return Ok(builder
            .status(StatusCode::OK)
            .header("HX-Push-Url", format!("/instances/{}", &path.id))
            .header("Content-Type", "text/html")
            .body(result.into())?);
    }

    Ok(redirect_login(builder, &ctx)?)
}

#[endpoint {
method = DELETE,
path = "/instances/{id}",
}]
pub async fn delete_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let builder = Response::builder();
    if Session::get_login(&ctx).is_some() {
        ctx.context()
            .client
            .delete_instance(&path_params.into_inner().id)
            .await
            .unwrap();

        return Ok(HXLocation::common(builder, "/instances")?);
    }
    Ok(redirect_login(builder, &ctx)?)
}

#[derive(Template)]
#[template(path = "instances.j2")]
pub struct InstancesTemplate {
    total_ram: u64,
    total_quota: u64,
    title: String,
    login: String,
    instances: Vec<Instance>,
}

#[endpoint {
method = GET,
path = "/instances"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let builder = Response::builder();
    if let Some(login) = Session::get_login(&ctx) {
        let title = String::from("Instances");
        let instances = ctx.context().client.get_instances().await.unwrap();
        let total_ram = instances
            .iter()
            .fold(0, |acc, i| i.max_physical_memory + acc);
        let total_quota = instances.iter().fold(0, |acc, i| i.quota + acc);
        let template = InstancesTemplate {
            total_ram,
            total_quota,
            title,
            login,
            instances,
        };
        let result = template.render().unwrap();

        return Ok(builder
            .status(StatusCode::OK)
            .header("HX-Push-Url", String::from("/instances"))
            .header("Content-Type", "text/html")
            .body(result.into())?);
    }

    Ok(redirect_login(builder, &ctx)?)
}

#[derive(Template)]
#[template(path = "instance-create.j2")]
pub struct InstanceCreateTemplate {
    title: String,
    login: String,
    images: HashMap<String, Vec<ImageOption>>,
    nictags: Vec<NicTag>,
}
pub struct ImageOption {
    pub id: String,
    pub title: String,
    pub name: String,
}

#[endpoint {
method = GET,
path = "/instance-create"
}]
pub async fn get_create(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let builder = Response::builder();

    if let Some(login) = Session::get_login(&ctx) {
        let title = String::from("Create Instance");

        let nictags = ctx.context().client.get_nictags().await.unwrap();

        // Group images by os + type
        let mut image_list = HashMap::<String, Vec<ImageOption>>::new();
        let mut images = ctx.context().client.get_images().await.unwrap();
        while let Some(image) = images.pop() {
            if image.manifest.state != "active" || image.manifest.disabled {
                continue;
            }
            let key =
                format!("{} {}", image.manifest.os, image.manifest.r#type);
            if let Some(image_vec) = image_list.get_mut(&key) {
                image_vec.push(ImageOption {
                    id: image.manifest.uuid.to_string(),
                    title: image.manifest.description.clone(),
                    name: format!(
                        "{} {}",
                        image.manifest.name, image.manifest.version
                    ),
                });
            } else {
                image_list.insert(
                    key,
                    vec![ImageOption {
                        id: image.manifest.uuid.to_string(),
                        title: image.manifest.description.clone(),
                        name: format!(
                            "{} {}",
                            image.manifest.name, image.manifest.version
                        ),
                    }],
                );
            }
        }

        let template = InstanceCreateTemplate {
            title,
            login,
            images: image_list,
            nictags,
        };
        let result = template.render().unwrap();

        return Ok(builder
            .status(StatusCode::OK)
            .header("HX-Push-Url", String::from("/instance-create"))
            .header("Content-Type", "text/html")
            .body(result.into())?);
    }

    Ok(redirect_login(builder, &ctx)?)
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct CreateRequestBody {
    pub alias: Option<String>,
    pub image_uuid: Uuid,
    pub ram: Option<u64>,
    pub quota: Option<u64>,
    pub nic_tag: String,
    pub ip: String,
    pub gateway: String,
}

#[endpoint {
method = POST,
path = "/instances",
}]
pub async fn post_create(
    ctx: RequestContext<Context>,
    request_body: TypedBody<CreateRequestBody>,
) -> Result<Response<Body>, HttpError> {
    let builder = Response::builder();
    let req = request_body.into_inner();
    if let Some(uuid) = Session::get_uuid(&ctx) {
        let exec_req = CreatePayload {
            alias: req.alias,
            brand: String::from("joyent"),
            resolvers: vec![String::from("8.8.8.8"), String::from("8.8.4.4")],
            ram: req.ram.unwrap_or_default(),
            max_lwps: 4096,
            autoboot: true,
            nics: vec![Nic {
                nic_tag: req.nic_tag,
                ips: vec![req.ip],
                gateways: vec![req.gateway],
                primary: true,
            }],
            image_uuid: req.image_uuid,
            quota: req.quota.unwrap_or_default(),
            owner_uuid: uuid,
        };
        ctx.context()
            .client
            .create_instance(exec_req)
            .await
            .unwrap();
        return Ok(HXLocation::common(builder, "/instances")?);
    }

    Ok(redirect_login(builder, &ctx)?)
}
