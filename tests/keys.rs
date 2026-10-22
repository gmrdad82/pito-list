mod common;

use common::*;
use pito_list::{Key, Keys, List, Row, Step};

const UP: &[Key] = &[Key::Char('k')];
const DOWN: &[Key] = &[Key::Char('j')];
const PAGE_UP: &[Key] = &[Key::Ctrl('b')];
const PAGE_DOWN: &[Key] = &[Key::Ctrl('f')];
const FIRST: &[Key] = &[Key::Char('<')];
const LAST: &[Key] = &[Key::Char('>')];
const OPEN: &[Key] = &[Key::Char('o'), Key::Char(' ')];

fn remapped() -> Keys {
    Keys::new()
        .up(UP)
        .down(DOWN)
        .page_up(PAGE_UP)
        .page_down(PAGE_DOWN)
        .first(FIRST)
        .last(LAST)
        .open(OPEN)
}

fn numbered(count: usize) -> List {
    List::new().with_rows((0..count).map(|n| Row::new([format!("row {n}")])))
}

#[test]
fn the_builders_replace_each_map_and_the_default_is_new() {
    let keys = remapped();
    assert_eq!(keys.up, UP);
    assert_eq!(keys.down, DOWN);
    assert_eq!(keys.page_up, PAGE_UP);
    assert_eq!(keys.page_down, PAGE_DOWN);
    assert_eq!(keys.first, FIRST);
    assert_eq!(keys.last, LAST);
    assert_eq!(keys.open, OPEN);
    assert_eq!(Keys::default(), Keys::new());
    assert_eq!(Keys::default().up, &[Key::Up]);
    assert_eq!(Keys::default().open, &[Key::Enter]);
    let only_up = Keys::new().up(UP);
    assert_eq!(only_up.down, Keys::new().down);
    assert_eq!(only_up.open, Keys::new().open);
    assert_ne!(only_up, Keys::new());
}

#[test]
fn an_app_remaps_its_keys_and_the_old_ones_do_nothing() {
    let mut list = numbered(30).keys(remapped());
    for old in [
        Key::Down,
        Key::Up,
        Key::PageDown,
        Key::PageUp,
        Key::End,
        Key::Home,
        Key::Enter,
    ] {
        assert_eq!(list.key(old), Step::Pass, "{old:?}");
        assert_eq!(list.selected(), Some(0), "{old:?}");
    }
    assert_eq!(list.key(Key::Char('j')), Step::Moved);
    assert_eq!(list.selected(), Some(1));
    assert_eq!(list.key(Key::Char('k')), Step::Moved);
    assert_eq!(list.selected(), Some(0));
    assert_eq!(list.key(Key::Char('k')), Step::Held);
    assert_eq!(list.key(Key::Ctrl('f')), Step::Moved);
    assert_eq!(list.selected(), Some(10));
    assert_eq!(list.key(Key::Ctrl('b')), Step::Moved);
    assert_eq!(list.selected(), Some(0));
    assert_eq!(list.key(Key::Char('>')), Step::Moved);
    assert_eq!(list.selected(), Some(29));
    assert_eq!(list.key(Key::Char('j')), Step::Held);
    assert_eq!(list.key(Key::Char('o')), Step::Open(29));
    assert_eq!(list.key(Key::Char(' ')), Step::Open(29));
    assert_eq!(list.key(Key::Char('<')), Step::Moved);
    assert_eq!(list.selected(), Some(0));
    assert_eq!(list.key(Key::Char('o')), Step::Open(0));
    let drawn = draw(&mut list, 30, 6);
    assert_eq!(drawn.text[1], "▌ row 0");
}

#[test]
fn a_key_in_no_map_passes_and_moves_nothing() {
    let mut list = numbered(5).keys(remapped());
    list.select(2);
    for key in [
        Key::Char('x'),
        Key::Ctrl('j'),
        Key::Alt('j'),
        Key::Char('J'),
        Key::Tab,
        Key::Esc,
        Key::Other,
    ] {
        assert_eq!(list.key(key), Step::Pass, "{key:?}");
        assert_eq!(list.selected(), Some(2), "{key:?}");
    }
    let mut plain = numbered(5);
    assert_eq!(plain.key(Key::Char('j')), Step::Pass);
    assert_eq!(plain.key(Key::Char('o')), Step::Pass);
    assert_eq!(plain.selected(), Some(0));
}

#[test]
fn an_empty_list_holds_on_its_mapped_keys_and_passes_the_rest() {
    let mut list = List::default().keys(remapped());
    assert!(list.is_empty());
    for key in [Key::Char('j'), Key::Char('k'), Key::Char('>'), Key::Char('o')] {
        assert_eq!(list.key(key), Step::Held, "{key:?}");
    }
    assert_eq!(list.key(Key::Down), Step::Pass);
    assert_eq!(list.selected(), None);
    let mut fresh = List::default();
    assert_eq!(fresh.len(), 0);
    assert_eq!(fresh.key(Key::Down), Step::Held);
    assert_eq!(fresh.key(Key::Enter), Step::Held);
    assert_eq!(draw(&mut fresh, 20, 3).text[1], "  Nothing here.");
}

#[cfg(feature = "crossterm")]
mod crossterm_keys {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use pito_list::{Key, Keys, List, Row, Step};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Key {
        Key::from(KeyEvent::new(code, modifiers))
    }

