use crate::{END_SEQUENCE, NULL_WORD, START_SEQUENCE, SYM_NULL};

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
const fn decode_word([a, b, c, d, e]: [u8; 5]) -> [u8; 4] {
    let q = (((a as u32 * 85 + b as u32) * 85 + c as u32) * 85 + d as u32) * 85 + e as u32;
    q.to_be_bytes()
}

/// Decodes ASCII85 encoded data.
/// 
/// The start (`<~`) and end (~>) are optional.
/// 
pub fn decode(mut data: &[u8]) -> Result<Vec<u8>, ()> {
    data.strip_prefix(START_SEQUENCE)
        .map(|stripped| data = stripped);
    data.strip_suffix(END_SEQUENCE)
        .map(|stripped| data = stripped);

    let mut stream = data.iter().filter(|&b| !b.is_ascii_whitespace());

    let mut out = Vec::with_capacity((data.len() + 4) / 5 * 4);


    let (tail_len, tail) = loop {
        match stream.next() {
            Some(&NULL_WORD) => out.extend_from_slice(&[0; 4]),

            Some(&a) => {
                let a = to_number(a).ok_or(())?;
                let (b, c, d, e) = match (
                    stream.next().map(|n| to_number(*n)).flatten(),
                    stream.next().map(|n| to_number(*n)).flatten(),
                    stream.next().map(|n| to_number(*n)).flatten(),
                    stream.next().map(|n| to_number(*n)).flatten(),
                ) {
                    (Some(b), Some(c), Some(d), Some(e)) => (b, c, d, e),
                    (None, _, _, _) => break (1, [a, SYM_NULL, SYM_NULL, SYM_NULL, SYM_NULL]),
                    (Some(b), None, _, _) => break (2, [a, b, SYM_NULL, SYM_NULL, SYM_NULL]),
                    (Some(b), Some(c), None, _) => break (3, [a, b, c, SYM_NULL, SYM_NULL]),
                    (Some(b), Some(c), Some(d), None) => break (4, [a, b, c, d, SYM_NULL]),
                };
                out.extend_from_slice(&decode_word([a, b, c, d, e]));
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
            assert_eq!(
                plain,
                decoded.unwrap(),
                "Couldn't decode test case #{} ({})",
                i,
                codec
            );
        }
    }
}
