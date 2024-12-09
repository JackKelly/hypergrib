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
    let metrics = Handle::current().metrics();
    let args = Args::parse();

    println!("Loading dataset {:?}", args.dataset);

    let dataset = match args.dataset {
        DatasetName::Gefs => Gefs::new()?,
    };

    let coord_labels = dataset.get_coord_labels().await.expect("get_coord_labels");
    // TODO: Write the coord labels to a metadata file. See:
    // https://github.com/JackKelly/hypergrib/discussions/17

    let n = metrics.spawned_tasks_count();
    println!("Runtime has had {} tasks spawned", n);

    Ok(())
}
