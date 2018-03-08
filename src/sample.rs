#![no_std]
#![allow(non_snake_case)]
#![feature(alloc)]
#![feature(proc_macro)]

extern crate parity_hash;
extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate alloc;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;
/// Bigint used for 256-bit arithmetic
extern crate bigint;

pub mod donation {
	use parity_hash::{H256, Address};
	use pwasm_ethereum::{read, write, sender, value};
	use bigint::U256;

	use pwasm_abi_derive::eth_abi;
	use alloc::Vec;

	static TOTAL_DONATED_KEY: H256 = H256([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
	static TOP_DONOR_KEY: H256 = H256([1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);

	#[eth_abi(DonationEndpoint, DonationClient)]
	pub trait DonationContract {
		/// The constructor
		fn constructor(&mut self);
		/// Total amount of donations
		#[constant]
		fn totalDonations(&mut self) -> U256;
		/// See who the top donor is
		#[constant]
		fn topDonor(&mut self) -> Address;
		/// Donate, whatever balance you send will be the donated amount
		fn donate(&mut self);
		/// Event declaration
		#[event]
		fn Donation(&mut self, indexed_from: Address, _value: U256);
	}

	pub struct DonationContractInstance;

	impl DonationContract for DonationContractInstance {
		fn constructor(&mut self) {
			write(&TOTAL_DONATED_KEY, &U256::from(0).into());
		}

		fn totalDonations(&mut self) -> U256 {
			read(&TOTAL_DONATED_KEY).into()
		}

		fn topDonor(&mut self) -> Address {
			let top_donor: H256 = read(&TOP_DONOR_KEY).into();
			Address::from(top_donor)
		}

		fn donate(&mut self) {
			let sender = sender();
			let donation = value();
			let total: U256 = read(&TOTAL_DONATED_KEY).into();
			write(&TOTAL_DONATED_KEY, &(total + donation).into());
			setTopDonor(sender, donation);
			self.Donation(sender, donation);
		}

	}

	fn setTopDonor(sender: Address, amount: U256) {
		let existing_top = read(&TOP_DONOR_KEY).into();
		if amount > existing_top {
			write(&TOP_DONOR_KEY, &H256::from(&sender).into());
		}
	}
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
	let mut endpoint = donation::DonationEndpoint::new(donation::DonationContractInstance{});
	// Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
	pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
	let mut endpoint = donation::DonationEndpoint::new(donation::DonationContractInstance{});
	endpoint.dispatch_ctor(&pwasm_ethereum::input());
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
	extern crate pwasm_test;
	extern crate std;
	use super::*;
	use self::pwasm_test::{ext_update, ext_get};
	use parity_hash::Address;
	use donation::DonationContract;

	#[test]
	fn should_succeed_donating_and_tally_up_correctly() {
		let mut contract = donation::DonationContractInstance{};
		let sender_one = Address::from("0xdeadbeef00000000000000000000000000000000");
		let sender_two = Address::from("0xca7cafe000000000000000000000000000000000");
		// Here we're creating an External context using ExternalBuilder and set the `sender` to the `owner_address`
		// so `pwasm_ethereum::sender()` in DonationContract::constructor() will return that `owner_address`
		ext_update(|e| e
			.sender(sender_one.clone())
			.value(500.into())
		);
		contract.constructor();
		assert_eq!(contract.totalDonations(), 0.into());
		contract.donate();
		assert_eq!(contract.totalDonations(), 500.into());
		
		ext_update(|e| e
			.sender(sender_two.clone())
			.value(250.into())
		);
		contract.donate();
		assert_eq!(contract.totalDonations(), 750.into());
		assert_eq!(contract.topDonor(), sender_one);
		// 2 log entries should be created
		//assert_eq!(ext_get().logs().len(), 2);
	}
}
