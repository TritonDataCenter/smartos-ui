/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::str::FromStr;

use crate::endpoints::{
    htmx_response, redirect_login, to_internal_error, Context, HXLocation,
    PathParams,
};
use crate::session::Session;

use smartos_shared::instance::{Brand, Instance, InstancePayload};
use smartos_shared::nictag::NicTag;

use askama::Template;
use dropshot::{endpoint, HttpError, Path, Query, RequestContext, TypedBody};
use http::StatusCode;
use hyper::{Body, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
pub struct InstanceCreateTemplate<'a> {
    title: String,
    images: Vec<&'a Image>,
    nictags: Vec<NicTag>,
    default_image_message: &'a str,
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
pub struct ImageOption {
    pub id: String,
    pub title: String,
    pub name: String,
    pub for_brand: Vec<String>,
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
        let mut default_image_message = "Select an Image";

        let nictags = ctx
            .context()
            .client
            .get_nictags()
            .await
            .map_err(to_internal_error)?;

        let images: Vec<Image> = if actual_brand == Brand::Other {
            default_image_message = "Select a Brand to choose an Image";
            vec![]
        } else {
            ctx.context()
                .client
                .get_images()
                .await
                .map_err(to_internal_error)?
        };

        let image_list = images
            .iter()
            .filter(|image| actual_brand.for_image_type(&image.manifest.r#type))
            .collect();

        let template = InstanceCreateTemplate {
            title,
            images: image_list,
            nictags,
            default_image_message,
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

// Needs to be updated to handle simple string payload
// #[endpoint {
// method = POST,
// path = "/provision",
// }]
// pub async fn post_provision(
//     ctx: RequestContext<Context>,
//     request_body: TypedBody<CreateRequestBody>,
// ) -> Result<Response<Body>, HttpError> {
//     let response = Response::builder();
//     let req = request_body.into_inner();
//     if Session::get_login(&ctx).is_some() {
//         let exec_req = CreatePayload {
//             alias: req.alias,
//             brand: String::from("joyent"),
//             resolvers: vec![String::from("8.8.8.8"), String::from("8.8.4.4")],
//             ram: req.ram.unwrap_or_default(),
//             max_lwps: 4096,
//             autoboot: true,
//             nics: vec![Nic {
//                 nic_tag: req.nic_tag,
//                 ips: vec![req.ip],
//                 gateways: vec![req.gateway],
//                 primary: true,
//             }],
//             image_uuid: req.image_uuid,
//             quota: req.quota.unwrap_or_default(),
//         };
//         ctx.context()
//             .client
//             .create_instance(exec_req)
//             .await
//             .map_err(to_internal_error)?;
//
//         return HXLocation::common(response, "/instances");
//     }
//
//     redirect_login(response, &ctx)
// }

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
    let req = request_body.into_inner();
    if Session::get_login(&ctx).is_some() {
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
