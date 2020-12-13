use generic_array::{ArrayLength, GenericArray, typenum::Unsigned};
use crate::{BlockCipher, NewBlockCipher, errors::InvalidLength};

/// Key for an algorithm that implements [`NewCipher`].
pub type Key<C> = GenericArray<u8, <C as NewCipher>::KeySize>;

/// Nonce for an algorithm that implements [`NewCipher`].
pub type Nonce<C> = GenericArray<u8, <C as NewCipher>::NonceSize>;

/// Cipher creation trait.
///
/// It can be used for creation of block modes, synchronous and asynchronous stream ciphers.
pub trait NewCipher: Sized {
    /// Key size in bytes
    type KeySize: ArrayLength<u8>;

    /// Nonce size in bytes
    type NonceSize: ArrayLength<u8>;

    /// Create new stream cipher instance from variable length key and nonce.
    fn new(key: &Key<Self>, nonce: &Nonce<Self>) -> Self;

    /// Create new stream cipher instance from variable length key and nonce.
    #[inline]
    fn new_var(key: &[u8], nonce: &[u8]) -> Result<Self, InvalidLength> {
        let kl = Self::KeySize::to_usize();
        let nl = Self::NonceSize::to_usize();
        if key.len() != kl || nonce.len() != nl {
            Err(InvalidLength)
        } else {
            let key = GenericArray::from_slice(key);
            let nonce = GenericArray::from_slice(nonce);
            Ok(Self::new(key, nonce))
        }
    }
}

/// Trait for initializing a stream cipher from a block cipher
pub trait FromBlockCipher {
    /// Block cipher
    type BlockCipher: BlockCipher;
    /// Nonce size in bytes
    type NonceSize: ArrayLength<u8>;

    /// Instantiate a stream cipher from a block cipher
    fn from_block_cipher(
        cipher: Self::BlockCipher,
        nonce: &GenericArray<u8, Self::NonceSize>,
    ) -> Self;
}

impl<C> NewCipher for C
where
    C: FromBlockCipher,
    C::BlockCipher: NewBlockCipher,
{
    type KeySize = <<Self as FromBlockCipher>::BlockCipher as NewBlockCipher>::KeySize;
    type NonceSize = <Self as FromBlockCipher>::NonceSize;

    fn new(key: &Key<Self>, nonce: &Nonce<Self>) -> C {
        C::from_block_cipher(
            <<Self as FromBlockCipher>::BlockCipher as NewBlockCipher>::new(key),
            nonce,
        )
    }

    fn new_var(key: &[u8], nonce: &[u8]) -> Result<Self, InvalidLength> {
        if nonce.len() != Self::NonceSize::USIZE {
            Err(InvalidLength)
        } else {
            C::BlockCipher::new_var(key)
                .map_err(|_| InvalidLength)
                .map(|cipher| {
                    let nonce = GenericArray::from_slice(nonce);
                    Self::from_block_cipher(cipher, nonce)
                })
        }
    }
}