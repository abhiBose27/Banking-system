use actix_web::{
    Error, HttpResponse, 
    body::BoxBody, 
    dev::{ServiceRequest, ServiceResponse}, 
    http::header::AUTHORIZATION, 
    middleware::Next, 
    web,
    HttpMessage
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use object::interfaces::authentication::{AuthContext, Claims, JwtConfig, Role};


pub fn issue_jwt(
    cfg: &JwtConfig,
    role: Role, 
    user_id: Uuid,
    ttl_seconds: usize,
    customer_id: Option<Uuid>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp() as usize;

    let (aud, secret) = match role {
        Role::Admin => (&cfg.admin_aud, &cfg.admin_secret),   
        Role::Client => (&cfg.client_aud, &cfg.client_secret)
    };

    let claims = Claims {
        sub: user_id.to_string(),
        role: if role == Role::Client {"client".to_string()} else {"admin".to_string()},
        iss: cfg.issuer.clone(),
        aud: aud.to_string(),
        exp: now + ttl_seconds,
        customer_id
    };
    let token = encode(
        &Header::new(Algorithm::HS256), 
        &claims,&EncodingKey::from_secret(secret)
    ).unwrap();
    Ok(token)
}

pub async fn internal_auth(
    mut req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse, Error> {
    // ── 0. Allow unauthenticated handshake (optional)
    if req.method() == actix_web::http::Method::GET && req.path() == "/" {
        return next.call(req).await;
    }

    // ── 1. Load JWT config
    let cfg = match req.app_data::<web::Data<JwtConfig>>() {
        Some(c) => c.clone(),
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

fn verify_token(token: &str, cfg: &JwtConfig) -> Result<AuthContext, ()> {
    // Try admin token first
    if let Ok(ctx) = verify_with_policy(
        token,
        &cfg.admin_secret,
        &cfg.issuer,
        &cfg.admin_aud,
        Role::Admin,
    ) {
        return Ok(ctx);
    }

    // Then client token
    if let Ok(ctx) = verify_with_policy(
        token,
        &cfg.client_secret,
        &cfg.issuer,
        &cfg.client_aud,
        Role::Client,
    ) {
        return Ok(ctx);
    }

    Err(())
}

fn verify_with_policy(
    token: &str,
    secret: &[u8],
    issuer: &str,
    audience: &str,
    fallback_role: Role,
) -> Result<AuthContext, ()> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &validation,
    ).map_err(|_| ())?;

    let role = match data.claims.role.as_str() {
        "admin" => Role::Admin,
        "client" => Role::Client,
        _ => fallback_role,
    };

    Ok(AuthContext {
        role,
        user_id: data.claims.sub,
        token: token.to_string(),
        customer_id: data.claims.customer_id
    })
}
