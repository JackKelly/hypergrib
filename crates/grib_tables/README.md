## grib_tables

The basic idea is to keep it simple.

When we're indexing a GRIB dataset:
- Load the WMO and GDAL CSVs into memory as a simple "database" with two HashMaps: one keyed on
the numerical identifier, and one keyed on the abbreviation string. (Although, actually, maybe
we only need to create one hashmap because, if a dataset has .idx files, then we know we only
need the hashmap keyed on the abbreviation string... although maybe we'll want to also load some
GRIB files.)
- Maybe give the option to only load some of the CSVs. e.g. we don't need the Canadian local
tables if we're loading a NOAA dataset. But this might be over complicating things for a small
reduction in memory footprint.
- Save the decoded metadata into the JSON that we create for each dataset. Maybe have a mapping
from the abbreviation string to the full ProductTemplate variant.

Then, when users are reading the dataset, we don't need to load any of the GRIB tables because
the relevant metadata will already be captured.

## Related

- https://github.com/JackKelly/rust-playground/tree/main/grib_tables: A previous design sketch
  that I made, where we try to faithfully capture the GRIB tables hierarchy (Discipline > Category > Parameter).
  But this feels overkill. It might be _slightly_ faster because we can create a perfect hash at compile time.
  But then I realised that we only need to look things up in the GRIB table whilst indexing a new GRIB dataset.
  When "normal users" use the dataset, they can just read the metadata that we create for the dataset.