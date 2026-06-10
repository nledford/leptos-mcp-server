//! Transport framing for newline-delimited JSON-RPC over stdio.

use std::io::{self, BufRead};

pub(crate) enum LineRead {
    Line(String),
    Oversized,
}

pub(crate) fn read_limited_line<R: BufRead>(
    reader: &mut R,
    max_bytes: usize,
) -> io::Result<Option<LineRead>> {
    let mut bytes = Vec::new();

    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Ok(Some(LineRead::Line(decode_line(bytes))))
            };
        }

        if let Some(newline_index) = available.iter().position(|byte| *byte == b'\n') {
            let take = newline_index + 1;
            if bytes.len() + newline_index > max_bytes {
                let take_until_oversized = max_bytes.saturating_sub(bytes.len()) + 1;
                reader.consume(take_until_oversized);
                return Ok(Some(LineRead::Oversized));
            }
            bytes.extend_from_slice(&available[..take]);
            reader.consume(take);
            return Ok(Some(LineRead::Line(decode_line(bytes))));
        }

        let take = available.len();
        if bytes.len() + take > max_bytes {
            let take_until_oversized = max_bytes.saturating_sub(bytes.len()) + 1;
            reader.consume(take_until_oversized);
            return Ok(Some(LineRead::Oversized));
        }

        bytes.extend_from_slice(available);
        reader.consume(take);
    }
}

