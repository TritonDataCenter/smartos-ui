/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::collections::HashMap;
use std::str::FromStr;

use crate::endpoints::{
    htmx_response, redirect_login, to_internal_error, Context, HXLocation,
    PathParams,
};
use crate::session::Session;

use smartos_shared::instance::{
    Brand, Instance, InstancePayload, PayloadContainer,
};
use smartos_shared::nictag::NicTag;

use askama::Template;
use dropshot::{endpoint, HttpError, Path, Query, RequestContext, TypedBody};
use http::StatusCode;
use hyper::{Body, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use smartos_shared::http_server::to_bad_request;
use smartos_shared::image::Image;
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
        let total_ram =
            instances.iter().fold(0, |acc, i| i.max_physical_memory + acc);
        let total_quota = instances.iter().fold(0, |acc, i| {
            if let Some(quota) = i.quota {
                return quota + acc;
            }
            acc
        });
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
    images: HashMap<String, Vec<Image>>,
    selected_image: Option<Image>,
    nictags: Vec<NicTag>,
    alias: String,
    brand: Brand,
    image_uuid: String,
    ram: String,
    quota: String,
    nic_tag: String,
    nic_setup: String,
    nic_ips: String,
    nic_gateways: String,
    resolvers: String,
    vcpus: String,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct ProvisionQuery {
    #[serde(default)]
    alias: String,
    #[serde(default)]
    brand: String,
    #[serde(default)]
    image_uuid: String,
    #[serde(default)]
    ram: String,
    #[serde(default)]
    quota: String,
    #[serde(default)]
    nic_tag: String,
    #[serde(default)]
    nic_setup: String,
    #[serde(default)]
    nic_ips: String,
    #[serde(default)]
    nic_gateways: String,
    #[serde(default)]
    resolvers: String,
    #[serde(default)]
    vcpus: String,
}

#[endpoint {
method = GET,
path = "/provision"
}]
pub async fn get_provision(
    ctx: RequestContext<Context>,
    query: Query<ProvisionQuery>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();

    if Session::get_login(&ctx).is_some() {
        let mut selected_image = None;
        let ProvisionQuery {
            alias,
            brand,
            image_uuid,
            ram,
            quota,
            nic_tag,
            nic_setup,
            nic_ips,
            nic_gateways,
            resolvers,
            vcpus,
        } = query.into_inner();
        let actual_brand = Brand::from_str(&brand).unwrap_or_default();

        let title = String::from("Create Instance");

        let nictags = ctx
            .context()
            .client
            .get_nictags()
            .await
            .map_err(to_internal_error)?;

        let mut image_list = HashMap::<String, Vec<Image>>::new();
        let mut images = ctx
            .context()
            .client
            .get_images()
            .await
            .map_err(to_internal_error)?;

        while let Some(image) = images.pop() {
            if image_uuid == image.manifest.uuid.to_string() {
                selected_image = Some(image.clone())
            }
            let virt_type = if image.is_for_hvm() {
                "Hardware Virtualization"
            } else {
                "Native Virtualization"
            };
            let key = format!(
                "{} ({} {})",
                virt_type, image.manifest.os, image.manifest.r#type
            );
            if let Some(image_vec) = image_list.get_mut(&key) {
                image_vec.push(image);
            } else {
                image_list.insert(key, vec![image]);
            }
        }

        let template = InstanceCreateTemplate {
            title,
            images: image_list,
            selected_image,
            nictags,
            alias,
            brand: actual_brand,
            image_uuid,
            ram,
            quota,
            nic_tag,
            nic_setup,
            nic_ips,
            nic_gateways,
            resolvers,
            vcpus,
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
content_type = "application/x-www-form-urlencoded"
}]
pub async fn post_provision(
    ctx: RequestContext<Context>,
    request_body: TypedBody<InstancePayload>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();

    if Session::get_login(&ctx).is_some() {
        let req = request_body.into_inner();

        let PayloadContainer { uuid } =
            serde_json::from_str(&req.payload).map_err(to_bad_request)?;

        ctx.context().client.provision(req).await.map_err(to_internal_error)?;
        let mut location = HXLocation::new_with_common("/instances");
        location.values = Some(json!({
            "longRunning": true,
            "allowedPaths": [],
            "notification": {
                "heading": "Instance created",
                "body": format!("Instance {} has been created.", uuid)
            }
        }));
        return location.serve(response);
    }

    redirect_login(response, &ctx)
}

#[derive(Template)]
#[template(path = "validate_create_result.j2")]
pub struct ValidateTemplate {
    success: bool,
    message: String,
}

#[endpoint {
method = POST,
path = "/provision/validate",
content_type = "application/x-www-form-urlencoded"
}]
pub async fn post_provision_validate(
    ctx: RequestContext<Context>,
    request_body: TypedBody<InstancePayload>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if Session::get_login(&ctx).is_some() {
        let req = request_body.into_inner();
        let validation = ctx
            .context()
            .client
            .validate_create(req)
            .await
            .map_err(to_internal_error)?;
        let template = ValidateTemplate {
            success: validation.success,
            message: validation.message,
        };
        let result = template.render().map_err(to_internal_error)?;
        return response
            .status(StatusCode::OK)
            .body(result.into())
            .map_err(to_internal_error);
    }

    redirect_login(response, &ctx)
}
