use gimme_3d::server::server;

#[tokio::main]
async fn main() {
    server::run().await;
}
