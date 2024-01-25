/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::collections::HashMap;

use crate::endpoints::{
    htmx_response, redirect_login, to_internal_error, Context, HXLocation,
    PathParams,
};
use crate::session::Session;

use smartos_shared::instance::{CreatePayload, Instance, Nic};
use smartos_shared::nictag::NicTag;

use askama::Template;
use dropshot::{endpoint, HttpError, Path, RequestContext, TypedBody};
use hyper::{Body, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "instance.j2")]
pub struct InstanceTemplate {
    title: String,
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
    let response = Response::builder();
    if Session::get_login(&ctx).is_some() {
        let id = path_params.into_inner().id;
        let instance = ctx
            .context()
            .client
            .get_instance(&id)
            .await
            .map_err(to_internal_error)?;

        let title = if let Some(alias) = &instance.alias {
            format!("Instance: {}", alias)
        } else {
            format!("Instance: {}", instance.uuid)
        };

        let template = InstanceTemplate { title, instance };
        let result = template.render().map_err(to_internal_error)?;
        return htmx_response(
            response,
            &format!("/instances/{}", &id),
            result.into(),
        );
    }

    redirect_login(response, &ctx)
}

#[endpoint {
method = DELETE,
path = "/instances/{id}",
}]
pub async fn delete_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if Session::get_login(&ctx).is_some() {
        ctx.context()
            .client
            .delete_instance(&path_params.into_inner().id)
            .await
            .map_err(to_internal_error)?;

        return HXLocation::common(response, "/instances");
    }
    redirect_login(response, &ctx)
}

#[derive(Template)]
#[template(path = "instances.j2")]
pub struct InstancesTemplate<'a> {
    total_ram: u64,
    total_quota: u64,
    title: &'a str,
    instances: Vec<Instance>,
}

#[endpoint {
method = GET,
path = "/instances"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if Session::get_login(&ctx).is_some() {
        let instances = ctx
            .context()
            .client
            .get_instances()
            .await
            .map_err(to_internal_error)?;
        let total_ram = instances
            .iter()
            .fold(0, |acc, i| i.max_physical_memory + acc);
        let total_quota = instances.iter().fold(0, |acc, i| i.quota + acc);
        let template = InstancesTemplate {
            total_ram,
            total_quota,
            title: "Instances",
            instances,
        };
        let result = template.render().map_err(to_internal_error)?;
        return htmx_response(response, "/instances", result.into());
    }

    redirect_login(response, &ctx)
}

#[derive(Template)]
#[template(path = "provision.j2")]
pub struct InstanceCreateTemplate {
    title: String,
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
path = "/provision"
}]
pub async fn get_provision(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();

    if Session::get_login(&ctx).is_some() {
        let title = String::from("Create Instance");

        let nictags = ctx
            .context()
            .client
            .get_nictags()
            .await
            .map_err(to_internal_error)?;

        // Group images by os + type
        let mut image_list = HashMap::<String, Vec<ImageOption>>::new();
        let mut images = ctx
            .context()
            .client
            .get_images()
            .await
            .map_err(to_internal_error)?;
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
            images: image_list,
            nictags,
        };
        let result = template.render().map_err(to_internal_error)?;
        return htmx_response(response, "/provision", result.into());
    }

    redirect_login(response, &ctx)
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
path = "/provision",
}]
pub async fn post_provision(
    ctx: RequestContext<Context>,
    request_body: TypedBody<CreateRequestBody>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    let req = request_body.into_inner();
    if Session::get_login(&ctx).is_some() {
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
        };
        ctx.context()
            .client
            .create_instance(exec_req)
            .await
            .map_err(to_internal_error)?;

        return HXLocation::common(response, "/instances");
    }

    redirect_login(response, &ctx)
}
