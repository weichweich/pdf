#[macro_use]
extern crate criterion;

use std::iter;

use criterion::Criterion;
use pdf_ascii85::decode;

const EXAMPLE_PLAIN: &[u8; 269] = b"Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.";


pub fn bench_decode(c: &mut Criterion) {
    c.bench_function("decode-empty", |b| b.iter(|| decode(b"<~~>")));

    let encoded = r#"<~9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,
    O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKY
    i(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIa
    l(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G
    >uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c~>"#
        .as_bytes();

    c.bench_function("decode-example", |b| b.iter(|| decode(encoded)));

    let decoded: Vec<u8> = (0..u8::MAX)
        .into_iter()
        .cycle()
        .take(10 * 1024 * 1024)
        .collect();
    let encoded = pdf_ascii85::encode(&decoded[..]);
    c.bench_function("decode-10mb", |b| b.iter(|| decode(&encoded[..])));

    let decoded: Vec<u8> = iter::repeat(b'z').take(10 * 1024 * 1024).collect();
    let encoded = pdf_ascii85::encode(&decoded[..]);
    c.bench_function("decode-z", |b| b.iter(|| decode(&encoded[..])));
}

pub fn bench_encode(c: &mut Criterion) {
    c.bench_function("encode-empty", |b| b.iter(|| decode(b"")));

    c.bench_function("encode-example", |b| b.iter(|| decode(&EXAMPLE_PLAIN[..])));

    let decoded: Vec<u8> = (0..u8::MAX)
        .into_iter()
        .cycle()
        .take(10 * 1024 * 1024)
        .collect();
    c.bench_function("encode-10mb", |b| b.iter(|| decode(&decoded[..])));

    let decoded: Vec<u8> = iter::repeat(0_u8).take(10 * 1024 * 1024).collect();
    c.bench_function("encode-z", |b| b.iter(|| decode(&decoded[..])));
}

criterion_group!(group_decode, bench_decode);
criterion_group!(group_encode, bench_encode);
criterion_main!(group_decode, group_encode);
