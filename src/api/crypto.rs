use crypto::aes::{cbc_encryptor, KeySize};
use crypto::blockmodes::PkcsPadding;
use crypto::buffer;
use crypto::buffer::{ReadBuffer, WriteBuffer};
use crypto::buffer::{RefReadBuffer, RefWriteBuffer};
use num_bigint::BigInt;
use rustc_serialize::hex::ToHex;

pub fn aes_encrypt(data: String, key: &str) -> Vec<u8> {
    let mut encryptor = cbc_encryptor(
        KeySize::KeySize128,
        key.as_bytes(),
        "0102030405060708".as_bytes(),
        PkcsPadding,
    );

    let mut final_result = Vec::<u8>::new();
    let mut buffer = [0; 1024];
    let mut read_buffer = RefReadBuffer::new(data.as_bytes());
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor
            .encrypt(&mut read_buffer, &mut write_buffer, true)
            .unwrap();
        final_result.extend_from_slice(write_buffer.take_read_buffer().take_remaining());
        match result {
            buffer::BufferResult::BufferUnderflow => break,
            buffer::BufferResult::BufferOverflow => {}
        }
    }

    final_result
}

pub fn encrypt(text: &str, exponent: &str, modulus: &str) -> Option<String> {
    let mut rev = Vec::<u8>::new();
    for byt in text.as_bytes() {
        rev.push(*byt);
    }
    rev.reverse();

    let radix = 16;
    let bi_text = BigInt::parse_bytes(rev.to_hex().as_bytes(), radix)?;
    let bi_exp = BigInt::parse_bytes(exponent.as_bytes(), radix)?;
    let bi_mod = BigInt::parse_bytes(modulus.as_bytes(), radix)?;
    let bi_ret = bi_text.modpow(&bi_exp, &bi_mod);

    Some(bi_ret.to_str_radix(radix))
}
