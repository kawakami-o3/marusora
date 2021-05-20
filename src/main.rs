use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{fs, io, panic, path};
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

//#[derive(Debug, Serialize, Deserialize)]
//struct Entry {
//    key: String,
//    value: String,
//}
//
//#[derive(Debug, Serialize, Deserialize)]
//struct EntryList {
//    list: Vec<Entry>,
//}
//
//impl EntryList {
//    fn new() -> Self {
//        EntryList { list: Vec::new() }
//    }
//
//    fn len(&self) -> usize {
//        self.list.len()
//    }
//
//    fn is_empty(&self) -> bool {
//        self.list.is_empty()
//    }
//
//    fn push(&mut self, e: Entry) {
//        self.list.push(e)
//    }
//
//    fn get(&self, i: usize) -> &Entry {
//        &self.list[i]
//    }
//}

/*
struct EntryListVisitor {
    marker: marker::PhantomData<fn() -> EntryList>,
}

impl EntryListVisitor {
    fn new() -> Self {
        EntryListVisitor {
            marker: marker::PhantomData,
        }
    }
}

impl<'de> de::Visitor<'de> for EntryListVisitor {
    type Value = EntryList;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("hogehoge")
    }

    //fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        //M: de::MapAccess<'de>,
        M: de::SeqAccess<'de>,
    {
        let mut list = EntryList::new();

        //while let Some((key, value)) = access.next_entry()? {
        while let Some(value) = access.next_element::<Entry>()? {
            println!("{:?}", value);
        }

        Ok(list)
    }
}

impl<'de> de::Deserialize<'de> for EntryList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(EntryListVisitor::new())
    }
}
*/

#[derive(Debug, Serialize, Deserialize)]
struct App {
    //entries: EntryList,
    entries: Vec<(String, String)>,

    deck: Vec<usize>,
    target_idx: usize,
    number: usize, // Number of questions

    mode: Mode,
}

impl App {
    fn new() -> Self {
        App {
            //entries: EntryList::new(),
            entries: Vec::new(),
            deck: Vec::new(),
            target_idx: 0,
            number: 0,
            mode: Mode::Question,
        }
    }

    fn restore(&mut self, file: &mut fs::File) -> io::Result<()> {
        let mut serialized = String::new();
        file.read_to_string(&mut serialized)?;

        if let Ok(app) = serde_json::from_str::<App>(&serialized) {
            *self = app;
        }

        Ok(())
    }

    fn save(&self, file_path: &str) -> io::Result<()> {
        let serialized = serde_json::to_string(self).unwrap();

        let mut file = match fs::File::create(file_path) {
            Ok(f) => f,
            Err(e) => {
                panic!("{}", e)
            }
        };

        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn has_data(&self) -> bool {
        !self.entries.is_empty()
    }

    fn add(&mut self, entry: Vec<&str>) {
        //self.entries.push(Entry {
        //    key: String::from(entry[0]),
        //    value: String::from(entry[1]),
        //});

        self.entries
            .push((String::from(entry[0]), String::from(entry[1])));
    }

    fn load_files(&mut self, files: &[path::PathBuf]) -> io::Result<()> {
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
        self.number = if number < 0 || number as usize > self.entries.len() {
            self.entries.len()
        } else {
            number as usize
        };

        self.deck = Vec::new();
        let mut candidates: Vec<usize> = (0..self.entries.len()).collect();

        let mut rng = thread_rng();

        while self.deck.len() < self.number {
            let i = rng.gen_range(0..candidates.len());
            self.deck.push(candidates[i]);
            candidates.remove(i);
        }

        self.target_idx = 0;
    }

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
        //&self.entries.get(self.deck[self.target_idx]).key
        &self.entries[self.deck[self.target_idx]].0
    }

    fn get_answer(&self) -> &String {
        //&self.entries.get(self.deck[self.target_idx]).value
        &self.entries[self.deck[self.target_idx]].1
    }

    fn get_progress_percent(&self) -> u16 {
        (100 * (self.target_idx + 1) / self.number) as u16
    }

    fn study_again(&mut self) {
        self.deck.push(self.deck[self.target_idx]);
        self.number += 1;
    }

    fn is_question_mode(&self) -> bool {
        self.mode == Mode::Question
    }

    fn is_answer_mode(&self) -> bool {
        self.mode == Mode::Answer
    }

    fn is_doen_mode(&self) -> bool {
        self.mode == Mode::Done
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Mode {
    Question,
    Answer,
    Done,
}

struct View {
    terminal: Terminal<TermionBackend<RawTerminal<io::Stdout>>>,
}

impl View {
    fn new(stdout: RawTerminal<io::Stdout>) -> Self {
        View {
            // TODO error handling
            terminal: Terminal::new(TermionBackend::new(stdout)).unwrap(),
        }
    }

    fn display(&mut self, app: &App) -> io::Result<()> {
        // ターミナルサイズが変わった場合に備えて
        self.terminal.clear()?;

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

            let answer = if app.is_question_mode() {
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

    //#[structopt(name = "FILE", parse(from_os_str), default_value = "~/.marusora")]
    #[structopt(short, long, default_value = "marusora.save")]
    //save: path::PathBuf,
    save: String,

    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<path::PathBuf>,
}

// https://github.com/fdehau/tui-rs/issues/177
fn setup_panic() -> io::Result<()> {
    let raw_handle = io::stdout().into_raw_mode()?;

    raw_handle.suspend_raw_mode()?;

    //let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        raw_handle
            .suspend_raw_mode()
            .unwrap_or_else(|e| log::error!("Could not suspend raw mode: {}", e));
        //default_hook(info);
        better_panic::Settings::new().create_panic_handler()(info);
    }));

    Ok(())
}

fn main() -> io::Result<()> {
    setup_panic()?;

    let opt = Opt::from_args();

    let mut app = App::new();

    if let Ok(mut f) = fs::File::open(&opt.save) {
        print!("Load saved file, '{}'? [Yn] ", opt.save);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input == "\n" || input == "Y\n" || input == "y\n" {
            app.restore(&mut f)?;
        }
    }

    // TODO refactor
    if !app.has_data() {
        app.load_files(&opt.files)?;
        app.prepare(opt.number);
    }

    let stdout = io::stdout().into_raw_mode()?;
    let mut view = View::new(stdout);

    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut bytes = stdin.bytes();

    loop {
        view.display(&app)?;

        match bytes.next().unwrap().unwrap() {
            b'q' => {
                app.save(&opt.save)?;
                break;
            }
            b'r' => {
                if app.is_answer_mode() {
                    app.study_again();
                }
            }
            _ => {}
        }

        app.update();

        if app.is_doen_mode() {
            break;
        }
    }

    Ok(())
}
