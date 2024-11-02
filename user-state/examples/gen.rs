use std::fmt::Display;

use anyhow::Result;
use chrono::{DateTime, Days, Utc};
use duckdb::Connection;
use fake::{
    faker::{chrono::zh_cn::DateTimeBetween, internet::en::SafeEmail, name::zh_cn::Name},
    Dummy, Fake, Faker,
};
use nanoid::nanoid;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

use sqlx::{Executor, PgPool};

//generate 10000 users and run them in a tx and repeat 500 times
/*
create table if NOT EXISTS user_stats(
  email varchar(128) NOT NULL PRIMARY KEY,
  name varchar(64) NOT NULL,
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
  last_visited_at timestamptz NOT NULL,
  last_watched_at timestamptz NOT NULL,
  recent_watched int[],
  viewed_but_not_started int[],
  started_but_not_finished int[],
  finished int[],
  last_email_notification timestamptz NOT NULL,
  last_in_app_notification timestamptz NOT NULL,
  last_sms_notification timestamptz NOT NULL
);
 */
#[derive(Debug, Clone, Serialize, Deserialize, Dummy, sqlx::Type)]
#[sqlx(type_name = "gender", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum Gender {
    Female,
    Male,
    Unknown,
}
impl Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Male => write!(f, "male"),
            Gender::Female => write!(f, "female"),
            Gender::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Dummy)]
struct UserState {
    #[dummy(faker = "UniqueEmail")]
    email: String,
    #[dummy(faker = "Name()")]
    name: String,
    gender: Gender,
    #[dummy(faker = "DateTimeBetween(before(365*5), before(90))")]
    created_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(30), now())")]
    last_visited_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), now())")]
    last_watched_at: DateTime<Utc>,
    #[dummy(faker = "IntList(50, 100000, 100000)")]
    recent_watched: Vec<i32>,
    #[dummy(faker = "IntList(50, 200000, 100000)")]
    viewed_but_not_started: Vec<i32>,
    #[dummy(faker = "IntList(50, 300000, 100000)")]
    started_but_not_finished: Vec<i32>,
    #[dummy(faker = "IntList(50, 400000, 100000)")]
    finished: Vec<i32>,
    #[dummy(faker = "DateTimeBetween(before(45), now())")]
    last_email_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(15), now())")]
    last_in_app_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), now())")]
    last_sms_notification: DateTime<Utc>,
}
#[allow(unused)]
#[tokio::main]
async fn main() -> Result<()> {
    let mut conn = duckdb().await?;

    for i in 1..=500 {
        let user: Vec<_> = (0..10000).map(|_| Faker.fake()).collect();
        //use pgsql
        let time = std::time::Instant::now();
        pgsql(&user).await?;
        println!(
            "{}: Inserted 10000 users for pgsql cost {:?}",
            i,
            time.elapsed()
        );
        //use duckdb
        let time = std::time::Instant::now();

        batch_insert_rows(&mut conn, &user).await;
        println!(
            "{}: Inserted 10000 users for duckdb cost {:?}",
            i,
            time.elapsed()
        );
    }

    Ok(())
}
async fn duckdb() -> Result<Connection> {
    let conn = Connection::open("state.db")?;
    // conn.execute("PRAGMA journal_mode=OFF;", [])?;
    conn.execute("PRAGMA memory_limit='2GB';", [])?;
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS user_stats (
            email VARCHAR(128) NOT NULL PRIMARY KEY,
            name VARCHAR(64) NOT NULL,
            gender VARCHAR(16) DEFAULT 'unknown',
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            last_visited_at TIMESTAMPTZ,
            last_watched_at TIMESTAMPTZ,
            recent_watched JSON,
            viewed_but_not_started JSON,
            started_but_not_finished JSON,
            finished JSON,
            last_email_notification TIMESTAMPTZ,
            last_in_app_notification TIMESTAMPTZ,
            last_sms_notification TIMESTAMPTZ
        );
        "#,
        [],
    )?;
    Ok(conn)
}

