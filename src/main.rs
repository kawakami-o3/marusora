use rand::prelude::*;
use std::io::prelude::*;
use std::{fs, io, path};
use structopt::StructOpt;
use termion::raw::IntoRawMode;
use tui::{
    backend::TermionBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Gauge, Widget},
    Terminal,
};

#[derive(Debug)]
struct App {
    words: Vec<(String, String)>,

    deck: Vec<usize>,
    target_idx: i32,
    number: usize, // Number of questions
}

impl App {
    fn new() -> App {
        App {
            words: Vec::new(),
            deck: Vec::new(),
            target_idx: -1,
            number: 0,
        }
    }

    fn add(&mut self, entry: Vec<&str>) {
        self.words
            .push((String::from(entry[0]), String::from(entry[1])));
    }

    fn load_files(&mut self, files: &Vec<path::PathBuf>) -> io::Result<()> {
        for f in files {
            let mut file = fs::File::open(f)?;

            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            let v: Vec<&str> = buffer.split('\n').collect();

            for ln in v {
                let w: Vec<&str> = ln.split(',').collect();

                if ln.len() >= 2 {
                    self.add(w);
                }
            }
        }

        Ok(())
    }

    fn prepare(&mut self, number: i32) {
        self.number = if number < 0 || number as usize > self.words.len() {
            self.words.len()
        } else {
            number as usize
        };
        self.deck = (0..self.number).collect();
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

    fn update_target(&mut self) -> Option<i32> {
        if self.target_idx >= 0 {
            self.deck.remove(self.target_idx as usize);
        }

        if self.deck.is_empty() {
            return None;
        }

        let mut rng = thread_rng();
        self.target_idx = rng.gen_range(0..self.deck.len()) as i32;

        Some(self.target_idx)
    }

    fn get_number(&self) -> usize {
        self.number
    }

    fn get_question_no(&self) -> usize {
        self.number - self.deck.len() + 1
    }

    fn get_question(&self) -> &String {
        &self.words[self.deck[self.target_idx as usize]].0
    }

    fn get_answer(&self) -> &String {
        &self.words[self.deck[self.target_idx as usize]].1
    }

    fn get_progress_percent(&self) -> u16 {
        (100 * (self.number - self.deck.len() + 1) / self.number) as u16
    }
}

struct Label<'a> {
    text: &'a str,
    x: u16,
    y: u16,
}

impl<'a> Default for Label<'a> {
    fn default() -> Label<'a> {
        Label {
            text: "",
            //x: 6,
            //y: 3,
            x: 0,
            y: 0,
        }
    }
}

impl<'a> Widget for Label<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        //let x = 6;
        //let y = 3;
        buf.set_string(
            area.left() + self.x,
            area.top() + self.y,
            self.text,
            Style::default(),
        );
    }
}

impl<'a> Label<'a> {
    //fn new() -> Label<'a> {
    //    Label {
    //        text: "",
    //        x: 0,
    //        y: 0,
    //    }
    //}

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

#[derive(StructOpt, Debug)]
#[structopt(name = "marusora")]
struct Opt {
    // Number of questions
    #[structopt(short, long, default_value = "-1")]
    number: i32,

    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<path::PathBuf>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let mut app = App::new();

    app.load_files(&opt.files)?;

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut bytes = stdin.bytes();

    app.prepare(opt.number);

    let mut mode = Mode::Question;

    loop {
        if mode == Mode::Question {
            let result = app.update_target();

            if result == None {
                break;
            }
        }

        // ターミナルサイズが変わった場合に備えて
        terminal.clear()?;

        terminal.draw(|f| {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("marusora")
                .border_type(BorderType::Rounded);
            f.render_widget(block, f.size());

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(20),
                        //Constraint::Percentage(50),
                        //Constraint::Percentage(50),
                        //Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let label = format!("{}/{}", app.get_question_no(), app.get_number());

            let gauge = Gauge::default()
                .block(Block::default().title("progress").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(app.get_progress_percent())
                .label(label);
            f.render_widget(gauge, chunks[0]);

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                //.direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Length(2),
                        //Constraint::Percentage(50),
                        //Constraint::Percentage(50),
                        //Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            let statement = format!("{}. {}", app.get_question_no(), app.get_question());
            let label = Label::default().text(statement.as_str());
            f.render_widget(label, main_chunks[0]);

            let answer = if mode == Mode::Question {
                "-"
            } else {
                app.get_answer()
            };
            //let label = Label::default().text(app.get_answer());
            let label = Label::default().text(answer);
            f.render_widget(label, main_chunks[1]);

            //let message = format!("> rest: {}, target: {}", app.deck.len(), app.target_idx);
            //let label = Label::new().text(message.as_str());
            //f.render_widget(label, chunks[2]);
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
