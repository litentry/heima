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

pub trait Storage<K, V> {
	fn get(&self, key: &K) -> Option<V>;
	fn insert(&self, key: K, value: V) -> Result<(), ()>;
	fn remove(&self, key: &K) -> Result<(), ()>;
	fn contains_key(&self, key: &K) -> bool;
}
