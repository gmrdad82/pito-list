use std::borrow::Cow;
use std::hint::black_box;
use std::time::{Duration, Instant};

use pito_list::{Cell, Column, Key, List, ListView, Mark, Part, Row, Source, Styles};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

const WIDTH: u16 = 150;
const HEIGHT: u16 = 40;
const ROWS: usize = 20_000;
const LAZY: usize = 100_000;
const GROUP: usize = 25;

fn row(n: usize) -> Row {
    Row::new([
        Cell::new(format!("item {n} ținută 日本語")),
        Cell::new("v1 → v2").faint(),
        Cell::new("owner"),
        Cell::new("running").style(Style::new().fg(Color::Green)),
        Cell::new("a message that is long enough to be clipped at the right edge"),
    ])
    .mark(Mark::new("⧗").style(Style::new().fg(Color::Green)))
}

struct Grouped;

impl Source for Grouped {
    fn len(&self) -> usize {
        LAZY
    }

    fn row(&self, index: usize) -> Cow<'_, Row> {
        Cow::Owned(if index.is_multiple_of(GROUP) {
            Row::heading([
                Part::new(format!("Group {}", index / GROUP))
                    .style(Style::new().add_modifier(Modifier::BOLD)),
                Part::new(format!(" · {}", GROUP - 1)).faint(),
            ])
        } else {
            row(index)
        })
    }

    fn selectable(&self, index: usize) -> bool {
        !index.is_multiple_of(GROUP)
    }

    fn mark_width(&self) -> u16 {
        1
    }
}

fn run<S: Source>(
    list: &mut List<S>,
    frames: u32,
    styles: Styles,
    columns: &[Column],
) -> (Duration, Duration) {
    let area = Rect::new(0, 0, WIDTH, HEIGHT);
    let mut buffer = Buffer::empty(area);
    let mut total = Duration::ZERO;
    let mut worst = Duration::ZERO;
    for frame in 0..frames {
        buffer.reset();
        let key = if frame % 64 < 48 {
            Key::Down
        } else {
            Key::PageUp
        };
        list.key(key);
        let started = Instant::now();
        ListView::new(list, columns)
            .styles(styles)
            .header(true)
            .end(Some("· end ·"))
            .render(area, &mut buffer);
        let took = started.elapsed();
        black_box(&buffer);
        total += took;
        worst = worst.max(took);
    }
    (total / frames.max(1), worst)
}

fn main() {
    let frames: u32 = std::env::args()
        .nth(1)
        .and_then(|text| text.parse().ok())
        .unwrap_or(20_000);
    let styles = Styles::new()
        .selected(
            Style::new()
                .fg(Color::Rgb(0xff, 0xcf, 0x5c))
                .add_modifier(Modifier::BOLD),
        )
        .faint(Style::new().add_modifier(Modifier::DIM));
    let columns = [
        Column::new("Name", 10, 24).pinned(),
        Column::new("Version", 8, 12).priority(4),
        Column::new("Owner", 5, 12).priority(2),
        Column::new("State", 5, 10).priority(3),
        Column::new("Message", 8, 0),
    ];
    let mut owned = List::new().with_rows((0..ROWS).map(row));
    let (mean, worst) = run(&mut owned, frames, styles, &columns);
    println!(
        "pito-list bench: {frames} frames of a {ROWS}-row list at {WIDTH}x{HEIGHT}: mean {:.1} µs, worst {:.1} µs",
        mean.as_secs_f64() * 1e6,
        worst.as_secs_f64() * 1e6
    );
    let mut lazy = List::from_source(Grouped);
    let (mean, worst) = run(&mut lazy, frames, styles, &columns);
    println!(
        "pito-list bench: {frames} frames of a lazy {LAZY}-row list in groups of {GROUP} at {WIDTH}x{HEIGHT}: mean {:.1} µs, worst {:.1} µs",
        mean.as_secs_f64() * 1e6,
        worst.as_secs_f64() * 1e6
    );
}
