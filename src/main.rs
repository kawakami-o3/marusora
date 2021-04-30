use std::*;
use std::io::prelude::*;
use rand::prelude::*;

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
        Some(word)
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

    app.prepare();

    loop {
        match app.next() {
            None => {
                break;
            }
            Some(w) => {
                println!("{:?}", w);
            }
        }
    }
}

