use ratatui::{buffer::Buffer, style::Style};
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

fn fits(text: &str, room: usize) -> bool {
    let mut used = 0usize;
    for (_, cells) in text.graphemes(true).filter_map(glyph) {
        used = used.saturating_add(cells);
        if used > room {
            return false;
        }
    }
    true
}

pub(crate) fn cells(text: &str) -> u16 {
    u16::try_from(width(text)).unwrap_or(u16::MAX)
}

pub(crate) fn blank(buf: &mut Buffer, x: u16, y: u16, room: u16, style: Style) {
    let area = buf.area;
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

pub(crate) fn put(buf: &mut Buffer, x: u16, y: u16, room: u16, text: &str, style: Style) {
    let area = buf.area;
    if y < area.top() || y >= area.bottom() || x < area.left() || x >= area.right() {
        return;
    }
    let room = usize::from(room.min(area.right() - x));
    if room == 0 {
        return;
    }
    let overflow = !fits(text, room);
    let limit = if overflow { room - 1 } else { room };
    let mut used = 0usize;
    for (grapheme, cells) in text.graphemes(true).filter_map(glyph) {
        if cells == 0 {
            continue;
        }
        if used.saturating_add(cells) > limit {
            break;
        }
        let at = x.saturating_add(u16::try_from(used).unwrap_or(u16::MAX));
        buf.set_stringn(at, y, grapheme, cells, style);
        used += cells;
    }
    if overflow {
        let at = x.saturating_add(u16::try_from(used).unwrap_or(u16::MAX));
        buf.set_stringn(at, y, ELLIPSIS, 1, style);
    }
}