    #[test]
    fn super_hyper_and_meta_make_any_key_other() {
        for held in [KeyModifiers::SUPER, KeyModifiers::HYPER, KeyModifiers::META] {
            assert_eq!(key(KeyCode::Char('j'), held), Key::Other, "{held:?}");
            assert_eq!(key(KeyCode::Down, held), Key::Other, "{held:?}");
            assert_eq!(key(KeyCode::Enter, held), Key::Other, "{held:?}");
            assert_eq!(
                key(KeyCode::Char('d'), held | KeyModifiers::CONTROL),
                Key::Other,
                "{held:?}"
            );
            assert_eq!(key(KeyCode::Tab, held | KeyModifiers::SHIFT), Key::Other);
        }
    }

    #[test]
    fn control_and_alt_on_chars() {
        let both = KeyModifiers::CONTROL | KeyModifiers::ALT;
        assert_eq!(key(KeyCode::Char('@'), both), Key::Char('@'));
        assert_eq!(key(KeyCode::Char('Q'), both), Key::Char('Q'));
        assert_eq!(
            key(KeyCode::Char('U'), KeyModifiers::CONTROL),
            Key::Ctrl('u')
        );
        assert_eq!(
            key(KeyCode::Char('u'), KeyModifiers::CONTROL),
            Key::Ctrl('u')
        );
        assert_eq!(
            key(
                KeyCode::Char('U'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            ),
            Key::Ctrl('u')
        );
        assert_eq!(key(KeyCode::Char('x'), KeyModifiers::ALT), Key::Alt('x'));
        assert_eq!(key(KeyCode::Char('X'), KeyModifiers::ALT), Key::Alt('X'));
        assert_eq!(key(KeyCode::Char('é'), KeyModifiers::NONE), Key::Char('é'));
    }

    #[test]
    fn alt_on_a_non_char_key_is_other() {
        for code in [
            KeyCode::Enter,
            KeyCode::Tab,
            KeyCode::Left,
            KeyCode::PageDown,
            KeyCode::Backspace,
        ] {
            assert_eq!(key(code, KeyModifiers::ALT), Key::Other, "{code:?}");
        }
    }

    #[test]
    fn named_keys_convert() {
        let none = KeyModifiers::NONE;
        let cases = [
            (KeyCode::Tab, KeyModifiers::SHIFT, Key::BackTab),
            (KeyCode::Tab, none, Key::Tab),
            (KeyCode::BackTab, KeyModifiers::SHIFT, Key::BackTab),
            (KeyCode::BackTab, none, Key::BackTab),
            (KeyCode::Enter, none, Key::Enter),
            (KeyCode::Esc, none, Key::Esc),
            (KeyCode::Backspace, none, Key::Backspace),
            (KeyCode::Delete, none, Key::Delete),
            (KeyCode::Home, none, Key::Home),
            (KeyCode::End, none, Key::End),
            (KeyCode::PageUp, none, Key::PageUp),
            (KeyCode::PageDown, none, Key::PageDown),
            (KeyCode::Left, none, Key::Left),
            (KeyCode::Right, none, Key::Right),
            (KeyCode::Up, none, Key::Up),
            (KeyCode::Down, none, Key::Down),
            (KeyCode::Down, KeyModifiers::SHIFT, Key::Down),
            (KeyCode::Up, KeyModifiers::CONTROL, Key::Up),
            (KeyCode::F(5), none, Key::Other),
            (KeyCode::Insert, none, Key::Other),
            (KeyCode::Null, none, Key::Other),
        ];
        for (code, modifiers, expected) in cases {
            assert_eq!(key(code, modifiers), expected, "{code:?} {modifiers:?}");
        }
    }

    #[test]
    fn a_repeat_converts_and_a_release_does_not() {
        let repeat = KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Repeat,
            state: KeyEventState::NONE,
        };
        assert_eq!(Key::from(repeat), Key::Char('j'));
        let release = KeyEvent {
            kind: KeyEventKind::Release,
            ..repeat
        };
        assert_eq!(Key::from(release), Key::Other);
    }

    #[test]
    fn crossterm_events_drive_a_remapped_list() {
        const DOWN: &[Key] = &[Key::Char('j'), Key::Ctrl('n')];
        const OPEN: &[Key] = &[Key::Char('l')];
        let mut list = List::new()
            .with_rows((0..4).map(|n| Row::new([format!("row {n}")])))
            .keys(Keys::new().down(DOWN).open(OPEN));
        let none = KeyModifiers::NONE;
        assert_eq!(list.key(key(KeyCode::Char('j'), none)), Step::Moved);
        assert_eq!(
            list.key(key(KeyCode::Char('N'), KeyModifiers::CONTROL)),
            Step::Moved
        );
        assert_eq!(list.selected(), Some(2));
        assert_eq!(
            list.key(key(KeyCode::Char('j'), KeyModifiers::SUPER)),
            Step::Pass
        );
        assert_eq!(
            list.key(key(KeyCode::Char('j'), KeyModifiers::ALT)),
            Step::Pass
        );
        assert_eq!(list.key(key(KeyCode::Enter, none)), Step::Pass);
        assert_eq!(list.key(key(KeyCode::Char('l'), none)), Step::Open(2));
        assert_eq!(list.key(key(KeyCode::F(5), none)), Step::Pass);
    }
}
