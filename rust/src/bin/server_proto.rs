use axum::{
    extract::{Path, Query},
    response::Response,
    routing::get,
    Router,
};
use bytes::Bytes;
use axum::http::{header, StatusCode};
use jsonvsproto::proto::{Address, Order, User, UserList};
use prost::Message;
use rand::Rng;
use serde::Deserialize;

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
        address: Some(Address {
            street: format!("{} {} St", rng.gen_range(1..=999), random_str(6)),
            city: random_str(7),
            country: "US".to_string(),
            zip_code: format!("{}", rng.gen_range(10000..=99999)),
        }),
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

fn proto_response(bytes: Vec<u8>) -> Response<axum::body::Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-protobuf")
        .body(axum::body::Body::from(Bytes::from(bytes)))
        .unwrap()
}

async fn get_user(Path(uid): Path<i32>) -> Response<axum::body::Body> {
    proto_response(make_user(uid).encode_to_vec())
}

#[derive(Deserialize)]
struct CountQuery {
    count: Option<usize>,
}

async fn get_users(Query(q): Query<CountQuery>) -> Response<axum::body::Body> {
    let count = q.count.unwrap_or(100);
    let user_list = UserList {
        users: (0..count as i32).map(make_user).collect(),
    };
    proto_response(user_list.encode_to_vec())
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/user/:uid", get(get_user))
        .route("/users", get(get_users));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8004").await.unwrap();
    println!("Proto server running on :8004");
    axum::serve(listener, app).await.unwrap();
}
