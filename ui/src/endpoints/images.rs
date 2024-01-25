/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{
    htmx_response, redirect_login, to_internal_error, Context, HXLocation,
    PathParams,
};
use crate::session::Session;
use smartos_shared::image::Image;

use askama::Template;
use dropshot::{endpoint, HttpError, Path, RequestContext};
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

#[derive(Template)]
#[template(path = "image.j2")]
pub struct ImageTemplate {
    title: String,
    image: Image,
}

#[endpoint {
method = GET,
path = "/images/{id}"
}]
pub async fn get_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();

    if Session::get_login(&ctx).is_some() {
        let id = path_params.into_inner().id;
        let image = ctx
            .context()
            .client
            .get_image(&id)
            .await
            .map_err(to_internal_error)?;

        let template = ImageTemplate {
            title: format!("Image: {}", image.manifest.name),
            image,
        };
        let result = template.render().map_err(to_internal_error)?;

        return htmx_response(
            response,
            &format!("/images/{}", id),
            result.into(),
        );
    }

    redirect_login(response, &ctx)
}

#[endpoint {
method = DELETE,
path = "/images/{id}",
}]
pub async fn delete_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if Session::get_login(&ctx).is_some() {
        ctx.context()
            .client
            .delete_image(&path_params.into_inner().id)
            .await
            .map_err(to_internal_error)?;

        return HXLocation::common(response, "/images");
    }
    redirect_login(response, &ctx)
}
