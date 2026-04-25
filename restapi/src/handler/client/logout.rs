use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, post, web};
use deadpool_redis::Pool;

use object::interfaces::authentication::AuthContext;

use crate::cache::redis::{is_logged_in_with_token, logout_user};


#[post("/logout")]
async fn client_logout(
    request: HttpRequest,
    redis_pool: web::Data<Pool>
) -> impl Responder {
    let auth = request.extensions().get::<AuthContext>().cloned().unwrap();

    // Get the connection from connection pool
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Redis Error {e}");
            return HttpResponse::InternalServerError().body("Error: Redis Unavailable")
        }
    };

    // Check if the user is logged in
    let is_logged_in = is_logged_in_with_token(&auth.token, &mut conn).await;
    if let Err(e) = is_logged_in {
        eprintln!("Error: Redis error {e}");
        return HttpResponse::InternalServerError().finish();
    }
    if let false = is_logged_in.unwrap() {
        return HttpResponse::BadRequest().body("Not logged in");
    }

    // Delete and logout
    if let Err(e) = logout_user(&auth.token, &mut conn).await {
        eprintln!("Error: Redis Error {e}");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json("Logged out.")
}