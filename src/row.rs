use ratatui::style::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Paint {
    Ink,
    Faint,
    Own(Style),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Cell {
    pub(crate) text: String,
    pub(crate) paint: Paint,
}

impl Cell {
    pub fn new(text: impl Into<String>) -> Self {
        Cell {
            text: text.into(),
            paint: Paint::Ink,
        }
    }

    pub fn faint(mut self) -> Self {
        self.paint = Paint::Faint;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.paint = Paint::Own(style);
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl From<&str> for Cell {
    fn from(text: &str) -> Self {
        Cell::new(text)
    }
}

impl From<String> for Cell {
    fn from(text: String) -> Self {
        Cell::new(text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Mark {
    pub(crate) text: String,
    pub(crate) paint: Paint,
}

impl Mark {
    pub fn new(text: impl Into<String>) -> Self {
        Mark {
            text: text.into(),
            paint: Paint::Ink,
        }
    }

    pub fn faint(mut self) -> Self {
        self.paint = Paint::Faint;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.paint = Paint::Own(style);
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Row {
    pub(crate) mark: Option<Mark>,
    pub(crate) cells: Vec<Cell>,
}

impl Row {
    pub fn new<I, C>(cells: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Cell>,
    {
        Row {
            mark: None,
            cells: cells.into_iter().map(Into::into).collect(),
        }
    }

    pub fn mark(mut self, mark: Mark) -> Self {
        self.mark = Some(mark);
        self
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn leading(&self) -> Option<&Mark> {
        self.mark.as_ref()
    }
}
