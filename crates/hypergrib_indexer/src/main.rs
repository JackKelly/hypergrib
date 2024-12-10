use std::time::Duration;

use clap::{Parser, ValueEnum};
use hypergrib::GetCoordLabels;
use hypergrib_indexer::datasets::gefs::Gefs;
use tokio::runtime::Handle;

/// Create a manifest from GRIB `.idx` files.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    dataset: DatasetName,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, ValueEnum)]
enum DatasetName {
    /// The Global Ensemble Forecast System (GEFS) is a weather model created
    /// by the US National Centers for Environmental Prediction (NCEP) that
    /// generates 21 separate forecasts (ensemble members). See:
    /// https://www.ncei.noaa.gov/products/weather-climate-models/global-ensemble-forecast
    Gefs,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Loading dataset {:?}", args.dataset);

    let dataset = match args.dataset {
        DatasetName::Gefs => Gefs::new()?,
    };

    let spawn_handle = tokio::spawn(async {
        for _ in 0..10 {
            println!("---------------");
            print_tokio_metrics();
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    // let coord_labels = dataset.get_coord_labels().await.expect("get_coord_labels");

    let mut handles = Vec::new();
    let store = dataset.coord_labels_builder().idx_store().clone();
    for _ in 0..7 {
        let store_cloned = store.clone();
        handles.push(tokio::spawn(async move {
            store_cloned.list_with_delimiter(None).await
        }));
    }

    // TODO: Write the coord labels to a metadata file. See:
    // https://github.com/JackKelly/hypergrib/discussions/17

    for handle in handles {
        let _ = handle.await?;
    }
    spawn_handle.await?;
    Ok(())
}

fn print_tokio_metrics() {
    let metrics = Handle::current().metrics();
    println!(
        "Runtime has had {} tasks spawned",
        metrics.spawned_tasks_count()
    );
    println!("num_alive_tasks: {}", metrics.num_alive_tasks());
    println!("num_workers: {}", metrics.num_workers());
    println!("global_queue_depth: {}", metrics.global_queue_depth());
    println!("num_blocking_threads: {}", metrics.num_blocking_threads());
    println!(
        "num_idle_blocking_threads: {}",
        metrics.num_idle_blocking_threads()
    );
    println!(
        "budget_forced_yield_count: {}",
        metrics.budget_forced_yield_count()
    );
    println!("blocking_queue_depth: {}", metrics.blocking_queue_depth());
    println!(
        "io_driver_fd_registered_count: {}",
        metrics.io_driver_fd_registered_count()
    );
    println!("io_driver_ready_count: {}", metrics.io_driver_ready_count());
}
