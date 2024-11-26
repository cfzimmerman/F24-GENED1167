//! ### Convert
//! Tools for converting raw CAISO csv datasets into
//! more digestible csvs that compute functions operate
//! against.

use anyhow::bail;
use chrono::{NaiveDateTime, Timelike};
use csv::StringRecord;
use plotters::style::{full_palette, RGBColor};
use serde::{Deserialize, Serialize};
use std::array;
use std::fmt::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct EnergyPriceCsvRow {
    pub timestamp: String,
    pub hour: u32,
    pub minute: u32,
    // locational marginal price
    pub lmp_avg: f64,
}

pub fn convert_energy_price_csv(inputs: &[impl AsRef<Path>], output: &Path) -> anyhow::Result<()> {
    let mut out_csv = csv::Writer::from_path(output)?;
    for input in inputs {
        let mut reader = csv::ReaderBuilder::new().flexible(true).from_path(input)?;

        for line in reader.records().skip(3) {
            let line = line?;
            if line.len() != 17 {
                bail!("Unexpected csv row format: {line:?}");
            }

            let lmp_sum = line
                .iter()
                .skip(5)
                .take(3)
                .map(|entry| entry.parse::<f64>())
                .try_fold(0., |acc, el| el.map(|num| num + acc))?;
            let timestamp_string = line[1].to_string();
            let timestamp = NaiveDateTime::parse_from_str(&timestamp_string, "%Y-%m-%d %H:%M:%S")?;
            out_csv.serialize(&EnergyPriceCsvRow {
                timestamp: timestamp_string,
                hour: timestamp.hour(),
                minute: timestamp.minute(),
                // lmp_sum adds the three different zones. This averages them.
                lmp_avg: lmp_sum / 3.,
            })?;
        }
    }
    Ok(())
}

pub fn write_energy_price_averages(output: &Path, prices: &[f64]) -> anyhow::Result<()> {
    let mut csv = csv::Writer::from_path(output)?;

    let mut buf = String::new();
    csv.write_record(["prices".as_bytes()])?;

    for price in prices.iter() {
        write!(&mut buf, "{price}")?;
        csv.write_record([&buf])?;
        buf.clear();
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

    pub total: f64,
    pub battery: f64,
    pub biogas: f64,
    pub biomass: f64,
    pub coal: f64,
    pub geothermal: f64,
    pub imports: f64,
    pub large_hydro: f64,
    pub natural_gas: f64,
    pub nuclear: f64,
    pub other: f64,
    pub small_hydro: f64,
    pub solar: f64,
    pub wind: f64,

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
    const HEADER_KEYWORDS: [(&'static str, RGBColor); 19] = [
        ("Timestamp", full_palette::BLACK),
        ("Beginning", full_palette::BLACK),
        ("Ending", full_palette::BLACK),
        ("Date", full_palette::BLACK),
        ("Hour", full_palette::BLACK),
        ("Total", full_palette::BLACK),
        ("Batteries", full_palette::RED_500),
        ("Biogas", full_palette::GREEN_300),
        ("Biomass", full_palette::GREEN_700),
        ("Coal", full_palette::BLACK),
        ("Geothermal", full_palette::RED_300),
        ("Imports", full_palette::GREY_500),
        ("Large Hydro", full_palette::PURPLE_500),
        ("Natural Gas", full_palette::PINK_300),
        ("Nuclear", full_palette::BLUE_300),
        ("Other", full_palette::GREY_700),
        ("Small Hydro", full_palette::PURPLE_300),
        ("Solar", full_palette::YELLOW_800),
        ("Wind", full_palette::BLUE_900),
    ];

    fn validate(header: &StringRecord) -> anyhow::Result<()> {
        for ((keyword, _), col_name) in Self::HEADER_KEYWORDS.iter().zip(header.iter()) {
            if !col_name.contains(keyword) {
                bail!("Expected column '{col_name}' to have keyword {keyword}");
            }
        }
        Ok(())
    }
    pub fn source_keys() -> impl ExactSizeIterator<Item = (&'static str, RGBColor)> {
        Self::HEADER_KEYWORDS.iter().copied().skip(5)
    }

    pub fn sources(&self) -> [f64; 14] {
        [
            self.total,
            self.battery,
            self.biogas,
            self.biomass,
            self.coal,
            self.geothermal,
            self.imports,
            self.large_hydro,
            self.natural_gas,
            self.nuclear,
            self.other,
            self.small_hydro,
            self.solar,
            self.wind,
        ]
    }
}

pub fn write_energy_gen_averages(output: &Path, gen: &[[f64; 14]]) -> anyhow::Result<()> {
    let mut csv = csv::Writer::from_path(output)?;
    let mut bufs: [String; 14] = array::from_fn(|_| String::new());

    for (key, buf) in EnergyGenCsvRow::source_keys().zip(&mut bufs) {
        write!(buf, "{}", key.0)?;
    }
    csv.write_record(&bufs)?;

    for dist in gen.iter() {
        for (val, buf) in dist.iter().copied().zip(&mut bufs) {
            buf.clear();
            write!(buf, "{val}")?;
        }
        csv.write_record(&bufs)?;
    }

    Ok(())
}

pub fn write_energy_value_averages(
    output: &Path,
    averages: &[f64; 14],
    qtys: &[f64; 14],
) -> anyhow::Result<()> {
    let mut csv = csv::Writer::from_path(output)?;
    let mut bufs = [
        "source".to_string(),
        "avg_price".to_string(),
        "net_mwh".to_string(),
    ];
    csv.write_record(&bufs)?;

    for (((label, _), &avg_price), &qty) in EnergyGenCsvRow::source_keys()
        .zip(averages.iter())
        .zip(qtys.iter())
    {
        for buf in bufs.iter_mut() {
            buf.clear();
        }
        write!(&mut bufs[0], "{label}")?;
        write!(&mut bufs[1], "{avg_price:.2}")?;
        write!(&mut bufs[2], "{qty}")?;
        csv.write_record(&bufs)?;
    }

    Ok(())
}
