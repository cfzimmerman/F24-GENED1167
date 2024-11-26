//! ### Graph
//! Displays results from the `compute` module in shareable format.

use anyhow::anyhow;
use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::chart::SeriesLabelPosition;
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::IntoSegmentedCoord;
use plotters::prelude::Rectangle;
use plotters::prelude::SegmentValue;
use plotters::series::Histogram;
use plotters::series::LineSeries;
use plotters::style::full_palette::BLUE_600;
use plotters::style::Color;
use plotters::style::RGBColor;
use plotters::style::BLACK;
use plotters::style::RED;
use plotters::style::WHITE;
use std::array;
use std::cmp::Ordering;
use std::path::Path;

use crate::compute::Compute;
use crate::convert::EnergyGenCsvRow;

pub struct Graphing<'a> {
    path: &'a Path,
}

impl<'a> Graphing<'a> {
    const CHART_COLOR: RGBColor = WHITE;

    pub fn new(path: &'a Path) -> Self {
        Graphing { path }
    }

    pub fn daily_price(&self, prices: &[f64]) -> anyhow::Result<()> {
        let root = BitMapBackend::new(self.path, (1080, 720)).into_drawing_area();
        root.fill(&Self::CHART_COLOR)?;

        let max_price = prices.iter().fold(prices[0], |acc, el| el.max(acc));
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(72)
            .y_label_area_size(72)
            .margin(20)
            .caption("Daily average price/MWh", ("sans-serif", 40.))
            .build_cartesian_2d(0..(prices.len()), 0f64..max_price)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .bold_line_style(WHITE.mix(0.3))
            .y_desc("$/MWh")
            .x_desc("Time of day")
            .axis_desc_style(("sans-serif", 30))
            .x_label_formatter(&|&idx| {
                let (hour, minute) = Compute::idx_5min_to_time(idx);
                format!("{hour:02}:{minute:02}")
            })
            .y_label_formatter(&|price| format!("${:02}", price))
            .x_labels(24)
            .y_labels(10)
            .x_label_style(("sans-serif", 16))
            .y_label_style(("sans-serif", 16))
            .draw()?;

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(RED.mix(0.5).filled())
                .data(prices.iter().enumerate().map(|(idx, &val)| (idx, val))),
        )?;

        root.present()?;

        Ok(())
    }

    pub fn daily_gen(&self, gen: &[[f64; 14]]) -> anyhow::Result<()> {
        let root = BitMapBackend::new(self.path, (1080, 720)).into_drawing_area();
        root.fill(&Self::CHART_COLOR)?;

        let gen_min = gen
            .iter()
            .flat_map(|arr| arr.iter().skip(1))
            .min_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .ok_or_else(|| anyhow!("Failed to compute chart min"))?;
        let gen_max = gen
            .iter()
            .flat_map(|arr| arr.iter().skip(1))
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .ok_or_else(|| anyhow!("Failed to compute chart max"))?;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(72)
            .y_label_area_size(84)
            .margin(20)
            .caption("Daily average generation by source", ("sans-serif", 40.))
            .build_cartesian_2d(0..(gen.len()), (*gen_min - 250.)..(*gen_max + 250.))?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .bold_line_style(WHITE.mix(0.3))
            .y_desc("MWh")
            .x_desc("Time of day")
            .axis_desc_style(("sans-serif", 30))
            .x_label_formatter(&|&idx| {
                let (hour, minute) = Compute::idx_5min_to_time(idx);
                format!("{hour:02}:{minute:02}")
            })
            .x_labels(24)
            .y_labels(10)
            .x_label_style(("sans-serif", 16))
            .y_label_style(("sans-serif", 16))
            .draw()?;

        for (src_idx, (label, color)) in EnergyGenCsvRow::source_keys().enumerate().skip(1) {
            chart
                .draw_series(LineSeries::new(
                    gen.iter()
                        .enumerate()
                        .map(|(timeslice, arr)| (timeslice, arr[src_idx])),
                    color.stroke_width(3),
                ))?
                .label(label)
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });
        }

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .position(SeriesLabelPosition::UpperRight)
            .label_font(("Calibri", 14))
            .draw()?;

        root.present()?;

        Ok(())
    }

    pub fn avg_value(&self, values: &[f64; 14]) -> anyhow::Result<()> {
        // Don't display `total`
        let mut val_iter = values.iter().copied().skip(1);
        let values: [f64; 13] = array::from_fn(|_| val_iter.next().unwrap());

        let mut label_iter = EnergyGenCsvRow::source_keys().skip(1);
        let labels: [&str; 13] = array::from_fn(|_| label_iter.next().unwrap().0);

        let root = BitMapBackend::new(self.path, (1080, 720)).into_drawing_area();
        root.fill(&Self::CHART_COLOR)?;

        let max_price = values.iter().fold(values[0], |acc, el| el.max(acc));

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(72)
            .y_label_area_size(72)
            .margin(20)
            .caption("Daily average price/MWh", ("sans-serif", 40.))
            .build_cartesian_2d(
                (0..(values.len() - 1)).into_segmented(),
                0f64..(max_price * 1.1),
            )?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .y_desc("$/MWh")
            .x_desc("Electricity source")
            .axis_desc_style(("sans-serif", 30))
            .x_label_formatter(&|seg| match seg {
                SegmentValue::Last | SegmentValue::Exact(_) => "".to_string(),
                SegmentValue::CenterOf(idx) => labels[*idx].to_string(),
            })
            .y_label_formatter(&|price| format!("${price:.2}"))
            .x_labels(20)
            .y_labels(20)
            .x_label_style(("sans-serif", 16))
            .y_label_style(("sans-serif", 16))
            .draw()?;

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(BLUE_600.filled())
                .data(values.iter().enumerate().map(|(idx, &val)| (idx, val))),
        )?;

        root.present()?;

        Ok(())
    }
}
