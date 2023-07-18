use digest::dev::{feed_rand_16mib, fixed_reset_test};
use hex_literal::hex;
use md4::{Digest, Ed2k, Md4};

digest::new_test!(md4_main, "md4", Md4, fixed_reset_test);

#[test]
fn md4_rand() {
    let mut h = Md4::new();
    feed_rand_16mib(&mut h);
    assert_eq!(
        h.finalize()[..],
        hex!("07345abfb6192d85bf6a211381926120")[..]
    );
}

// https://wiki.anidb.net/Ed2k-hash#How_is_an_ed2k_hash_calculated_exactly?
#[test]
fn ed2k_9728000() {
    let s = hex!("00");
    let mut h = Ed2k::new();
    let n = 9728000;
    let mut content = Vec::with_capacity(s.len() * n);
    for _ in 0..n {
        content.extend(&s);
    }
    assert_eq!(content.len(), 9728000_usize);
    h.update(content);
    assert_eq!(
        h.finalize()[..],
        hex!("d7def262a127cd79096a108e7a9fc138")[..]
    );
}

// https://wiki.anidb.net/Ed2k-hash#How_is_an_ed2k_hash_calculated_exactly?
#[test]
fn ed2k_19456000() {
    let s = hex!("00");
    let mut h = Ed2k::new();
    let n = 19456000;
    let mut content = Vec::with_capacity(s.len() * n);
    for _ in 0..n {
        content.extend(&s);
    }
    assert_eq!(content.len(), 19456000_usize);
    h.update(content);
    assert_eq!(
        h.finalize()[..],
        hex!("194ee9e4fa79b2ee9f8829284c466051")[..]
    );
}
