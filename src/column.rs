#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Column<'a> {
    pub(crate) title: &'a str,
    pub(crate) min: u16,
    pub(crate) preferred: u16,
    pub(crate) rank: u8,
    pub(crate) pinned: bool,
}

impl<'a> Column<'a> {
    pub const fn new(title: &'a str, min: u16, preferred: u16) -> Self {
        Column {
            title,
            min,
            preferred,
            rank: 0,
            pinned: false,
        }
    }

    pub const fn priority(mut self, rank: u8) -> Self {
        self.rank = rank;
        self
    }

    pub const fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }
}
