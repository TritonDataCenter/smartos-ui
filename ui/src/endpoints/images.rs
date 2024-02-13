/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::filters;
use crate::endpoints::{
    htmx_response, redirect_login, to_internal_error, Context, HXLocation,
    PathParams,
};
use crate::session::Session;

use smartos_shared::image::{Image, ImageImportParams};

use askama::Template;
use dropshot::{endpoint, HttpError, Path, RequestContext, TypedBody};
use hyper::{Body, Response};
use serde_json::json;

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

        let template = ImagesTemplate { title: "Images", images };
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
        let id = path_params.into_inner().id;
        ctx.context()
            .client
            .delete_image(&id)
            .await
            .map_err(to_internal_error)?;

        let mut location = HXLocation::new_with_common("/images");
        location.values = Some(json!({
            "allowedPaths": [format!("/images/{}", id)],
            "notification": {
                "heading": "Image deletion complete",
                "body": format!("Image {} has been deleted", id)
            }
        }));
        return location.serve(response);
    }
    redirect_login(response, &ctx)
}

#[derive(Template)]
#[template(path = "import.j2")]
pub struct ImportTemplate {
    title: String,
    images: Vec<Image>,
}

// TODO: if you navigate here, then quickly navigate elsewhere before it has
// loaded, you will be redirected here when it later loads, instead of aborting
// the request like the other side-panel navigation entries (need to grok how to
// make hx-sync work here too.)
#[endpoint {
method = GET,
path = "/import",
}]
pub async fn get_import_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if Session::get_login(&ctx).is_some() {
        let mut images = ctx
            .context()
            .client
            .get_available_images()
            .await
            .map_err(to_internal_error)?;
        images.reverse();
        let template =
            ImportTemplate { title: "Available Images".to_string(), images };

        let result = template.render().map_err(to_internal_error)?;
        return htmx_response(response, "/import", result.into());
    }

    redirect_login(response, &ctx)
}

#[endpoint {
method = POST,
path = "/import/{id}",
content_type = "application/x-www-form-urlencoded"
}]
pub async fn post_import_index(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
    request_body: TypedBody<ImageImportParams>,
) -> Result<Response<Body>, HttpError> {
    let request = request_body.into_inner();
    let id = &path_params.into_inner().id;
    let response = Response::builder();
    if Session::get_login(&ctx).is_some() {
        ctx.context()
            .client
            .import_image(id, &request)
            .await
            .map_err(to_internal_error)?;

        let mut location = HXLocation::new_with_common("/images");
        location.values = Some(json!({
            "allowedPaths": [],
            "notification": {
                "heading": "Image import complete",
                "body": format!("Image {} has been imported and is ready to use.", id)
            }
        }));
        return location.serve(response);
    }
    redirect_login(response, &ctx)
}
