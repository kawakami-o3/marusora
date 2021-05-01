use std::*;
use std::io::prelude::*;
use rand::prelude::*;
use termion::raw::IntoRawMode;
use tui::{
    Terminal,
    backend::TermionBackend,
    widgets::{Block, Borders},
    widgets::canvas::Canvas,
    layout::{Layout, Constraint, Direction},
    style::Color,
};


//#[derive(Copy)]
//struct pair {
//}

#[derive(Debug)]
struct App {
    words: Vec<(String, String)>,

    //deck: Vec<(String, String)>,
    deck: Vec<usize>,
}

impl App {
    fn new() -> App {
        App {
            words: Vec::new(),
            deck: Vec::new(),
        }
    }

    fn add(&mut self, entry: Vec<&str>) {
        self.words.push((String::from(entry[0]), String::from(entry[1])));
    }

    fn prepare(&mut self) {
        self.deck = (0..self.words.len()).collect();
    }

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
}


struct View {
    terminal: Terminal<TermionBackend<termion::raw::RawTerminal<io::Stdout>>>,
}

impl View {
    fn new() -> View {

        let stdout = io::stdout().into_raw_mode().unwrap();
        let backend = TermionBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        View {
            terminal,
        }
    }

    fn clear(&mut self) -> io::Result<()> {
        self.terminal.clear()
    }

    fn draw(&mut self, word: &(String, String)) -> io::Result<()> {

        self.terminal.draw(|f| {

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                //.margin(1)
                .constraints(
                    [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                    //Constraint::Percentage(10)
                    ].as_ref()
                ).split(f.size());

            let block = Block::default()
                .title("Block")
                .borders(Borders::ALL);

            let canvas = Canvas::default()
                .block(block)
                .paint(|ctx| {
                    ctx.print(0.0, 0.0, "0", Color::White);
                    ctx.print(1.0, 1.0, "1", Color::White);
                    ctx.print(1.0, 3.0, "2\nh", Color::White);
                })
                .x_bounds([0.0,5.0])
                .y_bounds([-5.0,5.0]);


            //f.render_widget(block, chunks[0]);
            f.render_widget(canvas, chunks[0]);

            let block = Block::default()
                .title("Block 2")
                .borders(Borders::ALL);

            f.render_widget(block, chunks[1]);
        })
    }

    fn draw_all(&mut self, word: &(String, String)) -> io::Result<()> {
        Ok(())
    }
}




fn main() {
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


    let mut view = View::new();
    view.clear();

    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut bytes = stdin.bytes();

    app.prepare();
    loop {
        let w = app.next();
        if w == None {
            break;
        }
        let w = w.unwrap();

        view.draw(&w).unwrap();
        let b = bytes.next().unwrap().unwrap();
        match b {
            b'q' => {
                break;
            }
            _ => {
                view.draw_all(&w).unwrap();
            }
        }

    }
}

