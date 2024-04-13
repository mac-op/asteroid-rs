use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = init()
        .size(640, 480)
        .title("Hello World")
        .build();

    rl.set_target_fps(60);
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
        d.draw_text("HELLO", 12, 12, 20, Color::RED);
    }
}
