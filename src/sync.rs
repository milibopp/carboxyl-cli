use std::io::{ self, Write };
use std::sync::{ Arc, RwLock, RwLockReadGuard, RwLockWriteGuard };
use std::thread;
use std::time::Duration;


#[derive(Clone)]
pub struct SyncWriter {
    buffer: Arc<RwLock<Vec<u8>>>,
}

impl SyncWriter {
    pub fn new() -> SyncWriter {
        SyncWriter { buffer: Arc::new(RwLock::new(vec![])) }
    }

    pub fn contents(&self) -> Option<RwLockReadGuard<Vec<u8>>> {
        self.buffer.read().ok()
    }

    fn write_guard(&mut self) -> io::Result<RwLockWriteGuard<Vec<u8>>> {
        self.buffer.write()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "poisoned"))
    }
}

impl Write for SyncWriter {
    fn write(&mut self, stuff: &[u8]) -> io::Result<usize> {
        self.write_guard()
            .and_then(|mut b| b.write(stuff))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write_guard()
            .and_then(|mut b| b.flush())
    }
}


pub fn check_timeout<F: FnMut() -> bool>(mut predicate: F, retries: u32) {
    for _ in 0..retries {
        thread::sleep(Duration::from_millis(1));
        if predicate() { return; }
    }
    panic!("check timed out")
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;

    #[test]
    fn sync_writer_writes_to_inner_buffer() {
        let mut writer = SyncWriter::new();
        writer.write(&[3, 1]).unwrap();
        assert_eq!(&*writer.contents().unwrap(), &[3, 1]);
    }
}
