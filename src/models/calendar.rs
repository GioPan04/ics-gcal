use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(FromRow, Deserialize, Serialize)]
pub struct Calendar {
    // pub id: u32,
    // pub uuid: String,
    pub remote_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}
