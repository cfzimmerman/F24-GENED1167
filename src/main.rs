use clap::Parser;
use energy_analysis::{compute::Compute, convert, graph::Graphing};
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
enum Args {
    /// Takes a raw 5-min zone price data CSV from
    /// https://www.eia.gov/electricity/wholesalemarkets/data.php?rto=caiso
    /// and simplifies it into a form more suitable for processing.
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

    /// Takes a raw 5-min energy generation source data CSV from
    /// https://www.eia.gov/electricity/wholesalemarkets/data.php?rto=caiso
    /// and simplifies it into a form more suitable for processing.
    /*
    cargo run parse-gen-csv \
        --caiso-csv \
        data/caiso_gen_all_5min_2023Q4.csv \
        data/caiso_gen_all_5min_2024Q1.csv \
        data/caiso_gen_all_5min_2024Q2.csv \
        data/caiso_gen_all_5min_2024Q3.csv \
        --output-csv results/gen.csv
        */
    ParseGenCsv {
        /// A list of input CSV files to aggregate into a single output.
        /// Expected file format is that of `caiso_gen_all_5min_202*Q*.csv`
        #[clap(short, long, num_args = 1.., value_delimiter = ' ')]
        caiso_csv: Vec<PathBuf>,

        /// An output file that the simplified inputs are written to
        #[clap(short, long)]
        output_csv: PathBuf,
    },

    /// Takes the output of parse-price-csv and records the price
    /// five-minute averages into the output csv. The same data
    /// is charted in the graph-price-minutes function.
    // cargo run write-price-minutes results/prices_raw.csv results/prices_avg.csv
    WritePriceMinutes {
        /// A csv of the form output by parse-price-csv
        csv_in: PathBuf,

        /// Where the output csv will be written
        csv_out: PathBuf,
    },

    /// Takes the output of parse-gem-csv and records the generation
    /// distribution five-minute averages into the output csv. The
    /// same data is charted in the graph-gen-minutes function.
    // cargo run write-gen-minutes results/gen_raw.csv results/gen_avg.csv
    WriteGenMinutes {
        /// A csv of the form output by parse-gen-csv
        csv_in: PathBuf,

        /// Where the output csv will be written
        csv_out: PathBuf,
    },

    /// Takes the output of parse-price-csv and renders it as a png at
    /// the given output_png location.
    // cargo run graph-price-minutes results/prices.csv results/prices.png
    GraphPriceMinutes {
        /// A csv of the form output by ParsePriceCsv
        price_csv: PathBuf,

        /// Where the output PNG file will be written.
        output_png: PathBuf,
    },

    /// Takes the output of parse-price-csv and renders it as a png at
    /// the given output_png location.
    // cargo run graph-gen-minutes results/gen.csv results/gen.png
    GraphGenMinutes {
        gen_csv: PathBuf,
        output_png: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Args::parse() {
        Args::ParsePriceCsv {
            caiso_csv: input,
            output_csv: output,
        } => {
            convert::convert_energy_price_csv(&input, &output)?;
        }
        Args::ParseGenCsv {
            caiso_csv,
            output_csv,
        } => {
            convert::convert_energy_gen_csv(&caiso_csv, &output_csv)?;
        }
        Args::WritePriceMinutes { csv_in, csv_out } => {
            let prices = Compute::new(&csv_in).average_price_5min()?;
            convert::write_energy_price_averages(&csv_out, &prices)?;
        }
        Args::WriteGenMinutes { csv_in, csv_out } => {
            let gen = Compute::new(&csv_in).average_gen_5min()?;
            convert::write_energy_gen_averages(&csv_out, &gen)?;
        }
        Args::GraphPriceMinutes {
            price_csv,
            output_png,
        } => {
            let prices = Compute::new(&price_csv).average_price_5min()?;
            Graphing::new(&output_png).daily_price(&prices)?;
        }
        Args::GraphGenMinutes {
            gen_csv,
            output_png,
        } => {
            let gen = Compute::new(&gen_csv).average_gen_5min()?;
            Graphing::new(&output_png).daily_gen(&gen)?;
        }
    }
    Ok(())
}
