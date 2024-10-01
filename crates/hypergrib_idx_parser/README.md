## hypergrib_idx_parser

Parse `.idx` files that act as a "table of contents" into GRIB messages.

`hypergrib_idx_parser` takes as input a slice of bytes holding the contents of the `.idx` file, and returns an iterator over the rows in that `.idx` file. Each row is decoded into an `MessageLocation` struct.
