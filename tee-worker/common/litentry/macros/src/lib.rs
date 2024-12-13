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

use cargo_toml::{Dependency, Manifest};
use proc_macro::TokenStream;
use quote::quote;
use std::fs;

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
			// go through all dependencies and visit the ones that has `path`, which means they are local
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
