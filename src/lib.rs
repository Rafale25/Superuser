use std::collections::HashMap;

use crate::manual::Manual;
use crate::manual::ManualBoard;
use notan::draw::*;
use notan::prelude::*;
use notan::random::rand::seq::SliceRandom;
use notan::random::rand::thread_rng;
use serde::Deserialize;

mod manual;

/*
clear: clear terminal
cat: print content of file
ls: see files/folder in current folder
ssh [ip]: connect to a host
dc: disconnect from host
print: print a manual file
cd: navigate files (low priority because needs filesystem to work)

hack [ip]: start puzzle associated to host
*/

#[derive(Deserialize)]
enum File {
    Manual { id: String },
}

#[derive(Deserialize)]
struct Host {
    fs: HashMap<String, File>,
    hacked: bool,
    position: (f32, f32),
    puzzle_kind: String,
}

#[derive(Deserialize, Default)]
struct Network {
    hosts: HashMap<String, Host>,
}

#[derive(Default)]
enum ConsoleState {
    #[default]
    AwaitingCommand,
    AwaitingAnswer {
        answer: String,
        host: String,
    },
}

#[derive(Default)]
struct Console {
    lines: Vec<String>,
    prompt: String,
    state: ConsoleState,
    current_host: String,
}

impl Console {
    pub fn get_full_prompt(&self) -> String {
        match &self.state {
            ConsoleState::AwaitingCommand => format!("root@{}> {}", self.current_host, self.prompt),
            ConsoleState::AwaitingAnswer { answer: _, host } => {
                format!("hacking {}> {}", host, self.prompt)
            }
        }
    }
    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }
}

#[derive(Deserialize)]
struct Puzzle {
    //manual_name: String,
    answers: Vec<(String, String)>,
}

#[derive(AppState)]
pub struct State {
    font: Font,
    console: Console,
    time: f32,
    manual_board: ManualBoard,
    previous_mouse_pos: (i32, i32),
    network: Network,
    manuals: HashMap<String, Manual>,
    puzzles: HashMap<String, Puzzle>,
}

impl State {
    fn handle_command(&mut self) {
        let prompt = &self.console.prompt;
        let mut args = prompt.split_whitespace();
        let Some(cmd) = args.next() else {return};
        match cmd {
            "clear" => self.console.lines.clear(),
            "ls" => {
                for key in self.network.hosts[&self.console.current_host].fs.keys() {
                    self.console.add_line(key.clone());
                }
            }
            "print" => {
                let Some(arg) = args.next() else {self.console.add_line("Filename expected".to_string());return};
                let Some(File::Manual { id }) = self.network.hosts[&self.console.current_host].fs.get(arg) else {self.console.add_line("Not a manual page".to_string());return};
                // TODO: actually print the manual
                self.manual_board.print_manual(self.manuals[id].clone());
            }
            "ssh" => {
                if let Some(arg) = args.next() {
                    if self.network.hosts.contains_key(arg) {
                        if self.network.hosts[arg].hacked {
                            self.console.current_host = arg.to_owned();
                        } else {
                            self.console
                                .add_line(format!("Connection refused: access denied"));
                        }
                    } else {
                        self.console
                            .add_line(format!("Connection refused: no host found"));
                    }
                } else {
                    self.console.add_line(format!("Expected address"));
                }
            }
            "hack" => {
                if let Some(arg) = args.next() {
                    if self.network.hosts.contains_key(arg) {
                        let host = &self.network.hosts[arg];
                        if !host.hacked {
                            //TODO puzzle
                            let puzzle = &self.puzzles[&host.puzzle_kind];
                            let (query, answer) = puzzle.answers.choose(&mut thread_rng()).unwrap();
                            self.console.state = ConsoleState::AwaitingAnswer {
                                answer: answer.to_string(),
                                host: arg.to_owned(),
                            };
                            self.console.add_line(query.to_owned());
                            //self.network.hosts.get_mut(arg).unwrap().hacked = true;
                        } else {
                            self.console.add_line(format!("Error: already root"));
                        }
                    } else {
                        self.console
                            .add_line(format!("Connection refused: no host found"));
                    }
                } else {
                    self.console.add_line(format!("Expected address"));
                }
            }
            "dc" => self.console.current_host = "127.0.0.1".to_string(),
            _ => self.console.add_line(format!("Unknown command: {cmd}")),
        }
    }