pub(crate) fn decode_line(mut bytes: Vec<u8>) -> String {
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::MAX_JSON_RPC_LINE_BYTES;
    use std::cmp;
    use std::io::{BufReader, Read};

    struct InstrumentedReader {
        input: Vec<u8>,
        position: usize,
        chunks: Vec<usize>,
        fill_buf_calls: Vec<(usize, usize)>,
        consume_calls: Vec<usize>,
    }

    impl InstrumentedReader {
        fn new(input: Vec<u8>, chunks: Vec<usize>) -> Self {
            Self {
                input,
                position: 0,
                chunks,
                fill_buf_calls: Vec::new(),
                consume_calls: Vec::new(),
            }
        }
    }

    impl Read for InstrumentedReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let available = self.fill_buf()?;
            let take = cmp::min(available.len(), buf.len());
            buf[..take].copy_from_slice(&available[..take]);
            self.consume(take);
            Ok(take)
        }
    }

    impl BufRead for InstrumentedReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            let chunk = self
                .chunks
                .get(self.fill_buf_calls.len())
                .copied()
                .unwrap_or(usize::MAX);
            let end = cmp::min(self.input.len(), self.position.saturating_add(chunk));
            self.fill_buf_calls
                .push((self.position, end - self.position));
            Ok(&self.input[self.position..end])
        }

        fn consume(&mut self, amt: usize) {
            self.consume_calls.push(amt);
            self.position += amt;
        }
    }

    #[test]
    fn max_sized_stdin_line_with_newline_is_accepted() {
        let input = format!("{}\n", "x".repeat(MAX_JSON_RPC_LINE_BYTES));
        let mut reader = BufReader::new(input.as_bytes());
        let line = read_limited_line(&mut reader, MAX_JSON_RPC_LINE_BYTES)
            .expect("line reader should succeed")
            .expect("line should be present");

        match line {
            LineRead::Line(line) => assert_eq!(line.len(), MAX_JSON_RPC_LINE_BYTES),
            LineRead::Oversized => panic!("line at the exact limit should be accepted"),
        }
    }

    #[test]
    fn read_limited_line_returns_newline_terminated_frames_in_order() {
        let mut reader = BufReader::new(b"abc\ndef\n".as_slice());

        let first = read_limited_line(&mut reader, 10)
            .expect("line reader should succeed")
            .expect("first line should be present");
        let second = read_limited_line(&mut reader, 10)
            .expect("line reader should succeed")
            .expect("second line should be present");

        match first {
            LineRead::Line(line) => assert_eq!(line, "abc"),
            LineRead::Oversized => panic!("first line should fit"),
        }
        match second {
            LineRead::Line(line) => assert_eq!(line, "def"),
            LineRead::Oversized => panic!("second line should fit"),
        }
    }

    #[test]
    fn read_limited_line_returns_final_eof_frame_then_none() {
        let mut reader = BufReader::new(b"abc".as_slice());

        let line = read_limited_line(&mut reader, 10)
            .expect("line reader should succeed")
            .expect("line should be present");
        let eof = read_limited_line(&mut reader, 10).expect("line reader should succeed");

        match line {
            LineRead::Line(line) => assert_eq!(line, "abc"),
            LineRead::Oversized => panic!("line should fit"),
        }
        assert!(eof.is_none());
    }

    #[test]
    fn exact_limit_stdin_line_without_newline_is_accepted_at_eof() {
        let mut reader = BufReader::new(b"abc".as_slice());
        let line = read_limited_line(&mut reader, 3)
            .expect("line reader should succeed")
            .expect("line should be present");

        match line {
            LineRead::Line(line) => assert_eq!(line, "abc"),
            LineRead::Oversized => panic!("line at exact limit should be accepted"),
        }
    }

    #[test]
    fn limit_plus_one_line_with_newline_consumes_only_violation_byte() {
        let input = b"abcd\nnext\n".to_vec();
        let mut reader = InstrumentedReader::new(input, vec![10]);

        let line = read_limited_line(&mut reader, 3)
            .expect("line reader should succeed")
            .expect("line should be present");

        match line {
            LineRead::Line(_) => panic!("limit-plus-one line should be rejected"),
            LineRead::Oversized => {}
        }
        assert_eq!(reader.consume_calls, vec![4]);
        assert_eq!(reader.position, 4);
    }

    #[test]
    fn invalid_utf8_stdin_line_is_lossily_decoded() {
        let mut reader = BufReader::new([0xff, b'\n'].as_slice());
        let line = read_limited_line(&mut reader, 10)
            .expect("line reader should succeed")
            .expect("line should be present");

        match line {
            LineRead::Line(line) => assert_eq!(line, "�"),
            LineRead::Oversized => panic!("invalid UTF-8 within the limit should be decoded"),
        }
    }

    #[test]
    fn oversized_unterminated_stdin_line_is_rejected() {
        let input = "x".repeat(MAX_JSON_RPC_LINE_BYTES + 1);
        let mut reader = BufReader::new(input.as_bytes());
        let line = read_limited_line(&mut reader, MAX_JSON_RPC_LINE_BYTES)
            .expect("line reader should succeed")
            .expect("line should be present");

        match line {
            LineRead::Line(_) => panic!("unterminated oversized line should be rejected"),
            LineRead::Oversized => {}
        }
    }

    #[test]
    fn read_limited_line_stops_at_hard_cap_without_discard() {
        let input = vec![b'x'; MAX_JSON_RPC_LINE_BYTES + 1];
        let mut reader = InstrumentedReader::new(input, vec![MAX_JSON_RPC_LINE_BYTES, 1]);

        let line = read_limited_line(&mut reader, MAX_JSON_RPC_LINE_BYTES)
            .expect("line reader should succeed")
            .expect("line should be present");

        match line {
            LineRead::Line(_) => panic!("unterminated oversized line should be rejected"),
            LineRead::Oversized => {}
        }
        assert_eq!(
            reader.fill_buf_calls,
            vec![(0, MAX_JSON_RPC_LINE_BYTES), (MAX_JSON_RPC_LINE_BYTES, 1)],
            "reader should stop immediately after observing byte MAX_JSON_RPC_LINE_BYTES + 1"
        );
        assert_eq!(
            reader.consume_calls,
            vec![MAX_JSON_RPC_LINE_BYTES, 1],
            "reader should consume only through the hard-cap violation byte"
        );
        assert_eq!(reader.position, MAX_JSON_RPC_LINE_BYTES + 1);
    }
}
