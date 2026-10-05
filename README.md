# pito-list

Selectable list rows for ratatui apps, in the style of HEY's terminal UI: the
selected row is drawn as a bold "▌ " marker plus its text in the app's selected
style, the other rows as two blank cells plus their cells in their own styles,
and the columns pad to align across rows. It's a ratatui 0.30 widget with no
backend feature, and it has no words of its own: every word, style and key
comes from the app, so any language works.

```toml
pito-list = { git = "https://github.com/gmrdad82/pito-list", tag = "v0.2.0" }
```

Turn on the `crossterm` feature for `Key::from(crossterm::event::KeyEvent)`
(crossterm 0.29); without it the crate has no backend dependency. The
conversion turns Alt, Super, Meta or Hyper on a non-character key into
`Key::Other`, keeps Alt apart on a character (`Key::Alt('y')`), and passes
Ctrl+Alt plus a character on as that character, which is how AltGr arrives on
some platforms, so diacritics can still be typed.

## What it does

- **Rows.** A `Row` is a list of `Cell`s, each with a text and a style (the
  app's ink by default, `faint()` for secondary text, or `style(..)` for its
  own), and an optional leading `Mark` such as a state sign. Marks align: the
  widest mark sets one column for the whole list.
- **The selected row** is drawn whole in the app's selected style (bold accent,
  say), across the full width, behind a "▌ " marker; cell and mark styles give
  way to it. `cursor(..)` replaces the marker text.
- **Columns** have a title, a minimum and a preferred width, and a drop
  priority. Each one pads to its width so the rows align. When the width runs
  out the highest `priority(n)` drops first (the later column on a tie), a
  `pinned()` column drops never, and a column that is left shrinks no
  further than its minimum. Spare width goes to the columns up to their
  preferred widths, left to right.
- **Right-aligned columns.** `right()` pads a column on the left instead of the
  right, for numbers: its cells and its header title end at the column's right
  edge. A text wider than the column is clipped with "…" just as in a
  left-aligned one.
- **The last column takes what's left** (a message, say), is never dropped
  and is clipped with "…". Its minimum is always kept free, and its preferred
  width isn't used.
- **A list** (`List`) holds the rows, the selection and the scroll offset. The
  offset keeps the selection in view when the list is drawn, and an end line
  scrolls in with the last row. `page_up`/`page_down` move by the drawn page
  (one less than the rows on screen), `first`/`last` jump.
