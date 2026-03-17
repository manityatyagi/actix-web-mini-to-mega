use actix_web::{web, App, HttpServer, HttpResponse, Responder, get, dev::{ServiceRequest, ServiceResponse}, Error, middleware::{from_fn, Next}};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
struct UserRequest {
    name: String,
    id: i32
}

#[derive(Debug, Serialize)]
struct UserResponse {
    name: String,
    id: i32,
    created_at: DateTime<Utc>
}

#[get("/")]
async fn all() -> impl Responder {
    HttpResponse::Ok().body("Server is running".to_string())
}

async fn user_info(data: web::Path<UserRequest>) -> impl Responder {
  let user_data = data.into_inner();
  HttpResponse::Ok().json(UserResponse {
     name: user_data.name,
     id: user_data.id,
     created_at: Utc::now()
  })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new( || {
        App::new()
             .wrap(from_fn(simple_middleware))
             .service(all)
             .service(
                web::scope("/users")
                   .route("/{name}/{id}", web::get().to(user_info))
             )
    })
    .bind(("192.168.31.102", 8080))?
    .run()
    .await
}

async fn simple_middleware<B>(req: ServiceRequest, next: Next<B>) -> Result<ServiceResponse<B>, Error> {
    let method = req.method();
    let path = req.path().to_string();

    println!("Incoming Reqest {} {}", method, path);

    let res = next.call(req).await?;
    Ok(res)
}