    fn handle_console_prompt(&mut self) {
        match &self.console.state {
            ConsoleState::AwaitingCommand => self.handle_command(),
            ConsoleState::AwaitingAnswer { answer, host } => {
                if answer == &self.console.prompt {
                    self.network.hosts.get_mut(host).unwrap().hacked = true;
                    if self.network.hosts.values().all(|x| x.hacked) {
                        self.console.add_line("Congrats, you win".to_string());
                    }
                } else {
                    self.console.add_line("Hacking attempt failed".to_string());
                }
                self.console.state = ConsoleState::AwaitingCommand;
            }
        };
    }
}

pub fn setup(app: &mut App, gfx: &mut Graphics) -> State {
    app.window().set_size(1280, 720);
    let font = gfx
        .create_font(include_bytes!("ShareTechMono-Regular.ttf"))
        .unwrap();

    // BUG: set_size not being applied directly when not in html page, making width and height wrong
    // println!("{} {}", app.window().width(), app.window().height());
    // println!("{} {}", gfx.size().0, gfx.size().1);

    let mut state = State {
        font,
        time: 0.0,
        console: Default::default(),
        manual_board: ManualBoard::new(app.window().width(), app.window().height()),
        previous_mouse_pos: (0, 0),
        network: serde_json::from_str(include_str!("network_desc.json"))
            .map_err(|err| {
                notan::log::error!("{err}");
                err
            })
            .unwrap_or_default(),
        puzzles: serde_json::from_str(include_str!("puzzles.json")).unwrap(),
        manuals: HashMap::new(),
    };
    state.console.current_host = "127.0.0.1".to_string();
    state.manuals.insert(
        "commands".to_string(),
        Manual::new(
            // setup manuals (move this in its own function or something)
            (0, 0),
            (300, 200),
            include_str!("manuals/commands.txt").to_string(),
            (209, 190, 161),
        ),
    );
    state.manuals.insert(
        "hack".to_string(),
        Manual::new(
            // setup manuals (move this in its own function or something)
            (0, 0),
            (250, 100),
            include_str!("manuals/hack.txt").to_string(),
            (220, 252, 199),
        ),
    );
    state.manuals.insert(
        "puzzle1".to_string(),
        Manual::new(
            (0, 0),
            (500, 250),
            include_str!("manuals/puzzle1.txt").to_string(),
            (220, 152, 199),
        ),
    );

    state
        .manual_board
        .print_manual(state.manuals["commands"].clone());
    // state
    //     .manual_board
    //     .print_manual(state.manuals["hack"].clone());
    // state
    //     .manual_board
    //     .print_manual(state.manuals["puzzle1"].clone());

    state
}

pub fn event(app: &mut App, state: &mut State, event: Event) {
    match event {
        Event::ReceivedCharacter(c) if !c.is_ascii_control() => {
            if state.console.prompt.len() < 70 {
                state.console.prompt.push(c);
            }
        }
        Event::KeyDown { key } if key == KeyCode::Back => {
            state.console.prompt.pop();
        }
        Event::MouseDown { button: _, x, y } => {
            state
                .manual_board
                .mouse_drag((x, y), state.previous_mouse_pos);
        }
        Event::MouseMove { x, y } => {
            if app.mouse.is_down(MouseButton::Left) {
                state
                    .manual_board
                    .mouse_drag((x, y), state.previous_mouse_pos);
            }
            state.previous_mouse_pos = (x, y);
        }
        _ => {}
    }
}

