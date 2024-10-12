## hypergrib_idx_parser

Parse the body of an `.idx` file. `.idx` files are like a "table of contents" into GRIB messages.

`hypergrib_idx_parser` takes as input a slice of bytes holding the contents of the `.idx` file, and returns an iterator over the decoded rows in that `.idx` file. Each row is decoded into an `MessageLocation` struct.

Note that the code for decoding and encoding `.idx` _filenames_ is in the main `hypergrib` crate.

`hypergrib_idx_parser` will probably eventually be moved into `gribberish`, after https://github.com/mpiannucci/gribberish/issues/63 is implemented.
