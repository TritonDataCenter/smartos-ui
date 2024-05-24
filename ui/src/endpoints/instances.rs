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
    filters, htmx_response, redirect_login, AsJson, Context, NotificationKind,
    NotificationTemplate, PathParams,
};
use crate::session;

use smartos_shared::{
    http_server::to_bad_request, http_server::to_internal_error, image::Image,
    image::Type as ImageType, instance::Brand, instance::Info,
    instance::Instance, instance::InstancePayload, instance::InstanceView,
    instance::PayloadContainer, nictag::NicTag, sysinfo::Sysinfo,
};

use askama::Template;
use dropshot::{endpoint, HttpError, Path, Query, RequestContext, TypedBody};
use http::StatusCode;
use hyper::{Body, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slog::{error, info};
use uuid::Builder as UuidBuilder;

#[derive(Template)]
#[template(path = "instance.j2")]
pub struct InstanceTemplate {
    title: String,
    instance_enum: Instance,
    json: Option<String>,
    info: Option<Info>,
    image: Option<Image>,
}

#[endpoint {
method = GET,
path = "/instances/{id}",
}]
pub async fn get_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
    query_params: Query<AsJson>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if !session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }
    let id = path_params.into_inner().id;

    let instance_enum = ctx
        .context()
        .vminfod
        .get_instance(&id)
        .await
        .map_err(to_internal_error)?;

    let title = format!("Instance: {}", instance_enum.alias());

    let mut info: Option<Info> = None;
    let mut json_string = None;
    let mut location = format!("/instances/{}", &id);
    if let Some(as_json) = query_params.into_inner().json {
        if as_json {
            json_string = Some(
                ctx.context()
                    .vminfod
                    .get_instance_json(&id)
                    .await
                    .map_err(to_internal_error)?,
            );
            location = format!("/instances/{}?json=true", id)
        }
    } else if instance_enum.is_hvm() && instance_enum.state() == "running" {
        info = Some(
            ctx.context()
                .executor
                .info(&instance_enum.uuid())
                .await
                .map_err(to_internal_error)?,
        );
    }

    let image_uuid = instance_enum.image_uuid();

    let image = if image_uuid != UuidBuilder::nil().into_uuid() {
        match ctx.context().executor.get_image(&image_uuid).await {
            Ok(image) => Some(image),
            Err(error) => {
                error!(
                    ctx.log,
                    "Failed getting image details for {}: {}",
                    image_uuid,
                    error.to_string()
                );
                None
            }
        }
    } else {
        None
    };

    let template = InstanceTemplate {
        title,
        instance_enum,
        json: json_string,
        info,
        image,
    };
    let result = template.render().map_err(to_internal_error)?;

    htmx_response(response, &location, result.into())
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
    if !session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let id = path_params.into_inner().id;
    let instance = ctx
        .context()
        .vminfod
        .get_instance_view(&id)
        .await
        .map_err(to_internal_error)?;
    let template = if ctx.context().executor.delete_instance(&id).await.is_ok()
    {
        NotificationTemplate {
            id: ctx.request_id,
            entity_id: id.to_string(),
            kind: NotificationKind::Ok,
            subject: String::from("Instance deleted"),
            message: format!(
                "Instance {} successfully deleted",
                instance.alias
            ),
            timeout: Some(String::from("8s")),
            redirect: Some(String::from("/instances")),
            created_at: format!("/instances/{}", id),
        }
    } else {
        NotificationTemplate {
            id: ctx.request_id,
            entity_id: id.to_string(),
            kind: NotificationKind::Error,
            subject: String::from("Instance could not be deleted"),
            message: format!("Failed to delete instance {}", instance.alias),
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
    if !session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let id = path_params.into_inner().id;
    let template = if ctx.context().executor.start_instance(&id).await.is_ok() {
        NotificationTemplate {
            id: ctx.request_id,
            entity_id: id.to_string(),
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
            entity_id: id.to_string(),
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
    if !session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let id = path_params.into_inner().id;
    let template = if ctx.context().executor.stop_instance(&id).await.is_ok() {
        NotificationTemplate {
            id: ctx.request_id,
            entity_id: id.to_string(),
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
            entity_id: id.to_string(),
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
    provisioned_ram: u64,
    total_ram: u64,
    provisioned_quota: u64,
    total_quota: u64,
    provisioned_cpu: f32,
    total_cpu: u64,
    title: &'a str,
    instances: Vec<(InstanceView, String)>,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct InstanceListParams {
    #[serde(default)]
    pub reload: Option<bool>,
}

#[endpoint {
method = GET,
path = "/instances"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
    query_params: Query<InstanceListParams>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if !session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }
    let mut instances: Vec<(InstanceView, String)> = Vec::new();

    let Sysinfo { cpu_count, mib_of_memory, zpool_size_in_gib, .. } = ctx
        .context()
        .executor
        .get_sysinfo()
        .await
        .map_err(to_internal_error)?;

    let images = ctx
        .context()
        .executor
        .get_images(&ctx.log)
        .await
        .map_err(to_internal_error)?;
    let image_count = images.len();

    let mut instance_views = ctx
        .context()
        .vminfod
        .get_instances()
        .await
        .map_err(to_internal_error)?;

    let provisioned_ram = instance_views.iter().fold(0, |acc, i| {
        if i.ram == 0 {
            mib_of_memory + acc
        } else {
            i.ram + acc
        }
    });
    let provisioned_quota = instance_views.iter().fold(0, |acc, i| {
        if i.disk_usage == 0 {
            (zpool_size_in_gib * 1024) + acc
        } else {
            i.disk_usage + acc
        }
    });

    let provisioned_cpu = instance_views.iter().fold(0.0, |acc, i| {
        if i.cpu == 0.0 {
            acc + cpu_count as f32
        } else {
            i.cpu + acc
        }
    });

    for instance in instance_views.drain(..) {
        let image_name = if let Some(image) =
            images.iter().find(|&i| i.manifest.uuid == instance.image_uuid)
        {
            format!("{}@{}", image.manifest.name, image.manifest.version)
        } else {
            instance.image_uuid.to_string().clone()
        };
        instances.push((instance, image_name));
    }

    let template = InstancesTemplate {
        image_count,
        provisioned_ram,
        total_ram: mib_of_memory,
        provisioned_quota,
        total_quota: zpool_size_in_gib,
        provisioned_cpu,
        total_cpu: cpu_count,
        title: "Instances",
        instances,
    };
    let result = template.render().map_err(to_internal_error)?;

    if query_params.into_inner().reload.is_some() {
        let event_data = json!({
            "filterTable": {
                "selector": "#filter-instances"
            }
        })
        .to_string();
        response
            .status(StatusCode::OK)
            .header("HX-Push-Url", "/instances")
            .header("HX-Trigger-After-Swap", event_data)
            .header("Content-Type", "text/html")
            .body(result.into())
            .map_err(to_internal_error)
    } else {
        htmx_response(response, "/instances", result.into())
    }
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
    primary_disk_size: u64,
    root_authorized_keys: String,
    delegate_dataset: String,
    root_pw: String,
    bootrom: String,
    cpu_cap: String,
    user_script: String,
    cloudinit_data: String,
    dns_domain: String,
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
    primary_disk_size: u64,
    #[serde(default)]
    root_authorized_keys: String,
    #[serde(default)]
    delegate_dataset: String,
    #[serde(default)]
    root_pw: String,
    #[serde(default)]
    bootrom: String,
    #[serde(default)]
    cpu_cap: String,
    #[serde(default)]
    user_script: String,
    #[serde(default)]
    cloudinit_data: String,
    #[serde(default)]
    dns_domain: String,
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
    if !session::is_valid(&ctx) {
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
        primary_disk_size,
        root_authorized_keys,
        delegate_dataset,
        root_pw,
        bootrom,
        cpu_cap,
        user_script,
        cloudinit_data,
        dns_domain,
    } = query.into_inner();

    let mut selected_brand = Brand::default();

    let title = String::from("Create Instance");

    let nictags = ctx
        .context()
        .executor
        .get_nictags()
        .await
        .map_err(to_internal_error)?;

    let mut image_list = BTreeMap::<String, Vec<Image>>::new();
    let mut images = ctx
        .context()
        .executor
        .get_images(&ctx.log)
        .await
        .map_err(to_internal_error)?;

    while let Some(image) = images.pop() {
        if image_uuid == image.manifest.uuid.to_string() {
            // Use the previously chosen brand if exists, else choose a default
            if let Ok(request_brand) = Brand::from_str(&brand) {
                info!(ctx.log, "brand from request: {}", request_brand);

                // make sure this brand is valid for the image type
                // needed if user selects a brand, then changes the image the
                // conditions we have set for brands will not work as expected
                if image.valid_brand(&request_brand) {
                    info!(ctx.log, "using chosen brand: {}", request_brand);
                    selected_brand = request_brand
                } else {
                    selected_brand = image.default_brand();
                    info!(
                        ctx.log,
                        "using default brand instead: {}", selected_brand
                    );
                }
            } else {
                selected_brand = image.default_brand();
                info!(ctx.log, "brand from image default: {}", selected_brand);
            }
            selected_image = Some(image.clone())
        }

        if let Some(image_vec) = image_list.get_mut(&image.group_name()) {
            image_vec.push(image);
        } else {
            image_list.insert(image.group_name(), vec![image]);
        }
    }

    let template = InstanceCreateTemplate {
        title,
        images: image_list,
        selected_image,
        nictags,
        alias,
        brand: selected_brand,
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
        primary_disk_size,
        root_authorized_keys,
        delegate_dataset,
        root_pw,
        bootrom,
        cpu_cap,
        user_script,
        cloudinit_data,
        dns_domain,
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
    if !session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let req = request_body.into_inner();

    let PayloadContainer { uuid } =
        serde_json::from_str(&req.payload).map_err(to_bad_request)?;

    let result = ctx
        .context()
        .executor
        .provision(req)
        .await
        .map_err(to_internal_error)?;

    let exec_result = if result.status().is_success() {
        let buttons = vec![
            Button {
                text: String::from("Instance Details"),
                classes: vec![
                    String::from("btn-primary"),
                    String::from("btn-instance-details"),
                ],
                attributes: vec![
                    format!("data-hx-get=\"/instances/{}\"", uuid),
                    String::from("data-hx-target=\"#main\""),
                    String::from("data-hx-select=\"#content\""),
                ],
            },
            Button {
                text: String::from("Instance List"),
                classes: vec![
                    String::from("btn-clear"),
                    String::from("btn-instance-list"),
                ],
                attributes: vec![
                    String::from("data-hx-get=\"/instances\""),
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
#[template(path = "validate.j2")]
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
    if !session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let req = request_body.into_inner();
    let validation = ctx
        .context()
        .executor
        .validate_create(req)
        .await
        .map_err(to_internal_error)?;

    let trigger = json!({
        "validationResult": {
            "valid": validation.success
        }
    })
    .to_string();

    let template = ValidateTemplate {
        success: validation.success,
        message: validation.message,
    };
    let template_result = template.render().map_err(to_internal_error)?;
    response
        .status(StatusCode::OK)
        .header("HX-Trigger", trigger)
        .body(template_result.into())
        .map_err(to_internal_error)
}
