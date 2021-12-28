use std::io::Error;
use std::mem;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use crate::buf::*;
use crate::count_down_latch::CountDownLatch;
use crate::writer::FileWriter;

// enum ThreadState {
//     Init(Box<dyn FnOnce() -> Option<thread::JoinHandle<()>>>),
//     Run()
// }

struct ThreadedWriter {
    latch: Arc<CountDownLatch>,
    buf: Arc<(Mutex<DoubleBuffer>, Condvar)>,
    terminated: Arc<AtomicBool>,
    writer_thread: Option<thread::JoinHandle<()>>,
}

impl ThreadedWriter {
    fn new(
        latch: Arc<CountDownLatch>,
        writer: FileWriter,
        buf: Arc<(Mutex<DoubleBuffer>, Condvar)>,
        flush_interval: u64,
    ) -> Self {
        let l = Arc::clone(&latch);
        let b = Arc::clone(&buf);
        let terminated = Arc::new(AtomicBool::new(false));
        let t = Arc::clone(&terminated);
        let writer_thread = thread::Builder::new()
            .name("[LOG]".into())
            .spawn(move || Self::thread_loop(l, b, writer, flush_interval, t))
            .map_err(|e| {
                eprintln!("fail to spawn writer thread: {}", e);
            })
            .ok(); // started

        ThreadedWriter {
            latch,
            buf,
            terminated,
            writer_thread,
        }
    }

    // call in main (main wait on latch, until writer thread loop start)
    fn start(&mut self) {
        // TODO: thread start here or start by new?
        self.latch.wait();
    }

    // call in main
    fn stop(&mut self) {
        self.terminated.store(true, Ordering::Relaxed);

        let (_, cvar) = &*self.buf;
        cvar.notify_one();

        if let Some(h) = self.writer_thread.take() {
            h.join().expect("writer thread join failed");
        }
        println!("writer quit");
    }

    fn thread_loop(
        latch: Arc<CountDownLatch>,
        buf: Arc<(Mutex<DoubleBuffer>, Condvar)>,
        mut writer: FileWriter,
        flush_interval: u64,
        terminated: Arc<AtomicBool>,
    ) {
        latch.count_down();
        println!("writer started");

        let mut buffers_to_write = BufferVec::with_capacity(INITIAL_BUFFERS_NUM);
        let mut backup1 = Some(Box::new(Buffer::new()));
        let mut backup2 = Some(Box::new(Buffer::new()));

        loop {
            // mutex lock access double_buffer
            {
                let (buf, cvar) = &*buf;
                let mut db = buf.lock().unwrap();
                if db.buffers.is_empty() {
                    db = cvar
                        .wait_timeout(db, Duration::from_secs(flush_interval))
                        .unwrap()
                        .0;
                }
                // if we enter this section, curr_buffer is ALWAYS moved and renewed
                assert!(!backup1.is_none(), "backup buffer #1 is empty!");
                let r = mem::replace(&mut db.curr_buffer, backup1.take().unwrap());
                db.buffers.push(r);
                // swap for write
                mem::swap(&mut db.buffers, &mut buffers_to_write);
                // in case next_buffer need to refill
                if db.next_buffer.is_none() {
                    assert!(!backup2.is_none(), "backup buffer #2 is empty!");
                    db.next_buffer = backup2.take();
                }
            }
            for v in buffers_to_write.iter() {
                writer.append(v);
            }
            // drop used buffers, avoid thrashing(but leave 2 for refill)
            // if bufs_to_write.len() > 2 {
            //     bufs_to_write.resize_with(2, || Box::new(Buffer::new()));
            // }
            // refill backs (place these outside CS to improve performance)
            if backup1.is_none() {
                backup1 = Some(Box::new(Buffer::new()));
            }
            if backup2.is_none() {
                backup2 = Some(Box::new(Buffer::new()));
            }

            buffers_to_write.clear();
            writer.flush();

            if terminated.load(Ordering::Relaxed) {
                break;
            }
        }
        writer.flush();
    }
}

pub struct AsyncLogging {
    double_buffer: Arc<(Mutex<DoubleBuffer>, Condvar)>,
    thread_writer: ThreadedWriter,
}

impl AsyncLogging {
    pub fn new(basename: String, roll_size: usize, flush_interval: u64) -> Result<Self, Error> {
        let file_writer = FileWriter::new(&basename, roll_size)?;
        let buf = Arc::new((Mutex::new(DoubleBuffer::default()), Condvar::new()));
        Ok(AsyncLogging {
            double_buffer: Arc::clone(&buf),
            thread_writer: ThreadedWriter::new(
                Arc::new(CountDownLatch::new(1)),
                file_writer,
                buf,
                flush_interval,
            ),
        })
    }

    pub fn start(&mut self) {
        self.thread_writer.start();
    }

    pub fn append(&self, line: String) {
        let (lock, cvar) = &*self.double_buffer;
        let mut db = lock.lock().unwrap();

        let sz = line.len();
        if sz < db.curr_buffer.avail() {
            db.curr_buffer.append(line.as_bytes());
        } else {
            let backup = db
                .next_buffer
                .take()
                .unwrap_or_else(|| Box::new(Buffer::new()));
            let r = mem::replace(&mut db.curr_buffer, backup);
            db.buffers.push(r);
            db.curr_buffer.append(line.as_bytes());
            cvar.notify_one();
        }
    }

    pub fn stop(&mut self) {
        self.thread_writer.stop();
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_fixed_buffer_new() {
        let b = FixedBuffer::<16>::new();
        assert_eq!(16, b.data.capacity());
    }

    #[test]
    fn test_fixed_buffer_append() {
        let mut b = FixedBuffer::<8>::new();
        let n = b.append(b"hello");
        assert_eq!(n, 5);
    }

    #[test]
    fn test_fixed_buffer_append_overflow() {
        let mut b = FixedBuffer::<8>::new();
        let n = b.append(b"hello world");
        assert_eq!(n, 8);
    }

    #[test]
    fn test_simple_log() {
        let mut logger = AsyncLogging::new(String::from("/tmp/async"), 32, 1).unwrap();
        for i in 1..4 {
            logger.append(format!("line {}\n", i));
        }
        // logger.stop();
    }

    #[test]
    fn test_rotate_log() {
        let mut logger = AsyncLogging::new(String::from("/tmp/async"), 8, 1).unwrap();
        logger.append("abcdefg\n".into());
        thread::sleep(Duration::from_secs(2));
        logger.append("hijklmn\n".into());
        // logger.stop();
    }

    #[test]
    fn test_multi_thread_rotate_log() {
        let mut logger = Arc::new(AsyncLogging::new(String::from("/tmp/async"), 32, 1).unwrap());
        let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
        for i in 1..4 {
            let l = Arc::clone(&logger);
            let th = thread::Builder::new()
                .name(format!("Job#{}", i))
                .spawn(move || {
                    thread::sleep(Duration::from_secs(i as u64)); // give time to rotate to new file, TODO: maybe we need indexed rotation
                    let dt = Local::now().format("%Y%m%d-%H:%M:%S").to_string();
                    l.append(format!("{} [Job#{}] abcdefghijklmn\n", dt, i));
                })
                .unwrap();
            threads.push(th);
        }
        for v in threads {
            v.join().unwrap();
        }
        // logger.stop();
    }
}
*/
