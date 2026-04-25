use actix_web::{Error, HttpMessage, HttpResponse, body::BoxBody, dev::{ServiceRequest, ServiceResponse}, http::header::AUTHORIZATION, middleware::Next, web};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use object::interfaces::authentication::{AuthContext, Claims, JwtConfig, Role};


fn verify_token(token: &str, cfg: &JwtConfig) -> Result<AuthContext, ()> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[cfg.issuer.clone()]);
    validation.set_audience(&[cfg.client_aud.clone()]);
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&cfg.client_secret),
        &validation,
    ).map_err(|_| ())?;

    Ok(AuthContext { 
        user_id: data.claims.sub, 
        role: Role::Client, 
        token: token.to_string(), 
        customer_id: data.claims.customer_id 
    })
    
}

pub fn issue_jwt(
    cfg: &JwtConfig,
    ttl_seconds: usize,
    user_id: Uuid,
    customer_id: Uuid,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        role: "client".to_string(),
        iss: cfg.issuer.clone(),
        aud: cfg.client_aud.to_string(),
        exp: now + ttl_seconds,
        customer_id
    };
    let token = encode(
        &Header::new(Algorithm::HS256), 
        &claims,&EncodingKey::from_secret(&cfg.client_secret)
    ).unwrap();
    Ok(token)
}

pub async fn client_auth(
    mut req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse, Error> {
    // ── 0. Allow unauthenticated handshake (optional)
    if req.method() == actix_web::http::Method::GET && req.path() == "/" {
        return next.call(req).await;
    }

    // ── 1. Load JWT config
    let cfg = match req.app_data::<web::Data<JwtConfig>>() {
        Some(c) => c,
        None => {
            return Ok(req.into_response(
                HttpResponse::Unauthorized().body("JWT config missing")
            ));
        }
    };

    // ── 2. Extract Bearer token
    let token = match req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t,
        None => {
            return Ok(req.into_response(
                HttpResponse::Unauthorized().body("Missing Bearer token")
            ));
        }
    };
    // ── 3. Verify JWT (admin first, then client)
    let auth_ctx = match verify_token(token, &cfg) {
        Ok(ctx) => ctx,
        Err(_) => {
            return Ok(req.into_response(
                HttpResponse::Unauthorized().body("Invalid token")
            ));
        }
    };

    // ── 4. Authorization rules (Model B)
    if matches!(auth_ctx.role, Role::Client) {
        let forbidden =
            (req.method() == actix_web::http::Method::POST && req.path() == "/api/account") ||
            (req.method() == actix_web::http::Method::POST && req.path() == "/api/customer");

        if forbidden {
            return Ok(req.into_response(
                HttpResponse::Forbidden().body("Clients cannot create accounts or customers")
            ));
        }
    }

    // ── 5. Defense in depth
    req.headers_mut().remove("role");
    req.headers_mut().remove("customer_id");

    // ── 6. Inject trusted auth context
    req.extensions_mut().insert(auth_ctx);
    next.call(req).await
}
