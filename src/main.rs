use std::io::Write;

use dotenv_codegen::dotenv;
use sqlx::sqlite::{SqlitePool, SqliteQueryResult};
use sqlx::Row;
use tokio::{self};
#[derive(Debug, Clone)]
struct Water {
    /// Current timestamp, automatically created
    timestamp: String,
    /// The current water intake (ml)
    water_intake: i32,
    /// The target water intake(ml)
    target: i32,
}
impl Water {
    fn new(water_intake: i32, target: i32) -> Self {
        Self {
            timestamp: chrono::Local::now().date_naive().to_string(),
            water_intake,
            target,
        }
    }
    async fn insert_water(&self, pool: &SqlitePool) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("INSERT INTO water (date, water_intake, target) VALUES (?,?,?)")
            .bind(self.timestamp.as_str())
            .bind(self.water_intake)
            .bind(self.target)
            .execute(pool)
            .await
    }
    async fn update_water(
        id: i32,
        pool: &SqlitePool,
        water_intake: i32,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("UPDATE water SET water_intake = ? WHERE id = ?")
            .bind(water_intake)
            .bind(id)
            .execute(pool)
            .await
    }
}
async fn display_db(pool: &SqlitePool) {
    let rows = sqlx::query("SELECT date, water_intake, target FROM water")
        .fetch_all(pool)
        .await
        .unwrap();
    for row in rows {
        let date: &str = row.get("date");
        let water_intake: i32 = row.get("water_intake");
        let target: i32 = row.get("target");
        let water_intake = water_intake as f32;
        let target = target as f32;
        let percent = (water_intake * 100.0) / target;
        println!(
            "Results: date = {}, water_intake = {}, target = {}, percent = {}",
            date, water_intake, target, percent
        );
    }
}
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect(dotenv!("DATABASE_URL"))
        .await
        .expect("Could not find database file, make sure it is the right file");
    let start_query = r#"
    CREATE TABLE IF NOT EXISTS water (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TEXT NOT NULL,
        water_intake INTEGER NOT NULL,
        target INTEGER NOT NULL
    )
    "#;
    sqlx::query(start_query).execute(&pool).await?;
    loop {
        print!("Enter your options: 0(exit), 1(add), 2(view), 3 (update): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input: i32 = input.trim().parse().unwrap();
        match input {
            0 => break,
            1 => {
                print!("Enter your current intake and target separated by comma: ");
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let (water_intake, target) = input.trim().split_once(",").unwrap();
                let water_intake: i32 = water_intake.parse().unwrap();
                let target: i32 = target.parse().unwrap();
                let data = Water::new(water_intake, target);
                data.insert_water(&pool).await.unwrap();
                println!("Data succesfully inserted!");
                display_db(&pool).await;
            }
            2 => display_db(&pool).await,
            3 => {
                let latest_id: i32 = sqlx::query("SELECT MAX(id) as max_id FROM water")
                    .fetch_one(&pool)
                    .await?
                    .get("max_id");

                if latest_id == 0 {
                    println!("No entries found in the database.");
                    continue;
                }

                let current_entry =
                    sqlx::query("SELECT date, water_intake, target FROM water WHERE id = ?")
                        .bind(latest_id)
                        .fetch_one(&pool)
                        .await?;

                let date: String = current_entry.get("date");
                let current_intake: i32 = current_entry.get("water_intake");
                let target: i32 = current_entry.get("target");

                println!(
                    "Latest entry (ID: {}): Date: {}, Current Water Intake: {}, Target: {}",
                    latest_id, date, current_intake, target
                );

                print!("Enter the additional water intake: ");
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let additional_intake: i32 = input.trim().parse().unwrap();

                let new_intake = current_intake + additional_intake;

                match Water::update_water(latest_id, &pool, new_intake).await {
                    Ok(_) => {
                        println!("Successfully updated water intake");
                        let updated_row = sqlx::query(
                            "SELECT date, water_intake, target FROM water WHERE id = ?",
                        )
                        .bind(latest_id)
                        .fetch_one(&pool)
                        .await?;
                        let date: String = updated_row.get("date");
                        let water_intake: i32 = updated_row.get("water_intake");
                        let target: i32 = updated_row.get("target");
                        println!(
                            "Updated entry: Date: {}, Water Intake: {}, Target: {}",
                            date, water_intake, target
                        );
                    }
                    Err(e) => println!("Failed to update water intake: {}", e),
                }
            }
            _ => {
                println!("Invalid option!");
                break;
            }
        }
    }

    Ok(())
}
