use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::{
    Column, List, Row,
    row::Paint,
    text::{self, blank, put},
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
}

impl Styles {
    pub const fn new() -> Self {
        Styles {
            selected: Style::new(),
            ink: Style::new(),
            faint: Style::new(),
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
    count: usize,
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
            count,
        };
        if count == 0 {
            return layout;
        }
        let flex = count - 1;
        let width = usize::from(width);
        let lead = usize::from(lead);
        let gap = usize::from(gap);
        layout.kept[..flex].fill(true);
        let need = |kept: &[bool; MAX_COLUMNS]| {
            let mut total = lead.saturating_add(usize::from(columns[flex].min));
            for at in 0..flex {
                if kept[at] {
                    total = total
                        .saturating_add(usize::from(columns[at].min))
                        .saturating_add(gap);
                }
            }
            total
        };
        while need(&layout.kept) > width {
            let mut pick: Option<usize> = None;
            for at in 0..flex {
                if layout.kept[at] && !columns[at].pinned {
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
        let mut extra = width.saturating_sub(need(&layout.kept));
        let mut used = lead;
        for (at, column) in columns.iter().enumerate().take(flex) {
            if !layout.kept[at] {
                continue;
            }
            let min = usize::from(column.min);
            let grow = usize::from(column.preferred).saturating_sub(min).min(extra);
            extra -= grow;
            let size = min + grow;
            layout.widths[at] = small(size);
            used = used.saturating_add(size).saturating_add(gap);
        }
        layout.widths[flex] = small(width.saturating_sub(used));
        layout.kept[flex] = true;
        layout
    }
}

fn columns_line<'r>(
    buf: &mut Buffer,
    y: u16,
    from: u16,
    layout: &Layout,
    gap: u16,
    mut cell: impl FnMut(usize) -> Option<(&'r str, Style)>,
) {
    let mut x = from;
    for at in 0..layout.count {
        if !layout.kept[at] {
            continue;
        }
        let size = layout.widths[at];
        if let Some((text, style)) = cell(at) {
            put(buf, x, y, size, text, style);
        }
        x = x.saturating_add(size).saturating_add(gap);
    }
}

#[derive(Debug)]
pub struct ListView<'a> {
    list: &'a mut List,
    columns: &'a [Column<'a>],
    styles: Styles,
    cursor: &'a str,
    gap: u16,
    header: bool,
    end: Option<&'a str>,
    empty: Option<&'a str>,
}

impl<'a> ListView<'a> {
    pub fn new(list: &'a mut List, columns: &'a [Column<'a>]) -> Self {
        ListView {
            list,
            columns,
            styles: Styles::new(),
            cursor: CURSOR,
            gap: GAP,
            header: false,
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
    layout: Layout,
    gap: u16,
    indent: u16,
    lead: u16,
    cursor: &'a str,
    mark_width: u16,
    area: Rect,
}

impl Line<'_> {
    fn row(&self, buf: &mut Buffer, y: u16, row: &Row, chosen: bool) {
        let area = self.area;
        let mut x = area.x;
        let room = area.width;
        if chosen {
            blank(buf, x, y, room, self.styles.selected);
            put(buf, x, y, self.indent, self.cursor, self.styles.selected);
        }
        x = x.saturating_add(self.indent);
        if let Some(mark) = &row.mark {
            let style = if chosen {
                self.styles.selected
            } else {
                self.styles.resolve(mark.paint)
            };
            put(buf, x, y, self.mark_width, &mark.text, style);
        }
        x = x.saturating_add(self.lead - self.indent);
        columns_line(buf, y, x, &self.layout, self.gap, |at| {
            let cell = row.cells.get(at)?;
            let style = if chosen {
                self.styles.selected
            } else {
                self.styles.resolve(cell.paint)
            };
            Some((cell.text.as_str(), style))
        });
    }

    fn header(&self, buf: &mut Buffer, y: u16, columns: &[Column]) {
        let x = self.area.x.saturating_add(self.lead);
        columns_line(buf, y, x, &self.layout, self.gap, |at| {
            Some((columns[at].title, self.styles.faint))
        });
    }

    fn note(&self, buf: &mut Buffer, y: u16, text: &str) {
        let x = self.area.x.saturating_add(self.indent);
        let room = self.area.width.saturating_sub(self.indent);
        put(buf, x, y, room, text, self.styles.faint);
    }
}

impl Widget for ListView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        for y in area.top()..area.bottom() {
            blank(buf, area.x, y, area.width, Style::new());
        }
        let ListView {
            list,
            columns,
            styles,
            cursor,
            gap,
            header,
            end,
            empty,
        } = self;
        let indent = text::cells(cursor);
        let mark_width = list.mark_width();
        let lead = indent
            .saturating_add(mark_width)
            .saturating_add(u16::from(mark_width > 0));
        let line = Line {
            styles,
            layout: Layout::new(columns, area.width, lead, gap),
            gap,
            indent,
            lead,
            cursor,
            mark_width,
            area,
        };
        let mut y = area.y;
        if header {
            line.header(buf, y, columns);
            y += 1;
        }
        let height = usize::from(area.bottom().saturating_sub(y));
        list.follow(height, end.is_some());
        if list.is_empty() {
            if let Some(text) = empty
                && height > 0
            {
                line.note(buf, y, text);
            }
            return;
        }
        let selected = list.selected();
        for offset in 0..height {
            let index = list.top().saturating_add(offset);
            let at = y.saturating_add(small(offset));
            match list.rows().get(index) {
                Some(row) => line.row(buf, at, row, selected == Some(index)),
                None => {
                    if let Some(text) = end
                        && index == list.len()
                    {
                        line.note(buf, at, text);
                    }
                    break;
                }
            }
        }
    }
}
