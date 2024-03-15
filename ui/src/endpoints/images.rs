/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use crate::endpoints::{
    filters, htmx_response, redirect_login, AsJson, Context, NotificationKind,
    NotificationTemplate, PathParams,
};
use crate::session::Session;

use smartos_shared::image::{Image, ImageImportParams};

use askama::Template;
use dropshot::{endpoint, HttpError, Path, Query, RequestContext, TypedBody};
use http::StatusCode;
use hyper::{Body, Response};
use smartos_shared::http_server::{to_internal_error, GenericResponse};

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
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let images =
        ctx.context().executor.get_images().await.map_err(to_internal_error)?;

    let template = ImagesTemplate { title: "Images", images };
    let result = template.render().map_err(to_internal_error)?;

    htmx_response(response, "/images", result.into())
}

#[derive(Template)]
#[template(path = "image.j2")]
pub struct ImageTemplate {
    title: String,
    image: Image,
    json: Option<String>,
}

#[endpoint {
method = GET,
path = "/images/{id}"
}]
pub async fn get_by_id(
    ctx: RequestContext<Context>,
    path_params: Path<PathParams>,
    query_params: Query<AsJson>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let id = path_params.into_inner().id;
    let image = ctx
        .context()
        .executor
        .get_image(&id)
        .await
        .map_err(to_internal_error)?;

    let mut location = format!("/images/{}", id);
    let mut json_string = None;
    if let Some(as_json) = query_params.into_inner().json {
        if as_json {
            json_string = Some(
                ctx.context()
                    .executor
                    .get_image_json(&id)
                    .await
                    .map_err(to_internal_error)?,
            );
            // TODO: Back button works but the "View JSON" button no longer works
            location = format!("/images/{}?json=true", id)
        }
    }

    let template = ImageTemplate {
        title: format!("Image: {}", image.manifest.name),
        image,
        json: json_string,
    };
    let result = template.render().map_err(to_internal_error)?;

    htmx_response(response, &location, result.into())
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
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }
    let id = path_params.into_inner().id;
    let image = ctx
        .context()
        .executor
        .get_image(&id)
        .await
        .map_err(to_internal_error)?;
    let template = if ctx.context().executor.delete_image(&id).await.is_ok() {
        NotificationTemplate {
            id: ctx.request_id,
            entity_id: id.to_string(),
            kind: NotificationKind::Ok,
            subject: String::from("Image deleted"),
            message: format!(
                "Image {} ({}) successfully deleted",
                image.manifest.name, id
            ),
            timeout: Some(String::from("8s")),
            redirect: Some(String::from("/images")),
            created_at: format!("/images/{}", id),
        }
    } else {
        NotificationTemplate {
            id: ctx.request_id,
            entity_id: id.to_string(),
            kind: NotificationKind::Error,
            subject: String::from("Image could not be deleted"),
            message: format!(
                "Failed to delete image {} ({})",
                image.manifest.name, id
            ),
            timeout: Some(String::from("8s")),
            redirect: None,
            created_at: format!("/images/{}", id),
        }
    };

    let template_result = template.render().map_err(to_internal_error)?;

    response
        .status(StatusCode::OK)
        .body(template_result.into())
        .map_err(to_internal_error)
}

#[derive(Template)]
#[template(path = "import.j2")]
pub struct ImportTemplate {
    title: String,
    images: Vec<Image>,
}

#[endpoint {
method = GET,
path = "/import",
}]
pub async fn get_import_index(
    ctx: RequestContext<Context>,
) -> Result<Response<Body>, HttpError> {
    let response = Response::builder();
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let mut images = ctx
        .context()
        .executor
        .get_available_images()
        .await
        .map_err(to_internal_error)?;

    let installed_images =
        ctx.context().executor.get_images().await.map_err(to_internal_error)?;

    images.retain(|available| {
        !installed_images
            .iter()
            .any(|installed| installed.manifest.uuid == available.manifest.uuid)
    });

    images.reverse();

    let template =
        ImportTemplate { title: "Available Images".to_string(), images };

    let result = template.render().map_err(to_internal_error)?;
    htmx_response(response, "/import", result.into())
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
    let response = Response::builder();
    if !Session::is_valid(&ctx) {
        return redirect_login(response, &ctx);
    }

    let id = &path_params.into_inner().id;

    let template_result;

    if let Ok(result) = ctx
        .context()
        .executor
        .import_image(id, &request_body.into_inner())
        .await
    {
        let kind = (&result).try_into().unwrap_or_default();
        let import_response: GenericResponse =
            result.json().await.map_err(to_internal_error)?;

        let subject = match kind {
            NotificationKind::Ok => String::from("Image Import Complete"),
            NotificationKind::Error => String::from("Image Import Failed"),
        };

        let template = NotificationTemplate {
            id: ctx.request_id,
            entity_id: id.to_string(),
            kind,
            subject,
            message: import_response.message,
            timeout: Some(String::from("8s")),
            redirect: None,
            created_at: format!("/import/{}", id),
        };
        template_result = template.render().map_err(to_internal_error)?;
    } else {
        let template = NotificationTemplate {
            id: ctx.request_id,
            entity_id: id.to_string(),
            kind: NotificationKind::Error,
            subject: String::from("Import failed"),
            message: format!("Failed to import image: {}", id),
            timeout: Some(String::from("8s")),
            redirect: None,
            created_at: format!("/import/{}", id),
        };
        template_result = template.render().map_err(to_internal_error)?;
    }

    response
        .status(StatusCode::OK)
        .body(template_result.into())
        .map_err(to_internal_error)
}
