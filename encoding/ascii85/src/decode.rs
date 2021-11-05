use crate::{END_SEQUENCE, NULL_CHAR, NULL_WORD, START_SEQUENCE};

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct DecodeError {
    /// The index of the character that caused the error.
    error_index: usize,
}

impl DecodeError {
    pub fn new_with_index(start: &u8, error: &u8) -> Self {
        let ptr_start: *const u8 = start;
        let ptr_error: *const u8 = error;

        Self {
            error_index: (ptr_error as usize) - (ptr_start as usize),
        }
    }
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
fn decode_word(word: [u8; 5]) -> Option<[u8; 4]> {
    let mut q = word[0] as u32;
    q = q * 85 + word[1] as u32;
    q = q * 85 + word[2] as u32;
    q = q * 85 + word[3] as u32;
    q = q.checked_mul(85)?.checked_add(word[4] as u32)?;
    Some(q.to_be_bytes())
}

/// Fills the byte buffer using the iterator. Returns the number of elements
/// that where copied to the buffer.
fn fill_from_iter<'a>(stream: &mut impl Iterator<Item = &'a u8>, buf: &mut [u8]) -> usize {
    for (i, digit) in buf.iter_mut().enumerate() {
        if let Some(&b) = stream.next() {
            *digit = b;
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
pub fn decode(mut data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    // get ref to first element or return empty vec if data is empty. We use the ref
    // to the first element in case there is an invalid character in the data. The
    // first element is then used to calculate the index of the char
    let first = if let Some(first) = data.get(0) {
        first
    } else {
        return Ok(Vec::with_capacity(0));
    };

    if let Some(stripped) = data.strip_prefix(START_SEQUENCE) {
        data = stripped;
    }
    if let Some(stripped) = data.strip_suffix(END_SEQUENCE) {
        data = stripped;
    }

    let mut stream = data.iter().filter(|b| !b.is_ascii_whitespace());

    let mut out = Vec::with_capacity((data.len() + 4) / 5 * 4);

    loop {
        match stream.next() {
            Some(&NULL_WORD) => out.extend_from_slice(&[0; 4]),

            Some(a) => {
                let mut buf = [NULL_CHAR; 5];
                buf[0] = *a;

                let copied = fill_from_iter(&mut stream, &mut buf[1..]);
                for digit in &mut buf {
                    *digit = to_number(*digit).ok_or_else(|| DecodeError::new_with_index(first, a))?;
                }
                let word = decode_word(buf).ok_or_else(|| DecodeError::new_with_index(first, a))?;
                if copied < 4 {
                    // The buffer was not filled. The stream stopped while copying.
                    out.extend_from_slice(&word[..copied]);
                } else {
                    out.extend_from_slice(&word);
                }
            }
            None => break,
        }
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

    #[test]
    fn correct_error_index() {
        let mut example = crate::tests::EXAMPLE_CODEC.to_owned();
        // replace a word in ascii85 with a word that is out of bounds.
        example.replace_range(9..14, "uuuuu");

        // should report the index at which the error occurred (10 in this case)
        let decoded = decode(example.as_bytes());
        assert_eq!(decoded, Err(DecodeError { error_index: 10 }));

        // should be able to decode up to the invalid word
        let decoded = decode(&example.as_bytes()[..10]);
        assert!(decoded.is_ok());
    }

    #[test]
    fn errors_on_max_invalide() {
        let decoded = decode(&[0x75, 0x75, 0x75, 0x75, 0x75]);
        assert!(matches!(decoded, Err(DecodeError { error_index: _ })));
    }
}
