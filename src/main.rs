use std::io::{stdout, Write};
use std::{cmp, fs};

use crossterm::{cursor, execute, queue, style::Print, terminal, tty::IsTty};

use backend::Backend;

mod backend;

#[derive(Default)]
struct Coord {
    col: usize,
    row: usize,
}

struct Editor<W: Write> {
    w: W,
    backend: Backend,
    screen_size: Coord,
    visible_cursor: Coord,
    cursor: Coord,
    offset: Coord,
    current_row: usize,
}

impl<W: Write> Editor<W> {
    fn new(w: W, data: String) -> anyhow::Result<Self> {
        let stdin = std::io::stdin();
        if !stdin.is_tty() {
            return Err(anyhow::Error::msg("not in tty"));
        }

        Ok(Editor {
            w,
            backend: Backend::new(data),
            screen_size: terminal::size().map(|(c, r)| Coord {
                col: c as usize,
                row: r as usize,
            })?,
            visible_cursor: Coord::default(),
            cursor: Coord::default(),
            offset: Coord::default(),
            current_row: 0,
        })
    }

    fn run(&mut self) -> anyhow::Result<()> {
        use crossterm::{
            event,
            event::{Event::*, KeyCode::*, KeyEvent, KeyModifiers},
        };

        terminal::enable_raw_mode()?;
        execute!(self.w, terminal::EnterAlternateScreen)?;
        self.clear_screen()?;

        loop {
            self.rerender()?;

            match event::read()? {
                Key(KeyEvent { code: Esc, .. })
                | Key(KeyEvent {
                    code: Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    break;
                }
                Key(KeyEvent { code: Char(c), .. }) => {
                    self.backend
                        .insert(self.current_row, self.visible_cursor.col, c);
                    self.action_right(1);
                }
                Key(KeyEvent {
                    code: direction @ (Left | Right | Up | Down),
                    ..
                }) => match direction {
                    Left => {
                        self.action_left(1);
                    }
                    Right => {
                        self.action_right(1);
                    }
                    Up => {
                        self.action_up(1);
                    }
                    Down => {
                        self.action_down(1);
                    }
                    _ => unimplemented!(),
                },
                Key(KeyEvent { code: Enter, .. }) => {
                    unimplemented!()
                }
                Resize(c, r) => {
                    (self.screen_size.col, self.screen_size.row) = (c as usize, r as usize);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn action_up(&mut self, mut n: usize) {
        n = cmp::min(self.current_row, n);
        self.current_row -= n;

        if self.visible_cursor.row < n {
            // sub overflow line to offset
            self.offset.row = self.offset.row.saturating_sub(n - self.visible_cursor.row);
        }
        self.visible_cursor.row = self.visible_cursor.row.saturating_sub(n);
    }

    fn action_down(&mut self, mut n: usize) {
        let content_row = self.backend.row_length();

        // add n more than content row
        if self.current_row + n > content_row - 1 {
            n -= (self.current_row + n) - (content_row - 1)
        }
        self.current_row += n;

        self.visible_cursor.row += n;
        self.offset.row += self
            .visible_cursor
            .row
            .saturating_sub(self.screen_size.row - 1); // add overflow line to offset
        if self.offset.row + self.screen_size.row > content_row {
            self.offset.row = content_row - self.screen_size.row;
        }

        self.visible_cursor.row = cmp::min(self.visible_cursor.row, self.screen_size.row - 1);
    }

    fn action_left(&mut self, n: usize) {
        self.cursor.col = self.visible_cursor.col;
        self.cursor.col = self.cursor.col.saturating_sub(n);
    }

    fn action_right(&mut self, n: usize) {
        if let Some(t) = self.backend.get_row(self.current_row) {
            self.cursor.col += n;
            self.cursor.col = cmp::min(self.cursor.col, t.len());
        }
    }

    fn normalize_visible_cursor(&mut self) {
        if let Some(t) = self.backend.get_row(self.current_row) {
            self.visible_cursor.col = self.cursor.col;
            self.visible_cursor.col = cmp::min(self.visible_cursor.col, t.len());
        }
    }

    fn clear_screen(&mut self) -> anyhow::Result<()> {
        execute!(
            self.w,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }

    fn rerender(&mut self) -> anyhow::Result<()> {
        execute!(self.w, cursor::Hide, cursor::MoveTo(0, 0))?;

        // TODO: preserve row when resize
        self.normalize_visible_cursor();

        for i in 0..self.screen_size.row {
            if let Some(line) = self.backend.get_row(self.offset.row + i) {
                queue!(
                    self.w,
                    Print(line),
                    terminal::Clear(terminal::ClearType::UntilNewLine)
                )?;
                if i < self.screen_size.row - 1 {
                    queue!(self.w, Print("\r\n"))?;
                }
            } else {
                queue!(self.w, terminal::Clear(terminal::ClearType::UntilNewLine))?;
            }
        }

        queue!(
            self.w,
            cursor::MoveTo(
                self.visible_cursor.col as u16,
                self.visible_cursor.row as u16
            ),
            cursor::Show,
        )?;

        self.w.flush()?;
        Ok(())
    }
}

impl<W: Write> Drop for Editor<W> {
    fn drop(&mut self) {
        let _ = execute!(self.w, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn main() -> anyhow::Result<()> {
    let data = fs::read_to_string("Cargo.lock")?;

    let mut editor = Editor::new(stdout(), data)?;
    editor.run()?;
    Ok(())
}
