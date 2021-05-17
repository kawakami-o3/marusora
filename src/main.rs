use rand::prelude::*;
use std::io::prelude::*;
use std::{fs, io, path, panic};
use structopt::StructOpt;
use termion::raw::{IntoRawMode, RawTerminal};
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
    target_idx: usize,
    number: usize, // Number of questions

    mode: Mode,
}

impl App {
    fn new() -> App {
        App {
            words: Vec::new(),
            deck: Vec::new(),
            target_idx: 0,
            number: 0,
            mode: Mode::Question,
        }
    }

    fn add(&mut self, entry: Vec<&str>) {
        self.words
            .push((String::from(entry[0]), String::from(entry[1])));
    }

    fn load_files(&mut self, files: &Vec<path::PathBuf>) -> Result<(), io::Error> {
        for f in files {
            let mut file = fs::File::open(f)?;

            let mut buffer = String::new();
            match file.read_to_string(&mut buffer) {
                Ok(_) => {},
                Err(e) => return Err(e),
            };
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

        self.deck = Vec::new();
        let mut candidates: Vec<usize> = (0..self.words.len()).collect();

        let mut rng = thread_rng();


        while self.deck.len() < self.number {
            let i = rng.gen_range(0..candidates.len());
            self.deck.push(candidates[i]);
            candidates.remove(i);
        }

        self.target_idx = 0;
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

    fn update(&mut self) {
        self.mode = if self.mode == Mode::Question {
            Mode::Answer
        } else {
            Mode::Question
        };

        if self.mode == Mode::Question {
            self.target_idx += 1;
        }

        if self.deck.len() <= self.target_idx {
            self.mode = Mode::Done;
        }
    }

    fn get_number(&self) -> usize {
        self.number
    }

    fn get_question_no(&self) -> usize {
        //self.number - self.deck.len() + 1
        self.target_idx + 1
    }

    fn get_question(&self) -> &String {
        &self.words[self.deck[self.target_idx]].0
    }

    fn get_answer(&self) -> &String {
        &self.words[self.deck[self.target_idx]].1
    }

    fn get_progress_percent(&self) -> u16 {
        (100 * (self.target_idx + 1) / self.number) as u16
    }

    fn study_again(&mut self) {
        self.deck.push(self.deck[self.target_idx]);
        self.number += 1;
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

#[derive(Debug, PartialEq)]
enum Mode {
    Question,
    Answer,
    Done,
}

struct View {
    terminal: Terminal<TermionBackend<RawTerminal<io::Stdout>>>,
}

impl View {
    fn new(stdout: RawTerminal<io::Stdout>) -> View {
        View {
            // TODO error handling
            terminal: Terminal::new(TermionBackend::new(stdout)).unwrap(),
        }
    }

    fn display(&mut self, app: &App) -> io::Result<()> {
        // ターミナルサイズが変わった場合に備えて
        match self.terminal.clear() {
            Ok(()) => {},
            Err(e) => return Err(e),
        };

        self.terminal.draw(|f| {
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

            let answer = if app.mode == Mode::Question {
                "-"
            } else {
                app.get_answer()
            };
            //let label = Label::default().text(app.get_answer());
            let label = Label::default().text(answer);
            f.render_widget(label, main_chunks[1]);

            //let message = format!("> rest: {}, target: {}, deck: {:?}", app.deck.len(), app.target_idx, app.deck);
            //let label = Label::default().text(message.as_str());
            //f.render_widget(label, main_chunks[2]);
        })

    }
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


// https://github.com/fdehau/tui-rs/issues/177
fn setup_panic() -> Result<(), io::Error> {
    let raw_handle = match io::stdout().into_raw_mode() {
        Ok(t) => t,
        Err(e) => return Err(e),
    };
    //let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        raw_handle.suspend_raw_mode()
            .unwrap_or_else(|e| log::error!("Could not suspend raw mode: {}", e));
        //default_hook(info);
        better_panic::Settings::new().create_panic_handler()(info);
    }));

    Ok(())
}

fn main() {
    match setup_panic() {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    };

    let opt = Opt::from_args();

    let mut app = App::new();

    match app.load_files(&opt.files) {
        Ok(()) => {},
        Err(error) => {
            panic!("{:?}", error);
        }
    }

    let stdout = io::stdout().into_raw_mode();

    let mut view = match stdout {
        Ok(s) => View::new(s),
        Err(error) => {
            panic!("{:?}", error);
        }
    };

    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut bytes = stdin.bytes();

    app.prepare(opt.number);

    loop {
        match view.display(&app) {
            Ok(()) => {},
            Err(error) => {
                //view.reset();
                panic!("{:?}", error);
            }
        }

        match bytes.next().unwrap().unwrap() {
            b'q' => {
                break;
            }
            b'r' => {
                if app.mode == Mode::Answer {
                    app.study_again();
                }
            }
            _ => {
            }
        }

        app.update();

        if app.mode == Mode::Done {
            break;
        }
    }

}
