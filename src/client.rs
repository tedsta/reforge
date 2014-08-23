#![crate_name = "spacegame"]
#![desc = "spacegame awesome mmo"]
#![crate_type = "bin"]

extern crate native;
extern crate rsfml;

use rsfml::graphics::{RenderWindow, Color};
use rsfml::window::{VideoMode, ContextSettings, event, keyboard, Close};

#[cfg(target_os="macos")]
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}


fn main () -> () {
    // Create the window of the application
    let setting: ContextSettings = ContextSettings::default();
    let mut window: RenderWindow =
        match RenderWindow::new(VideoMode::new_init(800, 600, 32),
                                "spacegame",
                                Close,
                                &setting) {
            Some(window) => window,
            None => fail!("Cannot create a new Render Window.")
        };

    while window.is_open() {
        loop {
            match window.poll_event() {
                event::Closed => window.close(),
                event::KeyPressed{code, ..} => match code {
                    keyboard::Escape      => {window.close(); break},
                    _                     => {}
                } ,
                event::NoEvent => break,
                _ => {}
            }
        }
        
        // Clear the screen
        window.clear(&Color::black());

        // Display things on screen
        window.display()
    }

}