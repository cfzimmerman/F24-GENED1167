//! ### Compute
//! Calculations on energy price and production caiso data
//! preprocessed through the `convert` module.

use crate::convert::{EnergyGenCsvRow, EnergyPriceCsvRow};
use anyhow::bail;
use chrono::NaiveDateTime;
use csv::DeserializeRecordsIntoIter;
use std::{array, cmp::Ordering, fs::File, iter::Peekable, path::Path};

pub struct Compute<'a> {
    path: &'a Path,
}

struct PriceGenIter {
    prices: Peekable<DeserializeRecordsIntoIter<File, EnergyPriceCsvRow>>,
    gen: Peekable<DeserializeRecordsIntoIter<File, EnergyGenCsvRow>>,
}

impl<'a> Compute<'a> {
    const MINS_PER_DAY: usize = 24 * 60;
    const MINS_INCR: usize = 5;

    // I assume every timeslot has an equal number of data points. Allow up to
    // this many missed times before warning that the data doesn't look
    // how I think it does.
    const MAX_WINDOW_MISS: usize = 12;

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

    pub fn average_gen_5min(&self) -> anyhow::Result<Vec<[f64; 14]>> {
        self.average_gen_5min_custom(|_| ())
    }

    pub fn average_gen_solar_battery(&self) -> anyhow::Result<Vec<[f64; 14]>> {
        let battery_idx = Self::battery_idx();
        let solar_idx = Self::solar_idx();

        self.average_gen_5min_custom(|row| {
            row[solar_idx] += row[battery_idx];
            row[battery_idx] = 0.;
        })
    }

    fn average_gen_5min_custom(
        &self,
        gen_mod: impl Fn(&mut [f64; 14]),
    ) -> anyhow::Result<Vec<[f64; 14]>> {
        let mut reader = csv::Reader::from_path(self.path)?;
        let mut results: Vec<[f64; 14]> = (0..(Self::MINS_PER_DAY / Self::MINS_INCR))
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
            let mut sources = line.sources();
            gen_mod(&mut sources);
            let idx = Self::time_to_idx_5min(line.hour, line.minute);
            for (res_src, src_val) in results[idx].iter_mut().zip(sources.iter()) {
                *res_src += src_val;
            }
            counts[idx] += 1;
        }

        for (total, ct) in results.iter_mut().zip(&counts) {
            if ct.max(&counts[0]) - ct.min(&counts[0]) > Self::MAX_WINDOW_MISS {
                bail!(
                    "Distrib is not even: diff({}, {ct}) > {}",
                    counts[0],
                    Self::MAX_WINDOW_MISS
                );
            }
            for val in total.iter_mut() {
                *val /= *ct as f64;
            }
        }

        Ok(results)
    }

    pub fn average_price_5min(&self) -> anyhow::Result<Vec<f64>> {
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
            if ct.max(&counts[0]) - ct.min(&counts[0]) > Self::MAX_WINDOW_MISS {
                bail!("Distrib is not even: diff({}, {ct}) > target", counts[0]);
            }
            *total /= *ct as f64;
        }

        Ok(results)
    }

    pub fn average_value_5min(
        price_csv: &Path,
        gen_csv: &Path,
    ) -> anyhow::Result<([f64; 14], [f64; 14])> {
        Self::average_value_5min_custom(price_csv, gen_csv, |_| ())
    }

    pub fn average_value_solar_battery(
        price_csv: &Path,
        gen_csv: &Path,
    ) -> anyhow::Result<([f64; 14], [f64; 14])> {
        let battery_idx = Self::battery_idx();
        let solar_idx = Self::solar_idx();
        Self::average_value_5min_custom(price_csv, gen_csv, |row| {
            row[solar_idx] += row[battery_idx];
            row[battery_idx] = 0.;
        })
    }

    fn battery_idx() -> usize {
        const BATTERY_IDX: usize = 1;
        let mut key_iter = EnergyGenCsvRow::source_keys();
        let keys: [&'static str; 14] = array::from_fn(|_| key_iter.next().unwrap().0);
        assert!(keys[BATTERY_IDX] == "Batteries");
        BATTERY_IDX
    }

    fn solar_idx() -> usize {
        const SOLAR_IDX: usize = 12;
        let mut key_iter = EnergyGenCsvRow::source_keys();
        let keys: [&'static str; 14] = array::from_fn(|_| key_iter.next().unwrap().0);
        assert!(keys[SOLAR_IDX] == "Solar");
        SOLAR_IDX
    }

    fn average_value_5min_custom(
        price_csv: &Path,
        gen_csv: &Path,
        gen_mod: impl Fn(&mut [f64; 14]),
    ) -> anyhow::Result<([f64; 14], [f64; 14])> {
        let mut accs = [0f64; 14];
        let mut qtys = [0f64; 14];

        for (price, gen) in Self::try_iter_price_gen(price_csv, gen_csv)? {
            let mut sources = gen.sources();
            gen_mod(&mut sources);
            for (idx, qty) in sources.iter().copied().enumerate() {
                qtys[idx] += qty.abs();
                accs[idx] += qty * price.lmp_avg;
            }
        }

        for (idx, total) in accs.iter_mut().enumerate() {
            if qtys[idx] != 0. {
                *total /= qtys[idx];
            }
        }

        Ok((accs, qtys))
    }

    /// Creates an iterator over joined price + generation data occuring at the same
    /// timestamps. The data is spotty at places, and this ensures the timestamps
    /// line up between the two.
    fn try_iter_price_gen(prices_csv: &Path, gen_csv: &Path) -> anyhow::Result<PriceGenIter> {
        Ok(PriceGenIter {
            prices: csv::Reader::from_path(prices_csv)?
                .into_deserialize()
                .peekable(),
            gen: csv::Reader::from_path(gen_csv)?
                .into_deserialize()
                .peekable(),
        })
    }
}

impl Iterator for PriceGenIter {
    type Item = (EnergyPriceCsvRow, EnergyGenCsvRow);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (price, gen) = match (self.prices.peek(), self.gen.peek()) {
                (Some(Err(e)), _) | (_, Some(Err(e))) => {
                    eprintln!("{e}");
                    return None;
                }
                (None, _) | (_, None) => return None,
                (Some(Ok(p)), Some(Ok(g))) => (p, g),
            };
            let price_time =
                NaiveDateTime::parse_from_str(&price.timestamp, "%Y-%m-%d %H:%M:%S").ok()?;
            let gen_time =
                NaiveDateTime::parse_from_str(&gen.local_timestamp_start, "%Y-%m-%d %H:%M:%S")
                    .ok()?;
            match price_time.cmp(&gen_time) {
                Ordering::Equal => break,
                Ordering::Greater => {
                    // println!(
                    //     "Unequal time: price {} v. gen {}",
                    //     &price.timestamp, &gen.local_timestamp_start
                    // );
                    self.gen.next();
                }
                Ordering::Less => {
                    // println!(
                    //     "Unequal time: price {} v. gen {}",
                    //     &price.timestamp, &gen.local_timestamp_start
                    // );
                    self.prices.next();
                }
            }
        }
        let (Some(Ok(price)), Some(Ok(gen))) = (self.prices.next(), self.gen.next()) else {
            return None;
        };
        debug_assert!(price.timestamp == gen.local_timestamp_start);
        Some((price, gen))
    }
}
