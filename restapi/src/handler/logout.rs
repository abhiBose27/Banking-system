use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, post, web};
use deadpool_redis::{Pool, redis::AsyncTypedCommands};
use object::interfaces::authentication::{AuthContext};


#[post("/logout")]
async fn client_logout(
    request: HttpRequest,
    pool: web::Data<Pool>
) -> impl Responder {
    let auth = request.extensions().get::<AuthContext>().cloned().unwrap();
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("Error: Redis Unavailable"),
    };
    let exists= match conn.exists::<_>(&auth.token).await {
        Ok(e) =>  e,
        Err(_) => return HttpResponse::InternalServerError().body("Error: Redis Unavailable")
    };

    if !exists {
        return HttpResponse::BadRequest().body("Not logged in");
    }

    match conn.get_del::<_>(&auth.token).await {
        Ok(usr) => {
            match usr {
                Some(u) => {
                    if let Err(_) = conn.get_del::<_>(&u).await {
                        return HttpResponse::InternalServerError().body("Error: Redis cannot delete");
                    }
                },
                None => return HttpResponse::InternalServerError().body("Error: Redis Error"),
            };
        },
        Err(_) => return HttpResponse::InternalServerError().body("Error: Redis cannot delete")
    };
    HttpResponse::Ok().json("Logged out.")
}