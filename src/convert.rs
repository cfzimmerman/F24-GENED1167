//! ### Convert
//! Tools for converting raw CAISO csv datasets into
//! more digestible csvs that compute functions operate
//! against.

use anyhow::bail;
use chrono::{NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct EnergyPriceCsvRow {
    pub timestamp: String,
    pub hour: u32,
    pub minute: u32,
    // locational marginal price
    pub lmp_avg: f32,
}

pub fn convert_energy_price_csv(inputs: &[impl AsRef<Path>], output: &Path) -> anyhow::Result<()> {
    let mut out_csv = csv::Writer::from_path(output)?;
    for input in inputs {
        let mut reader = csv::ReaderBuilder::new().flexible(true).from_path(input)?;

        for line in reader.records().skip(4) {
            let line = line?;
            if line.len() != 17 {
                bail!("Unexpected csv row format: {line:?}");
            }

            let lmp_sum = line
                .iter()
                .skip(5)
                .take(3)
                .map(|entry| entry.parse::<f32>())
                .try_fold(0., |acc, el| el.map(|num| num + acc))?;
            let timestamp = NaiveDateTime::parse_from_str(&line[1], "%Y-%m-%d %H:%M:%S")?;
            out_csv.serialize(&EnergyPriceCsvRow {
                timestamp: line[0].to_string(),
                hour: timestamp.hour(),
                minute: timestamp.minute(),
                // lmp_sum adds the three different zones. This averages them.
                lmp_avg: lmp_sum / 3.,
            })?;
        }
    }
    Ok(())
}
