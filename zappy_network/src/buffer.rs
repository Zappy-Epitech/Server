use std::collections::VecDeque;
use std::io::{self, Read, Write};

pub const MAX_BUFFER_SIZE: usize = 1024 * 1024;

/// A circular buffer implementation to manage network I/O.
pub struct CircularBuffer {
    buffer: VecDeque<u8>,
}

impl CircularBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    /// Appends data to the buffer. Returns false if the overflow limit is reached.
    pub fn push(&mut self, data: &[u8]) -> bool {
        if self.buffer.len() + data.len() > MAX_BUFFER_SIZE {
            return false;
        }
        self.buffer.extend(data);
        true
    }

    /// Reads data directly from a stream into the buffer.
    /// Returns the number of bytes read, or an error.
    /// Returns an error of ErrorKind::OutOfMemory if the buffer overflows.
    pub fn read_from<R: Read>(&mut self, stream: &mut R) -> io::Result<usize> {
        let mut temp = [0; 4096];
        let n = stream.read(&mut temp)?;
        if n > 0 {
            if !self.push(&temp[..n]) {
                return Err(io::Error::new(io::ErrorKind::OutOfMemory, "Buffer overflow"));
            }
        }
        Ok(n)
    }

    /// Writes data from the buffer directly to a stream.
    /// Returns the number of bytes written, or an error.
    pub fn write_to<W: Write>(&mut self, stream: &mut W) -> io::Result<usize> {
        if self.buffer.is_empty() {
            return Ok(0);
        }
        
        let (slice1, _) = self.buffer.as_slices();
        if !slice1.is_empty() {
            let n = stream.write(slice1)?;
            self.buffer.drain(..n);
            Ok(n)
        } else {
            Ok(0)
        }
    }

    /// Extracts a complete line (ending with \n) if available.
    /// The returned string does not include the trailing \n.
    pub fn extract_line(&mut self) -> Option<String> {
        if let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.buffer.drain(..=pos).collect();
            let line_str = String::from_utf8_lossy(&line[..line.len() - 1]);
            Some(line_str.trim_end_matches('\r').to_string())
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_extract_line() {
        let mut buffer = CircularBuffer::new();
        buffer.push(b"hello\nworld\npartial");
        
        assert_eq!(buffer.extract_line(), Some("hello".to_string()));
        assert_eq!(buffer.extract_line(), Some("world".to_string()));
        assert_eq!(buffer.extract_line(), None);
    }

    #[test]
    fn test_push_overflow() {
        let mut buffer = CircularBuffer::new();
        let large_data = vec![0u8; MAX_BUFFER_SIZE];
        
        assert!(buffer.push(&large_data));
        assert!(!buffer.push(b"extra"));
    }

    #[test]
    fn test_extract_line_with_carriage_return() {
        let mut buffer = CircularBuffer::new();
        buffer.push(b"command\r\n");
        assert_eq!(buffer.extract_line(), Some("command".to_string()));
    }
}
