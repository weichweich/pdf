use std::convert::TryInto;

/// The character `u` in the ASCII table represents the null byte (0x00).
const NULL_CHAR: u8 = b'u';

/// The character `z` in the ASCII table represents 4 null bytes (0x0000_0000).
const NULL_WORD: u8 = b'z';

fn sym_85(byte: u8) -> Option<u8> {
    match byte {
        b @ 0x21..=0x75 => Some(b - 0x21),
        _ => None,
    }
}

fn word_85([a, b, c, d, e]: [u8; 5]) -> Option<[u8; 4]> {
    let (a, b, c, d, e) = (sym_85(a)?, sym_85(b)?, sym_85(c)?, sym_85(d)?, sym_85(e)?);
    let q = (((a as u32 * 85 + b as u32) * 85 + c as u32) * 85 + d as u32) * 85 + e as u32;
    Some(q.to_be_bytes())
}

pub fn decode(mut data: &[u8]) -> Result<Vec<u8>, ()> {
    let mut out = Vec::with_capacity((data.len() + 4) / 5 * 4);

    data.strip_prefix(b"<~").map(|stripped| data = stripped);
    data.strip_suffix(b"~>").map(|stripped| data = stripped);

    let mut stream = data.iter().filter(|&b| !b.is_ascii_whitespace());

    // parse the middle of the buffer
    let mut buf = [NULL_CHAR; 5];
    while let Some(char) = stream.next() {
        match char {
            // null word shortcut
            &NULL_WORD => out.extend_from_slice(&[0x00_u8; 4]),

            // Parse ascii85 word
            char => {
                let mut index = 1;
                // insert current char
                buf[0] = *char;

                // fill buffer if possible
                for char in stream.by_ref().take(buf.len() - 1) {
                    buf[index] = *char;
                    index += 1;
                }

                // parse word
                let parsed_word = word_85(buf).ok_or(())?;
                out.extend_from_slice(&parsed_word[..index - 1]);
            }
        }
    }

    Ok(out)
}

fn divmod(n: u32, m: u32) -> (u32, u32) {
    (n / m, n % m)
}

fn a85(n: u32) -> u8 {
    n as u8 + 0x21
}

fn base85_chunk(c: [u8; 4]) -> [u8; 5] {
    let n = u32::from_be_bytes(c);
    let (n, e) = divmod(n, 85);
    let (n, d) = divmod(n, 85);
    let (n, c) = divmod(n, 85);
    let (a, b) = divmod(n, 85);

    [a85(a), a85(b), a85(c), a85(d), a85(e)]
}

pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity((data.len() / 4) * 5 + 10);
    let mut chunks = data.chunks_exact(4);
    for chunk in chunks.by_ref() {
        let c: [u8; 4] = chunk.try_into().unwrap();
        if c == [0; 4] {
            buf.push(b'z');
        } else {
            buf.extend_from_slice(&base85_chunk(c));
        }
    }

    let r = chunks.remainder();
    if !r.is_empty() {
        let mut c = [0; 4];
        c[..r.len()].copy_from_slice(r);
        let out = base85_chunk(c);
        buf.extend_from_slice(&out[..r.len() + 1]);
    }
    buf.extend_from_slice(b"~>");
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successfull_decode() {
        let tests = vec![
            (&b""[..], &"<~~>"[..]),
            (&b""[..], &"~>"[..]),
            (&b""[..], &"<~"[..]),
            (&b"M"[..], &"9`~>"[..]),
            (&b"Ma"[..], &"9jn~>"[..]),
            (&b"Man"[..], &"9jqo~>"[..]),
            (&b"Man "[..], &"9jqo^~>"[..]),
            (&b"Man X"[..], &"9jqo^=9~>"[..]),
            (&b"Man XY"[..], &"9jqo^=BP~>"[..]),
            (&b"Man XYZ"[..], &"9jqo^=BSf~>"[..]),
            (&b"Man XYZ "[..], &"9jqo^=BSfM~>"[..]),
            (&[0; 4], &"z"[..]),
            (&[0; 4], &"<~z"[..]),
            (&[0; 4], &"z~>"[..]),
            (&[0; 4], &"<~z~>"[..]),
            (&[0; 16], &"zzzz"[..]),
        ];

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

    #[test]
    fn successfull_decode_big() {
        let decoded: Vec<u8> = (0..u8::MAX).into_iter().cycle().take(128).collect();
        let encoded = encode(&decoded[..]);
        assert!(decode(&encoded[..]).is_ok());
    }
}
