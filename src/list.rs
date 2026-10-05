use ratatui::layout::Rect;

use crate::{Key, Row, text};

const PAGE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Keys {
    pub up: &'static [Key],
    pub down: &'static [Key],
    pub page_up: &'static [Key],
    pub page_down: &'static [Key],
    pub first: &'static [Key],
    pub last: &'static [Key],
    pub open: &'static [Key],
}

impl Keys {
    pub const fn new() -> Self {
        Keys {
            up: &[Key::Up],
            down: &[Key::Down],
            page_up: &[Key::PageUp],
            page_down: &[Key::PageDown],
            first: &[Key::Home],
            last: &[Key::End],
            open: &[Key::Enter],
        }
    }

    pub const VIM: Keys = Keys {
        up: &[Key::Up, Key::Char('k')],
        down: &[Key::Down, Key::Char('j')],
        page_up: &[Key::PageUp, Key::Ctrl('u')],
        page_down: &[Key::PageDown, Key::Ctrl('d')],
        first: &[Key::Home, Key::Char('g')],
        last: &[Key::End, Key::Char('G')],
        open: &[Key::Enter],
    };

    pub const fn up(mut self, keys: &'static [Key]) -> Self {
        self.up = keys;
        self
    }

    pub const fn down(mut self, keys: &'static [Key]) -> Self {
        self.down = keys;
        self
    }

    pub const fn page_up(mut self, keys: &'static [Key]) -> Self {
        self.page_up = keys;
        self
    }

    pub const fn page_down(mut self, keys: &'static [Key]) -> Self {
        self.page_down = keys;
        self
    }

    pub const fn first(mut self, keys: &'static [Key]) -> Self {
        self.first = keys;
        self
    }

    pub const fn last(mut self, keys: &'static [Key]) -> Self {
        self.last = keys;
        self
    }

    pub const fn open(mut self, keys: &'static [Key]) -> Self {
        self.open = keys;
        self
    }
}

impl Default for Keys {
    fn default() -> Self {
        Keys::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Step {
    Pass,
    Held,
    Moved,
    Open(usize),
}

#[derive(Debug, Clone)]
pub struct List {
    rows: Vec<Row>,
    selected: usize,
    top: usize,
    page: usize,
    mark_width: u16,
    keys: Keys,
}

impl Default for List {
    fn default() -> Self {
        List::new()
    }
}

impl List {
    pub fn new() -> Self {
        List {
            rows: Vec::new(),
            selected: 0,
            top: 0,
            page: PAGE,
            mark_width: 0,
            keys: Keys::new(),
        }
    }

    pub fn keys(mut self, keys: Keys) -> Self {
        self.keys = keys;
        self
    }

    pub fn with_rows(mut self, rows: impl IntoIterator<Item = Row>) -> Self {
        self.set_rows(rows);
        self
    }

    pub fn set_rows(&mut self, rows: impl IntoIterator<Item = Row>) {
        self.rows.clear();
        self.mark_width = 0;
        for row in rows {
            self.push(row);
        }
        self.selected = self.selected.min(self.rows.len().saturating_sub(1));
        self.top = self.top.min(self.rows.len().saturating_sub(1));
    }

    pub fn push(&mut self, row: Row) {
        if let Some(mark) = &row.mark {
            self.mark_width = self.mark_width.max(text::cells(&mark.text));
        }
        self.rows.push(row);
    }

    pub fn clear(&mut self) {
        self.rows.clear();
        self.mark_width = 0;
        self.selected = 0;
        self.top = 0;
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn selected(&self) -> Option<usize> {
        (!self.rows.is_empty()).then_some(self.selected)
    }

    pub fn selected_row(&self) -> Option<&Row> {
        self.rows.get(self.selected)
    }

    pub fn select(&mut self, index: usize) {
        self.selected = index.min(self.rows.len().saturating_sub(1));
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn page(&self) -> usize {
        self.page
    }

    pub(crate) fn mark_width(&self) -> u16 {
        self.mark_width
    }

    fn go(&mut self, to: usize) -> Step {
        let to = to.min(self.rows.len().saturating_sub(1));
        if self.rows.is_empty() || to == self.selected {
            return Step::Held;
        }
        self.selected = to;
        Step::Moved
    }

    pub fn up(&mut self) -> Step {
        self.go(self.selected.saturating_sub(1))
    }

    pub fn down(&mut self) -> Step {
        self.go(self.selected.saturating_add(1))
    }

    pub fn page_up(&mut self) -> Step {
        self.go(self.selected.saturating_sub(self.page))
    }

    pub fn page_down(&mut self) -> Step {
        self.go(self.selected.saturating_add(self.page))
    }

    pub fn first(&mut self) -> Step {
        self.go(0)
    }

    pub fn last(&mut self) -> Step {
        self.go(usize::MAX)
    }

    pub fn key(&mut self, key: Key) -> Step {
        let keys = self.keys;
        if keys.up.contains(&key) {
            self.up()
        } else if keys.down.contains(&key) {
            self.down()
        } else if keys.page_up.contains(&key) {
            self.page_up()
        } else if keys.page_down.contains(&key) {
            self.page_down()
        } else if keys.first.contains(&key) {
            self.first()
        } else if keys.last.contains(&key) {
            self.last()
        } else if keys.open.contains(&key) {
            match self.selected() {
                Some(index) => Step::Open(index),
                None => Step::Held,
            }
        } else {
            Step::Pass
        }
    }

    pub fn hit(&self, area: Rect, header: bool, column: u16, row: u16) -> Option<usize> {
        let first = area.y.saturating_add(u16::from(header));
        if column < area.x || column >= area.right() || row < first || row >= area.bottom() {
            return None;
        }
        let index = self.top.saturating_add(usize::from(row - first));
        (index < self.rows.len()).then_some(index)
    }

    pub(crate) fn follow(&mut self, height: usize, tail: bool) {
        if height > 0 {
            self.page = height.saturating_sub(1).max(1);
        }
        let len = self.rows.len();
        if height == 0 || len == 0 {
            self.top = 0;
            return;
        }
        self.selected = self.selected.min(len - 1);
        let last = self.selected + 1 == len;
        let bottom = self.selected + 1 + usize::from(last && tail);
        if self.selected < self.top {
            self.top = self.selected;
        }
        if bottom > self.top.saturating_add(height) {
            self.top = bottom - height;
        }
        let lines = len + usize::from(tail);
        self.top = self.top.min(lines.saturating_sub(height));
    }
}
