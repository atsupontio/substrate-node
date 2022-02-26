use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce}; // Or `Aes128Gcm`
									  //use aes::Aes256;

//#[cfg(feature = "aes")]
use sp_std::vec::Vec;
// use typenum::U16;

#[test]
fn decrypt_is_working() {
	new_test_ext().execute_with(|| {
		const SENDER: u64 = 1;
		// Dispatch a signed extrinsic.
		// assert_ok!(AccountModule::register(
		// 	Origin::signed(SENDER.clone()),
		// 	Role::Organization,
		// 	str2vec("OK")
		// ));
		let _ = AccountModule::register(
			Origin::signed(SENDER.clone()),
			Role::Organization,
			str2vec("OK"),
		);
		let plaintext = b"plaintext message";
		let account_storage = AccountModule::account_storage(SENDER).unwrap();
		let key = Key::from_slice(b"an example very very secret key.");
		let cipher = Aes256Gcm::new(key);

		let enkey: Vec<u8> = account_storage.enkey.clone().unwrap();

		let nonce = Nonce::from_slice(b"unique nonce");
		// buffer.extend_from_slice(b"plaintext message");
		// let enkey_convert = generic_array::GenericArray::<u8, U16>::from_slice(&enkey);
		//assert_ok!()

		// let encrypted = cipher
		// 	.encrypt_in_place_detached(nonce, b"associated", &mut buffer)
		// 	.expect("encryption failure!");

		let decrypted = cipher.decrypt(nonce, enkey.as_ref()).expect("decryption failure!");

		assert_eq!(decrypted, plaintext);
	});
}
