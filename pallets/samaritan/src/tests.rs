use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
	// 	// Dispatch a signed extrinsic.
	// 	assert_ok!(Samaritan::do_something(Origin::signed(1), 42));
	// 	// Read pallet storage and assert an expected result.
	// 	assert_eq!(TemplateModule::something(), Some(42));
	});
}

// Things you need to _write_ test with
// assert_ok! e.g.: assert_ok!(Nicks::set_name(RuntimeOrigin::signed(2), b"Dave".to_vec()));
// assert_eq! e.g.: assert_eq!(Balances::total_balance(&2), 8);
// assert_noop! e.g.: assert_noop!(Nicks::set_name(RuntimeOrigin::signed(2), b"Dr. David Brubeck, III".to_vec()), Error::<Test>::TooLong,);



#[test]
fn create_samaritan_works() {

	let samaritan: Samaritan = sam;

	let name = name_too_long;
	let did_str = did_too_long;
	let did_str = string;
	let hash = hash;

	assert_noop!(Samaritan::create_samaritan(Origin::signed(1), name_too_long, did_str, hash), 
		Error::<Test>::NameOverflow);

	assert_noop!(Samaritan::create_samaritan(Origin::signed(1), name_too_long, did_str, hash), 
	Error::<Test>::DIDLengthOverflow);

}



#[test]
fn x() {}

#[test]
fn y() {}

#[test]
fn z() {}

