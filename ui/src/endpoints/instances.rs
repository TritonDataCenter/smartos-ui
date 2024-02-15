/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::collections::BTreeMap;
use std::str::FromStr;

use crate::endpoints::{
    filters, htmx_response, redirect_login, Context, NotificationKind,
    NotificationTemplate, PathParams,
};
use crate::session::Session;

use smartos_shared::http_server::to_internal_error;
use smartos_shared::instance::{
    Brand, Instance, InstancePayload, InstanceView, PayloadContainer,
};
use smartos_shared::nictag::NicTag;

use crate::endpoints::filters::format_word;
use askama::Template;
use dropshot::{endpoint, HttpError, Path, Query, RequestContext, TypedBody};
use http::StatusCode;
use hyper::{Body, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smartos_shared::http_server::to_bad_request;
use smartos_shared::image::Image;

#[derive(Template)]
#[template(path = "instance.j2")]
pub struct InstanceTemplate {
    title: String,
    instance_enum: Instance,
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
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }
    let id = path_params.into_inner().id;
    let title = String::from("Instance"); // XXX alias or uuid

    let instance_enum = ctx
        .context()
        .client
        .get_instance(&id)
        .await
        .map_err(to_internal_error)?;

    let template = InstanceTemplate { title, instance_enum };
    let result = template.render().map_err(to_internal_error)?;

    htmx_response(response, &format!("/instances/{}", &id), result.into())
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
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let id = path_params.into_inner().id;

    let template = if ctx.context().client.delete_instance(&id).await.is_ok() {
        NotificationTemplate {
            id: ctx.request_id,
            kind: NotificationKind::Ok,
            subject: String::from("Instance deleted"),
            message: format!("Instance {} successfully deleted", id),
            timeout: Some(String::from("8s")),
            redirect: Some(String::from("/instances")),
            created_at: format!("/instances/{}", id),
        }
    } else {
        NotificationTemplate {
            id: ctx.request_id,
            kind: NotificationKind::Error,
            subject: String::from("Instance could not be deleted"),
            message: format!("Failed to delete instance {}", id),
            timeout: Some(String::from("8s")),
            redirect: None,
            created_at: format!("/instances/{}", id),
        }
    };

    let template_result = template.render().map_err(to_internal_error)?;

    response
        .status(StatusCode::OK)
        .body(template_result.into())
        .map_err(to_internal_error)
}

#[endpoint {
method = POST,
path = "/instances/{id}/start",
}]
pub async fn start_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let id = path_params.into_inner().id;
    let template = if ctx.context().client.start_instance(&id).await.is_ok() {
        NotificationTemplate {
            id: ctx.request_id,
            kind: NotificationKind::Ok,
            subject: String::from("Started"),
            message: format!("Instance {} successfully started", id),
            timeout: Some(String::from("8s")),
            redirect: Some(format!("/instances/{}", id)),
            created_at: format!("/instances/{}", id),
        }
    } else {
        NotificationTemplate {
            id: ctx.request_id,
            kind: NotificationKind::Error,
            subject: String::from("Start Failed"),
            message: format!("Failed to start instance {}", id),
            timeout: Some(String::from("8s")),
            redirect: None,
            created_at: format!("/instances/{}", id),
        }
    };

    let template_result = template.render().map_err(to_internal_error)?;

    response
        .status(StatusCode::OK)
        .body(template_result.into())
        .map_err(to_internal_error)
}

#[endpoint {
method = POST,
path = "/instances/{id}/stop",
}]
pub async fn stop_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let id = path_params.into_inner().id;
    let template = if ctx.context().client.stop_instance(&id).await.is_ok() {
        NotificationTemplate {
            id: ctx.request_id,
            kind: NotificationKind::Ok,
            subject: String::from("Stopped"),
            message: format!("Instance {} successfully stopped", id),
            timeout: Some(String::from("8s")),
            redirect: Some(format!("/instances/{}", id)),
            created_at: format!("/instances/{}", id),
        }
    } else {
        NotificationTemplate {
            id: ctx.request_id,
            kind: NotificationKind::Error,
            subject: String::from("Stop Failed"),
            message: format!("Failed to stop instance {}", id),
            timeout: Some(String::from("8s")),
            redirect: None,
            created_at: format!("/instances/{}", id),
        }
    };

    let template_result = template.render().map_err(to_internal_error)?;

    response
        .status(StatusCode::OK)
        .body(template_result.into())
        .map_err(to_internal_error)
}

#[derive(Template)]
#[template(path = "instances.j2")]
pub struct InstancesTemplate<'a> {
    image_count: usize,
    total_ram: u64,
    total_quota: u64,
    total_cpu: f32,
    title: &'a str,
    instances: Vec<InstanceView>,
}

