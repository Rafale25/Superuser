//#![allow(dead_code)]

use manual::ManualBoard;
use notan::draw::*;
use notan::log;
use notan::prelude::*;
//use notan::text::*;

mod manual;

#[derive(Default)]
struct Console {
    lines: Vec<String>,
    prompt: String,
}

impl Console {
    pub fn get_full_prompt(&self) -> String {
        format!("root@192.168.0.7> {}", self.prompt)
    }
    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }
}

#[derive(AppState)]
pub struct State {
    font: Font,
    console: Console,
    time: f32,
    manual_board: ManualBoard,
    previous_mouse_pos: (i32, i32),
}

pub fn setup(app: &mut App, gfx: &mut Graphics) -> State {
    app.window().set_size(1280, 720);
    let font = gfx
        .create_font(include_bytes!("ShareTechMono-Regular.ttf"))
        .unwrap();

    // BUG: set_size not being applied directly when not in html page, making width and height wrong
    // println!("{} {}", app.window().width(), app.window().height());
    println!("{} {}", gfx.size().0 , gfx.size().1 );

    State {
        font,
        time: 0.0,
        console: Default::default(),
        manual_board: ManualBoard::new(app.window().width(), app.window().height()),
        previous_mouse_pos: (0, 0),
    }
}

pub fn event(app: &mut App, state: &mut State, event: Event) {
    match event {
        Event::ReceivedCharacter(c) if c != '\u{7f}' => {
            if state.console.prompt.len() < 70 {
                state.console.prompt.push(c);
            }
        }
        Event::KeyDown { key } if key == KeyCode::Back => {
            state.console.prompt.pop();
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
        state.console.prompt.clear();
    }
    state.time += app.system_timer.delta_f32();
}

pub fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::WHITE);

    let (width, height) = (gfx.size().0 as f32, gfx.size().1 as f32);


    // manual background
    draw.rect((width / 2.0, height * 0.25), (width / 2.0, height * 0.75))
        .color(Color::GRAY);

    for manual in state.manual_board.manuals.iter() {
        draw.rect(
            (manual.pos.0 as f32, manual.pos.1 as f32),
            (manual.size.0 as f32, manual.size.1 as f32),
        )
        .color(Color::RED);
    }

    // graph background
    draw.rect((width / 2.0, 0.0), (width / 2.0, height * 0.25))
        .color(Color::WHITE);

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
