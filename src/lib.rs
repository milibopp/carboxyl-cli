extern crate carboxyl;

use std::io::Write;
use std::thread;
use carboxyl::Stream;

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
    pub fn drive(mut self, output: Stream<String>) -> Stream<String> {
        let events = output.events();
        thread::spawn(move || {
            for text in events {
                self.writer.write(text.as_bytes()).unwrap();
                self.writer.flush().unwrap();
            }
        });
        Stream::never()
    }
}


#[cfg(test)]
mod test {
    use std::thread;
    use carboxyl::Sink;

    use super::*;
    use ::sync::SyncWriter;

    const SAMPLE: &'static str = "abc";

    fn check_timeout<F: FnMut() -> bool>(mut predicate: F, retries: u32) {
        for _ in 0..retries {
            thread::sleep_ms(1);
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
}
