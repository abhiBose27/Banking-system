use actix_web::{Error, HttpResponse, body::BoxBody, dev::{ServiceRequest, ServiceResponse}, http::header::HeaderName, middleware::Next, web};
use object::interfaces::authentication::ApiKeyConfig;

pub async fn admin_auth(
    req: ServiceRequest,
    next: Next<BoxBody>
) -> Result<ServiceResponse, Error> {
    // ── 0. Allow unauthenticated handshake (optional)
    if req.method() == actix_web::http::Method::GET && req.path() == "/" {
        return next.call(req).await;
    }

    let cfg = match req.app_data::<web::Data<ApiKeyConfig>>() {
        Some(c) => c,
        None => {
            return Ok(req.into_response(
                HttpResponse::Unauthorized().body("Api key config missing")
            ));
        }
    };

    match req
        .headers()
        .get(HeaderName::from_static("apikey"))
        .and_then(|v| v.to_str().ok())
    {
        Some(t) if t == cfg.private_key.as_str() => next.call(req).await,
        _ => {
            return Ok(req.into_response(
                HttpResponse::Unauthorized().body("Missing Api key")
            ));
        }
    }
}