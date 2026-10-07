mod common;

use std::{cell::Cell as Counter, sync::Arc};

use common::*;
use pito_list::{Bar, Build, Column, Count, List, ListView, Place, Row, Shared};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

const TWO: [Column<'static>; 2] = [
    Column::new("Name", 8, 12).pinned(),
    Column::new("Message", 8, 0),
];
const COUNT: Count<'static> = Count::new("–", " of ").group(",");

fn numbered(count: usize) -> List {
    List::new().with_rows((0..count).map(|n| Row::new([format!("row {n}"), "a message".into()])))
}

fn render<'a>(
    list: &'a mut List,
    width: u16,
    height: u16,
    tweak: impl FnOnce(ListView<'a>) -> ListView<'a>,
) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    let view = ListView::new(list, &TWO)
        .styles(STYLES)
        .header(true)
        .end(Some("· end ·"));
    tweak(view).render(area, &mut buf);
    buf
}

fn bar(buf: &Buffer, from: u16, to: u16) -> String {
    let x = buf.area.right() - 1;
    (from..to).map(|y| buf[(x, y)].symbol()).collect()
}

#[test]
fn a_list_that_fits_draws_as_before() {
    for (len, end) in [(0, true), (5, true), (8, false), (7, true)] {
        let mut plain = numbered(len);
        let mut barred = numbered(len);
        let tail = end.then_some("· end ·");
        let before = render(&mut plain, 30, 9, |view| view.end(tail));
        let after = render(&mut barred, 30, 9, |view| {
            view.end(tail).scrollbar(Some(Bar::LINE)).count(Some(COUNT))
        });
        assert_eq!(before, after, "{len} rows");
        let area = Rect::new(0, 0, 30, 9);
        assert_eq!(barred.hit(area, true, 29, 1), (len > 0).then_some(0));
        let shown = (len > 0).then_some((1, len, len));
        assert_eq!(barred.shown(), shown);
    }
}

#[test]
fn an_overflowing_list_gives_its_last_column_to_the_bar() {
    for select in [0, 17, 39] {
        let mut plain = numbered(40);
        let mut barred = numbered(40);
        plain.select(select);
        barred.select(select);
        let narrow = render(&mut plain, 29, 9, |view| view);
        let wide = render(&mut barred, 30, 9, |view| view.scrollbar(Some(Bar::LINE)));
        for y in 1..9 {
            for x in 0..29 {
                assert_eq!(narrow[(x, y)], wide[(x, y)], "{select}: {x},{y}");
            }
        }
        assert_eq!(plain.columns(), barred.columns());
        assert_eq!(plain.top(), barred.top());
        let area = Rect::new(0, 0, 30, 9);
        assert_eq!(barred.hit(area, true, 28, 1), Some(barred.top()));
        assert_eq!(barred.hit(area, true, 29, 1), None);
    }
    let mut list = numbered(40);
    let lines = |list: &mut List| {
        bar(
            &render(list, 30, 9, |view| view.scrollbar(Some(Bar::LINE))),
            1,
            9,
        )
    };
    assert_eq!(lines(&mut list), "┃╿││││││");
    list.select(20);
    assert_eq!(lines(&mut list), "││╽┃││││");
    list.select(39);
    assert_eq!(lines(&mut list), "││││││╽┃");
    list.select(1);
    let blocks = Bar::new(" ", "█");
    let drawn = render(&mut list, 30, 9, |view| view.scrollbar(Some(blocks)));
    assert_eq!(bar(&drawn, 1, 9), " ██     ");
    list.select(38);
    let drawn = render(&mut list, 30, 9, |view| view.scrollbar(Some(blocks)));
    assert_eq!(bar(&drawn, 1, 9), "     ██ ");
}

#[test]
fn the_count_takes_the_bottom_line_or_sits_at_the_foot_when_it_fits() {
    assert_eq!(COUNT.label(12, 12, 340).to_string(), "12 of 340");
    assert_eq!(
        COUNT.label(1_693, 1_701, 1_234_567).to_string(),
        "1,693–1,701 of 1,234,567"
    );
    assert_eq!(
        Count::new("-", "/").label(999, 1000, 1000).to_string(),
        "999-1000/1000"
    );
    let line = |width, place, last| {
        let mut list = numbered(3_400);
        list.select(if last { 3_399 } else { 1_700 });
        let count = COUNT.place(place);
        let buf = render(&mut list, width, 10, |view| {
            view.count(Some(count)).scrollbar(Some(Bar::LINE))
        });
        (list, read(&buf).text)
    };
    let (list, text) = line(40, Place::Line, false);
    assert_eq!(text[9], "                    1,694–1,701 of 3,400");
    assert_eq!(text[8], "▌ row 1700      a message              │");
    assert_eq!(list.shown(), Some((1_694, 1_701, 3_400)));
    assert_eq!(list.position(), Some((1_701, 3_400)));
    assert_eq!(list.page(), 7);
    let area = Rect::new(0, 0, 40, 10);
    assert_eq!(list.hit(area, true, 5, 8), Some(1_700));
    assert_eq!(list.hit(area, true, 5, 9), None);
    let (list, text) = line(50, Place::Foot, false);
    assert_eq!(
        text[9],
        "▌ row 1700      a message    1,693–1,701 of 3,400│"
    );
    assert_eq!(list.hit(Rect::new(0, 0, 50, 10), true, 5, 9), Some(1_700));
    let (_, text) = line(40, Place::Foot, false);
    assert_eq!(text[9], "▌ row 1700      a message              │");
    let (_, text) = line(50, Place::Foot, true);
    assert_eq!(
        text[9],
        "  · end ·                    3,393–3,400 of 3,400┃"
    );
}

struct Counted {
    built: Counter<usize>,
}

impl Build<usize> for Counted {
    fn row(&self, _: usize, item: &usize) -> Row {
        self.built.set(self.built.get() + 1);
        Row::new([format!("job {item}"), "queued".into()])
    }

    fn selectable(&self, _: usize, _: &usize) -> bool {
        true
    }
}

#[test]
fn a_shared_list_draws_its_bar_and_count_building_only_the_rows_on_screen() {
    let items: Arc<Vec<usize>> = Arc::new((0..100_000).collect());
    let builder = Counted {
        built: Counter::new(0),
    };
    let mut list = List::from_source(Shared::new(items, builder));
    list.select(50_000);
    let area = Rect::new(0, 0, 40, 12);
    let mut buf = Buffer::empty(area);
    ListView::new(&mut list, &TWO)
        .header(true)
        .scrollbar(Some(Bar::LINE))
        .count(Some(COUNT))
        .render(area, &mut buf);
    assert!(list.source().builder().built.get() <= 20);
    assert_eq!(list.shown(), Some((49_992, 50_001, 100_000)));
    let text = read(&buf).text;
    assert_eq!(text[11], "                49,992–50,001 of 100,000");
    assert_eq!(bar(&buf, 1, 11), "││││╽╿││││");
}
