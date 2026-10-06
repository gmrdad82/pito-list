use std::iter;

use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub(crate) const ELLIPSIS: &str = "…";

fn glyph(grapheme: &str) -> Option<(&str, usize)> {
    if grapheme.contains(char::is_control) {
        return grapheme.contains(['\t', '\n', '\r']).then_some((" ", 1));
    }
    Some((grapheme, grapheme.width()))
}

pub(crate) fn width(text: &str) -> usize {
    text.graphemes(true)
        .filter_map(glyph)
        .fold(0usize, |total, (_, cells)| total.saturating_add(cells))
}

fn within<'t>(runs: impl Iterator<Item = (&'t str, Style)>, room: usize) -> Option<usize> {
    let mut used = 0usize;
    for (text, _) in runs {
        for (_, cells) in text.graphemes(true).filter_map(glyph) {
            used = used.saturating_add(cells);
            if used > room {
                return None;
            }
        }
    }
    Some(used)
}

pub(crate) fn cells(text: &str) -> u16 {
    u16::try_from(width(text)).unwrap_or(u16::MAX)
}

fn small(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

pub(crate) fn blank(buf: &mut Buffer, clip: Rect, x: u16, y: u16, room: u16, style: Style) {
    let area = clip.intersection(buf.area);
    if y < area.top() || y >= area.bottom() {
        return;
    }
    let from = x.max(area.left());
    let to = x.saturating_add(room).min(area.right());
    for column in from..to {
        let cell = &mut buf[(column, y)];
        cell.reset();
        cell.set_style(style);
    }
}

pub(crate) fn put(
    buf: &mut Buffer,
    clip: Rect,
    x: u16,
    y: u16,
    room: u16,
    text: &str,
    style: Style,
) {
    put_runs(buf, clip, x, y, room, iter::once((text, style)), false);
}

pub(crate) fn put_runs<'t, I>(
    buf: &mut Buffer,
    clip: Rect,
    x: u16,
    y: u16,
    room: u16,
    runs: I,
    right: bool,
) where
    I: Iterator<Item = (&'t str, Style)> + Clone,
{
    let area = clip.intersection(buf.area);
    let (x, room) = if right {
        let visible = usize::from(room.min(area.right().saturating_sub(x)));
        let shift = small(within(runs.clone(), visible).map_or(0, |used| visible - used));
        (x.saturating_add(shift), room - shift)
    } else {
        (x, room)
    };
    if y < area.top() || y >= area.bottom() || x < area.left() || x >= area.right() {
        return;
    }
    let room = usize::from(room.min(area.right() - x));
    if room == 0 {
        return;
    }
    let overflow = within(runs.clone(), room).is_none();
    let limit = if overflow { room - 1 } else { room };
    let mut used = 0usize;
    let mut cut = None;
    'runs: for (text, style) in runs {
        for (grapheme, cells) in text.graphemes(true).filter_map(glyph) {
            if cells == 0 {
                continue;
            }
            if used.saturating_add(cells) > limit {
                cut = Some(style);
                break 'runs;
            }
            buf.set_stringn(x.saturating_add(small(used)), y, grapheme, cells, style);
            used += cells;
        }
    }
    if let (true, Some(style)) = (overflow, cut) {
        buf.set_stringn(x.saturating_add(small(used)), y, ELLIPSIS, 1, style);
    }
}
