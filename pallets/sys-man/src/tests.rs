//! pallet-sys-man functionalities test
use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system as system;
use pallet_utils::{Role, Status};
use serde_json::{json, Value};

fn str2vec(s: &str) -> Vec<u8> {
	s.as_bytes().to_vec()
}

fn generate_test_account(
	role: Role,
	level: Option<u8>,
	parent: Option<<mock::Test as system::Config>::AccountId>,
	children: Option<Vec<<mock::Test as system::Config>::AccountId>>,
) -> SysManAccount<Test> {
	let root_authority = SysManAccount::<Test> {
		role,
		status: Status::Active,
		level,
		parent,
		children,
		metadata: str2vec(
			r#"
			{
				"description": "Root authority",
        	}"#,
		),
	};

	root_authority
}

fn add_json_field(v: &Value, field_key: String, field_val: &str) -> Value {
	match v {
		Value::Object(map) => {
			let mut map = map.clone();

			map.insert(field_key.clone(), Value::String(field_val.to_string()));

			Value::Object(map)
		},
		v => v.clone(),
	}
}

#[test]
fn approve_sys_man_should_work() {
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let root_authority = generate_test_account(Role::SysMan, Some(0), None, None);

		// init genesis config
		SysMan::<Test>::insert(&id, root_authority);

		let metadata = str2vec(
			r#"
		{
			"description": "Root authority",
			"name": "new_sys_man",
			"email": "new_sys_man@gmail.com"
		}"#,
		);

		// Dispatch a signed extrinsic.
		assert_ok!(SysManModule::approve_sys_man(Origin::signed(1), 2u64, metadata.clone()));

		let _ = SysManModule::approve_sys_man(Origin::signed(1), 2u64, metadata.clone());

		let new_sys_man = SysMan::<Test>::get(2).unwrap();

		assert_eq!(new_sys_man.role, Role::SysMan);
		assert_eq!(new_sys_man.status, Status::Active);
		assert_eq!(new_sys_man.level, Some(1));
		assert_eq!(new_sys_man.metadata, metadata.clone());
	});
}

#[test]
fn approve_sys_man_should_fails() {
	// already_exists
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let root_authority = generate_test_account(Role::SysMan, Some(0), None, None);

		// init genesis config
		SysMan::<Test>::insert(&id, root_authority);

		let metadata = str2vec(
			r#"
		{
			"description": "Root authority",
			"name": "new_sys_man",
			"email": "new_sys_man@gmail.com"
		}"#,
		);

		assert_noop!(
			SysManModule::approve_sys_man(Origin::signed(1), 1u64, metadata.clone()),
			Error::<Test>::AlreadyRegistered
		);
	});

	// no authorization
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let mut root_authority = generate_test_account(Role::SysMan, Some(0), None, None);

		root_authority.role = Role::User;
		// init genesis config
		SysMan::<Test>::insert(&id, root_authority);

		let metadata = str2vec(
			r#"
		{
			"description": "Root authority",
			"name": "new_sys_man",
			"email": "new_sys_man@gmail.com"
		}"#,
		);

		assert_noop!(
			SysManModule::approve_sys_man(Origin::signed(1), 2u64, metadata.clone()),
			Error::<Test>::NoValidAuthorization
		);
	})
}

#[test]
fn revole_sys_man_should_work() {
	new_test_ext().execute_with(|| {
		let authority_id = 0u64;

		let revoke_target_id = 1u64;

		let authority = generate_test_account(Role::SysMan, Some(0), None, Some(vec![1u64]));

		let revoke_target = generate_test_account(Role::SysMan, Some(1), Some(0), None);

		SysMan::<Test>::insert(&authority_id, authority);

		SysMan::<Test>::insert(&revoke_target_id, revoke_target);

		let description = "violate management rules";

		assert_ok!(SysManModule::revoke_sys_man(
			Origin::signed(0),
			1u64,
			str2vec(description.clone())
		));

		let _ = SysManModule::revoke_sys_man(Origin::signed(0), 1u64, str2vec(description.clone()));

		assert_eq!(false, SysMan::<Test>::contains_key(&revoke_target_id));

		assert_eq!(true, SysManRevoked::<Test>::contains_key(&revoke_target_id));
	})
}

#[test]
fn revoke_sys_man_should_fail() {
	// not exist
	new_test_ext().execute_with(|| {
		let authority_id = 0u64;

		let revoke_target_id = 1u64;

		let authority = generate_test_account(Role::SysMan, Some(0), None, Some(vec![1u64]));

		let revoke_target = generate_test_account(Role::SysMan, Some(1), Some(0), None);

		SysMan::<Test>::insert(&authority_id, authority);

		SysMan::<Test>::insert(&revoke_target_id, revoke_target);

		let description = "violate management rules";

		assert_noop!(
			SysManModule::revoke_sys_man(Origin::signed(0), 2u64, str2vec(description.clone())),
			Error::<Test>::SysManNotExist
		);
	});

	//no authorization
	new_test_ext().execute_with(|| {
		let authority_id = 0u64;

		let revoke_target_id = 1u64;

		let authority = generate_test_account(Role::SysMan, Some(2), None, Some(vec![1u64]));

		let revoke_target = generate_test_account(Role::SysMan, Some(1), Some(0), None);

		SysMan::<Test>::insert(&authority_id, authority);

		SysMan::<Test>::insert(&revoke_target_id, revoke_target);

		let description = "violate management rules";

		assert_noop!(
			SysManModule::revoke_sys_man(Origin::signed(0), 1u64, str2vec(description.clone())),
			Error::<Test>::NoValidAuthorization
		);

		// already revoked
		new_test_ext().execute_with(|| {
			let authority_id = 0u64;

			let revoked_target_id = 1u64;

			let authority = generate_test_account(Role::SysMan, Some(0), None, Some(vec![1u64]));

			let revoked_target = generate_test_account(Role::SysMan, Some(1), Some(0), None);

			SysMan::<Test>::insert(&authority_id, authority);

			SysMan::<Test>::insert(&revoked_target_id, &revoked_target);

			SysManRevoked::<Test>::insert(&revoked_target_id, revoked_target);

			let description = "violate management rules";

			assert_noop!(
				SysManModule::revoke_sys_man(Origin::signed(0), 1u64, str2vec(description.clone())),
				Error::<Test>::AlreadyRevoked
			);
		})
	})
}

