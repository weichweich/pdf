use std::convert::TryInto;

use crate::{END_SEQUENCE, NULL_WORD};

fn encode_word(c: [u8; 4]) -> [u8; 5] {
    let mut n = u32::from_be_bytes(c);
    let mut out = [0_u8; 5];
    for i in (0..5).rev() {
        out[i] = (n % 85) as u8 + 33_u8;
        n /= 85;
    }

    out
}

/// Encodes the byte slice using ASCII85.
///
/// The ending sequence `~>` is appended to the end of the encoding.
/// Four zero bytes that are aligned correctly will be encoded as the character
/// `z`.
///
/// # Example
///
/// ```
/// use pdf_ascii85::encode;
///
/// let encoded = encode(&[1,2,3,4, 0,0,0,0]);
/// assert_eq!(encoded, b"!<N?+z~>");
/// ```
pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity((data.len() / 4) * 5 + 10);
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();
    for chunk in chunks {
        let c: [u8; 4] = chunk.try_into().expect("The chunk size was ensured with chunks_exact.");
        if c == [0; 4] {
            buf.push(NULL_WORD);
        } else {
            buf.extend_from_slice(&encode_word(c));
        }
    }

    if !remainder.is_empty() {
        let mut c = [0; 4];
        c[..remainder.len()].copy_from_slice(remainder);
        let out = encode_word(c);
        buf.extend_from_slice(&out[..remainder.len() + 1]);
    }
    buf.extend_from_slice(END_SEQUENCE);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successfull_encode() {
        let tests = crate::tests::encode_samples();

        for (i, (plain, codec)) in tests.into_iter().enumerate() {
            let encoded = String::from_utf8(encode(plain)).expect("must be valid utf-8");
            assert_eq!(encoded, codec, "Couldn't encode test case #{} ({})", i, codec);
        }
    }
}
