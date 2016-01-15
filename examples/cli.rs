extern crate carboxyl;
extern crate cli_driver;

use std::io::{ stdin, stdout };
use carboxyl::Stream;
use cli_driver::{ run, Quit, Input, ReadDriver, WriteDriver };

fn program(inputs: Stream<Input>) -> (Stream<String>, Stream<Quit>) {
    let outputs = inputs.filter_map(Input::line);
    let quit = inputs.filter_map(Input::end);
    (outputs, quit)
}

fn main() {
    let stdin_driver = ReadDriver::new(stdin());
    let stdout_driver = WriteDriver::new(stdout());
    run(stdin_driver, stdout_driver, program);
}
