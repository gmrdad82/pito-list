use std::iter;

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::{
    Column, List, Row, Source,
    row::{Kind, Paint},
    text::{self, blank, put, put_runs},
};

pub const CURSOR: &str = "▌ ";
pub const MAX_COLUMNS: usize = 32;
const GAP: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Styles {
    pub(crate) selected: Style,
    pub(crate) ink: Style,
    pub(crate) faint: Style,
    pub(crate) base: Style,
    pub(crate) header: Option<Style>,
    pub(crate) header_key: Option<Style>,
    pub(crate) cursor: Option<Style>,
    pub(crate) keep: bool,
}

impl Styles {
    pub const fn new() -> Self {
        Styles {
            selected: Style::new(),
            ink: Style::new(),
            faint: Style::new(),
            base: Style::new(),
            header: None,
            header_key: None,
            cursor: None,
            keep: false,
        }
    }

    pub const fn selected(mut self, style: Style) -> Self {
        self.selected = style;
        self
    }

    pub const fn ink(mut self, style: Style) -> Self {
        self.ink = style;
        self
    }

    pub const fn faint(mut self, style: Style) -> Self {
        self.faint = style;
        self
    }

    pub const fn base(mut self, style: Style) -> Self {
        self.base = style;
        self
    }

    pub const fn header(mut self, style: Style) -> Self {
        self.header = Some(style);
        self
    }

    pub const fn header_key(mut self, style: Style) -> Self {
        self.header_key = Some(style);
        self
    }

    pub const fn cursor(mut self, style: Style) -> Self {
        self.cursor = Some(style);
        self
    }

    pub const fn keep_colours(mut self, keep: bool) -> Self {
        self.keep = keep;
        self
    }

    fn resolve(&self, paint: Paint) -> Style {
        match paint {
            Paint::Ink => self.ink,
            Paint::Faint => self.faint,
            Paint::Own(style) => style,
        }
    }
}

impl Default for Styles {
    fn default() -> Self {
        Styles::new()
    }
}

struct Layout {
    widths: [u16; MAX_COLUMNS],
    kept: [bool; MAX_COLUMNS],
    right: [bool; MAX_COLUMNS],
    count: usize,
    flex: usize,
}

