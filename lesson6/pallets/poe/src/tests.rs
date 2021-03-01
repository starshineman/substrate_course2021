use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 4, 5];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		assert_eq!(Proofs::<Test>::get(&claim), (1, system::Module::<Test>::block_number()));
	})
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 4, 5];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()), 
			Error::<Test>::ClaimAlreadyExist
		);
	})
}

#[test]
fn create_claim_failed_when_claim_too_short() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()), 
			Error::<Test>::ClaimTooShort
		);
	})
}

#[test]
fn create_claim_failed_when_claim_too_long() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
		
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()), 
			Error::<Test>::ClaimTooLong
		);
	})
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 4, 5];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
	})
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 4, 5];
		
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()), 
			Error::<Test>::ClaimNotExist
		);
	})
}

#[test]
fn revoke_claim_failed_when_not_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 4, 5];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()), 
			Error::<Test>::NotClaimOwner
		);
	})
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 4, 5];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), 2, claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim), (2, system::Module::<Test>::block_number()));
	})
}

#[test]
fn transfer_claim_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 4, 5];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(3), 2, claim.clone()), 
			Error::<Test>::NotClaimOwner
		);
	})
}

#[test]
fn transfer_claim_when_not_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2, 4, 5];
		
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), 2, claim.clone()), 
			Error::<Test>::ClaimNotExist
		);
	})
}