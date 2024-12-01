#!/bin/bash

set -e

cargo run --release parse-price-csv \
  --caiso-csv \
  data/caiso_lmp_rt_5min_zones_2023Q4.csv \
  data/caiso_lmp_rt_5min_zones_2024Q1.csv \
  data/caiso_lmp_rt_5min_zones_2024Q2.csv \
  data/caiso_lmp_rt_5min_zones_2024Q3.csv \
  --output-csv data/prices.csv

cargo run --release parse-gen-csv \
  --caiso-csv \
  data/caiso_gen_all_5min_2023Q4.csv \
  data/caiso_gen_all_5min_2024Q1.csv \
  data/caiso_gen_all_5min_2024Q2.csv \
  data/caiso_gen_all_5min_2024Q3.csv \
  --output-csv data/gen.csv

cargo run --release write-price-minutes data/prices.csv results/prices_avg.csv

cargo run --release write-gen-minutes data/gen.csv results/gen_avg.csv

cargo run --release write-gen-solar-battery data/gen.csv results/gen_solar_battery.csv

cargo run --release write-value-minutes data/prices.csv data/gen.csv results/values_avg.csv

cargo run --release write-value-solar-battery data/prices.csv data/gen.csv results/values_solar_battery.csv

cargo run --release graph-price-minutes data/prices.csv results/prices.png

cargo run --release graph-gen-minutes data/gen.csv results/gen.png

cargo run --release graph-gen-solar-battery data/gen.csv results/gen_solar_battery.png

cargo run --release graph-value-minutes data/prices.csv data/gen.csv results/values.png

cargo run --release graph-value-solar-battery data/prices.csv data/gen.csv results/solar_battery.png