fn small(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

impl Layout {
    fn new(columns: &[Column], width: u16, lead: u16, gap: u16) -> Self {
        let count = columns.len().min(MAX_COLUMNS);
        let mut layout = Layout {
            widths: [0; MAX_COLUMNS],
            kept: [false; MAX_COLUMNS],
            right: [false; MAX_COLUMNS],
            count,
            flex: 0,
        };
        for (right, column) in layout.right.iter_mut().zip(columns) {
            *right = column.right;
        }
        if count == 0 {
            return layout;
        }
        layout.flex = columns[..count]
            .iter()
            .position(|column| column.flex)
            .unwrap_or(count - 1);
        layout.kept[..count].fill(true);
        while layout.need(columns, lead, gap) > usize::from(width) {
            let mut pick: Option<usize> = None;
            for at in 0..count {
                if at != layout.flex && layout.kept[at] && !columns[at].pinned {
                    match pick {
                        Some(held) if columns[held].rank > columns[at].rank => {}
                        _ => pick = Some(at),
                    }
                }
            }
            match pick {
                Some(at) => layout.kept[at] = false,
                None => break,
            }
        }
        layout
    }

    fn need(&self, columns: &[Column], lead: u16, gap: u16) -> usize {
        let mut total = usize::from(lead).saturating_add(usize::from(columns[self.flex].min));
        for (at, column) in columns.iter().enumerate().take(self.count) {
            if at != self.flex && self.kept[at] {
                total = total
                    .saturating_add(usize::from(column.min))
                    .saturating_add(usize::from(gap));
            }
        }
        total
    }

    fn fits(&self, columns: &[Column], at: usize) -> bool {
        at != self.flex && self.kept[at] && columns[at].fit.is_some()
    }

    fn size(&mut self, columns: &[Column], width: u16, lead: u16, gap: u16, fitted: &[u16]) {
        if self.count == 0 {
            return;
        }
        let width = usize::from(width);
        let mut extra = width.saturating_sub(self.need(columns, lead, gap));
        let mut used = usize::from(lead);
        for (at, column) in columns.iter().enumerate().take(self.count) {
            if at == self.flex || !self.kept[at] {
                continue;
            }
            let min = usize::from(column.min);
            let preferred = match column.fit {
                Some(cap) => fitted[at].min(cap),
                None => column.preferred,
            };
            let grow = usize::from(preferred).saturating_sub(min).min(extra);
            extra -= grow;
            let size = min + grow;
            self.widths[at] = small(size);
            used = used.saturating_add(size).saturating_add(usize::from(gap));
        }
        self.widths[self.flex] = small(width.saturating_sub(used));
    }

    fn places(&self, from: u16, edge: u16, gap: u16) -> impl Iterator<Item = (usize, u16, u16)> {
        (0..self.count)
            .filter(|&at| self.kept[at])
            .scan(from, move |x, at| {
                let start = *x;
                *x = start.saturating_add(self.widths[at]).saturating_add(gap);
                Some((at, start, self.widths[at].min(edge.saturating_sub(start))))
            })
            .filter(|&(_, _, width)| width > 0)
    }
}

fn fitted<S: Source>(
    list: &List<S>,
    columns: &[Column],
    layout: &Layout,
    header: bool,
    lines: usize,
) -> [u16; MAX_COLUMNS] {
    let mut widths = [0; MAX_COLUMNS];
    let count = layout.count;
    if !(0..count).any(|at| layout.fits(columns, at)) {
        return widths;
    }
    if header {
        for (at, width) in widths.iter_mut().enumerate().take(count) {
            if layout.fits(columns, at) {
                *width = text::cells(columns[at].title);
            }
        }
    }
    let from = list.top();
    let to = from.saturating_add(lines).min(list.len());
    for index in from..to {
        let row = list.row(index);
        if row.kind != Kind::Item {
            continue;
        }
        for (at, cell) in row.cells.iter().enumerate().take(count) {
            if !layout.kept[at] {
                continue;
            }
            if cell.rest {
                break;
            }
            if layout.fits(columns, at) {
                let cells = cell.pieces().fold(0usize, |total, (text, _)| {
                    total.saturating_add(text::width(text))
                });
                widths[at] = widths[at].max(small(cells));
            }
        }
    }
    widths
}

fn columns_line<'r, I>(
    buf: &mut Buffer,
    clip: Rect,
    y: u16,
    from: u16,
    layout: &Layout,
    gap: u16,
    mut cell: impl FnMut(usize) -> Option<(I, bool)>,
) where
    I: Iterator<Item = (&'r str, Style)> + Clone,
{
    let mut x = from;
    for at in 0..layout.count {
        if !layout.kept[at] {
            continue;
        }
        let size = layout.widths[at];
        if let Some((runs, rest)) = cell(at) {
            if rest {
                let room = clip.right().saturating_sub(x);
                put_runs(buf, clip, x, y, room, runs, layout.right[at]);
                return;
            }
            put_runs(buf, clip, x, y, size, runs, layout.right[at]);
        }
        x = x.saturating_add(size).saturating_add(gap);
    }
}

#[derive(Debug)]
pub struct ListView<'a, S = Vec<Row>> {
    list: &'a mut List<S>,
    columns: &'a [Column<'a>],
    styles: Styles,
    cursor: &'a str,
    gap: u16,
    header: bool,
    key: Option<usize>,
    end: Option<&'a str>,
    empty: Option<&'a str>,
}

