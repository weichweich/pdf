use std::convert::TryInto;

/// The character `u` in the ASCII table represents the null byte (0x00).
const NULL_CHAR: u8 = b'u';

/// The character `z` in the ASCII table represents 4 null bytes (0x0000_0000).
const NULL_WORD: u8 = b'z';

const START_SEQUENCE: &[u8; 2] = b"<~";

const END_SEQUENCE: &[u8; 2] = b"~>";

fn sym_85(byte: u8) -> Option<u8> {
    match byte {
        b @ 0x21..=0x75 => Some(b - 0x21),
        _ => None,
    }
}

fn word_85([a, b, c, d, e]: [u8; 5]) -> Option<[u8; 4]> {
    fn s(b: u8) -> Option<u32> {
        sym_85(b).map(|n| n as u32)
    }
    let (a, b, c, d, e) = (s(a)?, s(b)?, s(c)?, s(d)?, s(e)?);
    let q = (((a * 85 + b) * 85 + c) * 85 + d) * 85 + e;
    Some(q.to_be_bytes())
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, ()> {
    let mut out = Vec::with_capacity((data.len() + 4) / 5 * 4);

    let mut stream = data.iter().filter(|&b| !b.is_ascii_whitespace());

    let mut buf = [NULL_CHAR; 5];
    let mut index = 0;

    // Check and skip the start sequence
    for _ in 0..START_SEQUENCE.len() {
        stream.next().map(|char| {
            buf[index] = *char;
            index += 1;
        });
    }
    if &buf[..2] == START_SEQUENCE {
        // reset buffer
        buf = [NULL_CHAR; 5];
        index = 0;
    }

    // parse the middle of the buffer
    for char in stream {
        match (char, index) {
            // null word shortcut
            (&NULL_WORD, 0) => out.extend_from_slice(&[0x00_u8; 4]),

            // null word must be aligned at the beginning of a 5 char sequence
            (&NULL_WORD, _) => return Err(()),

            (&b'~', _) => break,

            // fill the buffer with chars
            (char, i) if i < buf.len() => {
                buf[i] = *char;
                index += 1;
            }

            // the buffer is full. Parse the word and clear the buffer.
            (char, _) => {
                // process full buffer
                let parsed_word = word_85(buf).ok_or(())?;
                out.extend_from_slice(&parsed_word);

                buf = [NULL_CHAR; 5];
                if char != &END_SEQUENCE[0] {
                    // set index to 1 since we already have the first char of the next round.
                    buf[0] = *char;
                    index = 1;
                } else {
                    index = 0;
                    break;
                }
            }
        }
    }

    // parse remainder of the buffer
    if (1..buf.len()).contains(&index) && &buf[..2] != END_SEQUENCE {
        let last = word_85(buf).ok_or(())?;
        out.extend_from_slice(&last[..index - 1]);
    } else if index == 5 {
        let parsed_word = word_85(buf).ok_or(())?;
        out.extend_from_slice(&parsed_word);
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
            (&b"M"[..], &"9`"[..]),
            (&b"Ma"[..], &"9jn"[..]),
            (&b"Man"[..], &"9jqo"[..]),
            (&b"Man "[..], &"9jqo^"[..]),
            (&b"Man X"[..], &"9jqo^=9"[..]),
            (&[0; 4], &"z"[..]),
            (&[0; 4], &"<~z"[..]),
            (&[0; 4], &"z~>"[..]),
            (&[0; 4], &"<~z~>"[..]),
            (&[0; 16], &"zzzz"[..]),
        ];

        for (i, (plain, codec)) in tests.into_iter().enumerate() {
            let decoded = decode(codec.as_bytes());
            assert!(decoded.is_ok(), "Error in test case #{} ({})", i, codec);
            assert_eq!(plain, decoded.unwrap(), "Couldn't decode test case #{} ({})", i, codec);
        }
    }
}
