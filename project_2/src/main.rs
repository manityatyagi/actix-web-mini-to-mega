use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Task {
    id: i64,
    title: String,
    description: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateTask {
    title: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTask {
    title: Option<String>,
    description: Option<String>,
}

async fn create_task(pool: web::Data<SqlitePool>, body: web::Json<CreateTask>) -> impl Responder {
    let now = Utc::now();

    let result = sqlx::query("INSERT INTO Task (title, description, created_at) VALUES (?, ?, ?)")
        .bind(&body.title)
        .bind(&body.description)
        .bind(now)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json("Task created"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_tasks(pool: web::Data<SqlitePool>) -> impl Responder {
    let tasks = sqlx::query_as::<_, Task>("SELECT id, title, description, created_at FROM Task")
        .fetch_all(pool.get_ref())
        .await;

    match tasks {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_task(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let task = sqlx::query_as::<_, Task>(
        "SELECT id, title, description, created_at FROM Task WHERE id = ?",
    )
    .bind(path.into_inner())
    .fetch_optional(pool.get_ref())
    .await;

    match task {
        Ok(Some(task)) => HttpResponse::Ok().json(task),
        Ok(None) => HttpResponse::NotFound().body("Task not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn update_task(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    body: web::Json<UpdateTask>,
) -> impl Responder {
    let id = path.into_inner();

    let existing = sqlx::query_as::<_, Task>(
        "SELECT id, title, description, created_at FROM Task WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await
    .unwrap();

    let Some(task) = existing else {
        return HttpResponse::NotFound().body("Task not found");
    };

    let title = body.title.clone().unwrap_or(task.title);
    let description = body.description.clone().unwrap_or(task.description);

    let result = sqlx::query("UPDATE Task SET title = ?, description = ? WHERE id = ?")
        .bind(title)
        .bind(description)
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Task updated"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn delete_task(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let result = sqlx::query("DELETE FROM Task WHERE id = ?")
        .bind(path.into_inner())
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                HttpResponse::NotFound().body("Task not found")
            } else {
                HttpResponse::Ok().body("Task deleted")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePool::connect("sqlite://app.db")
        .await
        .expect("DB connection failed");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Task (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::resource("/tasks")
                    .route(web::post().to(create_task))
                    .route(web::get().to(get_tasks)),
            )
            .service(
                web::resource("/tasks/{id}")
                    .route(web::get().to(get_task))
                    .route(web::put().to(update_task))
                    .route(web::delete().to(delete_task)),
            )
    })
    .bind(("192.168.31.102", 8080))?
    .run()
    .await
}
