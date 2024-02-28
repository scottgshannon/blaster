use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(version,about)]
pub struct Args {
    /// Number of workers to run simultaneously
    worker_count: u32,
    /// Number of requests to perform PER worker
    requests_per_worker: u32,
    /// URI of endpoint
    target_uri: String,
    /// use a single connection client shared between workers
    #[arg(short,long,default_value_t = false)]
    share_worker_connection: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!(
        "Running {} workers with {} requests per worker",
        args.worker_count, args.requests_per_worker
    );
    let shared_client = reqwest::Client::new();
    let threads: Vec<_> = (0..args.worker_count)
        .map(|i| {
            let request_count = args.requests_per_worker;
            let url = args.target_uri.clone();
            let client = if args.share_worker_connection {
                shared_client.clone()
            } else {
                reqwest::Client::new()
            };
            println!("Spawning {i}");
            return tokio::spawn(async move { run_requests(client, i, request_count, url).await });
        })
        .collect();
    for handle in threads {
        match handle.await.unwrap() {
            Ok((i, r)) => println!("Thread {i} complete after {r} requests"),
            Err(e) => println!("{:?}", e),
        };
    }
    Ok(())
}

async fn run_requests(
    client: reqwest::Client,
    thread_index: u32,
    num_count: u32,
    url: String,
) -> Result<(u32, u32)> {
    println!("Starting thread {thread_index}");
    for i in 0..num_count {
        println!("Thread {thread_index} requesting {i}");
        _ = client.post(&url).send().await?;
    }
    Ok((thread_index, num_count))
}
