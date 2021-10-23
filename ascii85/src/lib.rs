use std::convert::TryInto;

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

    let mut stream = data
        .iter()
        .filter(|&b| !matches!(b, b' ' | b'\n' | b'\r' | b'\t'));

    let mut symbols = stream.by_ref().take_while(|&b| b != &b'~');

    let (tail_len, tail) = loop {
        match symbols.next() {
            Some(b'z') => out.extend_from_slice(&[0; 4]),
            Some(a) => {
                let (b, c, d, e) = match (
                    symbols.next(),
                    symbols.next(),
                    symbols.next(),
                    symbols.next(),
                ) {
                    (Some(b), Some(c), Some(d), Some(e)) => (b, c, d, e),
                    (None, _, _, _) => break (1, [*a, b'u', b'u', b'u', b'u']),
                    (Some(b), None, _, _) => break (2, [*a, *b, b'u', b'u', b'u']),
                    (Some(b), Some(c), None, _) => break (3, [*a, *b, *c, b'u', b'u']),
                    (Some(b), Some(c), Some(d), None) => break (4, [*a, *b, *c, *d, b'u']),
                };
                out.extend_from_slice(&word_85([*a, *b, *c, *d, *e]).ok_or(())?);
            }
            None => break (0, [b'u'; 5]),
        }
    };

    if tail_len > 0 {
        let last = word_85(tail).ok_or(())?;
        out.extend_from_slice(&last[..tail_len - 1]);
    }

    match (stream.next(), stream.next()) {
        (Some(b'>'), None) => Ok(out),
        _ => Err(()),
    }
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
    fn base_85() {
        fn s(b: &[u8]) -> &str {
            std::str::from_utf8(b).unwrap()
        }
    
        let case = &b"hello world!"[..];
        let encoded = encode(case);
        assert_eq!(s(&encoded), "BOu!rD]j7BEbo80~>");
        let decoded = decode(&encoded).unwrap();
        assert_eq!(case, &*decoded);
    }
}
