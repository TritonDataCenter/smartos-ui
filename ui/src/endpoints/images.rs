/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{
    htmx_response, redirect_login, to_internal_error, Context,
};
use crate::session::Session;
use smartos_shared::image::Image;

use askama::Template;
use dropshot::{endpoint, HttpError, RequestContext};
use hyper::{Body, Response};

#[derive(Template)]
#[template(path = "images.j2")]
pub struct ImagesTemplate<'a> {
    title: &'a str,
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
        let images = ctx
            .context()
            .client
            .get_images()
            .await
            .map_err(to_internal_error)?;

        let template = ImagesTemplate {
            title: "Images",
            images,
        };
        let result = template.render().map_err(to_internal_error)?;

        return htmx_response(response, "/images", result.into());
    }

    redirect_login(response, &ctx)
}
