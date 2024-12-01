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
        --output-csv data/prices.csv
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
        --output-csv data/gen.csv
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
    // cargo run write-price-minutes data/prices.csv results/prices_avg.csv
    WritePriceMinutes {
        /// A csv of the form output by parse-price-csv
        csv_in: PathBuf,

        /// Where the output csv will be written
        csv_out: PathBuf,
    },

    /// Takes the output of parse-gen-csv and records the generation
    /// distribution five-minute averages into the output csv. The
    /// same data is charted in the graph-gen-minutes function.
    // cargo run write-gen-minutes data/gen.csv results/gen_avg.csv
    WriteGenMinutes {
        /// A csv of the form output by parse-gen-csv
        csv_in: PathBuf,

        /// Where the output csv will be written
        csv_out: PathBuf,
    },

    /// Same as write-gen-minutes but merges solar and battery columns.
    // cargo run write-gen-solar-battery data/gen.csv results/gen_solar_battery.csv
    WriteGenSolarBattery {
        /// A csv of the form output by parse-gen-csv
        csv_in: PathBuf,

        /// Where the output csv will be written
        csv_out: PathBuf,
    },

    /// Writes the values from graph-value-minutes into a CSV.
    // cargo run write-value-minutes data/prices.csv data/gen.csv results/values_avg.csv
    WriteValueMinutes {
        /// A csv of the form output by parse-price-csv
        price_csv: PathBuf,

        /// A csv of the form output by parse-gen-csv
        gen_csv: PathBuf,

        /// Where the output csv will be written
        csv_out: PathBuf,
    },

    /// Writes value-minutes under the hypothetical of merged solar + battery.
    // cargo run write-value-solar-battery data/prices.csv data/gen.csv
    // results/values_solar_battery.csv
    WriteValueSolarBattery {
        /// A csv of the form output by parse-price-csv
        price_csv: PathBuf,

        /// A csv of the form output by parse-gen-csv
        gen_csv: PathBuf,

        /// Where the output csv will be written
        csv_out: PathBuf,
    },

    /// Takes the output of parse-price-csv and renders it as a png at
    /// the given output_png location.
    // cargo run graph-price-minutes data/prices.csv results/prices.png
    GraphPriceMinutes {
        /// A csv of the form output by ParsePriceCsv
        price_csv: PathBuf,

        /// Where the output PNG file will be written.
        output_png: PathBuf,
    },

    /// Takes the output of parse-price-csv and renders it as a png at
    /// the given output_png location.
    // cargo run graph-gen-minutes data/gen.csv results/gen.png
    GraphGenMinutes {
        gen_csv: PathBuf,
        output_png: PathBuf,
    },

    /// graph-gen-minutes but merges the solar and battery columns
    // cargo run graph-gen-solar-battery data/gen.csv results/gen_solar_battery.png
    GraphGenSolarBattery {
        gen_csv: PathBuf,
        output_png: PathBuf,
    },

    /// Takes the output of both parse-price-csv and parse-gen-csv and
    /// writes a graph displaying the average dollar value of each type
    /// of electricity.
    // cargo run graph-value-minutes data/prices.csv data/gen.csv results/values.png
    GraphValueMinutes {
        /// A csv output by parse-price-csv
        price_csv: PathBuf,

        /// A csv output by parse-gen-csv
        gen_csv: PathBuf,

        /// A png file where the graph should be written.
        output_png: PathBuf,
    },

    /// Graphs value-minutes but adds solar + battery output into a
    /// single column.
    // cargo run graph-value-solar-battery data/prices.csv data/gen.csv results/solar-battery.png
    GraphValueSolarBattery {
        /// A csv output by parse-price-csv
        price_csv: PathBuf,

        /// A csv output by parse-gen-csv
        gen_csv: PathBuf,

        /// A png file where the graph should be written.
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
        Args::WriteGenSolarBattery { csv_in, csv_out } => {
            let gen = Compute::new(&csv_in).average_gen_solar_battery()?;
            convert::write_energy_gen_averages(&csv_out, &gen)?;
        }
        Args::WriteValueMinutes {
            price_csv,
            gen_csv,
            csv_out,
        } => {
            let (values, qtys) = Compute::average_value_5min(&price_csv, &gen_csv)?;
            convert::write_energy_value_averages(&csv_out, &values, &qtys)?;
        }
        Args::WriteValueSolarBattery {
            price_csv,
            gen_csv,
            csv_out,
        } => {
            let (values, qtys) = Compute::average_value_solar_battery(&price_csv, &gen_csv)?;
            convert::write_energy_value_averages(&csv_out, &values, &qtys)?;
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
            Graphing::new(&output_png).daily_gen(&gen, "Daily average generation by source")?;
        }
        Args::GraphGenSolarBattery {
            gen_csv,
            output_png,
        } => {
            let gen = Compute::new(&gen_csv).average_gen_solar_battery()?;
            Graphing::new(&output_png).daily_gen(&gen, "Daily average Solar + Battery")?;
        }
        Args::GraphValueMinutes {
            price_csv,
            gen_csv,
            output_png,
        } => {
            let (values, _qtys) = Compute::average_value_5min(&price_csv, &gen_csv)?;
            Graphing::new(&output_png).avg_value(&values, "Daily average price/MWh")?;
        }
        Args::GraphValueSolarBattery {
            price_csv,
            gen_csv,
            output_png,
        } => {
            let (values, _qtys) = Compute::average_value_solar_battery(&price_csv, &gen_csv)?;
            Graphing::new(&output_png).avg_value(&values, "Solar + Battery price/MWh")?;
        }
    }
    Ok(())
}
