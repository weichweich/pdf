#[macro_use]
extern crate criterion;

use criterion::Criterion;
use pdf_ascii85::decode;

pub fn criterion_benchmark(c: &mut Criterion) {
    let encoded = r#"<~9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,
    O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKY
    i(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIa
    l(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G
    >uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c~>"#
        .as_bytes();

    c.bench_function("encode-small", |b| b.iter(|| decode(encoded)));

    let encoded: Vec<u8> = (0..u8::MAX)
        .into_iter()
        .cycle()
        .take(100 * 1024 * 1024)
        .collect();
    c.bench_function("encode-100m", |b| b.iter(|| decode(&encoded[..])));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
