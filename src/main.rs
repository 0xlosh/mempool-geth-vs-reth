mod cli;
mod hwi;
mod runner;

#[tokio::main]
async fn main() {    
    if let Err(err) = cli::run().await {
        eprintln!("error: {err:?}");
    }
}
