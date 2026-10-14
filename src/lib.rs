#![doc = include_str!("../README.md")]

mod bar;
mod column;
mod key;
mod list;
mod row;
mod shared;
mod text;
mod view;

pub use bar::{Bar, Count, Place};
pub use column::Column;
pub use key::Key;
pub use list::{Keys, List, Paging, Source, Step};
pub use row::{Cell, Mark, Part, Row};
pub use shared::{Build, Shared};
pub use view::{CURSOR, ListView, MAX_COLUMNS, Styles};

#[cfg(doctest)]
#[doc = include_str!("../docs/guide.md")]
struct Guide;

#[cfg(doctest)]
#[doc = include_str!("../docs/install.md")]
struct Install;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/first-list.md")]
struct FirstList;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/keys-and-clicks.md")]
struct KeysAndClicks;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/columns.md")]
struct Columns;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/sections.md")]
struct Sections;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/ranges.md")]
struct Ranges;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/scrolling.md")]
struct Scrolling;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/styles.md")]
struct Themes;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/live-rows.md")]
struct LiveRows;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/big-lists.md")]
struct BigLists;

#[cfg(doctest)]
#[doc = include_str!("../docs/guides/measure.md")]
struct Measure;
