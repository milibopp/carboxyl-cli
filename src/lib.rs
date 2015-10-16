extern crate carboxyl;

use std::io::Write;
use std::thread;
use carboxyl::Stream;

#[cfg(test)]
mod sync;


pub struct IoDriver<W> {
    writer: W,
}

impl<W> IoDriver<W> {
    pub fn new(writer: W) -> IoDriver<W> {
        IoDriver { writer: writer }
    }
}

impl<W: 'static + Send + Write> IoDriver<W> {
    pub fn drive(mut self, output: Stream<String>) -> Stream<String> {
        thread::spawn(move || {
            for text in output.events() {
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

    #[test]
    fn writes_events_from_output_stream() {
        let writer = SyncWriter::new(vec![]);
        let driver = IoDriver::new(writer.clone());
        let sink = Sink::new();
        let output = sink.stream();
        let _input = driver.drive(output);
        sink.send("abc".to_string());
        thread::sleep_ms(1);
        assert_eq!(&(*writer.contents().unwrap())[..], "abc".as_bytes());
    }
}
