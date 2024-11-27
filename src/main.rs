use std::error::Error;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use dotenv_codegen::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::{SqlitePool, SqliteQueryResult, SqliteRow};
use sqlx::{Pool, Row, Sqlite};
use tokio::{self};
use tower_http::cors::{Any, CorsLayer};

type QueryResult = Result<SqliteQueryResult, sqlx::Error>;
const DATABASE_URL: &str = dotenv!("DATABASE_URL");
const URL: &str = if option_env!("URL").is_some() {
    dotenv!("URL")
} else {
    "0.0.0.0:3000"
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Water {
    /// Current timestamp, automatically created
    timestamp: String,
    /// The current water intake (ml)
    water_intake: i32,
    /// The target water intake(ml)
    target: i32,
    /// Percentage that is automatically calculate when creating a new water instance
    percentage: f32,
}
#[derive(Serialize, Deserialize, Debug)]
struct WaterPayload {
    water_intake: i32,
    target: i32,
}
#[derive(Serialize, Deserialize, Debug)]
struct UpdateWater {
    water_intake: i32,
}
#[derive(Serialize, Deserialize, Debug)]
struct ViewWaterById {
    id: i32,
}
impl Water {
    fn new(water_intake: i32, target: i32) -> Self {
        Self {
            timestamp: chrono::Local::now().date_naive().to_string(),
            water_intake,
            target,
            percentage: (water_intake as f32 * 100.0) / target as f32,
        }
    }

    async fn insert_water(&self, pool: &SqlitePool) -> QueryResult {
        sqlx::query("INSERT INTO water (date, water_intake, target) VALUES (?,?,?)")
            .bind(self.timestamp.as_str())
            .bind(self.water_intake)
            .bind(self.target)
            .execute(pool)
            .await
    }
    async fn update_water(id: i32, pool: &SqlitePool, water_intake: i32) -> QueryResult {
        sqlx::query("UPDATE water SET water_intake = ? WHERE id = ?")
            .bind(water_intake)
            .bind(id)
            .execute(pool)
            .await
    }
}
fn format_row(row: SqliteRow) -> String {
    let id: i32 = row.get("id");
    let date: &str = row.get("date");
    let water_intake: i32 = row.get("water_intake");
    let target: i32 = row.get("target");
    let water_intake = water_intake as f32;
    let target = target as f32;
    let percentage = ((water_intake * 100.0) / target).round();
    format!(
        "id = {} date = {}, water_intake = {}, target = {}, percentage = {}\n",
        id, date, water_intake, target, percentage
    )
}
async fn display_db(pool: &SqlitePool) -> String {
    let rows = sqlx::query("SELECT * FROM water")
        .fetch_all(pool)
        .await
        .unwrap();
    let mut db: String = String::new();
    for row in rows {
        let row = format_row(row);
        db.push_str(&row);
    }
    db
}
async fn get_water_by_id(pool: &SqlitePool, id: i32) -> String {
    if let Ok(query) = sqlx::query("SELECT * FROM water WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
    {
        format_row(query)
    } else {
        format!("Water with id {} not found", id)
    }
}
async fn create_connection() -> Pool<Sqlite> {
    let pool = SqlitePool::connect(DATABASE_URL).await.unwrap();
    let start_query = r#"
      CREATE TABLE IF NOT EXISTS water (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          date TEXT NOT NULL,
          water_intake INTEGER NOT NULL,
          target INTEGER NOT NULL
      )
      "#;
    sqlx::query(start_query).execute(&pool).await.unwrap();
    pool
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route("/", get(root))
        .route("/view_water", get(view_water))
        .route("/percentage", get(get_percentage))
        .route("/add_water", post(add_water))
        .route("/view_water_id", post(get_water_id))
        .route("/update_water", post(update_water))
        .layer(cors);
    let listener = tokio::net::TcpListener::bind(URL).await.unwrap();
    println!("Listening on {}", URL);

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
async fn root() -> &'static str {
    "Server is up and running!\n"
}
async fn add_water(Json(payload): Json<WaterPayload>) -> (StatusCode, Json<Water>) {
    println!("Got request");
    // For debug
    let pool = create_connection().await;
    let water = Water::new(payload.water_intake, payload.target);
    water.insert_water(&pool).await.unwrap();
    println!("Received request: {:?}", water);
    (StatusCode::CREATED, Json(water))
}
async fn view_water() -> String {
    println!("Received get request for water");
    let pool = create_connection().await;
    let water = display_db(&pool).await;
    if water.is_empty() {
        "There is no water".to_string()
    } else {
        water
    }
}
async fn update_water(Json(payload): Json<UpdateWater>) -> (StatusCode, Json<String>) {
    println!("Updating water...");
    let pool = create_connection().await;
    let latest_id: i32 = sqlx::query("SELECT MAX(id) as max_id FROM water")
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("max_id");
    if latest_id == 0 {
        println!("No entries found in the database.");
    }
    let current_entry = sqlx::query("SELECT date, water_intake, target FROM water WHERE id = ?")
        .bind(latest_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let date: String = current_entry.get("date");
    let current_intake: i32 = current_entry.get("water_intake");
    let target: i32 = current_entry.get("target");
    let latest_entry = format!(
        "Latest entry (ID: {}): Date: {}, Current Water Intake: {}, Target: {}",
        latest_id, date, current_intake, target
    );
    let new_intake = current_intake + payload.water_intake;
    Water::update_water(latest_id, &pool, new_intake)
        .await
        .unwrap();
    let updated_row = sqlx::query("SELECT date, water_intake, target FROM water WHERE id = ?")
        .bind(latest_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let date: String = updated_row.get("date");
    let water_intake: i32 = updated_row.get("water_intake");
    let target: i32 = updated_row.get("target");
    let updated_entry = format!(
        "Updated entry: Date: {}, Water Intake: {}, Target: {}",
        date, water_intake, target
    );
    let response = json!({
        "previous entry": latest_entry,
        "updated entry": updated_entry,
    })
    .to_string();
    println!("Sending response: {}", response);
    (StatusCode::OK, Json(response))
}
async fn get_percentage() -> String {
    println!("Viewing percentage");
    let pool = create_connection().await;
    if let Ok(query) = sqlx::query("SELECT water_intake, target FROM water")
        .fetch_one(&pool)
        .await
    {
        let water_intake: i32 = query.get("water_intake");
        let target: i32 = query.get("target");
        let percentage = ((water_intake as f32 * 100.0) / target as f32).round();

        format!("{percentage}%")
    } else {
        format!("There are no entries available")
    }
}
async fn get_water_id(Json(view_water_by_id): Json<ViewWaterById>) -> String {
    println!("Got request to get water with id {}", view_water_by_id.id);
    let conn = create_connection().await;
    let water = get_water_by_id(&conn, view_water_by_id.id).await;
    water
}
