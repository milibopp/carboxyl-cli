extern crate carboxyl;

use std::io::{ Read, Write, BufReader, BufRead };
use std::{ iter, thread };
use carboxyl::{ Sink, Stream };

#[cfg(test)]
mod sync;


pub struct WriteDriver<W> {
    writer: W,
}

impl<W> WriteDriver<W> {
    pub fn new(writer: W) -> WriteDriver<W> {
        WriteDriver { writer: writer }
    }
}

impl<W: 'static + Send + Write> WriteDriver<W> {
    pub fn drive(mut self, output: Stream<String>) {
        let events = output.events();
        thread::spawn(move || {
            for text in events {
                self.writer.write(text.as_bytes()).unwrap();
                self.writer.write("\n".as_bytes()).unwrap();
                self.writer.flush().unwrap();
            }
        });
    }
}

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


pub struct ReadDriver<R> {
    sink: Sink<Input>,
    reader: R,
}

impl<R> ReadDriver<R> {
    pub fn new(reader: R) -> ReadDriver<R> {
        ReadDriver { reader: reader, sink: Sink::new() }
    }
}

impl<R: 'static + Send + Read> ReadDriver<R> {
    pub fn stream(&self) -> Stream<Input> {
        self.sink.stream()
    }

    pub fn drive(self) {
        self.sink.feed_async(
            BufReader::new(self.reader)
                .lines()
                .filter_map(|r| r.ok())
                .map(Input::Line)
                .chain(iter::once(Input::End))
        );
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
    use std::time::Duration;
    use carboxyl::{ Sink, Stream };

    use super::*;
    use ::sync::SyncWriter;
    use std::sync::{ Arc, Mutex };

    const SAMPLE: &'static str = "abc";

    fn check_timeout<F: FnMut() -> bool>(mut predicate: F, retries: u32) {
        for _ in 0..retries {
            thread::sleep(Duration::from_millis(1));
            if predicate() { return; }
        }
        panic!("check timed out")
    }

    #[test]
    fn writes_events_from_output_stream() {
        let writer = SyncWriter::new();
        let sink = Sink::new();
        WriteDriver::new(writer.clone())
            .drive(sink.stream());
        sink.send(SAMPLE.to_string());
        let expected = {
            let mut s = SAMPLE.to_string();
            s.push('\n');
            s
        };
        check_timeout(|| &(*writer.contents().unwrap())[..] == expected.as_bytes(), 100);
    }

    #[test]
    fn reads_into_input_stream() {
        let reader = Cursor::new("abc\n".to_string().into_bytes());
        let driver = ReadDriver::new(reader);
        let inputs = driver.stream();
        let mut events = inputs.events();
        driver.drive();
        assert_eq!(events.next(), Some(Input::Line("abc".to_string())));
        assert_eq!(events.next(), Some(Input::End));
    }

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
