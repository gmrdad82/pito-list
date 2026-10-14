# Notices

The look of this crate (a bold marker on the selected row, padded columns and a
quiet end line) is in the style of HEY's terminal UI by 37signals
([basecamp/hey-cli](https://github.com/basecamp/hey-cli)). No code or text is
copied from it.

## Images

Every image in this repository is made from its own code and data, © 2026
Cătălin Ilinca, under the MIT licence like the code (see LICENSE); the names
and logos in them are as TRADEMARKS.md says.

- `docs/demo.gif`, `docs/images/range.gif`, `docs/images/scroll.gif` and
  `docs/images/live.gif`: clips of `examples/demo.rs`, recorded with prec from
  the shots in `render/terminal.toml` and made into GIFs with pconv.
- `docs/images/columns-96.png`, `docs/images/columns-46.png` and
  `docs/images/columns-38.png`: stills of `examples/demo.rs` at 96, 46 and 38
  columns, rendered with prec from the `narrow` shot.
- `docs/images/bench-bars.png` and `docs/images/frame-time-paired.png`: charts
  drawn with pito-compare from `docs/charts/bench.toml` (the output of
  `examples/bench.rs`) and `docs/charts/frame-time.toml` (the output of
  `examples/versus.rs`), measured on 2026-10-14.
