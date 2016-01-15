use std::iter;
use std::io::{ BufReader, BufRead, Read, Write };
use std::thread;
use carboxyl::{ Sink, Stream };

use ::Input;

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


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use carboxyl::Sink;

    use super::*;
    use ::Input;
    use ::sync::{ SyncWriter, check_timeout };

    #[test]
    fn writes_events_from_output_stream() {
        const SAMPLE: &'static str = "abc";

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
}
