mod routes;
use routes::run;

#[tokio::main]
async fn main() {
    run().await;
}
