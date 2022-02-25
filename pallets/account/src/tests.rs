use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

pub use aes_gcm::{AesGcm, Key, Nonce}; // Or `Aes128Gcm`
pub use aes_gcm::{NewAead};
use generic_array::ArrayLength;
//use aes::Aes256;

//#[cfg(feature = "aes")]
pub use aes_gcm::Aes256Gcm;
use aead::Buffer;
use typenum::U16;
use sp_std::vec::Vec;
pub use aes_gcm::AeadInPlace;
use sp_std::convert::From;

#[test]
fn decrypt_is_working() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(AccountModule::register(Origin::signed(1), Role::Organization, str2vec("OK")));
		let account_storage = AccountModule::account_storage(ALICE).unwrap();
		let key = Key::from_slice(b"an example very very secret key.");
		let cipher = Aes256Gcm::new(key);
		assert_eq!(account_storage.enkey.clone().unwrap(),"5174cdfe2ee1ef1d519a192ebcf4d6e8".as_bytes());
		let enkey: [u8;16] = account_storage.enkey.clone().unwrap().try_into().unwrap();
		//assert_eq!(enkey, *b"");
		//let nonce: generic_array::GenericArray<u8,U16>= *Nonce::from_slice(b"unique nonce");
		let nonce = Nonce::from_slice(b"unique nonce");  

		let mut buffer: Vec<u8> = Vec::new();
		buffer.extend_from_slice(b"plaintext message");
		let enkey_convert= generic_array::GenericArray::<u8,U16>::from_slice(&enkey);
		//assert_ok!()
		cipher.decrypt_in_place_detached(nonce, b"", &mut buffer, enkey_convert).expect("decryption failure!");
		assert_eq!(&buffer, b"plaintext message");

	});
}

