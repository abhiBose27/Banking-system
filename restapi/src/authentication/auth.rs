use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware::Next,
    Error,
};

pub async fn internal_auth<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody + 'static,
{
    let expected = std::env::var("INTERNAL_SERVICE_SECRET")
        .map_err(|_| ErrorUnauthorized("Internal auth not configured"))?;

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if token != Some(expected.as_str()) {
        return Err(ErrorUnauthorized("Unauthorized internal service"));
    }

    next.call(req).await
}
