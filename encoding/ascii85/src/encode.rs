use std::convert::TryInto;

use crate::{END_SEQUENCE, NULL_WORD};

fn divmod(n: u32, m: u32) -> (u32, u32) {
    (n / m, n % m)
}

fn a85(n: u32) -> u8 {
    n as u8 + 0x21
}

fn encode_word(c: [u8; 4]) -> [u8; 5] {
    let n = u32::from_be_bytes(c);
    let (n, e) = divmod(n, 85);
    let (n, d) = divmod(n, 85);
    let (n, c) = divmod(n, 85);
    let (a, b) = divmod(n, 85);

    [a85(a), a85(b), a85(c), a85(d), a85(e)]
}

pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity((data.len() / 4) * 5 + 10);
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();
    for chunk in chunks {
        let c: [u8; 4] = chunk.try_into().unwrap();
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
            assert_eq!(
                encoded,
                codec,
                "Couldn't encode test case #{} ({})",
                i,
                codec
            );
        }
    }
}