// async fn insert_rows(user: &Vec<UserState>) -> Result<()> {
//     let conn = Connection::open("state.db")?;
//     conn.execute(
//         r#"
//         CREATE TABLE IF NOT EXISTS user_stats (
//             email VARCHAR(128) NOT NULL PRIMARY KEY,
//             name VARCHAR(64) NOT NULL,
//             gender VARCHAR(16) DEFAULT 'unknown',
//             created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
//             last_visited_at TIMESTAMPTZ,
//             last_watched_at TIMESTAMPTZ,
//             recent_watched JSON,
//             viewed_but_not_started JSON,
//             started_but_not_finished JSON,
//             finished JSON,
//             last_email_notification TIMESTAMPTZ,
//             last_in_app_notification TIMESTAMPTZ,
//             last_sms_notification TIMESTAMPTZ
//         );
//         "#,
//         [],
//     )?;
//     for u in user {
//         // println!("Inserting {}", u.email);
//         conn.execute(
//             "INSERT INTO user_stats(email,name,gender,created_at,last_visited_at,last_watched_at,recent_watched,viewed_but_not_started,started_but_not_finished,finished,last_email_notification,last_in_app_notification,last_sms_notification)
//             VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)",
//             params![u.email,u.name,u.gender.to_string(), u.created_at, u.last_visited_at, u.last_watched_at, json!(u.recent_watched).to_string(), json!(u.viewed_but_not_started).to_string(), json!(u.started_but_not_finished).to_string(), json!(u.finished).to_string(), u.last_email_notification, u.last_in_app_notification, u.last_sms_notification],
//         )?;
//     }
//     Ok(())
// }

async fn batch_insert_rows(conn: &mut Connection, user: &[UserState]) -> Result<()> {
    let tx = conn.transaction()?;
    let mut stmt=tx.prepare("INSERT INTO user_stats(email,name,gender,created_at,last_visited_at,last_watched_at,recent_watched,viewed_but_not_started,started_but_not_finished,finished,last_email_notification,last_in_app_notification,last_sms_notification)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13);")?;

    for u in user.iter().cloned() {
        stmt.execute([
            u.email,
            u.name,
            u.gender.to_string(),
            u.created_at.to_string(),
            u.last_visited_at.to_string(),
            u.last_watched_at.to_string(),
            json!(u.recent_watched).to_string(),
            json!(u.viewed_but_not_started).to_string(),
            json!(u.started_but_not_finished).to_string(),
            json!(u.finished).to_string(),
            u.last_email_notification.to_string(),
            u.last_in_app_notification.to_string(),
            u.last_sms_notification.to_string(),
        ])?;
    }
    tx.commit()?;
    Ok(())
}

async fn pgsql(user: &[UserState]) -> Result<()> {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/state").await?;
    bulk_insert(user, &pool).await?;
    Ok(())
}
/*
create table if NOT EXISTS user_stats(
  email varchar(128) NOT NULL PRIMARY KEY,
  name varchar(64) NOT NULL,
  gender gender DEFAULT 'unknown',
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
  last_visited_at timestamptz,
  last_watched_at timestamptz,
  recent_watched int [],
  viewed_but_not_started int [],
  started_but_not_finished int [],
  finished int [],
  last_email_notification timestamptz,
  last_in_app_notification timestamptz,
  last_sms_notification timestamptz
);
*/
async fn bulk_insert(user: &[UserState], pool: &PgPool) -> Result<()> {
    let mut tx = pool.begin().await?;
    for u in user.iter().cloned() {
        let query=sqlx::query(r#"
            INSERT INTO user_stats(email,name,gender,created_at,last_visited_at,last_watched_at,recent_watched,viewed_but_not_started,started_but_not_finished,finished,last_email_notification,last_in_app_notification,last_sms_notification)
            VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)"#)
            .bind(u.email).bind(u.name).bind(u.gender).bind(u.created_at).bind(u.last_visited_at).bind(u.last_watched_at).bind(u.recent_watched).bind(u.viewed_but_not_started).bind(u.started_but_not_finished).bind(u.finished).bind(u.last_email_notification).bind(u.last_in_app_notification).bind(u.last_sms_notification);
        tx.execute(query).await?;
    }
    tx.commit().await?;
    Ok(())
}
fn before(days: u64) -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(days)).unwrap()
}
fn now() -> DateTime<Utc> {
    Utc::now()
}

struct IntList(pub i32, pub i32, pub i32); // does not handle locale, see locales module for more

impl Dummy<IntList> for Vec<i32> {
    fn dummy_with_rng<R: Rng + ?Sized>(v: &IntList, rng: &mut R) -> Vec<i32> {
        let (max, start, len) = (v.0, v.1, v.2);
        let size = rng.gen_range(0..max);
        (0..size)
            .map(|_| rng.gen_range(start..start + len))
            .collect()
    }
}

struct UniqueEmail;
const SAFE: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
impl Dummy<UniqueEmail> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &UniqueEmail, rng: &mut R) -> String {
        let email: String = SafeEmail().fake_with_rng(rng);
        let id = nanoid!(10, &SAFE);
        let at = email.find("@").unwrap();
        format!("{}.{}{}", &email[..at], id, &email[at..])
    }
}
