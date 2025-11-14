use ratatui::{
    style::Style,
    widgets::{Block, Sparkline as RatatuiSparkline},
};

pub fn create_tps_sparkline<'a>(
    data: &'a [u64],
    block: Block<'a>,
    style: Style,
) -> RatatuiSparkline<'a> {
    RatatuiSparkline::default()
        .block(block)
        .data(data)
        .style(style)
}