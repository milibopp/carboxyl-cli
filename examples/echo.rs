extern crate carboxyl;
extern crate carboxyl_cli;

use std::io::{ stdin, stdout };
use carboxyl::Stream;
use carboxyl_cli::{ run, Quit, Input };

fn echo(inputs: Stream<Input>) -> (Stream<String>, Stream<Quit>) {
    (inputs.filter_map(Input::line), inputs.filter_map(Input::end))
}

fn main() {
    run(stdin(), stdout(), echo);
}
