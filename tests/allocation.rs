use std::{
    alloc::{GlobalAlloc, Layout, System},
    borrow::Cow,
    cell::Cell as Counter,
};

use pito_list::{
    Cell, Column, Key, Keys, List, ListView, Mark, Paging, Part, Row, Source, Step, Styles,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

struct Counting;

thread_local! {
    static COUNTING: Counter<bool> = const { Counter::new(false) };
    static COUNT: Counter<usize> = const { Counter::new(0) };
}

fn note() {
    if COUNTING.with(Counter::get) {
        COUNT.with(|count| count.set(count.get() + 1));
    }
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note();
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        note();
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn allocations(work: impl FnOnce()) -> usize {
    COUNT.with(|count| count.set(0));
    COUNTING.with(|counting| counting.set(true));
    work();
    COUNTING.with(|counting| counting.set(false));
    COUNT.with(Counter::get)
}

const ACCENT: Style = Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD);
const DIM: Style = Style::new().add_modifier(Modifier::DIM);

#[test]
fn drawing_allocates_nothing() {
    assert!(allocations(|| drop(std::hint::black_box(Vec::<u8>::with_capacity(8)))) > 0);
    let styles = Styles::new().selected(ACCENT).faint(DIM);
    let columns = [
        Column::new("Name", 8, 14).pinned(),
        Column::new("Version", 6, 10).priority(3),
        Column::new("Owner", 5, 8).priority(1),
        Column::new("Message", 8, 0),
    ];
    let rows: Vec<Row> = (0..500)
        .map(|n| {
            Row::new([
                Cell::new(format!("ținută {n} 日本語")),
                Cell::new("v1 → v2").faint(),
                Cell::new("owner").style(Style::new().fg(Color::Green)),
                Cell::new("a long message that never fits\nin one line at all, however wide"),
            ])
            .mark(Mark::new("⧗").style(DIM))
        })
        .collect();
    let mut list = List::new().keys(Keys::VIM).with_rows(rows);
    let sizes = [
        (150, 40),
        (80, 12),
        (40, 6),
        (24, 3),
        (6, 2),
        (1, 1),
        (0, 0),
    ];
    let mut buffers: Vec<Buffer> = sizes
        .iter()
        .map(|&(width, height)| Buffer::empty(Rect::new(0, 0, width, height)))
        .collect();
    let counted = allocations(|| {
        for round in 0..30usize {
            for buffer in &mut buffers {
                let area = buffer.area;
                for header in [false, true] {
                    ListView::new(&mut list, &columns)
                        .styles(styles)
                        .header(header)
                        .end(Some("· end ·"))
                        .empty(Some("Nothing."))
                        .render(area, buffer);
                }
                std::hint::black_box(list.hit(area, true, 3, 3));
            }
            let key = [
                Key::Down,
                Key::Char('j'),
                Key::PageDown,
                Key::Char('G'),
                Key::Home,
                Key::Tab,
            ][round % 6];
            assert_ne!(list.key(key), Step::Open(0));
        }
    });
    assert_eq!(counted, 0);
    let mut empty = List::new();
    let counted = allocations(|| {
        for buffer in &mut buffers {
            let area = buffer.area;
            ListView::new(&mut empty, &columns)
                .styles(styles)
                .header(true)
                .empty(Some("Nothing."))
                .render(area, buffer);
        }
    });
    assert_eq!(counted, 0);
}

#[test]
fn a_base_and_right_columns_allocate_nothing() {
    let styles = Styles::new()
        .selected(ACCENT)
        .faint(DIM)
        .base(Style::new().fg(Color::White).bg(Color::Blue));
    let columns = [
        Column::new("Name", 8, 14).pinned(),
        Column::new("Size", 4, 10).priority(2).right(),
        Column::new("Owner", 5, 8).priority(1),
        Column::new("Total", 3, 0).right(),
    ];
    let rows: Vec<Row> = (0..500)
        .map(|n| {
            Row::new([
                Cell::new(format!("ținută {n} 日本語")),
                Cell::new(format!("{}", n * 1024)).faint(),
                Cell::new("owner"),
                Cell::new("1234567890 ".repeat(n % 7)),
            ])
            .mark(Mark::new("⧗").style(DIM))
        })
        .collect();
    let mut list = List::new().with_rows(rows);
    let mut buffers: Vec<Buffer> = [(150, 40), (40, 6), (12, 3), (1, 1), (0, 0)]
        .iter()
        .map(|&(width, height)| Buffer::empty(Rect::new(0, 0, width, height)))
        .collect();
    let counted = allocations(|| {
        for round in 0..30usize {
            for buffer in &mut buffers {
                let area = buffer.area;
                ListView::new(&mut list, &columns)
                    .styles(styles)
                    .header(round % 2 == 0)
                    .end(Some("· end ·"))
                    .render(area, buffer);
            }
            list.key(Key::PageDown);
        }
    });
    assert_eq!(counted, 0);
}

fn grouped(n: usize) -> Row {
    match n % 12 {
        0 => Row::heading([
            Part::new(format!("Group {n} ținută")).style(ACCENT),
            Part::new(" · 10").faint(),
        ]),
        11 => Row::blank(),
        5 => Row::new([Cell::new(
            "a detail that spans the rest of the row, wider than any area here 日本語",
        )
        .rest()])
        .mark(Mark::new("·")),
        _ => Row::new([
            Cell::parts([
                Part::new(format!("ținută {n}")),
                Part::new(" 日本語").faint(),
            ]),
            Cell::new(format!("{}", n * 1024)),
            Cell::parts(["1", "2"]).style(Style::new().fg(Color::Green)),
        ])
        .mark(Mark::new("⧗").style(DIM)),
    }
}

const GROUPED: [Column<'static>; 3] = [
    Column::new("Name", 8, 0).flex(),
    Column::new("Size", 4, 8).right().priority(1),
    Column::new("Count", 3, 5).right(),
];

const KEYS: [Key; 8] = [
    Key::Down,
    Key::Char('k'),
    Key::PageDown,
    Key::PageUp,
    Key::Char('G'),
    Key::Home,
    Key::Ctrl('d'),
    Key::Enter,
];

fn sizes() -> Vec<Buffer> {
    [(150, 40), (40, 6), (20, 2), (12, 1), (1, 1), (0, 0)]
        .iter()
        .map(|&(width, height)| Buffer::empty(Rect::new(0, 0, width, height)))
        .collect()
}

#[test]
fn sections_parts_rest_cells_a_flex_column_and_view_paging_allocate_nothing() {
    let styles = Styles::new()
        .selected(ACCENT)
        .faint(DIM)
        .base(Style::new().bg(Color::Blue));
    let mut list = List::new()
        .keys(Keys::VIM)
        .paging(Paging::View)
        .with_rows((0..600).map(grouped));
    let mut buffers = sizes();
    let counted = allocations(|| {
        for round in 0..60usize {
            for buffer in &mut buffers {
                let area = buffer.area;
                ListView::new(&mut list, &GROUPED)
                    .styles(styles)
                    .header(round % 2 == 0)
                    .end(Some("· end ·"))
                    .render(area, buffer);
                std::hint::black_box(list.hit(area, round % 2 == 0, 3, 1));
            }
            std::hint::black_box(list.key(KEYS[round % KEYS.len()]));
            if round % 5 == 0 {
                list.scroll_to(round * 7);
            }
        }
    });
    assert_eq!(counted, 0);
}

struct Lazy {
    len: usize,
    built: Counter<usize>,
    own: Counter<usize>,
}

impl Source for Lazy {
    fn len(&self) -> usize {
        self.len
    }

    fn row(&self, index: usize) -> Cow<'_, Row> {
        let before = COUNT.with(Counter::get);
        let row = grouped(index);
        self.own
            .set(self.own.get() + COUNT.with(Counter::get) - before);
        self.built.set(self.built.get() + 1);
        Cow::Owned(row)
    }

    fn mark_width(&self) -> u16 {
        1
    }
}

#[test]
fn a_lazy_list_allocates_only_what_its_rows_do() {
    let mut list = List::from_source(Lazy {
        len: 50_000,
        built: Counter::new(0),
        own: Counter::new(0),
    })
    .keys(Keys::VIM);
    let mut buffers = sizes();
    let counted = allocations(|| {
        for round in 0..60usize {
            for buffer in &mut buffers {
                let area = buffer.area;
                ListView::new(&mut list, &GROUPED)
                    .header(true)
                    .end(Some("· end ·"))
                    .render(area, buffer);
            }
            std::hint::black_box(list.key(KEYS[round % KEYS.len()]));
        }
    });
    let own = list.source().own.get();
    assert!(own > 0);
    assert_eq!(counted, own);
    assert!(list.source().built.get() < 60 * 100);
}
