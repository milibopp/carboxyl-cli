extern crate carboxyl;

use std::io::{ Read, Write, BufReader, BufRead };
use std::thread;
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
                self.writer.flush().unwrap();
            }
        });
    }
}


pub struct ReadDriver<R> {
    sink: Sink<String>,
    reader: R,
}

impl<R> ReadDriver<R> {
    pub fn new(reader: R) -> ReadDriver<R> {
        ReadDriver { reader: reader, sink: Sink::new() }
    }
}

impl<R: 'static + Send + Read> ReadDriver<R> {
    pub fn stream(&self) -> Stream<String> {
        self.sink.stream()
    }

    pub fn drive(self) {
        self.sink.feed_async(
            BufReader::new(self.reader)
                .lines()
                .filter_map(|r| r.ok())
        );
    }
}


#[cfg(test)]
mod test {
    use std::thread;
    use std::io::Cursor;
    use std::time::Duration;
    use carboxyl::Sink;

    use super::*;
    use ::sync::SyncWriter;

    const SAMPLE: &'static str = "abc";

    fn check_timeout<F: FnMut() -> bool>(mut predicate: F, retries: u32) {
        for _ in 0..retries {
            thread::sleep(Duration::from_millis(1));
            if predicate() { return; }
        }
        panic!()
    }

    #[test]
    fn writes_events_from_output_stream() {
        let writer = SyncWriter::new();
        let sink = Sink::new();
        WriteDriver::new(writer.clone())
            .drive(sink.stream());
        sink.send(SAMPLE.to_string());
        check_timeout(|| &(*writer.contents().unwrap())[..] == SAMPLE.as_bytes(), 100);
    }

    #[test]
    fn reads_into_input_stream() {
        let reader = Cursor::new("abc\n".to_string().into_bytes());
        let driver = ReadDriver::new(reader);
        let inputs = driver.stream();
        let mut events = inputs.events();
        driver.drive();
        assert_eq!(events.next(), Some("abc".to_string()));
    }
}
