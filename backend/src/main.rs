#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let http_listen = std::env::var("HTTP_LISTEN").expect("HTTP_LISTEN must set");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must set");

    backend::main(&http_listen, &database_url).await;
}
