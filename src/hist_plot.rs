use plotters::coord;
use plotters::prelude::DrawingArea;
use plotters::prelude::*;
use plotters::prelude::{ChartBuilder, LabelAreaPosition, SeriesLabelPosition};
use plotters::series::LineSeries;
use plotters::style::Color as PlottersColor;
use plotters::style::{IntoTextStyle as _, RGBColor};
use plotters_ratatui_backend::{
    widget_fn, AreaResult, Draw, Error, PlottersWidget, RatatuiBackend,
};

use crate::analysis;

pub fn histogram_widget(
    hist_data: &analysis::HistogramData,
) -> PlottersWidget<impl Draw, impl Fn(DrawingAreaErrorKind<Error>)> {
    let bins = hist_data.iter().map(|&(b, _)| b).collect::<Vec<f32>>();
    let counts = hist_data.iter().map(|&(_, c)| c).collect::<Vec<u32>>();

    let min_bin = bins.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_bin = bins.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let bin_width = if bins.len() > 1 {
        (max_bin - min_bin) / (bins.len() - 1) as f32
    } else {
        1.0
    };

    let min_cnt = *counts.iter().min().unwrap();
    let max_cnt = *counts.iter().max().unwrap();

    let draw_fn = move |area: DrawingArea<RatatuiBackend, coord::Shift>| -> AreaResult {
        let mut chart = ChartBuilder::on(&area)
            .x_label_area_size(10)
            .y_label_area_size(20)
            .margin(1)
            // .caption("Histogram Test", ("sans-serif", 50.0).into_font().color(&WHITE))
            .build_cartesian_2d(min_bin..max_bin, 0..max_cnt)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .bold_line_style(WHITE)
            .y_desc("Count")
            .x_desc("Bucket")
            .axis_desc_style(("sans-serif", 15).into_font().color(&WHITE))
            .axis_style(ShapeStyle {
                color: WHITE.into(),
                filled: false,
                stroke_width: 1,
            })
            .label_style(("sans-serif", 15).into_font().color(&WHITE))
            .draw()?;

        // Draw histogram bars
        for (i, &count) in counts.iter().enumerate() {
            let x0 = min_bin + i as f32 * bin_width;
            let x1 = x0 + bin_width;
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x0, 0), (x1, count)],
                MAGENTA.filled(), // nice bright color for dark bg
            )))?;
        }

        area.present()
    };

    return widget_fn(draw_fn);
}
