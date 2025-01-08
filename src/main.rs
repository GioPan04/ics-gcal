use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use uuid::Uuid;

mod models;

struct AppState {
    db: Pool<Sqlite>,
}

async fn fetch_calendar(cal: &models::calendar::Calendar) -> Result<String, reqwest::Error> {
    let mut client = reqwest::Client::new().get(&cal.remote_url);

    if cal.username.is_some() {
        client = client.basic_auth(cal.username.as_ref().unwrap(), cal.password.as_ref());
    }

    let res = client.send().await?.text().await?;

    Ok(res)
}

async fn get_calendar(cal_id: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let cal: Result<models::calendar::Calendar, _> =
        sqlx::query_as("SELECT * FROM calendar WHERE uuid = ? LIMIT 1")
            .bind(cal_id.as_str())
            .fetch_one(&state.db)
            .await;

    if cal.is_err() {
        return HttpResponse::NotFound().finish();
    }

    let cal = fetch_calendar(&cal.unwrap()).await;
    return match cal {
        Ok(cal) => HttpResponse::Ok().body(cal),
        Err(_) => HttpResponse::InternalServerError().finish(),
    };
}

#[derive(Debug, Serialize)]
struct AddCalendarResponse {
    pub uuid: String,
}

async fn add_calendar(
    body: web::Json<models::calendar::Calendar>,
    state: web::Data<AppState>,
) -> impl Responder {
    let uuid = Uuid::new_v4().to_string();

    let res: Result<(String,), _> = sqlx::query_as(
        "INSERT INTO calendar (uuid, remote_url, username, password) VALUES (?, ?, ?, ?) RETURNING uuid",
    )
    .bind(&uuid)
    .bind(&body.remote_url)
    .bind(&body.username)
    .bind(&body.password)
    .fetch_one(&state.db)
    .await;

    if res.is_err() {
        eprintln!("{:?}", res.err().unwrap());
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(AddCalendarResponse {
        uuid: res.unwrap().0,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_pool = SqlitePoolOptions::new()
        .connect("sqlite:database.db")
        .await?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: db_pool.clone(),
            }))
            .route("/calendar/", web::post().to(add_calendar))
            .route("/calendar/{cal_id}.ics", web::get().to(get_calendar))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;
    Ok(())
}
