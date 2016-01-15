extern crate carboxyl;

use std::io::{ Read, Write };
use carboxyl::Stream;
use drivers::{ ReadDriver, WriteDriver };

mod drivers;
#[cfg(test)]
mod sync;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Line(String),
    End,
}

impl Input {
    pub fn line(self) -> Option<String> {
        if let Input::Line(text) = self { Some(text) }
        else { None }
    }

    pub fn end(self) -> Option<Quit> {
        if let Input::End = self { Some(Quit) }
        else { None }
    }
}


pub fn run<R, W, P>(reader: R, writer: W, program: P)
    where R: 'static + Send + Read,
          W: 'static + Send + Write,
          P: Fn(Stream<Input>) -> (Stream<String>, Stream<Quit>)
{
    let read_driver = ReadDriver::new(reader);
    let write_driver = WriteDriver::new(writer);
    let (outputs, quit) = program(read_driver.stream());
    let mut quit_stream = quit.events();
    write_driver.drive(outputs);
    read_driver.drive();
    quit_stream.next();
}


#[cfg(test)]
mod test {
    use std::thread;
    use std::io::Cursor;
    use std::sync::{ Arc, Mutex };
    use std::time::Duration;
    use carboxyl::Stream;

    use super::*;
    use ::sync::{ SyncWriter, check_timeout };

    #[test]
    fn runs_echo_application() {
        let sample = b"abc\n";
        let writer = SyncWriter::new();
        run(
            Cursor::new(sample),
            writer.clone(),
            |inputs| (inputs.filter_map(Input::line), inputs.filter_map(Input::end))
        );
        check_timeout(|| &(*writer.contents().unwrap())[..] == sample, 100);
    }

    #[test]
    fn runs_forever_without_end_of_input() {
        let flag = Arc::new(Mutex::new(false));
        thread::spawn({
            let flag = flag.clone();
            move || {
                run(
                    Cursor::new(b""),
                    SyncWriter::new(),
                    |_| (Stream::never(), Stream::never())
                );
                *flag.lock().unwrap() = true;
            }
        });
        thread::sleep(Duration::from_millis(5));
        assert!(!*flag.lock().unwrap());
    }
}
