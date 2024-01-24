/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{redirect_login, Context};
use crate::session::Session;
use smartos_shared::image::Image;

use askama::Template;
use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response, StatusCode};

#[derive(Template)]
#[template(path = "images.j2")]
pub struct ImagesTemplate {
    title: String,
    images: Vec<Image>,
}

#[endpoint {
method = GET,
path = "/images"
}]
pub async fn get_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();

    if Session::get_login(&ctx).is_some() {
        let title = String::from("Instances");
        let images = ctx.context().client.get_images().await.unwrap();

        let template = ImagesTemplate { title, images };
        let result = template.render().unwrap();

        return Ok(response
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .header("HX-Push-Url", String::from("/images"))
            .body(result.into())?);
    }

    Ok(redirect_login(response, &ctx)?)
}
