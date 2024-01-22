/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{get_header, Context, PathParams};
use crate::session::Session;
use smartos_shared::instance::{Instance, ListInstance};

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

#[derive(Template)]
#[template(path = "instance-hx.j2")]
pub struct InstanceHxTemplate {
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
    let path = path_params.into_inner();
    let is_htmx = get_header(&ctx, "HX-Request").is_some();
    if let Some(login) = Session::get_login(&ctx) {
        let instance =
            ctx.context().client.get_instance(&path.id).await.unwrap();
        let title = format!("Instance: {}", instance.alias);

        let result = if is_htmx {
            let template = InstanceHxTemplate { title, instance };
            template.render().unwrap()
        } else {
            let template = InstanceTemplate {
                title,
                login,
                instance,
            };
            template.render().unwrap()
        };

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
    total_ram: u32,
    total_quota: u32,
    title: String,
    login: String,
    instances: Vec<ListInstance>,
}

#[derive(Template)]
#[template(path = "instances-hx.j2")]
pub struct InstancesHxTemplate {
    total_ram: u32,
    total_quota: u32,
    title: String,
    instances: Vec<ListInstance>,
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
        let total_ram = instances.iter().fold(0, |acc, i| i.ram + acc);
        let total_quota = instances.iter().fold(0, |acc, i| i.quota + acc);
        let result = if is_htmx {
            let template = InstancesHxTemplate {
                total_ram,
                total_quota,
                title,
                instances,
            };
            template.render().unwrap()
        } else {
            let template = InstancesTemplate {
                total_ram,
                total_quota,
                title,
                login,
                instances,
            };
            template.render().unwrap()
        };
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
}

#[derive(Template)]
#[template(path = "instance-create-hx.j2")]
pub struct InstanceCreateHxTemplate {
    title: String,
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
        let result = if is_htmx {
            let template = InstanceCreateHxTemplate { title };
            template.render().unwrap()
        } else {
            let template = InstanceCreateTemplate { title, login };
            template.render().unwrap()
        };
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
