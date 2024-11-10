//! ### Convert
//! Tools for converting raw CAISO csv datasets into
//! more digestible csvs that compute functions operate
//! against.

use anyhow::bail;
use chrono::{NaiveDateTime, Timelike};
use csv::StringRecord;
use serde::{Deserialize, Serialize};
use std::{ops::AddAssign, path::Path};

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

// repr(c) because field order matters a lot for csv parsing
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EnergyGenCsvRow {
    pub utc_timestamp: String,
    pub local_timestamp_start: String,
    pub local_timestamp_end: String,
    pub local_date: String,
    pub hour: u32,

    pub total: f32,
    pub battery: f32,
    pub biogas: f32,
    pub biomass: f32,
    pub coal: f32,
    pub geothermal: f32,
    pub imports: f32,
    pub large_hydro: f32,
    pub natural_gas: f32,
    pub nuclear: f32,
    pub other: f32,
    pub small_hydro: f32,
    pub solar: f32,
    pub wind: f32,

    #[serde(default)]
    pub minute: u32,
}

pub fn convert_energy_gen_csv(inputs: &[impl AsRef<Path>], output: &Path) -> anyhow::Result<()> {
    let mut out_csv = csv::Writer::from_path(output)?;
    for input in inputs {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_path(input)?;
        EnergyGenCsvRow::validate(
            &reader
                .records()
                .nth(3)
                .ok_or_else(|| anyhow::anyhow!("Empty CSV"))??,
        )?;

        let mut failed_lines = 0;
        for line in reader.deserialize::<EnergyGenCsvRow>() {
            let Ok(mut line) = line else {
                // println!("Skipping failed line: {line:?}");
                failed_lines += 1;
                continue;
            };

            // Compute timestamp manually for consistency with other conversions.
            let timestamp =
                NaiveDateTime::parse_from_str(&line.local_timestamp_start, "%Y-%m-%d %H:%M:%S")?;
            line.hour = timestamp.hour();
            line.minute = timestamp.minute();

            out_csv.serialize(line)?;
        }
        println!("{:?} had {failed_lines} failed lines", input.as_ref());
    }

    Ok(())
}

impl EnergyGenCsvRow {
    const HEADER_KEYWORDS: [&'static str; 19] = [
        "Timestamp",
        "Beginning",
        "Ending",
        "Date",
        "Hour",
        "Total",
        "Batteries",
        "Biogas",
        "Biomass",
        "Coal",
        "Geothermal",
        "Imports",
        "Large Hydro",
        "Gas",
        "Nuclear",
        "Other",
        "Small Hydro",
        "Solar",
        "Wind",
    ];

    fn validate(header: &StringRecord) -> anyhow::Result<()> {
        for (keyword, col_name) in Self::HEADER_KEYWORDS.iter().zip(header.iter()) {
            if !col_name.contains(keyword) {
                bail!("Expected column '{col_name}' to have keyword {keyword}");
            }
        }
        Ok(())
    }

    pub fn div_assign(&mut self, div: f32) {
        self.total /= div;
        self.battery /= div;
        self.biogas /= div;
        self.biomass /= div;
        self.coal /= div;
        self.geothermal /= div;
        self.imports /= div;
        self.large_hydro /= div;
        self.natural_gas /= div;
        self.nuclear /= div;
        self.other /= div;
        self.small_hydro /= div;
        self.solar /= div;
        self.wind /= div;
    }

    pub fn source_max() {
        // TODO: Restart here. Graphing needs source_max and source_min for bounds on the charts.
        // Include total?
    }
}

impl AddAssign for EnergyGenCsvRow {
    fn add_assign(&mut self, other: Self) {
        assert!(
            self.hour == other.hour && self.minute == other.minute,
            "Hour and minute values must be identical to add: {self:#?} += {other:#?}"
        );
        self.total += other.total;
        self.battery += other.battery;
        self.biogas += other.biogas;
        self.biomass += other.biomass;
        self.coal += other.coal;
        self.geothermal += other.geothermal;
        self.imports += other.imports;
        self.large_hydro += other.large_hydro;
        self.natural_gas += other.natural_gas;
        self.nuclear += other.nuclear;
        self.other += other.other;
        self.small_hydro += other.small_hydro;
        self.solar += other.solar;
        self.wind += other.wind;
    }
}
