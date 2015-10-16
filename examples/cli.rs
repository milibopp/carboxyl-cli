extern crate carboxyl;
extern crate cli_driver;

use std::io::stdout;
use carboxyl::Sink;
use cli_driver::WriteDriver;

fn main() {
    let sink = Sink::new();
    WriteDriver::new(stdout()).drive(sink.stream());
    sink.send("First line\n".to_string());
    sink.send("Second line\n".to_string());
}