#[endpoint {
method = GET,
path = "/instances"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let image_count = ctx
        .context()
        .client
        .get_images()
        .await
        .map_err(to_internal_error)?
        .len();

    let instances = ctx
        .context()
        .client
        .get_instances()
        .await
        .map_err(to_internal_error)?;

    let total_ram = instances.iter().fold(0, |acc, i| i.ram + acc);
    let total_quota = instances.iter().fold(0, |acc, i| i.disk_usage + acc);
    let total_cpu = instances.iter().fold(0.0, |acc, i| i.cpu + acc);
    let template = InstancesTemplate {
        image_count,
        total_ram,
        total_quota,
        total_cpu,
        title: "Instances",
        instances,
    };
    let result = template.render().map_err(to_internal_error)?;

    htmx_response(response, "/instances", result.into())
}

#[derive(Template)]
#[template(path = "provision.j2")]
pub struct InstanceCreateTemplate {
    title: String,
    images: BTreeMap<String, Vec<Image>>,
    selected_image: Option<Image>,
    nictags: Vec<NicTag>,
    alias: String,
    brand: Brand,
    image_uuid: String,
    ram: String,
    quota: String,
    nic_tag: String,
    ipv4_setup: String,
    ipv4_ip: String,
    ipv4_gateway: String,
    ipv4_prefix: String,
    ipv6_setup: String,
    ipv6_ip: String,
    ipv6_prefix: String,
    resolvers: String,
    vcpus: String,
    kernel_version: String,
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
    ipv4_setup: String,
    #[serde(default)]
    ipv4_ip: String,
    #[serde(default)]
    ipv4_prefix: String,
    #[serde(default)]
    ipv4_gateway: String,
    #[serde(default)]
    ipv6_setup: String,
    #[serde(default)]
    ipv6_ip: String,
    #[serde(default)]
    ipv6_prefix: String,
    #[serde(default)]
    resolvers: String,
    #[serde(default)]
    vcpus: String,
    #[serde(default)]
    kernel_version: String,
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
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }
    let mut selected_image = None;
    let ProvisionQuery {
        alias,
        brand,
        image_uuid,
        ram,
        quota,
        nic_tag,
        ipv4_setup,
        ipv4_ip,
        ipv4_prefix,
        ipv4_gateway,
        ipv6_setup,
        ipv6_ip,
        ipv6_prefix,
        resolvers,
        vcpus,
        kernel_version,
    } = query.into_inner();
    let actual_brand = Brand::from_str(&brand).unwrap_or_default();

    let title = String::from("Create Instance");

    let nictags =
        ctx.context().client.get_nictags().await.map_err(to_internal_error)?;

    let mut image_list = BTreeMap::<String, Vec<Image>>::new();
    let mut images =
        ctx.context().client.get_images().await.map_err(to_internal_error)?;

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
            virt_type,
            format_word(&image.manifest.os).unwrap_or_default(),
            format_word(&image.manifest.r#type).unwrap_or_default()
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
        ipv4_setup,
        ipv4_ip,
        ipv4_gateway,
        ipv4_prefix,
        ipv6_setup,
        ipv6_ip,
        ipv6_prefix,
        resolvers,
        vcpus,
        kernel_version,
    };
    let result = template.render().map_err(to_internal_error)?;
    htmx_response(response, "/provision", result.into())
}

struct Button {
    pub text: String,
    pub classes: Vec<String>,
    pub attributes: Vec<String>,
}

#[derive(Template)]
#[template(path = "modal.j2")]
pub struct ProvisionTemplate {
    kind: NotificationKind,
    subject: String,
    message: String,
    buttons: Option<Vec<Button>>,
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
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let req = request_body.into_inner();

    let PayloadContainer { uuid } =
        serde_json::from_str(&req.payload).map_err(to_bad_request)?;

    let result =
        ctx.context().client.provision(req).await.map_err(to_internal_error)?;

    let exec_result = if result.status().is_success() {
        let buttons = vec![
            Button {
                text: String::from("Ok"),
                classes: vec![String::from("btn-primary")],
                attributes: vec![
                    String::from("data-hx-get=\"/instances\""),
                    String::from("data-hx-target=\"#main\""),
                    String::from("data-hx-select=\"#content\""),
                ],
            },
            Button {
                text: String::from("View Instance"),
                classes: vec![String::from("btn-clear")],
                attributes: vec![
                    format!("data-hx-get=\"/instances/{}\"", uuid),
                    String::from("data-hx-target=\"#main\""),
                    String::from("data-hx-select=\"#content\""),
                ],
            },
        ];
        let message = result.text().await.expect("failed");
        let template = ProvisionTemplate {
            kind: NotificationKind::Ok,
            subject: String::from("Provision successful"),
            message: format!("Instance {} created. {}", uuid, message),
            buttons: Some(buttons),
        };
        template.render().map_err(to_internal_error)?
    } else {
        let message = result.text().await.expect("failed");
        let template = ProvisionTemplate {
            kind: NotificationKind::Error,
            subject: String::from("Provision failed"),
            message,
            buttons: None,
        };
        template.render().map_err(to_internal_error)?
    };

    response
        .status(StatusCode::OK)
        .body(exec_result.into())
        .map_err(to_internal_error)
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
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

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
    response
        .status(StatusCode::OK)
        .body(result.into())
        .map_err(to_internal_error)
}
