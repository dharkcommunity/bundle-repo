use actix_web::{
    get,
    http::{header::ContentType, StatusCode},
    web, HttpResponse, ResponseError,
};
use log::error;
use thiserror::Error;

use crate::validate::validate_resource_name;
use crate::AppState;

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Internal Error")]
    InternalError,

    #[error("Bad Request: {0}")]
    BadRequest(String),
}

impl ResponseError for ErrorKind {
    fn status_code(&self) -> StatusCode {
        match *self {
            ErrorKind::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::BadRequest(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }
}

#[get("/resource_version_amount/{resource_name}")]
pub async fn resource_version_amount(
    data: web::Data<AppState>,
    resource_name: web::Path<String>,
) -> Result<String, ErrorKind> {
    let resource_name = resource_name.into_inner();
    validate_resource_name(&resource_name).map_err(ErrorKind::BadRequest)?;

    let content = data
        .bucket
        .list_page(format!("{resource_name}/"), None, None, None, None)
        .await;

    match content {
        Ok(v) => Ok(v.0.contents.len().to_string()),
        Err(err) => {
            error!("Error while listing resource amount: {err}");
            Err(ErrorKind::InternalError)
        }
    }
}
