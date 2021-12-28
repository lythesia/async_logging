pub(crate) struct FixedBuffer<const N: usize> {
    data: Vec<u8>,
}

impl<const N: usize> FixedBuffer<N> {
    pub fn new() -> Self {
        FixedBuffer {
            data: Vec::with_capacity(N),
        }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn avail(&self) -> usize {
        N - self.data.len()
    }

    pub fn append(&mut self, buf: &[u8]) -> usize {
        let n = std::cmp::min(buf.len(), N - self.data.len());
        self.data.extend_from_slice(&buf[..n]);
        n
    }
}

// const TEST_BUFFER_SIZE: usize = 40;
// const SMALL_BUFFER_SIZE: usize = 4000;
const LARGE_BUFFER_SIZE: usize = 4000 * 1000;
pub(crate) const INITIAL_BUFFERS_NUM: usize = 16;

// 1. work threads use Arc<AsyncLogging>::append() with mutex
//    append should be able to access buffers
// 2. AsyncLogging has one thread with writer read from buffers and write to log file, thus it should
//    hold &buffers
pub(crate) type Buffer = FixedBuffer<LARGE_BUFFER_SIZE>;
pub(crate) type BufferVec = Vec<BufferPtr>;
pub(crate) type BufferPtr = Box<Buffer>;

pub(crate) struct DoubleBuffer {
    pub curr_buffer: BufferPtr,
    pub next_buffer: Option<BufferPtr>,
    pub buffers: BufferVec,
}

impl DoubleBuffer {
    pub(crate) fn default() -> Self {
        DoubleBuffer {
            curr_buffer: Box::new(Buffer::new()),
            next_buffer: None,
            buffers: Vec::with_capacity(INITIAL_BUFFERS_NUM),
        }
    }
}
