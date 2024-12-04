use hypergrib::{CoordLabels, GetCoordLabels};

const BUCKET_ID: &str = "noaa-gefs-pds";

pub struct Gefs;

impl Gefs {
    pub fn new() -> Self {
        Self
    }
}

impl GetCoordLabels for Gefs {
    async fn get_coord_labels(self) -> anyhow::Result<CoordLabels> {
        Ok(CoordLabels {
            reference_datetime: Vec::new(),
            ensemble_member: Vec::new(),
            forecast_step: Vec::new(),
            parameter: Vec::new(),
            vertical_level: Vec::new(),
        })
    }
}
