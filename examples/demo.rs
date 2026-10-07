use std::fmt::Write;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use pito_list::{Cell, Column, Key, Keys, List, ListView, Mark, Part, Row, Step, Styles};
use ratatui::{
    DefaultTerminal, Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
};

const BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
const FAINT: Style = Style::new().add_modifier(Modifier::DIM);
const TICK: Duration = Duration::from_millis(120);
const HINTS: &str = "↑↓ j k move · g G ends · v range · enter open · q quit";

const COLUMNS: [Column; 5] = [
    Column::new("Job", 10, 16).pinned(),
    Column::new("Version", 7, 14).priority(2),
    Column::new("Owner", 5, 8).priority(1),
    Column::new("Size", 6, 0).priority(3).right().fit(9),
    Column::new("Message", 8, 0),
];

struct Job {
    index: usize,
    done: u32,
    total: u32,
    rate: u32,
    verb: &'static str,
}

struct App {
    list: List,
    jobs: [Job; 2],
    anchor: Option<usize>,
    opened: Option<usize>,
    text: String,
}

fn running(name: &str, from: &str, to: &str, owner: &str) -> Row {
    Row::new([
        Cell::new(name),
        Cell::parts([Part::new(from), Part::new(format!(" → {to}")).faint()]),
        Cell::new(owner),
        Cell::new(""),
        Cell::new(""),
    ])
    .mark(Mark::new("⧗").style(Style::new().fg(Color::Yellow)))
}

fn queued(name: &str, version: &str, owner: &str, message: &str) -> Row {
    Row::new([
        Cell::new(name),
        Cell::new(version),
        Cell::new(owner),
        Cell::new("—").faint(),
        Cell::new(message).faint(),
    ])
    .mark(Mark::new("·").faint())
}

fn done(name: &str, version: &str, owner: &str, size: &str, message: &str) -> Row {
    Row::new([name, version, owner, size, message])
        .mark(Mark::new("✓").style(Style::new().fg(Color::Green)))
}

fn heading(title: &str, count: usize) -> Row {
    Row::heading([
        Part::new(title).style(BOLD),
        Part::new(format!(" · {count}")).faint(),
    ])
}

fn rows() -> Vec<Row> {
    vec![
        heading("Running", 2),
        running("deploy api", "v1.4", "v1.5", "web"),
        running("migrate store", "v12", "v13", "data"),
        Row::blank(),
        heading("Queued", 3),
        queued("build docs", "v0.9", "docs", "waiting for a runner"),
        queued("rotate logs", "daily", "ops", "starts at 02:00"),
        queued("resize images", "v2.1", "media", "after build docs"),
        Row::blank(),
        heading("Done", 5),
        done(
            "update tool",
            "0.1 → 0.2",
            "tools",
            "640 kB",
            "finished in 42 s",
        ),
        done("bench parser", "v3.0", "core", "1.2 MB", "0.2 ms a frame"),
        Row::new([
            Cell::new("release notes"),
            Cell::new("a note that runs across the columns after the name")
                .faint()
                .rest(),
        ])
        .mark(Mark::new("✓").style(Style::new().fg(Color::Green))),
        done("backup config", "nightly", "ops", "88 kB", "finished"),
        done("sync mirrors", "v5", "infra", "9.4 MB", "finished in 3 min"),
    ]
}

fn styles() -> Styles {
    Styles::new()
        .selected(
            Style::new()
                .bg(Color::Indexed(237))
                .add_modifier(Modifier::BOLD),
        )
        .faint(FAINT)
        .header(Style::new().fg(Color::DarkGray))
        .header_key(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .cursor(Style::new().fg(Color::Cyan))
        .keep_colours(true)
}

impl App {
    fn new() -> Self {
        let mut app = App {
            list: List::new().keys(Keys::VIM).with_rows(rows()),
            jobs: [
                Job {
                    index: 1,
                    done: 30,
                    total: 240,
                    rate: 3,
                    verb: "uploading the bundle",
                },
                Job {
                    index: 2,
                    done: 180,
                    total: 640,
                    rate: 7,
                    verb: "copying tables",
                },
            ],
            anchor: None,
            opened: None,
            text: String::with_capacity(32),
        };
        app.tick();
        app
    }

    fn tick(&mut self) {
        for job in &mut self.jobs {
            job.done = (job.done + job.rate) % job.total;
            let share = job.done * 100 / job.total;
            let warm = match share {
                0..40 => Color::Green,
                40..75 => Color::Yellow,
                _ => Color::Red,
            };
            let Some(row) = self.list.row_mut(job.index) else {
                continue;
            };
            let cells = row.cells_mut();
            self.text.clear();
            let _ = write!(self.text, "{}.{} MB", job.done / 10, job.done % 10);
            cells[3].set_text(&self.text);
            cells[3].set_style(Style::new().fg(warm));
            self.text.clear();
            let _ = write!(self.text, "{} · {share}%", job.verb);
            cells[4].set_text(&self.text);
        }
    }

    fn key(&mut self, press: KeyEvent) {
        match press.code {
            KeyCode::Char('v') => {
                self.anchor = match self.anchor {
                    Some(_) => None,
                    None => self.list.selected(),
                }
            }
            KeyCode::Esc => {
                self.anchor = None;
                self.opened = None;
            }
            _ => {
                if let Step::Open(index) = self.list.key(Key::from(press)) {
                    self.opened = Some(index);
                }
            }
        }
        self.list
            .select_range(self.anchor.zip(self.list.selected()));
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let rows = Rect {
            height: area.height.saturating_sub(2),
            ..area
        };
        let view = ListView::new(&mut self.list, &COLUMNS)
            .styles(styles())
            .header(true)
            .key_column(Some(3))
            .end(Some("· end ·"));
        frame.render_widget(view, rows);
        if area.height < 2 {
            return;
        }
        let note = match (self.list.range(), self.opened) {
            (Some((from, to)), _) => format!("range {} … {}", self.name(from), self.name(to)),
            (None, Some(index)) => format!("opened {}", self.name(index)),
            (None, None) => String::new(),
        };
        let line = area.bottom() - 1;
        let width = u16::try_from(note.chars().count()).unwrap_or(0);
        let buffer = frame.buffer_mut();
        buffer.set_string(area.x + 2, line, HINTS, FAINT);
        buffer.set_string(
            area.right().saturating_sub(width + 2),
            line,
            &note,
            Style::new().fg(Color::Cyan),
        );
    }

    fn name(&self, index: usize) -> &str {
        self.list
            .rows()
            .get(index)
            .and_then(|row| row.cells().first())
            .map_or("", Cell::text)
    }
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();
    let mut last = Instant::now();
    loop {
        terminal.draw(|frame| app.draw(frame))?;
        if event::poll(TICK.saturating_sub(last.elapsed()))?
            && let Event::Key(press) = event::read()?
            && press.kind == KeyEventKind::Press
        {
            if press.code == KeyCode::Char('q') {
                return Ok(());
            }
            app.key(press);
        }
        if last.elapsed() >= TICK {
            app.tick();
            last = Instant::now();
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}
