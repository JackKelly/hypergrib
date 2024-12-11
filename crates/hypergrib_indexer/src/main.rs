use std::time::Duration;

use tokio::{
    runtime::{Handle, RuntimeMetrics},
    task::JoinSet,
};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    println!("1. Metrics before submitting any requests:");
    let m = Handle::current().metrics();
    print_tokio_metrics(&m);

    // Submit GET requests:
    const N: usize = 20;
    println!("2. Submitting {N} requests...\n");
    let mut handles = JoinSet::new();
    let client = reqwest::Client::new();
    for _ in 0..N {
        handles.spawn(client.get("https://rust-lang.org").send());
    }

    println!("3. Metrics just after submitting all requests:");
    print_tokio_metrics(&m);

    // Join:
    handles.join_all().await;
    println!("4. Metrics just after receiving all requests:");
    print_tokio_metrics(&m);

    // Print metrics after all blocking threads have been dropped:
    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("5. Metrics 10 seconds after receiving all requests:");
    print_tokio_metrics(&m);

    Ok(())
}

fn print_tokio_metrics(m: &RuntimeMetrics) {
    print!("spawned_tasks:{:>3}, ", m.spawned_tasks_count());
    print!("alive_tasks:{:>3}, ", m.num_alive_tasks());
    print!("blocking_threads:{:>3}\n\n", m.num_blocking_threads());
}
