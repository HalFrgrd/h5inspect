use crate::ui;
use plotters::coord;
use plotters::prelude::DrawingArea;
use plotters::prelude::*;
use plotters::prelude::{ChartBuilder, LabelAreaPosition};
use plotters_ratatui_backend::{
    widget_fn, AreaResult, Draw, Error, PlottersWidget, RatatuiBackend,
};

use crate::analysis;
use crate::num_utils;

pub fn histogram_widget(
    hist_data: &analysis::HistogramData,
    _widget_height: u16,
    widget_width: u16,
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

    // let min_cnt = *counts.iter().min().unwrap();
    let max_cnt = *counts.iter().max().unwrap();

    let draw_fn = move |area: DrawingArea<RatatuiBackend, coord::Shift>| -> AreaResult {
        let mut chart = ChartBuilder::on(&area)
            .margin(1)
            .set_label_area_size(LabelAreaPosition::Left, 30)
            .set_label_area_size(LabelAreaPosition::Bottom, 10)
            // .caption("Histogram Test", ("sans-serif", 50.0).into_font().color(&WHITE))
            .build_cartesian_2d(min_bin..(max_bin + bin_width), 0..max_cnt)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            // .bold_line_style(WHITE)
            // .y_desc("Count")
            // .x_desc("Bucket")
            .y_label_formatter(&|y| num_utils::large_int_fmt(*y as u64))
            .x_labels(2u16.max(widget_width / 10).into())
            .x_label_formatter(&|x| num_utils::basic_float_fmt(*x))
            // .axis_desc_style(("sans-serif", 15).into_font().color(&WHITE))
            .axis_style(ShapeStyle {
                color: WHITE.into(),
                filled: true,
                stroke_width: 0,
            })
            .label_style(("sans-serif", 15).into_font().color(&WHITE))
            .draw()?;

        // Draw histogram bars
        for (i, &count) in counts.iter().enumerate() {
            let x0 = min_bin + i as f32 * bin_width;

            let num_lines: usize = 30;
            let step = bin_width / (num_lines as f32);
            for j in 6..(num_lines - 5) {
                let x = x0 + j as f32 * step;
                chart.draw_series(LineSeries::new(
                    vec![(x, 0), (x, count)],
                    &RGBColor(ui::MAGENTA_R, ui::MAGENTA_G, ui::MAGENTA_B),
                ))?;
            }
        }

        area.present()
    };

    return widget_fn(draw_fn);
}
