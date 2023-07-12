mod cli;
mod runner;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    if let Err(err) = cli::run().await {
        eprintln!("error: {err:?}");
    }
}