- **An optional header line** of the column titles, an optional end line (the
  app's words, say "· end ·") after the last row, and an optional line for an
  empty list, all in the faint style.
- **A base style.** Drawing clears every cell of its area to the app's base
  style (none by default, so the terminal's own background shows), so an app
  that paints a page background keeps it under the list: in the column gaps,
  the padding, the two marker cells of the other rows and the lines below the
  end. Every other style is drawn over it, and the selected row's style wins
  on its row: the base shows through only what the selected style leaves
  unset.
- **Keys in, steps out.** `List::key(Key)` takes the crate's own `Key` and
  returns a `Step`: `Moved`, `Held` (a list key that moved nothing, such as up
  on the first row), `Open(index)` for the open key, or `Pass` for a key that
  isn't the list's. It never reads input itself. The keys are the app's:
  `Keys::new()` is the arrows, page keys, home, end and enter, `Keys::VIM` adds
  j/k, g/G and ctrl+u/ctrl+d, and any set can be replaced.
- **Clicks.** `List::hit(area, header, column, row)` names the row under a
  terminal cell, following the scroll, or `None` for the header, the end line
  or a blank line.
- **Widths by cell.** Unicode widths throughout, so diacritics (ă, î, ș, ț),
  "…" and wide glyphs measure, pad and clip cleanly; a wide glyph is never cut
  in half. A newline, tab or carriage return inside a text is drawn as one
  space, other control characters are dropped, and every sum saturates.
- **Cheap.** Drawing writes straight into the buffer and allocates nothing;
  a 20,000-row list at 150×40 draws in about 0.2 ms (`cargo run --release
  --example bench`). Only the rows on screen are visited.

## The API

```text
pub enum Key { Char(char), Ctrl(char), Alt(char), Tab, BackTab, Enter, Esc, Backspace,
               Left, Right, Up, Down, Home, End, PageUp, PageDown, Delete, Other }   // non_exhaustive
pub struct Styles { selected, ink, faint, base }   // non_exhaustive
  Styles::new(); .selected(Style) .ink(Style) .faint(Style) .base(Style)
Cell::new(text) | From<&str> | From<String>; .faint() .style(Style); text()
Mark::new(text);                              .faint() .style(Style); text()
Row::new(cells).mark(Mark); cells(), leading()
Column::new(title, min, preferred).priority(u8).pinned().right()
pub struct Keys { up, down, page_up, page_down, first, last, open: &'static [Key] }   // non_exhaustive
  Keys::new(), Keys::VIM; .up(..) .down(..) .page_up(..) .page_down(..) .first(..) .last(..) .open(..)
pub enum Step { Pass, Held, Moved, Open(usize) }                                      // non_exhaustive
List::new().keys(Keys).with_rows(rows)
  set_rows(rows), push(row), clear(), len(), is_empty(), rows(), selected() -> Option<usize>,
  selected_row(), select(index), top(), page()
  up(), down(), page_up(), page_down(), first(), last() -> Step
  key(Key) -> Step
  hit(area, header, column, row) -> Option<usize>
ListView::new(&mut List, &[Column])            // Widget
  .styles(Styles).cursor(&str).gap(u16).header(bool).end(Option<&str>).empty(Option<&str>)
pub const CURSOR: &str;                        // "▌ ", the default marker
pub const MAX_COLUMNS: usize;                  // 32; a column past it is ignored
```

`Styles`, `Cell`, `Mark`, `Row`, `Column`, `Keys`, `Key` and `Step` are
`#[non_exhaustive]`: match enums with a wildcard arm and build the structs
with their constructors and methods, so a later release can add to them in a
minor version.

Drawing takes `&mut List` because it moves the scroll offset to keep the
selection in view; `page()` and `hit` answer for the last draw. Replace the
rows with `set_rows` when the data changes: the selection stays on its index,
clamped to the new length.

## Example

```rust,standalone_crate
use pito_list::{Cell, Column, Key, List, ListView, Mark, Row, Step, Styles};
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
};

const COLUMNS: [Column; 5] = [
    Column::new("Operation", 10, 18).pinned(),
    Column::new("Version", 8, 12).priority(2),
    Column::new("Owner", 5, 10).priority(1),
    Column::new("Size", 4, 8).priority(3).right(),
    Column::new("Message", 8, 0),
];

fn rows() -> Vec<Row> {
    vec![
        Row::new([
            Cell::new("deploy api"),
            Cell::new("v1 → v2").faint(),
            Cell::new("billing"),
            Cell::new("12 MB").faint(),
            Cell::new("uploading the bundle"),
        ])
        .mark(Mark::new("⧗").style(Style::new().fg(Color::Yellow))),
        Row::new(["update tool", "0.1 → 0.2", "tools", "640 kB", "finished"])
            .mark(Mark::new("✓").style(Style::new().fg(Color::Green))),
    ]
}

fn key(list: &mut List, key: Key) -> Option<usize> {
    match list.key(key) {
        Step::Open(index) => Some(index),
        _ => None,
    }
}

fn draw(frame: &mut Frame, list: &mut List) {
    let styles = Styles::new()
        .selected(Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .faint(Style::new().add_modifier(Modifier::DIM))
        .base(Style::new().bg(Color::Black));
    let view = ListView::new(list, &COLUMNS)
        .styles(styles)
        .header(true)
        .end(Some("· end ·"))
        .empty(Some("Nothing yet."));
    frame.render_widget(view, frame.area());
}

fn click(list: &List, area: ratatui::layout::Rect, column: u16, row: u16) -> Option<usize> {
    list.hit(area, true, column, row)
}

fn main() {
    let mut list = List::new().with_rows(rows());
    key(&mut list, Key::Down);
    click(&list, ratatui::layout::Rect::new(0, 0, 80, 10), 3, 1);
}
```
