mod common;

use std::{cell::Cell as Counter, sync::Arc};

use common::*;
use pito_list::{Build, Column, Key, List, ListView, Mark, Part, Row, Shared, Source};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

const BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
const TWO: [Column<'static>; 2] = [
    Column::new("Job", 8, 12).pinned(),
    Column::new("Size", 4, 0),
];

struct Job {
    name: String,
    kb: u64,
    group: bool,
}

fn job(n: usize) -> Job {
    Job {
        name: format!("job {n}"),
        kb: n as u64 * 3,
        group: n.is_multiple_of(10),
    }
}

fn sized(job: &Job, unit: &str) -> Row {
    if job.group {
        Row::heading([
            Part::new(job.name.clone()).style(BOLD),
            Part::new(" · 9").faint(),
        ])
    } else {
        Row::new([job.name.clone(), format!("{} {unit}", job.kb)]).mark(Mark::new("•").style(GREEN))
    }
}

fn row_of(_: usize, job: &Job) -> Row {
    sized(job, "kB")
}

struct Counted {
    unit: &'static str,
    built: Counter<usize>,
}

impl Build<Job> for Counted {
    fn row(&self, _: usize, job: &Job) -> Row {
        self.built.set(self.built.get() + 1);
        sized(job, self.unit)
    }

    fn selectable(&self, _: usize, job: &Job) -> bool {
        !job.group
    }
}

fn counted(jobs: &Arc<Vec<Job>>) -> Shared<Job, Counted> {
    let builder = Counted {
        unit: "kB",
        built: Counter::new(0),
    };
    Shared::new(Arc::clone(jobs), builder).marks(1)
}

fn render<S: Source>(list: &mut List<S>, width: u16, height: u16) -> Drawn {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    ListView::new(list, &TWO)
        .styles(STYLES)
        .header(true)
        .end(Some("· end ·"))
        .empty(Some("No jobs."))
        .render(area, &mut buf);
    read(&buf)
}

#[test]
fn shared_items_draw_like_the_same_rows_owned() {
    let jobs: Arc<Vec<Job>> = Arc::new((0..200).map(job).collect());
    let mut owned = List::new().with_rows(jobs.iter().map(|job| row_of(0, job)));
    let mut closure = List::from_source(
        Shared::new(Arc::clone(&jobs), |index, job: &Job| row_of(index, job)).marks(1),
    );
    let mut built = List::from_source(counted(&jobs));
    let keys = [
        Key::Down,
        Key::PageDown,
        Key::End,
        Key::Up,
        Key::PageUp,
        Key::Home,
    ];
    for key in keys {
        let step = owned.key(key);
        assert_eq!(closure.key(key), step);
        assert_eq!(built.key(key), step);
        for (width, height) in [(40, 12), (24, 4), (12, 2), (60, 30)] {
            let expected = render(&mut owned, width, height);
            for drawn in [
                render(&mut closure, width, height),
                render(&mut built, width, height),
            ] {
                assert_eq!(drawn.text, expected.text, "{key:?} {width}x{height}");
                assert_eq!(drawn.marks, expected.marks, "{key:?} {width}x{height}");
            }
            assert_eq!(closure.selected(), owned.selected());
            assert_eq!(built.top(), owned.top());
        }
    }
}

#[test]
fn only_the_items_on_screen_are_built() {
    let jobs: Arc<Vec<Job>> = Arc::new((0..50_000).map(job).collect());
    let mut list = List::from_source(counted(&jobs));
    for key in [Key::End, Key::PageUp, Key::Up, Key::Down, Key::Home] {
        list.key(key);
    }
    assert_eq!(list.source().builder().built.get(), 0);
    list.select(49_000);
    for _ in 0..3 {
        render(&mut list, 60, 12);
        let built = list.source().builder().built.replace(0);
        assert!((11..=22).contains(&built), "{built}");
        list.key(Key::PageDown);
    }
    list.last();
    assert_eq!(render(&mut list, 60, 12).text[11], "  · end ·");
    assert_eq!(Arc::strong_count(&jobs), 2);
}

#[test]
fn new_items_and_builder_state_take_effect_at_the_next_draw() {
    let jobs: Arc<Vec<Job>> = Arc::new((0..100).map(job).collect());
    let mut list = List::from_source(counted(&jobs));
    list.select(95);
    list.update(|shared| shared.builder_mut().unit = "KiB");
    let drawn = render(&mut list, 30, 4);
    assert!(drawn.text[3].starts_with("▌ • job 95 "), "{:?}", drawn.text);
    assert!(drawn.text[3].ends_with(" 285 KiB"), "{:?}", drawn.text);
    let fewer: Arc<Vec<Job>> = Arc::new((0..30).map(job).collect());
    list.update(|shared| shared.set_items(Arc::clone(&fewer)));
    assert_eq!(list.selected(), Some(29));
    assert_eq!(list.len(), 30);
    assert_eq!(render(&mut list, 30, 4).text[3], "  · end ·");
    list.update(|shared| shared.set_items(Arc::default()));
    assert_eq!(list.selected(), None);
    assert_eq!(render(&mut list, 30, 4).text[1], "  No jobs.");
    let mut named: List<Shared<Job>> = List::from_source(Shared::new(fewer, row_of));
    named.last();
    let drawn = render(&mut named, 30, 4);
    assert!(drawn.text[2].starts_with("▌ job 29 "), "{:?}", drawn.text);
    assert!(drawn.text[2].ends_with(" 87 kB"), "{:?}", drawn.text);
}

#[test]
fn shared_rows_borrow_and_draw_like_owned_rows() {
    let rows: Vec<Row> = (0..40)
        .map(|n| match n {
            7 => Row::new([format!("wide {n}"), "x".into()]).mark(Mark::new("⧗⧗")),
            n => sized(&job(n), "kB"),
        })
        .collect();
    let mut owned = List::new().with_rows(rows.clone());
    let vector: Arc<Vec<Row>> = Arc::new(rows.clone());
    let slice: Arc<[Row]> = rows.into();
    let mut by_vector = List::from_source(Arc::clone(&vector));
    let mut by_slice = List::from_source(Arc::clone(&slice));
    for key in [Key::Down, Key::PageDown, Key::End, Key::Home] {
        owned.key(key);
        by_vector.key(key);
        by_slice.key(key);
        let expected = render(&mut owned, 40, 10);
        assert_eq!(render(&mut by_vector, 40, 10).text, expected.text);
        assert_eq!(render(&mut by_slice, 40, 10).text, expected.text);
    }
    assert!(render(&mut by_vector, 40, 10).text[2].starts_with("▌ •  job 1"));
    by_vector.set_source(Arc::clone(&vector));
    assert!(matches!(
        by_vector.source().row(3),
        std::borrow::Cow::Borrowed(_)
    ));
}