pub fn update(app: &mut App, state: &mut State) {
    if app.keyboard.was_pressed(KeyCode::Return) {
        state.console.add_line(state.console.get_full_prompt());
        state.handle_console_prompt();
        state.console.prompt.clear();
    }
    state.time += app.system_timer.delta_f32();

    state
        .manual_board
        .update_printer(app.system_timer.delta_f32());
}

pub fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::WHITE);

    let (width, height) = (gfx.size().0 as f32, gfx.size().1 as f32);

    // manualboard background
    draw.rect((width / 2.0, height * 0.25), (width / 2.0, height * 0.75))
        .color(Color::GRAY);

    for manual in state.manual_board.manuals.iter().rev() {
        draw.rect(
            (manual.pos.0 as f32, manual.pos.1 as f32),
            (manual.size.0 as f32, manual.size.1 as f32),
        )
        .color(Color {
            r: manual.bg_color.0 as f32 / 255.0,
            g: manual.bg_color.1 as f32 / 255.0,
            b: manual.bg_color.2 as f32 / 255.0,
            a: 1.0,
        });

        draw.text(&state.font, &manual.text)
            .color(Color::BLACK)
            // .position(10.0, 5.0 + i_f * line_height)
            .position(manual.pos.0 as f32 + 6.0, manual.pos.1 as f32 + 6.0)
            .size(14.0);
    }

    // draw printing manual
    for mp in state.manual_board.manual_printing.iter().rev() {
        draw.rect(
            (mp.manual.pos.0 as f32, mp.manual.pos.1 as f32),
            (mp.manual.size.0 as f32, mp.manual.size.1 as f32),
        )
        .color(Color {
            r: mp.manual.bg_color.0 as f32 / 255.0,
            g: mp.manual.bg_color.1 as f32 / 255.0,
            b: mp.manual.bg_color.2 as f32 / 255.0,
            a: 1.0,
        });

        draw.text(&state.font, &mp.manual.text)
            .color(Color::BLACK)
            // .position(10.0, 5.0 + i_f * line_height)
            .position(mp.manual.pos.0 as f32 + 6.0, mp.manual.pos.1 as f32 + 6.0)
            .size(14.0);
    }

    // graph background
    draw.rect((width / 2.0, 0.0), (width / 2.0, height * 0.25))
        .color(Color::from_hex(0x303030ff));

    for (addr, host) in state.network.hosts.iter() {
        let color = if host.hacked {
            Color::GREEN
        } else {
            Color::RED
        };
        draw.circle(10.0)
            .position(width / 2.0 + host.position.0, host.position.1)
            .color(color);
        draw.text(&state.font, &addr)
            .position(width / 2.0 + host.position.0 + 13.0, host.position.1 - 9.0);
    }

    // terminal background
    draw.rect((0.0, 0.0), (width / 2.0, height))
        .color(Color::BLACK);

    let line_height = 16.0;
    let font_size = 21.0;
    let symbol_size = 10.1;
    let max_line_count = (gfx.size().1 - 5) / (line_height as i32);
    while state.console.lines.len() >= max_line_count as usize {
        state.console.lines.remove(0);
    }
    for (i, line) in state.console.lines.iter().enumerate() {
        let i_f = i as f32;
        draw.text(&state.font, line)
            .color(Color::GREEN)
            .position(10.0, 5.0 + i_f * line_height)
            .size(font_size);
    }
    let full_prompt = state.console.get_full_prompt();
    if (state.time * 2.0) as u64 % 2 == 0 {
        draw.rect(
            (
                10.0 + (full_prompt.len() as f32) * symbol_size,
                5.0 + (state.console.lines.len() as f32) * line_height + 2.0,
            ),
            (symbol_size, line_height),
        )
        .color(Color::GREEN);
    }
    draw.text(&state.font, &full_prompt)
        .color(Color::GREEN)
        .position(10.0, 5.0 + (state.console.lines.len() as f32) * line_height)
        .size(font_size);

    gfx.render(&draw);
}
