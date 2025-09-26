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

pub fn histogram_widget() -> PlottersWidget<impl Draw, impl Fn(DrawingAreaErrorKind<Error>)> {
    fn draw_fn(area: DrawingArea<RatatuiBackend, coord::Shift>) -> AreaResult {
        let mut chart = ChartBuilder::on(&area)
            .x_label_area_size(10)
            .y_label_area_size(20)
            .margin(1)
            .caption("Histogram Test", ("sans-serif", 50.0))
            .build_cartesian_2d((0u32..10u32).into_segmented(), 0u32..10u32)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(WHITE)
            .y_desc("Count")
            .x_desc("Bucket")
            .axis_desc_style(("sans-serif", 15))
            .draw()?;

        let data = [
            0u32, 1, 1, 1, 4, 2, 5, 7, 8, 6, 4, 2, 1, 8, 3, 3, 3, 4, 4, 3, 3, 3,
        ];

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(RED.filled())
                .data(data.iter().map(|x: &u32| (*x, 1))),
        )?;
        area.present()
    }

    return widget_fn(draw_fn);
}
