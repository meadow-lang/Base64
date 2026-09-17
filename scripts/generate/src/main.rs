//! Writes `src/Cases.mw` for base64.
//!
//! ```text
//! cargo run --release -- <package root>
//! ```
//!
//! Inputs, with what the crate makes of them: bytes encoded with every
//! alphabet and configuration; text decoded, valid and not; strings tried as
//! alphabets; and lengths. The library is ported by hand into `src/`, and the
//! crate's source is fingerprinted.

use base64::alphabet::{self, Alphabet};
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use base64::engine::{DecodePaddingMode, Engine};
use std::fmt::Write as _;
use std::path::PathBuf;

/// The crate version pinned in `Cargo.toml`.
const UPSTREAM_VERSION: &str = "0.22.1";

/// The fingerprint of the crate's source, which `src/` ports.
const SOURCES: u64 = 0x1e91_64e5_97a8_2171;

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "../..".into()));

    let print = fingerprint(include_str!(concat!(env!("OUT_DIR"), "/sources.rs.txt")));
    if print != SOURCES {
        eprintln!(
            "error: base64 is not the version src/ ports.\n\
             Compare its source in {} with the previous version, carry any change\n\
             into src/, then set SOURCES in scripts/generate/src/main.rs to\n\
             {print:#x}",
            env!("UPSTREAM_DIR")
        );
        std::process::exit(1);
    }

    let cases = cases();
    let path = root.join("src/Cases.mw");
    std::fs::write(&path, &cases).unwrap();
    eprintln!("wrote {} ({} bytes)", path.display(), cases.len());
}

/// FNV-1a: stable across builds, which `DefaultHasher` does not promise.
fn fingerprint(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

// --- encoding -----------------------------------------------------------------------

/// A number as `digits` base-64 digits, most significant first, each digit the
/// character `'0' + d`: `'0'` to `'o'`, one contiguous run of ASCII.
fn digits(out: &mut String, value: u64, digits: u32) {
    assert!(
        value < 1 << (6 * digits),
        "{value} does not fit in {digits} digits"
    );
    for k in (0..digits).rev() {
        out.push(char::from(b'0' + ((value >> (6 * k)) & 63) as u8));
    }
}

/// A string, as its length in bytes (3 digits) and then its bytes.
fn text(out: &mut String, s: &str) {
    digits(out, s.len() as u64, 3);
    out.push_str(s);
}

/// Bytes, as a string of their hex digits.
fn bytes(out: &mut String, b: &[u8]) {
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    text(out, &hex);
}

/// `text` as one Meadow string literal, broken with `\`-newline every `width`
/// characters. Only printable ASCII is written raw; a space that would start a
/// line is `\x20`, since a continuation drops leading whitespace.
fn long_literal(text: &str, width: usize) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / width * 4 + 2);
    out.push('"');
    for (i, c) in text.chars().enumerate() {
        let line_start = i > 0 && i % width == 0;
        if line_start {
            out.push_str("\\\n    ");
        }
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '$' => out.push_str("\\$"),
            ' ' if line_start => out.push_str("\\x20"),
            ' '..='~' => out.push(c),
            _ => {
                let _ = write!(out, "\\u{{{:X}}}", u32::from(c));
            }
        }
    }
    out.push('"');
    out
}

// --- cases --------------------------------------------------------------------------

/// A small deterministic generator, so that the cases are the same on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.next() as u8).collect()
    }
}

const ALPHABETS: [&Alphabet; 6] = [
    &alphabet::STANDARD,
    &alphabet::URL_SAFE,
    &alphabet::CRYPT,
    &alphabet::BCRYPT,
    &alphabet::IMAP_MUTF7,
    &alphabet::BIN_HEX,
];

/// An engine as `Tests.mw` rebuilds it: alphabet, encode padding, allow
/// trailing bits, padding mode.
#[derive(Clone, Copy)]
struct Spec(usize, bool, bool, usize);

impl Spec {
    fn random(rng: &mut Rng) -> Spec {
        Spec(rng.below(6), rng.chance(50), rng.chance(30), rng.below(3))
    }

    fn engine(self) -> GeneralPurpose {
        let mode = [
            DecodePaddingMode::Indifferent,
            DecodePaddingMode::RequireCanonical,
            DecodePaddingMode::RequireNone,
        ][self.3];
        let config = GeneralPurposeConfig::new()
            .with_encode_padding(self.1)
            .with_decode_allow_trailing_bits(self.2)
            .with_decode_padding_mode(mode);
        GeneralPurpose::new(ALPHABETS[self.0], config)
    }

    fn write(self, out: &mut String) {
        digits(out, self.0 as u64, 1);
        digits(out, u64::from(self.1), 1);
        digits(out, u64::from(self.2), 1);
        digits(out, self.3 as u64, 1);
    }
}

