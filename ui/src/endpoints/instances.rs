/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use std::collections::HashMap;

use crate::endpoints::{get_header, Context, PathParams};
use crate::session::Session;

use smartos_shared::instance::Instance;

use askama::Template;
use dropshot::{endpoint, HttpError, Path, RequestContext};
use hyper::{Body, Response, StatusCode};

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
    let path = path_params.into_inner();
    let is_htmx = get_header(&ctx, "HX-Request").is_some();
    if let Some(login) = Session::get_login(&ctx) {
        let instance =
            ctx.context().client.get_instance(&path.id).await.unwrap();
        let title = format!("Instance: {}", instance.alias);
        let template = InstanceTemplate {
            title,
            login,
            instance,
        };
        let result = template.render().unwrap();

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Push-Url", format!("/instances/{}", &path.id))
            .header("Content-Type", "text/html")
            .body(result.into())
            .unwrap());
    }

    if is_htmx {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Refresh", "true")
            .body(Body::empty())
            .unwrap());
    }

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", "/login?e=auth")
        .body(Body::empty())
        .unwrap())
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
    let is_htmx = get_header(&ctx, "HX-Request").is_some();
    let title = String::from("Instances");
    if let Some(login) = Session::get_login(&ctx) {
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

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .header("HX-Push-Url", String::from("/instances"))
            .body(result.into())
            .unwrap());
    }

    if is_htmx {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Refresh", "true")
            .body(Body::empty())
            .unwrap());
    }

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", "/login")
        .body(Body::empty())
        .unwrap())
}

#[derive(Template)]
#[template(path = "instance-create.j2")]
pub struct InstanceCreateTemplate {
    title: String,
    login: String,
    images: HashMap<String, Vec<ImageOption>>,
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
pub async fn create(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let is_htmx = get_header(&ctx, "HX-Request").is_some();
    let title = String::from("Create Instance");

    if let Some(login) = Session::get_login(&ctx) {
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
                    id: image.manifest.uuid.clone(),
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
                        id: image.manifest.uuid.clone(),
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
        };
        let result = template.render().unwrap();

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .header("HX-Push-Url", String::from("/instance-create"))
            .body(result.into())
            .unwrap());
    }

    if is_htmx {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Refresh", "true")
            .body(Body::empty())
            .unwrap());
    }

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", "/login")
        .body(Body::empty())
        .unwrap())
}
