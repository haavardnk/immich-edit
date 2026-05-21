#[tokio::main]
async fn main() -> anyhow::Result<()> {
    immich_edit_backend::run().await
}
