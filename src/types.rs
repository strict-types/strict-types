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
use std::fmt::Debug;
use std::hash::Hash;

use strict_encoding::{StrictDecode, StrictEncode};

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Default)]
#[derive(StrictEncode, StrictDecode)]
pub struct StrictVec<T, const MIN_LEN: u16 = 0>(Vec<T>)
where T: StrictEncode + StrictDecode;

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Default)]
#[derive(StrictEncode, StrictDecode)]
pub struct StrictSet<T, const MIN_LEN: u16 = 0>(BTreeSet<T>)
where T: Eq + Ord + Debug + StrictEncode + StrictDecode;
// TODO: Remove `Debug` requirement after strict_encoding update

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Default)]
#[derive(StrictEncode, StrictDecode)]
pub struct StrictMap<K, V, const MIN_LEN: u16 = 0>(BTreeMap<K, V>)
where
    K: Clone + Eq + Ord + Debug + StrictEncode + StrictDecode,
    V: Clone + StrictEncode + StrictDecode;
// TODO: Remove `Debug` and `Clone` requirements after strict_encoding update

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Default)]
#[derive(StrictEncode, StrictDecode)]
pub struct StrictStr<const MIN_LEN: u16 = 0>(String);
