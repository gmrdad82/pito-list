# Changelog

## v0.8.0 — 2026-10-08

### Added

- An opt-in scrollbar and a count of the rows on screen: `ListView::scrollbar` and `ListView::count`, with `List::shown()` and `List::position()` for an app's own footer.

### Changed

- The demo quits as every PITO TUI does, on ctrl+c pressed twice.

## v0.7.0 — 2026-10-07

### Added

- `Shared`, a ready source over an `Arc<Vec<T>>` of the app's own items that builds a row only when a draw or a key asks for it, through a closure or a `Build<T>` type; `Arc<Vec<Row>>` and `Arc<[Row]>` are sources too.
- A demo example, whose clip opens the README, which points to PITO at https://pitomd.com.
- A code of conduct, a security policy and CI on GitHub.

### Changed

- The licence's copyright line carries the owner's legal name with its diacritics, Cătălin Ilinca.

## v0.6.1 — 2026-10-06

### Changed

- The gate is tiered: `bin/gate --fast` for every commit, the full gate before a merge, nightly and before every tag.

### Fixed

- A list drew past the right edge of its area when its pinned columns' minimums didn't fit; every write now clips to the list's own area.

## v0.6.0 — 2026-10-06

### Added

- `Styles::cursor` for a marker drawn in its own style over the selected line, `List::columns()` with each drawn column's index, position and width, and `Column::fit` for columns as wide as their widest cell on screen.

## v0.5.0 — 2026-10-06

### Added

- `Cell::set_style`, `Cell::set_part_style` and `Mark::set_style`, which restyle a cell, one part or a mark in place.

## v0.4.0 — 2026-10-05

### Added

- A selected range for Visual modes, with the marker on the cursor row only, kept colours under the selection, a styled header with a key column, and rows rewritten in place without allocating.

## v0.3.0 — 2026-10-05

### Added

- Section headings and blank rows the cursor skips, cells of several styled parts, a cell running to the end of the row, a flexible column, lazy rows through the `Source` trait, and paging by cursor or view.

### Fixed

- With an end line and one line left for rows, the selected last row no longer scrolls out behind the end line.

## v0.2.0 — 2026-10-05

### Added

- A base style that blank cells take, so a page background an app paints shows under the list.

## v0.1.0 — 2026-10-05

### Added

- Selectable list rows for ratatui apps, with a bold marker on the selected row and padded columns that drop by priority, a clipped last column, scrolling, paging and click hits.
