//! # ASCII85 Encoding
//!
//! pdf-ascii85 implements ASCII85 encoding. Arbitrary bytes are mapped to the
//! characters in range `b'!'..b'u'`. For more details on the encoding scheme refere to [Wikipedia](https://en.wikipedia.org/wiki/Ascii85).
//!
//! The characted `z` encodes four zero bytes, whitespaces are skipped, start
//! (`<~`) and ending sequences (`~>`) are optional while decoding.
//!
//! ## Decode Example
//!
//! ```
//! use pdf_ascii85::decode;
//! const EXAMPLE_CODEC: &str = r#"<~9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKYi(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIal(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G>uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c~>"#;
//!
//! println!("{:?}", decode(EXAMPLE_CODEC.as_bytes()).unwrap());
//! ```
//!
//! ## Encode Example
//!
//! ```
//! use pdf_ascii85::encode;
//! const EXAMPLE_PLAIN: &[u8; 269] = b"Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.";
//!
//! println!("{}", String::from_utf8(encode(&EXAMPLE_PLAIN[..])).unwrap());
//! ```

mod decode;
mod encode;

pub use decode::{decode, DecodeError};
pub use encode::encode;

/// The character `u` in the ASCII table represents the null byte (0x00).
const NULL_CHAR: u8 = b'u';

/// The character `z` in the ASCII table represents 4 null bytes (0x0000_0000).
const NULL_WORD: u8 = b'z';

const START_SEQUENCE: &[u8; 2] = b"<~";

const END_SEQUENCE: &[u8; 2] = b"~>";

#[cfg(test)]
mod tests {
    pub(crate) const EXAMPLE_CODEC: &str = r#"9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKYi(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIal(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G>uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c~>"#;
    pub(crate) const EXAMPLE_PLAIN: &[u8; 269] = b"Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.";

    pub(crate) fn decode_samples() -> Vec<(&'static [u8], &'static str)> {
        return vec![
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
            (EXAMPLE_PLAIN, EXAMPLE_CODEC),
        ];
    }

    pub(crate) fn encode_samples() -> Vec<(&'static [u8], &'static str)> {
        return vec![
            (&b""[..], &"~>"[..]),
            (&b"M"[..], &"9`~>"[..]),
            (&b"Ma"[..], &"9jn~>"[..]),
            (&b"Man"[..], &"9jqo~>"[..]),
            (&b"Man "[..], &"9jqo^~>"[..]),
            (&b"Man X"[..], &"9jqo^=9~>"[..]),
            (&[0; 4], &"z~>"[..]),
            (&[0; 16], &"zzzz~>"[..]),
            (EXAMPLE_PLAIN, EXAMPLE_CODEC),
        ];
    }
}
