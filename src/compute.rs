//! ### Compute
//! Calculations on energy price and production caiso data
//! preprocessed through the `convert` module.

use crate::convert::{EnergyGenCsvRow, EnergyPriceCsvRow};
use anyhow::bail;
use std::path::Path;

pub struct Compute<'a> {
    path: &'a Path,
}

impl<'a> Compute<'a> {
    const MINS_PER_DAY: usize = 24 * 60;
    const MINS_INCR: usize = 5;

    // I assume every day has an equal number of data points. I allow up to
    // this many missed times before warning that the data doesn't look
    // how I think it does.
    const RESULT_MISS_MAX: usize = 8;

    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }

    /// Returns the index in a 24-hour block of five-minute windows that this time should fill.
    pub fn time_to_idx_5min(hour: u32, minute: u32) -> usize {
        ((hour * 60) + minute) as usize / Self::MINS_INCR
    }

    pub fn idx_5min_to_time(idx: usize) -> (u32, u32) {
        ((idx as u32 * 5) / 60, (idx as u32 * 5) % 60)
    }

    pub fn average_gen_5min(&self) -> anyhow::Result<Vec<[f32; 14]>> {
        let mut reader = csv::Reader::from_path(self.path)?;
        let mut results: Vec<[f32; 14]> = (0..(Self::MINS_PER_DAY / Self::MINS_INCR))
            .map(|idx| {
                let (hour, minute) = Self::idx_5min_to_time(idx);
                EnergyGenCsvRow {
                    hour,
                    minute,
                    ..Default::default()
                }
                .sources()
            })
            .collect();
        let mut counts = vec![0; results.len()];

        for line in reader.deserialize() {
            let line: EnergyGenCsvRow = line?;
            let sources = line.sources();
            let idx = Self::time_to_idx_5min(line.hour, line.minute);
            for (res_src, src_val) in results[idx].iter_mut().zip(sources.iter()) {
                *res_src += src_val;
            }
            counts[idx] += 1;
        }

        for (total, ct) in results.iter_mut().zip(&counts) {
            if ct.max(&counts[0]) - ct.min(&counts[0]) > Self::RESULT_MISS_MAX {
                bail!(
                    "Distrib is not even: diff({}, {ct}) > {}",
                    counts[0],
                    Self::RESULT_MISS_MAX
                );
            }
            for val in total.iter_mut() {
                *val /= *ct as f32;
            }
        }

        Ok(results)
    }

    pub fn average_price_5min(&self) -> anyhow::Result<Vec<f32>> {
        let mut reader = csv::Reader::from_path(self.path)?;

        // (60 mins / 5 min increments) * 24 hours
        let mut results = vec![0.; Self::MINS_PER_DAY / Self::MINS_INCR];
        let mut counts = vec![0; results.len()];

        for line in reader.deserialize() {
            let line: EnergyPriceCsvRow = line?;
            let idx = Self::time_to_idx_5min(line.hour, line.minute);
            results[idx] += line.lmp_avg;
            counts[idx] += 1;
        }

        for (total, ct) in results.iter_mut().zip(&counts) {
            // I assume every day has an equal number of data points. I allow up to
            // this many missed times before warning that the data doesn't look
            // how I think it does.
            if ct.max(&counts[0]) - ct.min(&counts[0]) > 8 {
                bail!("Distrib is not even: diff({}, {ct}) > target", counts[0]);
            }
            *total /= *ct as f32;
        }

        Ok(results)
    }
}