/// Text to decode: an encoding, often damaged.
fn random_encoded(rng: &mut Rng, spec: Spec) -> Vec<u8> {
    let n = rng.below(40);
    let source = rng.bytes(n);
    let mut e = spec.engine().encode(&source).into_bytes();
    let symbols = ALPHABETS[spec.0].as_str().as_bytes();
    let damage = rng.below(10);
    for _ in 0..damage {
        if e.is_empty() {
            e.push(b'=');
            continue;
        }
        let at = rng.below(e.len() + 1);
        match rng.below(8) {
            0 => {
                e.remove(at.min(e.len() - 1));
            }
            1 => e.insert(at, b'='),
            2 => e.insert(at, symbols[rng.below(64)]),
            3 => e.push(b'='),
            4 => e.insert(at, rng.next() as u8),
            5 => e.insert(at, b' '),
            6 => {
                let n = e.len();
                e.truncate(n - 1);
            }
            _ => {
                // Change the last symbol, often leaving trailing bits set.
                let n = e.len();
                let last = e.iter().rposition(|b| *b != b'=').unwrap_or(n - 1);
                e[last] = symbols[rng.below(64)];
            }
        }
    }
    if rng.chance(10) {
        let n = rng.below(12);
        e = rng
            .bytes(n)
            .iter()
            .map(|b| b"A/+=_-.x\n"[*b as usize % 9])
            .collect();
    }
    e
}

fn random_alphabet(rng: &mut Rng) -> String {
    let base = ALPHABETS[rng.below(6)].as_str().to_string();
    let mut b = base.into_bytes();
    match rng.below(6) {
        0 => {
            let i = rng.below(64);
            b[i] = b'=';
        }
        1 => {
            let i = rng.below(64);
            let j = rng.below(64);
            b[i] = b[j];
        }
        2 => {
            let i = rng.below(64);
            b[i] = [0u8, 31, 127, b'\t', b'~', b' '][rng.below(6)];
        }
        3 => {
            b.truncate(rng.below(70));
        }
        4 => b.push(b'!'),
        _ => b.swap(rng.below(64), rng.below(64)),
    }
    String::from_utf8(b).unwrap()
}

fn cases() -> String {
    let mut rng = Rng(0xba5e_6464_c0ff_ee42);
    let mut body = String::new();
    let mut counts = [0usize; 4];

    // Kind 0: bytes encoded.
    for n in 0..60 {
        for _ in 0..8 {
            let spec = Spec::random(&mut rng);
            let input = rng.bytes(n);
            counts[0] += 1;
            digits(&mut body, 0, 1);
            spec.write(&mut body);
            bytes(&mut body, &input);
            text(&mut body, &spec.engine().encode(&input));
        }
    }

    // Kind 1: text decoded.
    for _ in 0..6000 {
        let spec = Spec::random(&mut rng);
        let input = random_encoded(&mut rng, spec);
        counts[1] += 1;
        digits(&mut body, 1, 1);
        spec.write(&mut body);
        bytes(&mut body, &input);
        match spec.engine().decode(&input) {
            Ok(b) => {
                digits(&mut body, 1, 1);
                bytes(&mut body, &b);
            }
            Err(e) => {
                digits(&mut body, 0, 1);
                text(&mut body, &e.to_string());
            }
        }
    }

    // Kind 2: alphabets.
    for _ in 0..400 {
        let candidate = random_alphabet(&mut rng);
        counts[2] += 1;
        digits(&mut body, 2, 1);
        text(&mut body, &candidate);
        match Alphabet::new(&candidate) {
            Ok(a) => {
                digits(&mut body, 1, 1);
                text(&mut body, a.as_str());
            }
            Err(e) => {
                digits(&mut body, 0, 1);
                text(&mut body, &e.to_string());
            }
        }
    }

    // Kind 3: lengths.
    for n in (0..200).chain([1000, 4095, 4096, 65537, 1 << 30]) {
        counts[3] += 1;
        digits(&mut body, 3, 1);
        digits(&mut body, n as u64, 6);
        for padding in [false, true] {
            digits(
                &mut body,
                base64::encoded_len(n, padding).unwrap() as u64,
                6,
            );
        }
        #[allow(deprecated)]
        digits(&mut body, base64::decoded_len_estimate(n) as u64, 6);
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "-- GENERATED by scripts/generate.sh from base64 {UPSTREAM_VERSION}.
-- Do not edit: run the script again instead.
--
-- Inputs, with what the crate makes of them, for `Tests.mw`: {} encodings,
-- {} decodings, {} alphabets and {} lengths.
--
-- Copyright Marshall Pierce, and the Meadow port's authors.
-- Dual-licensed under Apache-2.0 or MIT: see COPYRIGHT.

-- Each case starts with its kind (1 base-64 digit), and its fields follow in
-- the order `Tests.mw` reads them. A string is its length in bytes (3 digits)
-- and then its bytes; bytes are a string of their hex digits; an engine is the
-- alphabet (standard, URL-safe, crypt, bcrypt, IMAP, BinHex), encode padding,
-- allow trailing bits and the padding mode (indifferent, canonical, none), a
-- digit each.
@cfg(test)
@pub(pkg) def cases =
  {}",
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        long_literal(&body, 96)
    );
    out
}
