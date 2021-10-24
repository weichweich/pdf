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

fn word_85([a, b, c, d, e]: [u8; 5]) -> [u8; 4] {
    let q = (((a as u32 * 85 + b as u32) * 85 + c as u32) * 85 + d as u32) * 85 + e as u32;
    q.to_be_bytes()
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, ()> {
    let mut out = Vec::with_capacity((data.len() + 4) / 5 * 4);

    let mut stream = data.iter().filter(|&b| !b.is_ascii_whitespace());

    let mut symbols = stream.by_ref().take_while(|&b| b != &b'~');

    let (tail_len, tail) = loop {
        match symbols.next() {
            Some(&NULL_WORD) => out.extend_from_slice(&[0; 4]),
            Some(&a) => {
                let a = sym_85(a).ok_or(())?;
                let (b, c, d, e) = match (
                    symbols.next().map(|n| sym_85(*n)).flatten(),
                    symbols.next().map(|n| sym_85(*n)).flatten(),
                    symbols.next().map(|n| sym_85(*n)).flatten(),
                    symbols.next().map(|n| sym_85(*n)).flatten(),
                ) {
                    (Some(b), Some(c), Some(d), Some(e)) => (b, c, d, e),
                    (None, _, _, _) => break (1, [a, 0, 0, 0, 0]),
                    (Some(b), None, _, _) => break (2, [a, b, 0, 0, 0]),
                    (Some(b), Some(c), None, _) => break (3, [a, b, c, 0, 0]),
                    (Some(b), Some(c), Some(d), None) => break (4, [a, b, c, d, 0]),
                };
                out.extend_from_slice(&word_85([a, b, c, d, e]));
            }
            None => break (0, [0; 5]),
        }
    };

    if tail_len > 0 {
        let last = word_85(tail);
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

    const EXAMPLE_CODEC: &str = r#"<~9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,
    O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKY
    i(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIa
    l(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G
    >uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c~>"#;
    const EXAMPLE_PLAIN: &[u8; 269] = b"Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.";

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
            // (&[0; 4], &"z"[..]),
            (&[0; 4], &"<~z"[..]),
            // (&[0; 4], &"z~>"[..]),
            (&[0; 4], &"<~z~>"[..]),
            // (&[0; 16], &"zzzz"[..]),
            (EXAMPLE_PLAIN, EXAMPLE_CODEC),
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
}
