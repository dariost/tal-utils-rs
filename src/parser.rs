use std::fmt;
use std::io::{self, BufRead};
use std::str::FromStr;

pub enum ParserError<T: FromStr>
where
    <T as FromStr>::Err: fmt::Display,
{
    Io(io::Error),
    Parse(<T as FromStr>::Err),
    Utf8(std::string::FromUtf8Error),
}

impl<T: FromStr> fmt::Display for ParserError<T>
where
    <T as FromStr>::Err: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::Io(e) => write!(f, "IO error: {}", e),
            ParserError::Parse(e) => write!(f, "Parse error: {}", e),
            ParserError::Utf8(e) => write!(f, "Utf8 error: {}", e),
        }
    }
}

impl<T: FromStr> fmt::Debug for ParserError<T>
where
    <T as FromStr>::Err: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<T: FromStr> std::error::Error for ParserError<T> where <T as FromStr>::Err: fmt::Display {}

pub trait Parser: BufRead {
    fn get<T: FromStr>(&mut self) -> Result<T, ParserError<T>>
    where
        <T as FromStr>::Err: fmt::Display,
    {
        let mut buffer: Vec<u8> = Vec::new();
        loop {
            let buf = match self.fill_buf() {
                Ok(x) => x,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(ParserError::Io(e)),
            };
            let buf_size = buf.len();
            if buf_size == 0 {
                return Err(ParserError::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF",
                )));
            }
            match buf
                .iter()
                .position(|&x| matches!(x, b' ' | b'\n' | b'\r' | b'\t'))
            {
                Some(0) if buffer.is_empty() => {
                    self.consume(1);
                }
                Some(i) => {
                    buffer.extend_from_slice(&buf[..i]);
                    self.consume(i + 1);
                    break;
                }
                None => {
                    buffer.extend_from_slice(buf);
                    self.consume(buf_size);
                }
            }
        }
        let s = String::from_utf8(buffer).map_err(ParserError::Utf8)?;
        s.parse().map_err(ParserError::Parse)
    }

    fn get_vec<T: FromStr>(&mut self, size: usize) -> Result<Vec<T>, ParserError<T>>
    where
        <T as FromStr>::Err: fmt::Display,
    {
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(self.get()?);
        }
        Ok(v)
    }

    fn get_ascii_str(&mut self) -> Result<Vec<u8>, ParserError<String>> {
        let s = self.get()?;
        Ok(s.into_bytes())
    }
}

impl<T: BufRead> Parser for T {}
