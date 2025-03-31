// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry. If not, see <https://www.gnu.org/licenses/>.

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use std::vec::Vec;

pub enum AbortStrategy<F> {
	// Immediately terminate the loop
	FailFast,
	// Terminate the loop based on the predicate function
	ContinueUntil(F),
	// Continue the loop until the end of the items
	ContinueUntilEnd,
}

pub enum LoopControls {
	Break,
	Continue,
}

pub fn loop_with_abort_strategy<F, T, E>(
	// List of items to iterate over
	items: Vec<T>,
	// Closure to perform action, returns a LoopControls value indicating whether to exit the loop
	mut action: impl FnMut(&T) -> Result<LoopControls, E>,
	// Control when to abort the loop
	abort_strategy: AbortStrategy<F>,
) -> Result<(), Vec<E>>
where
	F: Fn(&T) -> bool, // Type of the predicate function, takes a parameter of type T and returns a boolean
{
	let mut errors: Vec<E> = Vec::new();
	for (index, item) in items.iter().enumerate() {
		match action(item) {
			Ok(control) => match control {
				LoopControls::Break => break,
				LoopControls::Continue => continue,
			},
			Err(err) => {
				errors.push(err);
				match abort_strategy {
					AbortStrategy::FailFast => return Err(errors), // If FailFast is chosen, return the error immediately
					AbortStrategy::ContinueUntil(ref predicate) => {
						// If ContinueUntil is chosen, decide whether to return the error based on the predicate function or return the error when processing the last item
						if predicate(item) || index == items.len() - 1 {
							return Err(errors);
						}
					},
					AbortStrategy::ContinueUntilEnd => {
						// If ContinueUntilEnd is chosen, return the error when processing the last item
						if index == items.len() - 1 {
							return Err(errors);
						}
					},
				}
			},
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use litentry_primitives::{ErrorDetail as Error, ErrorString};

	use super::*;

	#[test]
	fn test_break() {
		let test_array = vec!["item1", "item2", "item3", "item4"];
		let mut result = "";

		let loop_result: Result<(), Vec<Error>> = loop_with_abort_strategy(
			test_array,
			|item| {
				result = *item;
				if *item == "item2" {
					return Ok(LoopControls::Break);
				}

				Ok(LoopControls::Continue)
			},
			AbortStrategy::FailFast::<fn(&_) -> bool>,
		);

		assert_eq!(loop_result.err(), None);
		assert_eq!(result, "item2");
	}

	#[test]
	fn test_fail_fast() {
		let test_array = vec!["item1", "item2", "item3", "item4"];
		let mut result = "";

		let loop_result: Result<(), Vec<Error>> = loop_with_abort_strategy(
			test_array,
			|item| {
				result = *item;
				if *item == "item2" {
					return Err(Error::DataProviderError(ErrorString::truncate_from(
						"test error".as_bytes().to_vec(),
					)));
				}

				Ok(LoopControls::Continue)
			},
			AbortStrategy::FailFast::<fn(&_) -> bool>,
		);

		let err = loop_result.err();
		assert_ne!(err, None);
		assert_eq!(err.unwrap().len(), 1);
		assert_eq!(result, "item2");
	}

	#[test]
	fn test_continue_until_match() {
		let test_array = vec!["item1", "item2", "item3", "item4"];
		let mut result = "";

		let loop_result: Result<(), Vec<Error>> = loop_with_abort_strategy(
			test_array,
			|item| {
				result = *item;

				Err(Error::DataProviderError(ErrorString::truncate_from(
					"test error".as_bytes().to_vec(),
				)))
			},
			AbortStrategy::ContinueUntil(|item: &&str| *item == "item3"),
		);

		let err = loop_result.err();
		assert_ne!(err, None);
		assert_eq!(err.unwrap().len(), 3);
		assert_eq!(result, "item3");
	}

	#[test]
	fn test_continue_until_not_match() {
		let test_array = vec!["item1", "item2", "item3", "item4"];
		let mut result = "";

		let loop_result: Result<(), Vec<Error>> = loop_with_abort_strategy(
			test_array,
			|item| {
				result = *item;

				Err(Error::DataProviderError(ErrorString::truncate_from(
					"test error".as_bytes().to_vec(),
				)))
			},
			AbortStrategy::ContinueUntil(|item: &&str| *item == "item5"),
		);

		let err = loop_result.err();
		assert_ne!(err, None);
		assert_eq!(err.unwrap().len(), 4);
		assert_eq!(result, "item4");
	}

	#[test]
	fn test_continue_until_end_success() {
		let test_array = vec!["item1", "item2", "item3", "item4"];
		let mut result = "";

		let loop_result: Result<(), Vec<Error>> = loop_with_abort_strategy(
			test_array,
			|item| {
				result = *item;

				if *item == "item4" {
					return Ok(LoopControls::Break);
				}
				Err(Error::DataProviderError(ErrorString::truncate_from(
					"test error".as_bytes().to_vec(),
				)))
			},
			AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
		);

		assert_eq!(loop_result.err(), None);
		assert_eq!(result, "item4");
	}

	#[test]
	fn test_continue_until_end_failure() {
		let test_array = vec!["item1", "item2", "item3", "item4"];
		let mut result = "";

		let loop_result: Result<(), Vec<Error>> = loop_with_abort_strategy(
			test_array,
			|item| {
				result = *item;

				Err(Error::DataProviderError(ErrorString::truncate_from(
					"test error".as_bytes().to_vec(),
				)))
			},
			AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
		);

		let err = loop_result.err();
		assert_ne!(err, None);
		assert_eq!(err.unwrap().len(), 4);
		assert_eq!(result, "item4");
	}
}
