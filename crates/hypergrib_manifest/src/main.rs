use clap::Parser;
use futures_util::StreamExt;
use object_store;
use url::Url;

/// Create a manifest from GRIB `.idx` files.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URL of the GRIB files. For example "s3://noaa-gefs-pds/gefs.20170101/00/"
    #[arg(long)]
    url: Url,

    /// Set this flag if accessing a bucket that requires authentication.
    #[arg(long)]
    sign: bool,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();

    println!("{}", args.url);

    // Get options, store, and path:
    let mut opts = vec![];
    if !args.sign {
        opts.push(("skip_signature", "true"));
    }
    let (store, path) = object_store::parse_url_opts(&args.url, opts).unwrap();

    // Get listing:
    let mut list_stream = store.list(Some(&path));

    // Print listing:
    let mut i = 0;
    while let Some(meta) = list_stream.next().await.transpose().unwrap() {
        println!("Name: {}, size: {}", meta.location, meta.size);
        i += 1;
        if i > 10 {
            break;
        }
    }
}
