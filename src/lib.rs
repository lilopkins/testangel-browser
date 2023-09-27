use std::sync::Mutex;

use lazy_static::lazy_static;
use testangel_engine::*;

#[derive(Default)]
struct State;

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, Mutex<State>>> = Mutex::new(Engine::new("Browser Automation", env!("CARGO_PKG_VERSION")));
}
