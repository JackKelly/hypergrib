use clap::Parser;
use futures_util::{stream::BoxStream, Stream, StreamExt, TryFutureExt};
use object_store::ObjectMeta;
use std::fs;
use std::future;
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

    // Get listing of .idx files:
    let mut list_stream = filter_by_ext(store.list(Some(&path)), "idx");

    // Print listing:
    let mut i = 0;
    while let Some(meta) = list_stream.next().await.transpose().unwrap() {
        println!("Name: {}, size: {}", meta.location, meta.size);

        // Write idx file to local filesystem
        let bytes = store
            .get(&meta.location)
            .and_then(|get_result| get_result.bytes());
        fs::write(
            meta.location.filename().expect("failed to get filename"),
            bytes.await.expect("failed to get bytes"),
        )
        .expect("failed to write local file");

        i += 1;
        if i > 10 {
            break;
        }
    }
}

/// Filter a stream of `object_store::Result<object_store::ObjectMeta>` to select only the items
/// which have a file extension which matches `extension`.
fn filter_by_ext<'a>(
    stream: impl Stream<Item = object_store::Result<ObjectMeta>> + 'a,
    extension: &'static str,
) -> impl Stream<Item = object_store::Result<ObjectMeta>> + 'a {
    stream.filter(move |list_result| {
        future::ready(list_result.as_ref().is_ok_and(|meta| {
            meta.location
                .extension()
                .is_some_and(|ext| ext == extension)
        }))
    })
}
