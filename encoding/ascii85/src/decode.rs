use crate::{END_SEQUENCE, NULL_CHAR, NULL_WORD, START_SEQUENCE};

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct Ascii85DecodeError {
    /// The index of the character that caused the error.
    error_index: usize,
}

impl Ascii85DecodeError {
    /// Up to which index the buffer contained valid ASCII85 encoded data.
    pub fn valid_up_to(&self) -> usize {
        self.error_index.saturating_sub(1)
    }
}

/// Maps an ASCII character to a number
const fn to_number(byte: u8) -> Option<u8> {
    match byte {
        b @ 0x21..=0x75 => Some(b - 0x21),
        _ => None,
    }
}

/// Decodes 5 ASCII85 bytes to 4 bytes.
///
/// Assumes that the 5 bytes are already mapped to numbers using `to_number`.
const fn decode_word(word: [u8; 5]) -> [u8; 4] {
    let mut q = word[0] as u32;
    q = q * 85 + word[1] as u32;
    q = q * 85 + word[2] as u32;
    q = q * 85 + word[3] as u32;
    q = q * 85 + word[4] as u32;
    q.to_be_bytes()
}

/// Fills the byte buffer using the iterator. Returns the number of elements
/// that where copied to the buffer.
fn fill_from_iter(stream: &mut impl Iterator<Item = (usize, u8)>, buf: &mut [u8]) -> usize {
    for (i, digit) in buf.iter_mut().enumerate() {
        if let Some(b) = stream.next() {
            *digit = b.1;
        } else {
            // the index where the iterator ended, index of the last byte written plus 1.
            return i;
        }
    }
    buf.len()
}

/// Decodes ASCII85 encoded data.
///
/// The start (`<~`) and end (`~>`) are optional.
/// The character `z` is interpreted as four zero bytes.
///
/// # Examples
///
/// ```
/// use pdf_ascii85::decode;
///
/// let encoded = "<~9jqo~>";
/// let decoded = decode(encoded.as_bytes()).unwrap();
///
/// assert_eq!(String::from_utf8(decoded), Ok("Man".to_string()));
/// ```
pub fn decode(mut data: &[u8]) -> Result<Vec<u8>, Ascii85DecodeError> {
    if let Some(stripped) = data.strip_prefix(START_SEQUENCE) {
        data = stripped;
    }
    if let Some(stripped) = data.strip_suffix(END_SEQUENCE) {
        data = stripped;
    }

    let mut stream = data
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, b)| !b.is_ascii_whitespace());

    let mut out = Vec::with_capacity((data.len() + 4) / 5 * 4);

    let (tail_len, tail) = loop {
        match stream.next() {
            Some((_, NULL_WORD)) => out.extend_from_slice(&[0; 4]),

            Some((index, a)) => {
                let mut buf = [NULL_CHAR; 5];
                buf[0] = a;

                let copied = fill_from_iter(&mut stream, &mut buf[1..]);
                for digit in &mut buf {
                    *digit = to_number(*digit).ok_or_else(|| Ascii85DecodeError { error_index: index })?;
                }
                if copied < 4 {
                    // The buffer was not filled. The stream
                    break (copied + 1, buf);
                }
                out.extend_from_slice(&decode_word(buf));
            }
            None => break (0, [0; 5]),
        }
    };

    if tail_len > 0 {
        let last = decode_word(tail);
        out.extend_from_slice(&last[..tail_len - 1]);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successfull_decode() {
        let tests = crate::tests::decode_samples();

        for (i, (plain, codec)) in tests.into_iter().enumerate() {
            let decoded = decode(codec.as_bytes());
            assert!(decoded.is_ok(), "Error in test case #{} ({})", i, codec);
            assert_eq!(plain, decoded.unwrap(), "Couldn't decode test case #{} ({})", i, codec);
        }
    }
}
