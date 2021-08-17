//! Various helpers for teeing readers and writers.
//!
//! [`TeeReader`] and [`TeeWriter`] respect the underlying `Read`er and `Write`er's method overrides.
//! (Except for vectored, because I haven't got around to that yet)
//! 
//! [`TeeReader`] supports teeing `Read`, `BufRead` and `Seek` readers.

use std::{fmt::Arguments, io::{BufRead, Read, Seek, Stderr, Write}};
trait ReadExt: Read {
    fn tee<W: Write>(self, out: W) -> TeeReader<Self, W> where Self: Sized;
    fn tee_dbg(self) -> TeeReader<Self, Stderr> where Self: Sized;
}

impl<R: Read> ReadExt for R {
    fn tee<W: Write>(self, out: W) -> TeeReader<Self, W> where Self: Sized {
        TeeReader::new(self, out)
    }

    fn tee_dbg(self) -> TeeReader<Self, Stderr> where Self: Sized {
        TeeReader::new_stderr(self)
    }
}

trait WriteExt: Write {
    fn tee<R: Write>(self, other: R) -> TeeWriter<Self, R> where Self: Sized;
    fn tee_dbg(self) -> TeeWriter<Self, Stderr> where Self: Sized;
}

impl<W: Write> WriteExt for W {
    fn tee<R: Write>(self, other: R) -> TeeWriter<Self, R> where Self: Sized {
        TeeWriter::new(self, other)
    }

    fn tee_dbg(self) -> TeeWriter<Self, Stderr> where Self: Sized {
        TeeWriter::new_stderr(self)
    }
}

/// A reader which tees its input to another writer.
pub struct TeeReader<R, W> {
    reader: R,
    writer: W,
}

impl<R: Read, W: Write> TeeReader<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
        }
    }
}

impl<R: Read> TeeReader<R, Stderr> {
    pub fn new_stderr(reader: R) -> Self {
        Self {
            reader,
            writer: std::io::stderr(),
        }
    }
}

impl<R, W> TeeReader<R, W> {
    pub fn reader_ref(&self) -> &R {
        &self.reader
    }

    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    pub fn writer_ref(&self) -> &W {
        &self.writer
    }

    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn into_reader_writer(self) -> (R, W) {
        (self.reader, self.writer)
    }
}

impl<R: Read, W: Write> Read for TeeReader<R, W> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.writer.write_all(&buf[..len])?;
        Ok(len)
    }

    // read_vectored omitted
    //TODO ?

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let start = buf.len();
        let len = self.reader.read_to_end(buf)?;
        self.writer.write_all(&buf[start..start + len])?;
        Ok(len)
    }

    // read_to_string omitted
    // The default impl calls `read_to_end` anyway.

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.reader.read_exact(buf)?;
        self.writer.write_all(&buf)?;
        Ok(())
    }

    // by_ref omitted  
}

impl<R: BufRead, W: Write> BufRead for TeeReader<R, W> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let initial_len = buf.len();
        let bytes_read = self.reader.read_until(byte, buf)?;
        self.writer.write_all(&buf[initial_len..initial_len + bytes_read])?;
        Ok(bytes_read)
    }

    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        let initial_len = buf.as_bytes().len();
        let bytes_read = self.reader.read_line(buf)?;
        self.writer.write_all(&buf.as_bytes()[initial_len..initial_len + bytes_read])?;
        Ok(bytes_read)
    }
}

impl<R: Seek, W> Seek for TeeReader<R, W> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.reader.stream_position()
    }
}

struct TeeWriter<L, R> {
    left: L,
    right: R,
}

impl<L: Write, R: Write> TeeWriter<L, R> {
    fn new(left: L, right: R) -> Self {
        Self {
            left,
            right,
        }
    }
}

impl<L: Write> TeeWriter<L, Stderr> {
    fn new_stderr(left: L) -> Self {
        Self {
            left,
            right: std::io::stderr(),
        }
    }
}

impl<L: Write, R: Write> Write for TeeWriter<L, R> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.left.write(&buf[..])?;
        self.right.write_all(&buf[..n])?;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.left.flush()?;
        self.right.flush()?;
        Ok(())
    }

    // write_vectored omitted

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.left.write_all(buf)?;
        self.right.write_all(buf)?;
        Ok(())
    }

    // write_all_vectored omitted

    fn write_fmt(&mut self, fmt: Arguments<'_>) -> std::io::Result<()> {
        self.left.write_fmt(fmt)?;
        self.right.write_fmt(fmt)?;
        Ok(())
    }

    // by_ref omitted
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, Read};

    use crate::TeeReader;

    #[test]
    fn basic_read() -> std::io::Result<()> {
        let text = b"Hello, world!\n";
        let mut buf = [0u8; 5];
        let mut debug_buf = [0u8; 5];

        let mut reader = TeeReader::<&[u8], &mut [u8]>::new(text, &mut debug_buf);
        assert_eq!(reader.read(&mut buf)?, 5);
        drop(reader);

        assert_eq!(buf, debug_buf);
        Ok(())
    }

    #[test]
    fn read_to_end() -> std::io::Result<()> {
        let text = b"Hello, world!\n";
        let mut buf = Vec::with_capacity(text.len());
        let mut debug_buf = [0u8; 14];
        let mut reader = TeeReader::<&[u8], &mut [u8]>::new(text, &mut debug_buf);
        assert_eq!(reader.read_to_end(&mut buf)?, 14);
        drop(reader);
        assert_eq!(buf, debug_buf);
        Ok(())
    }

    #[test]
    fn buf_read() -> std::io::Result<()> {
        let text = b"Hello, world!\n";
        let mut debug_buf = [0u8; 14];
        let mut reader = TeeReader::<&[u8], &mut [u8]>::new(text, &mut debug_buf);
        let mut buf = Vec::with_capacity(text.len());
        assert_eq!(reader.read_until(b',', &mut buf)?, 6);
        let mut string = String::new();
        assert_eq!(reader.read_line(&mut string)?, 8);
        drop(reader);
        assert_eq!(&debug_buf, text);
        Ok(())
    }
}
