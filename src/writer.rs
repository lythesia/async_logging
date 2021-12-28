use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;

use crate::buf::BufferPtr;

use chrono::Local;

pub(crate) struct FileWriter {
    f: File,
    basename: String,
    roll_size: usize,
    written_size: usize,
}

impl FileWriter {
    pub(crate) fn new(basename: &String, roll_size: usize) -> Result<Self, Error> {
        let f = Self::open_log_file(basename)?;
        Ok(Self {
            f,
            basename: basename.to_owned(),
            roll_size,
            written_size: 0,
        })
    }

    fn open_log_file(basename: &String) -> Result<File, Error> {
        let dt = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let pathbuf = PathBuf::from(format!("{}-{}.log", basename, dt));

        let file_path = pathbuf
            .to_str()
            .ok_or(Error::new(ErrorKind::InvalidInput, "invalid file path"))?;

        std::fs::OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(file_path)
    }

    pub(crate) fn append(&mut self, buf: &BufferPtr) {
        if self.written_size >= self.roll_size {
            self.rotate();
        }
        if let Err(e) = self.f.write_all(buf.data().as_slice()) {
            eprintln!("write log file failed: {}", e);
        }
        self.written_size += buf.data().len();
    }

    fn rotate(&mut self) {
        let _ = self.f.flush(); // flush previous file?
        match Self::open_log_file(&self.basename) {
            Ok(f) => {
                self.f = f;
                self.written_size = 0;
            }
            Err(e) => eprintln!("open log file failed: {}", e),
        };
    }

    pub(crate) fn flush(&mut self) {
        let _ = self.f.flush();
    }
}
