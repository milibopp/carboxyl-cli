extern crate carboxyl;
extern crate carboxyl_cli;

use carboxyl::Stream;
use carboxyl_cli::{run, Quit, Input};

fn echo(inputs: Stream<Input>) -> (Stream<String>, Stream<Quit>) {
    (inputs.filter_map(Input::line),
     inputs.filter_map(Input::end))
}

fn main() {
    use std::io::{stdin, stdout};

    run(stdin(), stdout(), echo);
}
