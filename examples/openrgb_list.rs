//! Lists the controllers an OpenRGB server reports (read-only): `cargo run --example openrgb_list`.
fn main() {
    match coreglow::openrgb::Client::connect("localhost", coreglow::openrgb::DEFAULT_PORT) {
        Ok(c) => {
            for ctl in &c.controllers {
                println!("{}: {} ({} LEDs)", ctl.index, ctl.name, ctl.leds);
            }
        }
        Err(e) => eprintln!("{e}"),
    }
}
