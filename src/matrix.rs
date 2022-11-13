use rand::distributions::Uniform;
use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng};
use std::fmt::{Display, Formatter};
use std::io::Write;
use termion::color::{Blue, Fg, White};
use termion::cursor::Goto;
use termion::screen::AlternateScreen;

const LINE_SPAWN_CHANCE: f32 = 0.1;
const LINE_MAX_LEN: u32 = 15;
const CHAR_OPTIONS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890-=!@#$%^&*()+[]{};',./<>?~|";

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum LineChar {
    #[default]
    None,
    Tail(char),
    Head(char),
}

impl Display for LineChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            // TODO: make head and tail color configurable.
            LineChar::None => write!(f, "  "),
            LineChar::Tail(c) => write!(f, " {}", c),
            LineChar::Head(c) => write!(f, " {}{}{}", Fg(White), c, Fg(Blue)),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
struct Line {
    chars: Vec<char>,
    pos: (usize, usize),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Matrix {
    size_x: usize,
    size_y: usize,
    lines: Vec<Line>,
}

impl Matrix {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        Matrix {
            size_x,
            size_y,
            lines: vec![],
        }
    }

    pub fn tick(&mut self) {
        self.move_lines();
        self.new_lines();
        self.end_lines();
    }

    fn new_lines(&mut self) {
        let mut rng = thread_rng();
        let d_spawn = Uniform::new(0.0, 1.0);
        let d_length = Uniform::new(3, LINE_MAX_LEN);

        // if the top 3 rows are all empty, have a chance to spawn a new line.
        let mut can_spawn = vec![true; self.size_x];
        for line in self.lines.iter() {
            if (line.pos.1 as i32 - line.len() as i32) < 3 {
                can_spawn[line.pos.0] = false;
            }
        }

        for col in (0..self.size_x).filter(|col| can_spawn[*col]) {
            if rng.sample(d_spawn) < LINE_SPAWN_CHANCE {
                let len = rng.sample(d_length);
                self.lines.push(Line::new(col, len));
            }
        }
    }

    fn move_lines(&mut self) {
        for line in self.lines.iter_mut() {
            line.pos.1 += 1;
            line.shift();
        }
    }

    fn end_lines(&mut self) {
        let mut ended_lines = vec![];

        for (i, line) in self.lines.iter().enumerate() {
            if line.pos.1 as i32 - line.len() as i32 > self.size_y as i32 {
                ended_lines.push(i);
            }
        }

        for ended_line in ended_lines.into_iter().rev() {
            self.lines.remove(ended_line);
        }
    }

    // TODO: remove buffer from internal struct.
    pub fn render<W: Write>(&self, screen: &mut AlternateScreen<W>) {
        let mut buff = vec![vec![LineChar::None; self.size_y]; self.size_x];

        for line in self.lines.iter() {
            let x = line.pos.0;
            let line_y = line.pos.1;
            for i in 0..line.len() - 1 {
                // Off the top of the screen.
                if i > line_y {
                    break;
                }
                let y = line_y - i;
                // Off the bottom of the screen.
                if y >= self.size_y {
                    continue;
                }

                let c = line.chars[i];
                buff[x][y] = LineChar::Tail(c);
            }
            // If on the screen, draw the head.
            if line_y < self.size_y {
                buff[x][line_y] = LineChar::Head(*line.chars.last().unwrap());
            }
        }

        for y in 0..self.size_y {
            write!(screen, "{}", Goto(1, (y + 1) as u16)).unwrap();

            for x in 0..self.size_x {
                write!(screen, "{}", buff[x][y]).unwrap();
            }
        }
        screen.flush().unwrap();
    }
}

impl Line {
    pub fn new(col: usize, len: u32) -> Self {
        let mut rng = thread_rng();
        Line {
            chars: CHAR_OPTIONS.chars().choose_multiple(&mut rng, len as usize),
            pos: (col, 0),
        }
    }

    pub fn shift(&mut self) {
        for i in (0..self.chars.len() - 1).rev() {
            self.chars[i + 1] = self.chars[i];
        }
        self.chars[0] = CHAR_OPTIONS.chars().choose(&mut thread_rng()).unwrap()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }
}
