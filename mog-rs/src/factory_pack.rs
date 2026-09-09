//! Encode/decode for the embedded factory bundle.
//!
//! The factory library (mog scripts plus their fixture siblings) is embedded in
//! `mog.exe` at build time. Some of those fixtures are deliberately realistic
//! secret-shaped strings (fake AWS/Stripe tokens used by the redaction/scan
//! mogs' golden tests). Embedding them verbatim put token-shaped plaintext in
//! the shipped binary, which trips secret scanners on the published catalog.
//!
//! So the bundle is packed here: a simple archive, deflate-compressed, then
//! XORed with a SHA-256 CTR keystream. This is OBFUSCATION, not secrecy: the key
//! ships in the binary right next to the ciphertext, and anyone can reverse it.
//! The only goal is that the fixture bytes do not appear as plaintext in the
//! file, so scanners see compressed+XORed noise. `mog setup` decodes it back to
//! byte-identical files at install time.
//!
//! Archive format (little-endian), pre-compression:
//!   u32 file_count
//!   repeated file_count times:
//!     u32 path_len,  path_len bytes  (UTF-8, forward-slash separators)
//!     u32 data_len,  data_len bytes
//!
//! This module is compiled BOTH into the crate (via `mod factory_pack`) and into
//! `build.rs` (via `#[path = "src/factory_pack.rs"]`), so it must stay
//! self-contained: only `std`, `flate2`, and `sha2`, no crate-internal imports.

// Shared by the crate and build.rs. `encode` is used only by build.rs and
// `decode` only by the crate, so each half looks dead in the other compile.
#![allow(dead_code)]

use std::io::{Read, Write};
use std::path::PathBuf;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};

/// Fixed obfuscation key. NOT a secret: it ships in the binary alongside the
/// ciphertext, so it provides no confidentiality. Its only job is to keep the
/// embedded fixture bytes from appearing as plaintext in the shipped file.
const PACK_KEY: [u8; 32] = [
    0x9e, 0x3d, 0x1c, 0x77, 0xb2, 0x4a, 0x05, 0xe8, 0x61, 0xd0, 0x3f, 0x8c, 0x27, 0xaa, 0x56, 0x14,
    0xf1, 0x0b, 0x93, 0x6e, 0xc5, 0x82, 0x37, 0x49, 0xda, 0x1f, 0x60, 0xb7, 0x2c, 0x98, 0x04, 0xe3,
];

/// XOR `buf` in place with a SHA-256 CTR keystream: block `i` (64 bytes) is
/// `SHA256(PACK_KEY || i_le_u64)` repeated as needed. Symmetric: applying it
/// twice restores the input.
pub fn keystream_xor(buf: &mut [u8]) {
    let mut counter: u64 = 0;
    let mut off = 0;
    while off < buf.len() {
        let mut hasher = Sha256::new();
        hasher.update(PACK_KEY);
        hasher.update(counter.to_le_bytes());
        let block = hasher.finalize();
        for b in block.iter() {
            if off >= buf.len() {
                break;
            }
            buf[off] ^= b;
            off += 1;
        }
        counter += 1;
    }
}

/// Serialize `files` into the archive format, deflate it, then XOR-obfuscate.
/// `files` are `(forward-slash relative path, contents)` pairs.
pub fn encode(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut archive = Vec::new();
    archive.extend_from_slice(&(files.len() as u32).to_le_bytes());
    for (path, data) in files {
        let pb = path.as_bytes();
        archive.extend_from_slice(&(pb.len() as u32).to_le_bytes());
        archive.extend_from_slice(pb);
        archive.extend_from_slice(&(data.len() as u32).to_le_bytes());
        archive.extend_from_slice(data);
    }

    let mut enc = ZlibEncoder::new(Vec::new(), Compression::best());
    enc.write_all(&archive).expect("zlib compress");
    let mut packed = enc.finish().expect("zlib finish");

    keystream_xor(&mut packed);
    packed
}

/// Reverse [`encode`]: de-obfuscate, inflate, parse the archive back into
/// `(path, contents)` pairs. Panics on a malformed pack (the pack is produced by
/// this crate's own build, so corruption is a build bug, not user input).
pub fn decode(packed: &[u8]) -> Vec<(PathBuf, Vec<u8>)> {
    let mut buf = packed.to_vec();
    keystream_xor(&mut buf);

    let mut archive = Vec::new();
    ZlibDecoder::new(&buf[..])
        .read_to_end(&mut archive)
        .expect("zlib inflate factory pack");

    let mut out = Vec::new();
    let mut pos = 0usize;
    let read_u32 = |a: &[u8], pos: &mut usize| -> usize {
        let v = u32::from_le_bytes(a[*pos..*pos + 4].try_into().unwrap()) as usize;
        *pos += 4;
        v
    };
    let count = read_u32(&archive, &mut pos);
    for _ in 0..count {
        let plen = read_u32(&archive, &mut pos);
        let path = String::from_utf8(archive[pos..pos + plen].to_vec()).expect("utf8 path");
        pos += plen;
        let dlen = read_u32(&archive, &mut pos);
        let data = archive[pos..pos + dlen].to_vec();
        pos += dlen;
        out.push((PathBuf::from(path), data));
    }
    out
}
