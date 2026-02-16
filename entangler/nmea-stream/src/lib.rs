#![no_std]

use embedded_io_async::{Read, ReadExactError};

// NMEA sentences are max 82 chars,
// 128 should be enough to buffer through until next '$'
const MAX_NMEA_SENTENCE_LENGTH: usize = 128;

pub struct NmeaReader<'a, R> {
    reader: &'a mut R,
    buffer: [u8; MAX_NMEA_SENTENCE_LENGTH],
    pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum NmeaReaderError<'a, EIO> {
    IO(EIO),
    Nmea(nmea::Error<'a>),
    ParseError,
    InvalidData,
}

impl<'a, R: Read> NmeaReader<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        Self {
            reader,
            buffer: [0; 128],
            pos: 0,
        }
    }

    pub async fn next(&mut self) -> Result<nmea::ParseResult, NmeaReaderError<'_, R::Error>> {
        let sentence = {
            let s = self.try_get_sentence().await?;
            nmea::parse_bytes(s).map_err(|_| NmeaReaderError::ParseError)?
        };
        Ok(sentence)
    }

    async fn try_get_sentence(&mut self) -> Result<&[u8], NmeaReaderError<'_, R::Error>> {
        let mut byte = [0u8; 1];
        loop {
            self.reader
                .read_exact(&mut byte)
                .await
                .map_err(|e| match e {
                    ReadExactError::UnexpectedEof => NmeaReaderError::InvalidData,
                    ReadExactError::Other(io_error) => NmeaReaderError::IO(io_error),
                })?;

            let b = byte[0];

            match b {
                b'$' => {
                    // Start of a new packet: reset buffer
                    self.buffer[0] = b'$';
                    self.pos = 1;
                }
                b'\n' if self.pos > 0 && self.buffer[self.pos - 1] == b'\r' => {
                    // End of packet (\r\n)
                    self.buffer[self.pos] = b;
                    self.pos += 1;
                    let end = self.pos;
                    self.pos = 0; // Reset for next call
                    return Ok(&self.buffer[..end]);
                }
                _ => {
                    if self.pos > 0 && self.pos < self.buffer.len() {
                        self.buffer[self.pos] = b;
                        self.pos += 1;
                    } else {
                        // Buffer overflow or data before first '$', ignore
                        self.pos = 0;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    extern crate std;

    use mock_embedded_io::{MockError, Source};

    #[tokio::test]
    async fn test_next_err() {
        let mut source = Source::new().error(MockError(embedded_io_async::ErrorKind::BrokenPipe));
        let mut reader = NmeaReader::new(&mut source);
        let res = reader.next().await;
        assert!(res.is_err_and(
            |e| e == NmeaReaderError::IO(MockError(embedded_io_async::ErrorKind::BrokenPipe))
        ));
    }

    #[tokio::test]
    async fn test_next() {
        let mut source = Source::new()
            .data("$GPRMC,235316.000,A,4003.9040,N,10512.5792,W,0.09,144.75,141112,,*19\r\n");
        let mut reader = NmeaReader::new(&mut source);
        let res = reader.next().await;
        assert!(res.is_ok_and(|x| nmea::SentenceType::from(&x) == nmea::SentenceType::RMC));
    }

    #[tokio::test]
    async fn test_next_parts() {
        let mut source = Source::new()
            .data("$GPRMC,235316.000,A,4003.9040,N")
            .data(",10512.5792,W,0.09,144.75,141112,,*19\r\n");
        let mut reader = NmeaReader::new(&mut source);
        let res = reader.next().await;
        assert!(res.is_ok_and(|x| nmea::SentenceType::from(&x) == nmea::SentenceType::RMC));
    }

    #[tokio::test]
    async fn test_next_multiple() {
        let mut source = Source::new()
            .data("$GPRMC,235316.000,A,4003.9040,N,10512.5792,W,0.09,144.75,141112,,*19\r\n")
            .data("$GPRMC,235316.000,A,4003.9040,N,10512.5792,W,0.09,144.75,141112,,*19\r\n");
        let mut reader = NmeaReader::new(&mut source);

        let res = reader.next().await;
        assert!(res.is_ok_and(|x| nmea::SentenceType::from(&x) == nmea::SentenceType::RMC));

        let res = reader.next().await;
        assert!(res.is_ok_and(|x| nmea::SentenceType::from(&x) == nmea::SentenceType::RMC));
    }

    #[tokio::test]
    async fn test_next_data_at_start() {
        let mut source = Source::new()
            .data("2.5792,W,0.09,144.75,141112,,*19$GPRMC,235316.000,")
            .data("A,4003.9040,N,10512.5792,W,0.09,144.75,141112,,*19\r\n");
        let mut reader = NmeaReader::new(&mut source);

        let res = reader.next().await;
        assert!(res.is_ok_and(|x| nmea::SentenceType::from(&x) == nmea::SentenceType::RMC));
    }

    #[tokio::test]
    async fn test_try_get_sentence() {
        let mut source = Source::new()
            .data("2.5792,W,0.09,144.75,141112,,*19$GPRMC,235316.000,")
            .data("A,4003.9040,N,10512.5792,W,0.09,144.75,141112,,*19\r\n");
        let mut reader = NmeaReader::new(&mut source);

        let res = reader.try_get_sentence().await.unwrap();
        assert_eq!(
            res,
            "$GPRMC,235316.000,A,4003.9040,N,10512.5792,W,0.09,144.75,141112,,*19\r\n".as_bytes()
        );
    }
}
