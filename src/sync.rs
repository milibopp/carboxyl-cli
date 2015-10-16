use std::io::{ self, Write };
use std::sync::{ Arc, RwLock, RwLockReadGuard };


#[derive(Clone)]
pub struct SyncWriter {
    buffer: Arc<RwLock<Vec<u8>>>,
}

impl SyncWriter {
    pub fn new(contents: Vec<u8>) -> SyncWriter {
        SyncWriter { buffer: Arc::new(RwLock::new(contents)) }
    }

    pub fn contents(&self) -> Option<RwLockReadGuard<Vec<u8>>> {
        self.buffer.read().ok()
    }
}

impl Write for SyncWriter {
    fn write(&mut self, stuff: &[u8]) -> io::Result<usize> {
        self.buffer.write()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "poisoned"))
            .and_then(|mut b| b.write(stuff))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.write()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "poisoned"))
            .and_then(|mut b| b.flush())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;

    #[test]
    fn sync_writer_writes_to_inner_buffer() {
        let mut writer = SyncWriter::new(vec![]);
        writer.write(&[3, 1]).unwrap();
        assert_eq!(&*writer.contents().unwrap(), &[3, 1]);
    }
}
