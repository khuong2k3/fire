use clap::Parser;
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::{
    io::{self, stdin, stdout, Stdout, Write},
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
    usize,
};

const FIRE_STRING: [char; 52] = [
    ' ', ',', ';', '+', 'l', 't', 'g', 't', 'i', '!', 'l', 'I', '?', '/', '\\', '|', ')', '(', '1',
    '}', '{', ']', '[', 'r', 'c', 'v', 'z', 'j', 'f', 't', 'J', 'U', 'O', 'Q', 'o', 'c', 'x', 'f',
    'X', 'h', 'q', 'w', 'W', 'B', '8', '&', '%', '$', '#', '@', '"', ';',
];

const AMP: usize = 20; // < FIRE_STRING.len()

struct Buffer {
    bufs: [Vec<usize>; 2],
    current: usize,
    width: usize,
    height: usize,
}

impl Buffer {
    fn get<'a>(&'a self) -> &'a Vec<usize> {
        &self.bufs[self.current]
    }

    fn next(&mut self) {
        self.current = (self.current + 1) % 2;
    }

    fn diff(&self, x: usize, y: usize) -> bool {
        let idx = y * self.width + x;

        self.bufs[0][idx] != self.bufs[1][idx]
    }

    fn check_resized(&self, new_w: usize, new_h: usize) -> bool {
        self.width != new_w || self.height != new_h
    }

    fn resize(&mut self, new_w: usize, new_h: usize) {
        if self.width < new_w {
            self.width = new_w;
        }

        if self.height < new_h {
            self.height = new_h;
        }

        if self.width >= new_w || self.height >= new_h {
            let size = new_w * new_h;
            self.width = new_w;
            self.height = new_h;

            self.bufs = [vec![0; size], vec![0; size]];
        }
    }

    fn genrate(&mut self) {
        for _i in 0..self.width {
            let index = rand::random::<usize>() % self.width;

            self.bufs[self.current][(self.height - 1) * self.width + index] =
                AMP + rand::random::<usize>() % (FIRE_STRING.len() - AMP);
        }

        for _i in 0..self.width {
            let index = rand::random::<usize>() % self.width;
            self.bufs[self.current][(self.height - 1) * self.width + index] = 0;
        }
    }

    fn update(&mut self) {
        self.genrate();

        let size = self.width * (self.height - 1) - 1;
        let bufs = &mut self.bufs;
        let new_idx = (self.current + 1) % 2;

        for i in 0..size {
            let new_char_val = (bufs[new_idx][i]
                + bufs[new_idx][i + 1]
                + bufs[new_idx][i + self.width]
                + bufs[new_idx][i + self.width + 1])
                / 4;

            bufs[self.current][i] = new_char_val;
        }

        let new_char_val =
            (bufs[new_idx][size] + bufs[new_idx][size + 1] + bufs[new_idx][size + self.width]) / 3;

        bufs[self.current][size] = new_char_val;
    }

    fn present(&self, stdout: &mut Stdout, force: bool) -> io::Result<()> {
        let content = self.get();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if force || self.diff(x, y) {
                    stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                    if content[idx] > 5 {
                        stdout
                            .queue(style::PrintStyledContent(FIRE_STRING[content[idx]].blue()))?;
                    } else {
                        stdout.queue(style::PrintStyledContent(FIRE_STRING[content[idx]].red()))?;
                    }
                }
            }
        }

        // stdout.queue(cursor::MoveTo(0, 0))?;

        // let formted = content.iter().map(|c| style::PrintStyledContent(FIRE_STRING[*c].blue()));
        // let formted = String::from_iter(content.iter().map(|c| {
        //     if *c > 5 {
        //         FIRE_STRING[*c].with(style::Color::Red)
        //     } else {
        //         FIRE_STRING[*c].with(style::Color::Blue)
        //     }
        //     .content()
        //     .to_owned()
        // }));

        // stdout.queue(style::Print(formted))?;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if force || self.diff(x, y) {
                    stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                    if content[idx] > 5 {
                        stdout
                            .queue(style::PrintStyledContent(FIRE_STRING[content[idx]].blue()))?;
                    } else {
                        stdout.queue(style::PrintStyledContent(FIRE_STRING[content[idx]].red()))?;
                    }
                }
            }
        }

        stdout.flush()?;

        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Fire duration
    #[arg(short, long, default_value_t = 5)]
    duration: u64,
}

fn main() {
    let args = Args::parse();
    let mut io_stdout = io::stdout();

    let (width, height) = crossterm::terminal::size().unwrap();
    let width = width as usize;
    let height = height as usize;

    let mut buffer = Buffer {
        current: 0,
        bufs: [vec![0; width * height], vec![0; width * height]],
        width,
        height,
    };

    buffer.update();

    io_stdout
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap()
        .queue(cursor::Hide)
        .unwrap();

    let (sx, rx) = mpsc::channel();
    let sx_ctrl = sx.clone();
    ctrlc::set_handler(move || {
        sx_ctrl.send(()).unwrap();
    }).expect("Error setting Ctrl-C handler");

    thread::spawn(move || loop {
        let (new_w, new_h) = crossterm::terminal::size().unwrap();
        let new_w = new_w as usize;
        let new_h = new_h as usize;
        let resized = buffer.check_resized(new_w, new_h);

        if resized {
            buffer.resize(new_w, new_h);
        }

        buffer.update();
        buffer.present(&mut io_stdout, resized).unwrap();

        buffer.next();
        if let Ok(()) = rx.try_recv() {
            break;
        }
        sleep(Duration::from_millis(30));
    });
    let stdin = stdin();
    let _ = stdin.lock();

    sleep(Duration::from_secs(args.duration));
    sx.send(()).unwrap();
    stdout().queue(cursor::Show).unwrap();
}
