extern crate carboxyl;
extern crate carboxyl_cli;

use carboxyl::Stream;
use carboxyl_cli::{run, Quit, Input};

fn analyse_input(line: String) -> String {
    match line.parse::<f64>() {
        Ok(number) => format!("got a number: {}", number),
        Err(_) => "not a number :(".to_string()
    }
}

fn echo(inputs: Stream<Input>) -> (Stream<String>, Stream<Quit>) {
    let lines = inputs.filter_map(Input::line);
    let quit = inputs.filter_map(Input::end);
    let output = lines.map(analyse_input);
    (output, quit)
}

fn main() {
    use std::io::{stdin, stdout};

    run(stdin(), stdout(), echo);
}
