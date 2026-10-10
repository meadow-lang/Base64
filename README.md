# base64

Encode bytes as Base64 text and decode it back
([RFC 4648](https://www.rfc-editor.org/rfc/rfc4648)), for
[Meadow](https://github.com/meadow-lang/meadow).

This package is a port of Rust's
[`base64`](https://github.com/marshallpierce/rust-base64) 0.22.1. It has the
same alphabets and padding rules, and the same error messages.

## AI disclosure

Base64 is written with AI coding agents: Anthropic's Claude, through Claude
Code. Most of the code, the tests, the documentation and the commit messages in
this repository were written by an agent, under the direction of the project's
author, who decides the design and what goes in. Read it, and rely on it, with
that in mind.

## Install

```sh
meadow add meadow-lang/Base64
```

## Use

```meadow
use Base64 (standard, urlSafeNoPad, encodeString, decode, decodeErrorMessage)

def main =
  ( encodeString standard "hello",       -- "aGVsbG8="
    encodeString urlSafeNoPad "hello",   -- "aGVsbG8"
    match decode standard "aGVsbG8" with
    | Ok bytes -> bytesToString bytes
    | Err e -> decodeErrorMessage e      -- "Invalid padding"
  )
```

An **engine** is an alphabet plus a config. The four the crate names are
`standard`, `standardNoPad`, `urlSafe` and `urlSafeNoPad`. To build your own,
call `engine alphabet config` with:

- an alphabet: `standardAlphabet`, `urlSafeAlphabet`, `cryptAlphabet`,
  `bcryptAlphabet`, `imapMutf7Alphabet`, `binHexAlphabet`, or one returned by
  `alphabet "…"` (which checks that the string has 64 distinct printable
  characters and no `=`);
- a config: `pad`, `noPad`, or a `Config` record with `encodePadding`,
  `decodeAllowTrailingBits` and `decodePaddingMode` (`Indifferent`,
  `RequireCanonical` or `RequireNone`).

| function | |
|---|---|
| `encode engine bytes`, `encodeString engine s` | Base64 text |
| `decode engine text`, `decodeBytes engine bytes` | `Ok bytes`, or `Err` with a `DecodeError` (`InvalidByte`, `InvalidLength`, `InvalidLastSymbol` or `InvalidPadding`) |
| `decodeErrorMessage`, `alphabetErrorMessage` | the crate's error text |
| `encodedLen n padding`, `decodedLenEstimate n` | buffer sizes |

## How it's made

`src/Base64.mw` is a hand translation of the crate's general-purpose engine.
It reports the same errors at the same offsets. **`src/Cases.mw`** is generated
test data:

- 480 encodings;
- 6,000 decodings of damaged and undamaged input, under every alphabet and
  config;
- 400 candidate alphabets;
- 205 buffer-length calculations.

Every expected result comes from calling the crate. Run `scripts/generate.sh`
to regenerate; it needs a Rust toolchain.

## Licence

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your
option, like the crate. See [COPYRIGHT](COPYRIGHT).
