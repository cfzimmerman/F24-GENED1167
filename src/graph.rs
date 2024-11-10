//! ### Graph
//! Displays results from the `compute` module in shareable format.

use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::series::Histogram;
use plotters::style::Color;
use plotters::style::RGBColor;
use plotters::style::RED;
use plotters::style::WHITE;
use std::path::Path;

pub struct Graphing<'a> {
    path: &'a Path,
}

impl<'a> Graphing<'a> {
    const CHART_COLOR: RGBColor = WHITE;

    pub fn new(path: &'a Path) -> Self {
        Graphing { path }
    }

    pub fn hourly_price(&self, prices: &[f32]) -> anyhow::Result<()> {
        let root = BitMapBackend::new(self.path, (1080, 720)).into_drawing_area();
        root.fill(&Self::CHART_COLOR)?;

        let max_price = prices.iter().fold(prices[0], |acc, el| el.max(acc));
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(72)
            .y_label_area_size(72)
            .margin(20)
            .caption("Daily average price/MWh", ("sans-serif", 40.))
            .build_cartesian_2d(0..(prices.len()), 0f32..max_price)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .bold_line_style(WHITE.mix(0.3))
            .y_desc("$/MWh")
            .x_desc("Time of day")
            .axis_desc_style(("sans-serif", 30))
            .x_label_formatter(&|&idx| format!("{:02}:{:02}", (idx * 5) / 60, (idx * 5) % 60))
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
}
