use rand::prelude::*;
use std::io::prelude::*;
use std::{env, fs, io};
use termion::raw::IntoRawMode;
use tui::{
    backend::TermionBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::canvas::Canvas,
    widgets::{Block, Borders, Widget},
    Terminal,
};

#[derive(Debug)]
struct App {
    words: Vec<(String, String)>,

    deck: Vec<usize>,
    target_idx: usize,
}

impl App {
    fn new() -> App {
        App {
            words: Vec::new(),
            deck: Vec::new(),
            target_idx: 0,
        }
    }

    fn add(&mut self, entry: Vec<&str>) {
        self.words
            .push((String::from(entry[0]), String::from(entry[1])));
    }

    fn prepare(&mut self) {
        self.deck = (0..self.words.len()).collect();
    }

    /*
    fn next(&mut self) -> Option<(String, String)> {
        if self.deck.is_empty() {
            return None;
        }

        let mut rng = thread_rng();
        let i = rng.gen_range(0..self.deck.len());

        let word = self.words[self.deck[i]].clone();
        self.deck.remove(i);
        Some((word.0.clone(), word.1.clone()))
    }
    */

    fn update_target(&mut self) {
        //if self.deck.is_empty() {
        //    self.target_idx = 0;
        //} else {
        //    let mut rng = thread_rng();
        //    self.target_idx = rng.gen_range(0..self.deck.len());
        //}
        let mut rng = thread_rng();
        self.target_idx = rng.gen_range(0..self.deck.len());
        self.deck.remove(self.target_idx);

        /*
        let word = self.words[self.deck[i]].clone();
        self.deck.remove(i);
        Some((word.0.clone(), word.1.clone()))
        */
    }

    fn get_question(&self) -> &String {
        &self.words[self.deck[self.target_idx]].0
    }

    fn get_answer(&self) -> &String {
        &self.words[self.deck[self.target_idx]].1
    }
}

struct Label<'a> {
    text: &'a str,
}

impl<'a> Default for Label<'a> {
    fn default() -> Label<'a> {
        Label { text: "" }
    }
}

impl<'a> Widget for Label<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let x = 6;
        let y = 3;
        buf.set_string(area.left() + x, area.top() + y, self.text, Style::default());
    }
}

impl<'a> Label<'a> {
    fn text(mut self, text: &'a str) -> Label<'a> {
        self.text = text;
        self
    }
}

#[derive(PartialEq)]
enum Mode {
    Question,
    Answer,
}

fn main() -> io::Result<()> {
    let argv: Vec<String> = env::args().collect();
    println!("{:?}", argv);

    let mut app = App::new();

    for path in &argv[1..] {
        let mut file = fs::File::open(path).expect("file not found");

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).expect("wrong");
        let v: Vec<&str> = buffer.split('\n').collect();

        for ln in v {
            let w: Vec<&str> = ln.split(',').collect();

            if ln.len() >= 2 {
                app.add(w);
            }
        }
    }

    //let mut view = View::new();
    //view.clear().unwrap();

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut bytes = stdin.bytes();

    app.prepare();

    let mut mode = Mode::Question;
    //let mut word = Some((String::new(), String::new()));
    loop {
        if mode == Mode::Question {
            app.update_target();
            //word = app.next();
            //if word == None {
            //    break;
            //}
        }

        //let w = word.clone().unwrap();
        //let w = word.unwrap();

        //view.draw(&word).unwrap();
        //view.draw(&w).unwrap();
        //let b = bytes.next().unwrap().unwrap();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                //.margin(1)
                .constraints(
                    [
                        Constraint::Length(10),
                        Constraint::Length(10),
                        //Constraint::Percentage(50),
                        //Constraint::Percentage(50),
                        //Constraint::Percentage(10)
                    ]
                    .as_ref(),
                )
                .split(f.size());

            /*
            let block = Block::default()
                .title("Block")
                .borders(Borders::ALL);

            let canvas = Canvas::default()
                .block(block)
                .paint(|ctx| {
                    //ctx.print(0.0, 0.0, self.word.0.as_str(), Color::White);
                    //ctx.print(0.0, 0.0, app.get_question().as_str(), Color::White);
                    ctx.print(0.0, 0.0, "0", Color::White);
                    //ctx.print(1.0, 1.0, "1", Color::White);
                    //ctx.print(1.0, 3.0, "2\nh", Color::White);
                })
                .x_bounds([-5.0,5.0])
                .y_bounds([-5.0,5.0]);
                */

            let label = Label::default().text(app.get_question());

            //f.render_widget(block, chunks[0]);
            //f.render_widget(canvas, chunks[0]);
            f.render_widget(label, chunks[0]);

            //let block = Block::default()
            //    .title("Block 2")
            //    .borders(Borders::ALL);

            let answer = if mode == Mode::Question {
                "-"
            } else {
                app.get_answer()
            };
            //let label = Label::default().text(app.get_answer());
            let label = Label::default().text(answer);
            f.render_widget(label, chunks[1]);
        })?;

        match bytes.next().unwrap().unwrap() {
            b'q' => {
                break;
            }
            _ => {
                //mode.next();
                mode = if mode == Mode::Question {
                    Mode::Answer
                } else {
                    Mode::Question
                };
            }
        }
    }

    Ok(())
}
