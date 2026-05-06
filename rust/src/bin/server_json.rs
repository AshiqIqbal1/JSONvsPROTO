use axum::{extract::{Path, Query}, response::Json, routing::get, Router};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Address {
    street: String,
    city: String,
    country: String,
    zip_code: String,
}

#[derive(Serialize)]
struct Order {
    order_id: String,
    product: String,
    price: f32,
    quantity: i32,
}

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
    email: String,
    age: i32,
    address: Address,
    orders: Vec<Order>,
}

#[derive(Serialize)]
struct UserList {
    users: Vec<User>,
}

fn random_str(n: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..n).map(|_| rng.gen_range(b'a'..=b'z') as char).collect()
}

fn make_user(uid: i32) -> User {
    let mut rng = rand::thread_rng();
    let n_orders = rng.gen_range(1..=5);
    User {
        id: uid,
        name: format!("{} {}", random_str(5), random_str(7)),
        email: format!("{}@{}.com", random_str(6), random_str(5)),
        age: rng.gen_range(18..=80),
        address: Address {
            street: format!("{} {} St", rng.gen_range(1..=999), random_str(6)),
            city: random_str(7),
            country: "US".to_string(),
            zip_code: format!("{}", rng.gen_range(10000..=99999)),
        },
        orders: (0..n_orders)
            .map(|_| Order {
                order_id: random_str(12),
                product: random_str(8),
                price: (rng.gen_range(100..=50000) as f32) / 100.0,
                quantity: rng.gen_range(1..=20),
            })
            .collect(),
    }
}

async fn get_user(Path(uid): Path<i32>) -> Json<User> {
    Json(make_user(uid))
}

#[derive(Deserialize)]
struct CountQuery {
    count: Option<usize>,
}

async fn get_users(Query(q): Query<CountQuery>) -> Json<UserList> {
    let count = q.count.unwrap_or(100);
    Json(UserList {
        users: (0..count as i32).map(make_user).collect(),
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/user/:uid", get(get_user))
        .route("/users", get(get_users));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8003").await.unwrap();
    println!("JSON server running on :8003");
    axum::serve(listener, app).await.unwrap();
}
