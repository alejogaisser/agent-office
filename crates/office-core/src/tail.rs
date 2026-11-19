//! Offset-based incremental reader for append-only JSONL files.
use std::fs::OpenOptions;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const CHUNK: usize = 64 * 1024;
pub const MAX_LINE: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub enum StartAt {
    Beginning,
    LastBytes(u64),
    End,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TailStats {
    pub oversized: u64,
}

#[derive(Debug)]
pub struct TailReader {
    pub path: PathBuf,
    pub offset: u64,
    pub partial: Vec<u8>,
    pub gone: bool,
    pub stats: TailStats,
    /// True when the start was cut (file larger than the initial budget).
    pub truncated_start: bool,
    skipping: bool,
}

fn open(path: &Path) -> io::Result<std::fs::File> {
    let mut o = OpenOptions::new();
    o.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
        o.share_mode(7);
    }
    o.open(path)
}

impl TailReader {
    pub fn new(path: impl Into<PathBuf>, start: StartAt) -> TailReader {
        let path = path.into();
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let (offset, skipping, truncated_start) = match start {
            StartAt::Beginning => (0, false, false),
            StartAt::End => (size, false, false),
            StartAt::LastBytes(n) if size > n => (size - n, true, true),
            StartAt::LastBytes(_) => (0, false, false),
        };
        TailReader {
            path,
            offset,
            partial: Vec::new(),
            gone: false,
            stats: TailStats::default(),
            truncated_start,
            skipping,
        }
    }

    /// Cheap check: has the file grown (or shrunk) since the last read?
    pub fn has_new_data(&self) -> bool {
        match std::fs::metadata(&self.path) {
            Ok(m) => m.len() != self.offset,
            Err(_) => !self.gone,
        }
    }

    pub fn read_new(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut f = match open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.gone = true;
                return Ok(Vec::new());
            }
            Err(e) => return Err(e),
        };
        self.gone = false;
        let len = f.metadata()?.len();
        if len < self.offset {
            self.offset = 0;
            self.partial.clear();
            self.skipping = false;
            self.truncated_start = false;
        }
        let mut lines = Vec::new();
        if len == self.offset {
            return Ok(lines);
        }
        f.seek(SeekFrom::Start(self.offset))?;
        let mut buf = vec![0u8; CHUNK];
        let mut remaining = len - self.offset;
        while remaining > 0 {
            let want = (remaining as usize).min(CHUNK);
            let n = f.read(&mut buf[..want])?;
            if n == 0 {
                break;
            }
            remaining -= n as u64;
            self.offset += n as u64;
            self.feed(&buf[..n], &mut lines);
        }
        Ok(lines)
    }

    fn feed(&mut self, mut data: &[u8], lines: &mut Vec<Vec<u8>>) {
        while !data.is_empty() {
            match data.iter().position(|&b| b == b'\n') {
                Some(i) => {
                    let (head, rest) = data.split_at(i);
                    data = &rest[1..];
                    if self.skipping {
                        self.skipping = false;
                        self.partial.clear();
                        continue;
                    }
                    self.partial.extend_from_slice(head);
                    if self.partial.len() > MAX_LINE {
                        self.stats.oversized += 1;
                        self.partial = Vec::new();
                        continue;
                    }
                    let mut line = std::mem::take(&mut self.partial);
                    if line.last() == Some(&b'\r') {
                        line.pop();
                    }
                    if !line.is_empty() {
                        lines.push(line);
                    }
                }
                None => {
                    if !self.skipping {
                        self.partial.extend_from_slice(data);
                        if self.partial.len() > MAX_LINE {
                            self.stats.oversized += 1;
                            self.partial = Vec::new();
                            self.skipping = true;
                        }
                    }
                    break;
                }
            }
        }
    }
}
