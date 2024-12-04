use clap::{Parser, ValueEnum};

/// Create a manifest from GRIB `.idx` files.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    dataset: Dataset,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, ValueEnum)]
enum Dataset {
    /// The Global Ensemble Forecast System (GEFS) is a weather model created
    /// by the US National Centers for Environmental Prediction (NCEP) that
    /// generates 21 separate forecasts (ensemble members). See:
    /// https://www.ncei.noaa.gov/products/weather-climate-models/global-ensemble-forecast
    Gefs,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();

    println!("Loading dataset {:?}", args.dataset);
}
