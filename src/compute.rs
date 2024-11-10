//! ### Compute
//! Calculations on energy price and production caiso data
//! preprocessed through the `convert` module.

use crate::convert::EnergyPriceCsvRow;
use anyhow::bail;
use std::path::Path;

pub struct Compute<'a> {
    path: &'a Path,
}

impl<'a> Compute<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }

    pub fn average_price_5min(&self) -> anyhow::Result<Vec<f32>> {
        const MINS_PER_DAY: usize = 24 * 60;
        const MINS_INCR: usize = 5;

        let mut reader = csv::Reader::from_path(self.path)?;

        // (60 mins / 5 min increments) * 24 hours
        let mut results = vec![0.; MINS_PER_DAY / MINS_INCR];
        let mut counts = vec![0; results.len()];

        for line in reader.deserialize() {
            let line: EnergyPriceCsvRow = line?;
            let idx = ((line.hour * 60) + line.minute) as usize / MINS_INCR;
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
