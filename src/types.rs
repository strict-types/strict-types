// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

pub struct StrictVec<T, const MIN_LEN: u16 = 0>(Vec<T>);

pub struct StrictSet<T, const MIN_LEN: u16 = 0>(BTreeSet<T>)
where T: Eq + Ord + Hash;

pub struct StrictMap<K, V, const MIN_LEN: u16 = 0>(BTreeMap<K, V>)
where K: Eq + Ord + Hash;

pub struct StrictStr<const MIN_LEN: u16 = 0>(String);
