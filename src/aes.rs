use core::mem::size_of;

use aes_crypto::{Aes128Dec, Aes128Enc, AesBlock, AesDecrypt, AesEncrypt};
use rijndael::{key::AES128Key, schedule::KeySchedule128};

use crate::pi::{pi, Pi};
use crate::types::*;

pub const AES_128_BLOCK_SIZE: usize = 16;

pub fn decrypt(ciphertext: &[u8], plaintext: &mut [u8], key: AesKey, iv: AesIv) {
    let dec = Aes128Dec::from(key);

    for block in 0..(ciphertext.len() / AES_128_BLOCK_SIZE) {
        let index = block * AES_128_BLOCK_SIZE;

        let arr: [_; AES_128_BLOCK_SIZE] = ciphertext[index..index + AES_128_BLOCK_SIZE]
            .try_into()
            .unwrap();

        dec.decrypt_block(AesBlock::new(arr))
            .store_to(&mut plaintext[index..index + AES_128_BLOCK_SIZE]);

        for i in 0..AES_128_BLOCK_SIZE {
            plaintext[index + i] ^= if block == 0 {
                &iv
            } else {
                &ciphertext[index - AES_128_BLOCK_SIZE..index]
            }[i]
        }
    }
}

pub fn encrypt(plaintext: &[u8], ciphertext: &mut [u8], key: AesKey, iv: AesIv) {
    let enc = Aes128Enc::from(key);

    for block in 0..(ciphertext.len() / AES_128_BLOCK_SIZE) {
        let index = block * AES_128_BLOCK_SIZE;

        let mut arr: [_; AES_128_BLOCK_SIZE] = plaintext[index..index + AES_128_BLOCK_SIZE]
            .try_into()
            .unwrap();

        for i in 0..AES_128_BLOCK_SIZE {
            arr[i] ^= if block == 0 {
                &iv
            } else {
                &plaintext[index - AES_128_BLOCK_SIZE..index]
            }[i]
        }

        enc.encrypt_block(AesBlock::new(arr))
            .store_to(&mut ciphertext[index..index + AES_128_BLOCK_SIZE]);
    }
}

pub fn set_key_iv(key: &[u8], iv: &[u8]) {
    let pi = pi();

    for (index, key) in KeySchedule128::expand(AES128Key::try_from(key).unwrap())
        .to_dec()
        .iter()
        .enumerate()
    {
        for word_index in 0..4 {
            pi.set_aes_expanded_key(
                ((index * 4 + word_index) * size_of::<u32>()) as u32,
                key[word_index],
            );
        }
    }

    for (index, word) in iv.chunks_exact(size_of::<u32>()).enumerate() {
        pi.set_aes_iv(
            (index * size_of::<u32>()) as u32,
            u32::from_be_bytes(word.try_into().unwrap()),
        )
    }
}

impl Pi {
    pub fn run_aes(&mut self, continuation: bool) {
        let ctrl = 0x80000000 | if continuation { 1 } else { 0x9A } | (((0x200 / 0x10) - 1) << 16);

        self.set_bb_aes_ctrl(ctrl);
    }

    pub fn aes_wait(&self) {
        while self.bb_aes_ctrl() & 0x80000000 != 0 {}
    }
}
