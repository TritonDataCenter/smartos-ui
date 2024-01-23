/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{get_header, Context};
use crate::session::Session;
use smartos_shared::image::Image;

use askama::Template;
use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};

#[derive(Template)]
#[template(path = "images.j2")]
pub struct ImagesTemplate {
    title: String,
    login: String,
    images: Vec<Image>,
}

#[endpoint {
method = GET,
path = "/images"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let is_htmx = get_header(&ctx, "HX-Request").is_some();
    let title = String::from("Instances");
    if let Some(login) = Session::get_login(&ctx) {
        let images = ctx.context().client.get_images().await.unwrap();

        let template = ImagesTemplate {
            title,
            login,
            images,
        };
        let result = template.render().unwrap();

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .header("HX-Push-Url", String::from("/images"))
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