impl<'a, S: Source> ListView<'a, S> {
    pub fn new(list: &'a mut List<S>, columns: &'a [Column<'a>]) -> Self {
        ListView {
            list,
            columns,
            styles: Styles::new(),
            cursor: CURSOR,
            gap: GAP,
            header: false,
            key: None,
            end: None,
            empty: None,
        }
    }

    pub fn styles(mut self, styles: Styles) -> Self {
        self.styles = styles;
        self
    }

    pub fn cursor(mut self, cursor: &'a str) -> Self {
        self.cursor = cursor;
        self
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn header(mut self, header: bool) -> Self {
        self.header = header;
        self
    }

    pub fn key_column(mut self, column: Option<usize>) -> Self {
        self.key = column;
        self
    }

    pub fn end(mut self, end: Option<&'a str>) -> Self {
        self.end = end;
        self
    }

    pub fn empty(mut self, empty: Option<&'a str>) -> Self {
        self.empty = empty;
        self
    }
}

struct Line<'a> {
    styles: Styles,
    chosen: Style,
    layout: Layout,
    gap: u16,
    indent: u16,
    lead: u16,
    cursor: &'a str,
    mark_width: u16,
    area: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lit {
    No,
    Range,
    Cursor,
}

impl Line<'_> {
    fn paint(&self, paint: Paint, chosen: bool) -> Style {
        match (chosen, self.styles.keep) {
            (false, _) => self.styles.resolve(paint),
            (true, false) => self.styles.selected,
            (true, true) => self.styles.resolve(paint).patch(self.styles.selected),
        }
    }

    fn row(&self, buf: &mut Buffer, y: u16, row: &Row, lit: Lit) {
        match row.kind {
            Kind::Item => self.item(buf, y, row, lit),
            Kind::Heading => self.heading(buf, y, row),
            Kind::Blank => {}
        }
    }

    fn item(&self, buf: &mut Buffer, y: u16, row: &Row, lit: Lit) {
        let area = self.area;
        let mut x = area.x;
        let room = area.width;
        let chosen = lit != Lit::No;
        if chosen {
            blank(buf, area, x, y, room, self.chosen);
        }
        if lit == Lit::Cursor {
            let style = self.styles.cursor.unwrap_or(self.styles.selected);
            put(buf, area, x, y, self.indent, self.cursor, style);
        }
        x = x.saturating_add(self.indent);
        if let Some(mark) = &row.mark {
            put(
                buf,
                area,
                x,
                y,
                self.mark_width,
                &mark.text,
                self.paint(mark.paint, chosen),
            );
        }
        x = x.saturating_add(self.lead - self.indent);
        columns_line(buf, area, y, x, &self.layout, self.gap, |at| {
            let cell = row.cells.get(at)?;
            let runs = cell
                .pieces()
                .map(move |(text, paint)| (text, self.paint(paint, chosen)));
            Some((runs, cell.rest))
        });
    }

    fn heading(&self, buf: &mut Buffer, y: u16, row: &Row) {
        let Some(cell) = row.cells.first() else {
            return;
        };
        let x = self.area.x.saturating_add(self.indent);
        let room = self.area.width.saturating_sub(self.indent);
        let runs = cell
            .pieces()
            .map(|(text, paint)| (text, self.styles.resolve(paint)));
        put_runs(buf, self.area, x, y, room, runs, false);
    }

    fn header(&self, buf: &mut Buffer, y: u16, columns: &[Column], key: Option<usize>) {
        let styles = self.styles;
        if let Some(style) = styles.header {
            blank(
                buf,
                self.area,
                self.area.x,
                y,
                self.area.width,
                styles.base.patch(style),
            );
        }
        let title = styles.header.unwrap_or(styles.faint);
        let x = self.area.x.saturating_add(self.lead);
        columns_line(buf, self.area, y, x, &self.layout, self.gap, |at| {
            let style = match styles.header_key {
                Some(style) if key == Some(at) => style,
                _ => title,
            };
            Some((iter::once((columns[at].title, style)), false))
        });
    }

    fn note(&self, buf: &mut Buffer, y: u16, text: &str) {
        let x = self.area.x.saturating_add(self.indent);
        let room = self.area.width.saturating_sub(self.indent);
        put(buf, self.area, x, y, room, text, self.styles.faint);
    }
}

impl<S: Source> Widget for ListView<'_, S> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            self.list.place(iter::empty());
            return;
        }
        let ListView {
            list,
            columns,
            styles,
            cursor,
            gap,
            header,
            key,
            end,
            empty,
        } = self;
        for y in area.top()..area.bottom() {
            blank(buf, area, area.x, y, area.width, styles.base);
        }
        let indent = text::cells(cursor);
        let mark_width = list.mark_width();
        let lead = indent
            .saturating_add(mark_width)
            .saturating_add(u16::from(mark_width > 0));
        let y = area.y.saturating_add(u16::from(header));
        let height = usize::from(area.bottom().saturating_sub(y));
        let selected = list.follow(height, end.is_some());
        let mut layout = Layout::new(columns, area.width, lead, gap);
        let fitted = fitted(list, columns, &layout, header, height);
        layout.size(columns, area.width, lead, gap, &fitted);
        list.place(layout.places(area.x.saturating_add(lead), area.right(), gap));
        let line = Line {
            styles,
            chosen: styles.base.patch(styles.selected),
            layout,
            gap,
            indent,
            lead,
            cursor,
            mark_width,
            area,
        };
        if header {
            line.header(buf, area.y, columns, key);
        }
        if list.is_empty() {
            if let Some(text) = empty
                && height > 0
            {
                line.note(buf, y, text);
            }
            return;
        }
        let len = list.len();
        for offset in 0..height {
            let index = list.top().saturating_add(offset);
            let at = y.saturating_add(small(offset));
            if index < len {
                let lit = if selected == Some(index) {
                    Lit::Cursor
                } else if list.in_range(index) {
                    Lit::Range
                } else {
                    Lit::No
                };
                line.row(buf, at, &list.row(index), lit);
                continue;
            }
            if let Some(text) = end
                && index == len
            {
                line.note(buf, at, text);
            }
            break;
        }
    }
}