#[test]
fn approve_org_should_work() {
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let authority = generate_test_account(Role::SysMan, Some(0), None, None);

		// init genesis config
		SysMan::<Test>::insert(&id, authority);

		let metadata = str2vec(
			r#"
		{
			"description": "Organization",
			"name": "test_organization",
			"email": "test_organization@gmail.com",
			"address": "Hanoi, Vietnam",
			"phone": "0987654321"
		}"#,
		);

		// Dispatch a signed extrinsic.
		assert_ok!(SysManModule::approve_org(Origin::signed(1), 1u64, metadata.clone()));

		let _ = SysManModule::approve_org(Origin::signed(1), 1u64, metadata.clone());

		let new_org = Org::<Test>::get(1).unwrap();

		assert_eq!(new_org.role, Role::Organization);
		assert_eq!(new_org.status, Status::Active);
		assert_eq!(new_org.level, None);
		assert_eq!(new_org.metadata, metadata.clone());
	})
}

#[test]
fn approve_org_should_fails() {
	// no authorization
	new_test_ext().execute_with(|| {
		let metadata = str2vec(
			r#"
		{
			"description": "Organization",
			"name": "test_organization",
			"email": "test_organization@gmail.com",
			"address": "Hanoi, Vietnam",
			"phone": "0987654321"
		}"#,
		);

		// Dispatch a signed extrinsic.
		assert_noop!(
			SysManModule::approve_org(Origin::signed(1), 1u64, metadata.clone()),
			Error::<Test>::SysManNotExist
		);
	});

	// organization already approved
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let authority = generate_test_account(Role::SysMan, Some(0), None, None);

		let org = generate_test_account(Role::Organization, Some(0), None, None);

		// init genesis config
		SysMan::<Test>::insert(&id, authority);

		Org::<Test>::insert(&id, org);

		let metadata = str2vec(
			r#"
		{
			"description": "Organization",
			"name": "test_organization",
			"email": "test_organization@gmail.com",
			"address": "Hanoi, Vietnam",
			"phone": "0987654321"
		}"#,
		);

		// Dispatch a signed extrinsic.
		assert_noop!(
			SysManModule::approve_org(Origin::signed(1), id, metadata.clone()),
			Error::<Test>::AlreadyRegistered
		);
	});

	// organization already revoked
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let authority = generate_test_account(Role::SysMan, Some(0), None, None);

		let org = generate_test_account(Role::Organization, Some(0), None, None);

		// init genesis config
		SysMan::<Test>::insert(&id, authority);

		OrgRevoked::<Test>::insert(&id, org);

		let metadata = str2vec(
			r#"
		{
			"description": "Organization",
			"name": "test_organization",
			"email": "test_organization@gmail.com",
			"address": "Hanoi, Vietnam",
			"phone": "0987654321"
		}"#,
		);

		// Dispatch a signed extrinsic.
		assert_noop!(
			SysManModule::approve_org(Origin::signed(1), id, metadata.clone()),
			Error::<Test>::AlreadyRevoked
		);
	})
}

#[test]
fn revoke_org_should_work() {
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let authority = generate_test_account(Role::SysMan, Some(0), None, None);

		let mut org = generate_test_account(Role::Organization, None, None, None);

		// init genesis config
		SysMan::<Test>::insert(&id, authority);

		Org::<Test>::insert(&id, &org);

		let description = "organization remover";

		assert_ok!(SysManModule::revoke_org(
			Origin::signed(id.clone()),
			id.clone(),
			str2vec(description.clone())
		));

		let _ = SysManModule::revoke_org(
			Origin::signed(id.clone()),
			id.clone(),
			str2vec(description.clone()),
		);

		let mut metadata = json!(org.metadata);

		metadata = add_json_field(&metadata, "revoke_description".to_string(), description);

		org.metadata = metadata.to_string().as_bytes().to_vec();

		assert_eq!(org, OrgRevoked::<Test>::get(&id).unwrap());

		assert_eq!(false, Org::<Test>::contains_key(&id));
	})
}

#[test]
fn revoke_org_should_fail() {
	// revoke org not exist
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let authority = generate_test_account(Role::SysMan, Some(0), None, None);

		// init genesis config
		SysMan::<Test>::insert(&id, authority);

		let description = "organization remover";

		assert_noop!(
			SysManModule::revoke_org(Origin::signed(id), id, str2vec(description)),
			Error::<Test>::RevokedOrgNotExist
		);
	});

	// revokeOrg already revoked
	new_test_ext().execute_with(|| {
		let id = 1u64;
		let authority = generate_test_account(Role::SysMan, Some(0), None, None);

		let org = generate_test_account(Role::Organization, None, None, None);

		// init genesis config
		SysMan::<Test>::insert(&id, authority);

		Org::<Test>::insert(&id, &org);

		OrgRevoked::<Test>::insert(&id, org);

		let description = "organization remover";

		assert_noop!(
			SysManModule::revoke_org(Origin::signed(id), id, str2vec(description)),
			Error::<Test>::AlreadyRevoked
		);
	})
}
