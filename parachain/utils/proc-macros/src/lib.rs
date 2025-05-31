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
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

#![allow(clippy::tabs_in_doc_comments)]

use cargo_toml::{Dependency, Manifest};
use proc_macro::TokenStream;
use quote::quote;
use reuse::handle_reuse;
use std::fs;
use syn::{parse_macro_input, Error};
mod reuse;

/**
This macro is used to reuse implementations when the rust's trait system cannot gracefully express the abstraction.

This works similar with `#[cfg(..)]` that sets the target only appear on the specified cases.

# Usage:

```
use litentry_proc_macros::reuse;

#[reuse(x, y)] // Define the cases that the following implementation expands for each one
mod __ {  // Leave mod name with double discards, which is to be replaced by the cases
	#[x]  // This item would only appear on case `x`
	fn u() {
		__
	}

	#[y]  // This item would only appear on case `y`
	fn v(a: String) {
		println!("hello world!")
	}

	#[x]  // Specifying multiple cases indicates that the item would appear on all of them
	#[y] .// This behaviour is designed to be different from `#[cfg(..)]`
	fn a() -> i32 {
		#[x]  // This statement would only appear on case `x`
		let p = 1;
		#[y]  // This statement would only appear on case `y`
		let p = 2;
		p + 1
	}


	fn g<#[x] 'a, #[y] T>(#[x] a: i32, #[y] a: u32) {}
}

```
Expands to:
```
mod x {
	fn a() -> i32 {
		let p = 1;
		p + 1
	}
	fn u() {
		println!("hello world!");
	}
	fn g<'a>(a: i32) {}
}

mod y {
	fn a() -> i32 {
		let p = 2;
		p + 1
	}
	fn v(a: String) {
		println!("hello world!");
	}
	fn g<T>(a: u32) {}
}

```
*/
#[proc_macro_attribute]
pub fn reuse(args: TokenStream, input: TokenStream) -> TokenStream {
	handle_reuse(parse_macro_input!(args), parse_macro_input!(input))
		.unwrap_or_else(Error::into_compile_error)
		.into()
}

#[proc_macro]
pub fn local_modules(_item: TokenStream) -> TokenStream {
	let mut deps: Vec<String> = vec![];
	read_module_names("", ".", &mut deps);
	let output = quote! {
			{
				let deps: Vec<&str> = vec![
				#(#deps),*
				];
				deps
			}
	};
	output.into()
}

fn read_module_names(path: &str, relative_to: &str, module_names: &mut Vec<String>) {
	let current_path = relative_to.to_string() + "/" + path;
	let cargo_file = current_path.to_string() + "/Cargo.toml";
	let contents = fs::read_to_string(&cargo_file)
		.unwrap_or_else(|_| panic!("Should have been able to read the file: {}", cargo_file));
	let manifest = Manifest::from_str(&contents)
		.unwrap_or_else(|_| panic!("Could not parse manifest file locate at {}", cargo_file));
	if let Some(package) = manifest.package {
		let module_name = package.name.replace('-', "_");
		// skip package if it is unnamed or it was already visited
		if !package.name.is_empty() && !module_names.contains(&module_name) {
			module_names.push(module_name);
			// go through all dependencies and visit the ones that has `path`, which means they are
			// local
			manifest.dependencies.values().for_each(|dep| {
				if let Dependency::Detailed(details) = dep {
					if let Some(path) = &details.path {
						read_module_names(path, &current_path, module_names)
					}
				}
			});
		}
	}
}
