use clap::Parser;
use energy_analysis::{compute::Compute, convert, graph::Graphing};
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
enum Args {
    /// Takes a raw 5-min zone price data CSV from
    /// https://www.eia.gov/electricity/wholesalemarkets/data.php?rto=caiso
    /// and simplifies it into a form more suitable for sqlite queries.
    /*
    cargo run parse-price-csv \
        --caiso-csv \
        data/caiso_lmp_rt_5min_zones_2023Q4.csv \
        data/caiso_lmp_rt_5min_zones_2024Q1.csv \
        data/caiso_lmp_rt_5min_zones_2024Q2.csv \
        data/caiso_lmp_rt_5min_zones_2024Q3.csv \
        --output-csv results/prices.csv
    */
    ParsePriceCsv {
        /// A list of input CSV files to aggregate into a single output.
        /// Expected file format is that of `caiso_lmp_rt_5min_zones_202*Q*.csv`
        #[clap(short, long, num_args = 1.., value_delimiter = ' ')]
        caiso_csv: Vec<PathBuf>,

        /// An output file that the simplified inputs are written to
        #[clap(short, long)]
        output_csv: PathBuf,
    },
    /// Takes the output of parse-price-csv and renders it as a png at
    /// the given output_png location.
    // cargo run graph-price-hourly results/prices.csv results/prices.png
    GraphPriceHourly {
        /// A csv of the form output by ParsePriceCsv
        price_csv: PathBuf,

        /// Where the output PNG file will be written.
        output_png: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::ParsePriceCsv {
            caiso_csv: input,
            output_csv: output,
        } => {
            convert::convert_energy_price_csv(&input, &output)?;
        }
        Args::GraphPriceHourly {
            price_csv,
            output_png,
        } => {
            let prices = Compute::new(&price_csv).average_price_5min()?;
            Graphing::new(&output_png).hourly_price(&prices)?;
        }
    }
    Ok(())
}
