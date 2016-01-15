extern crate carboxyl;
extern crate cli_driver;

use std::io::{ stdin, stdout };
use carboxyl::Stream;
use cli_driver::{ run, Quit, Input };

fn program(inputs: Stream<Input>) -> (Stream<String>, Stream<Quit>) {
    let outputs = inputs.filter_map(Input::line);
    let quit = inputs.filter_map(Input::end);
    (outputs, quit)
}

fn main() {
    run(stdin(), stdout(), program);
}
