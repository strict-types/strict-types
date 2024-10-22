// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2024 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2024 UBIDECO Institute
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Strict value core types.

use std::fmt::Debug;

use amplify::hex::ToHex;
// use amplify::num::apfloat::ieee;
use amplify::num::{i1024, u1024, u24, u40, u48, u56};
use amplify::Wrapper;
use encoding::{FieldName, StrictEnum, VariantName};
use indexmap::IndexMap;

#[macro_export]
macro_rules! ston {
    (~) => {
        $crate::StrictVal::none()
    };
    ($tag:literal) => {
        $crate::StrictVal::from($val)
    };
    ($val:expr) => {
        $crate::StrictVal::from($val)
    };
    ($tag:ident) => {
        $crate::StrictVal::enumer(vname!(stringify!($tag)))
    };
    ($tag:literal $val:expr) => {
        $crate::StrictVal::union($tag, $val)
    };
    ($tag:ident $val:expr) => {
        $crate::StrictVal::union(vname!(stringify!($tag)), $val)
    };
    ($($val:expr ),*) => {
        $crate::StrictVal::tuple([
            $( $crate::sv!($val) ),*
        ])
    };
    ($($tag:ident $val:expr ),*) => {
        $crate::StrictVal::struc([
            $( (stringify!($tag), $crate::sv!($val)) ),*
        ])
    };
}

#[macro_export]
macro_rules! sv {
    ($val:expr) => {
        $crate::StrictVal::from($val)
    };
}

#[macro_export]
macro_rules! svnum {
    ($val:expr) => {
        $crate::StrictVal::num($val)
    };
}

#[macro_export]
macro_rules! svstr {
    ($val:expr) => {
        $crate::StrictVal::str($val)
    };
}

#[macro_export]
macro_rules! svbytes {
    ($val:expr) => {
        $crate::StrictVal::bytes($val)
    };
}

#[macro_export]
macro_rules! svnewtype {
    ($val:expr) => {
        $crate::StrictVal::newtype($val)
    };
}

#[macro_export]
#[deprecated(since = "2.7.1", note = "use `ston!` macro instead")]
macro_rules! svtuple {
    ($val:expr) => {
        $crate::StrictVal::tuple($val)
    };
}

#[macro_export]
#[deprecated(since = "2.7.1", note = "use `ston!` macro instead")]
macro_rules! svstruct {
    ($($tag:ident => $val:expr ),*) => {
        $crate::StrictVal::struc([
            $( (stringify!($tag), $crate::sv!($val)) ),*
        ])
    };
}

#[macro_export]
macro_rules! svenum {
    ($tag:literal) => {
        $crate::StrictVal::enumer($tag)
    };
    ($tag:ident) => {
        $crate::StrictVal::enumer(vname!(stringify!($tag)))
    };
}

#[macro_export]
#[deprecated(since = "2.7.1", note = "use `ston!` macro instead")]
macro_rules! svunion {
    ($tag:literal => $val:expr) => {
        $crate::StrictVal::union($tag, $val)
    };
    ($tag:ident => $val:expr) => {
        $crate::StrictVal::union(vname!(stringify!($tag)), $val)
    };
}

#[macro_export]
macro_rules! svnone {
    () => {
        $crate::StrictVal::none()
    };
}

#[macro_export]
macro_rules! svsome {
    ($val:expr) => {
        $crate::StrictVal::some($val)
    };
}

#[macro_export]
macro_rules! svlist {
    ($val:expr) => {
        $crate::StrictVal::list($val)
    };
}

#[macro_export]
macro_rules! svset {
    ($val:expr) => {
        $crate::StrictVal::set($val)
    };
}

#[macro_export]
macro_rules! svtable {
    ($val:expr) => {
        $crate::StrictVal::table($val)
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, From)]
#[display(inner)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase", untagged)
)]
#[non_exhaustive]
pub enum StrictNum {
    #[from(u8)]
    #[from(u16)]
    #[from(u24)]
    #[from(u32)]
    #[from(u40)]
    #[from(u48)]
    #[from(u56)]
    #[from(u64)]
    Uint(u64),

    // TODO: Do conversion of number types in to amplify_num
    //#[from(u128)]
    //#[from(u256)]
    //#[from(u512)]
    #[from]
    BigUint(u1024),

    #[from(i8)]
    #[from(i16)]
    #[from(i32)]
    #[from(i64)]
    Int(i64),

    // TODO: Do conversion of number types in to amplify_num
    //#[from(i128)]
    //#[from(i256)]
    //#[from(i512)]
    #[from]
    BigInt(i1024),
    // TODO: Do conversion of number types in to amplify_num
    /*
    #[from(half::bf16)]
    #[from(ieee::Half)]
    #[from(ieee::Single)]
    #[from(ieee::Double)]
    #[from(ieee::Quad)]
    #[from(ieee::Oct]
    #[from(f32)]
    #[from(f64)]
    Float(ieee::Oct),
    */
    // TODO: Addnon-zero
}

// TODO: Do conversion of number types in to amplify_num

impl StrictNum {
    pub fn unwrap_uint<N: TryFrom<u64>>(self) -> N
    where N::Error: Debug {
        let StrictNum::Uint(v) = self else {
            panic!("StrictNum expected to be an unsigned 128-bit integer");
        };
        v.try_into().expect("StrictNum is too large for the selected uint representation")
    }
}

/// A tag specifying enum or union variant used in strict value representation.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(inner)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase", untagged)
)]
pub enum EnumTag {
    #[from]
    Name(VariantName),
    #[from]
    Ord(u8),
}

impl From<&'static str> for EnumTag {
    fn from(tag: &'static str) -> Self { EnumTag::Name(VariantName::from(tag)) }
}

impl EnumTag {
    pub fn unwrap_ord(&self) -> u8 {
        match self {
            EnumTag::Name(name) => {
                panic!("enum tag value expected to be a numeric value and not '{name}' string")
            }
            EnumTag::Ord(tag) => *tag,
        }
    }
}

// TODO: Move to amplify crate
#[derive(
    Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Default, From
)]
#[wrapper(Deref, AsSlice, BorrowSlice, RangeOps, FromHex)]
#[wrapper_mut(DerefMut, BorrowSliceMut, RangeMut)]
#[display(ToHex::to_hex)]
pub struct Blob(pub Vec<u8>);

impl ToHex for Blob {
    #[inline]
    fn to_hex(&self) -> String { self.0.to_hex() }
}

#[cfg(feature = "serde")]
mod _serde {
    use amplify::hex::FromHex;
    use serde_crate::de::Error;
    use serde_crate::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    impl Serialize for Blob {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
            if serializer.is_human_readable() {
                serializer.serialize_str(&self.to_hex())
            } else {
                self.0.serialize(serializer)
            }
        }
    }

    impl<'de> Deserialize<'de> for Blob {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                Blob::from_hex(&hex).map_err(D::Error::custom)
            } else {
                Vec::<u8>::deserialize(deserializer).map(Blob)
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase", untagged)
)]
pub enum StrictVal {
    #[from(())]
    Unit,

    #[from(u8)]
    #[from(u16)]
    #[from(u24)]
    #[from(u32)]
    #[from(u40)]
    #[from(u48)]
    #[from(u56)]
    #[from(u64)]
    //#[from(u128)]
    //#[from(u256)]
    //#[from(u512)]
    //#[from(u1024)]
    #[from(i8)]
    #[from(i16)]
    #[from(i32)]
    #[from(i64)]
    //#[from(i128)]
    //#[from(i256)]
    //#[from(i512)]
    //#[from(i1024)]
    //#[from(f32)]
    //#[from(f64)]
    //#[from(half::bf16)]
    //#[from(ieee::Half)]
    //#[from(ieee::Single)]
    //#[from(ieee::Double)]
    //#[from(ieee::Quad)]
    //#[from(ieee::Oct)]
    Number(StrictNum),

    // Covers Unicode & ASCII strings and characters
    #[from]
    String(String),

    // Covers byte strings and fixed-size byte arrays
    #[from]
    // TODO: Use Blob type with hex representation
    Bytes(Blob),

    // TODO: Use confined collection
    Struct(IndexMap<FieldName, StrictVal>),

    #[from]
    Enum(EnumTag),

    Union(EnumTag, Box<StrictVal>),

    // Covers both variable- and fixed-size non-byte and non-unicode arrays.
    // May be used for representing tuples.
    List(Vec<StrictVal>),

    Set(Vec<StrictVal>),

    // Tuple has to go after List and Set since otherwise they can be miss-classified by serde
    // readers
    // TODO: Use confined collection
    Tuple(Vec<StrictVal>),

    // May be used to represent structures.
    // it is not a hash/tree map since StrictVal doesn't implement Hash
    // TODO: Create dedicated key type and convert to a HashMap
    Map(Vec<(StrictVal, StrictVal)>),
}

impl From<&str> for StrictVal {
    fn from(value: &str) -> Self { StrictVal::String(value.to_string()) }
}

impl From<&StrictVal> for StrictVal {
    fn from(value: &StrictVal) -> Self { value.clone() }
}

impl StrictVal {
    pub fn num(n: impl Into<StrictNum>) -> Self { StrictVal::Number(n.into()) }
    pub fn str(s: impl ToString) -> Self { StrictVal::String(s.to_string()) }
    pub fn bytes(s: impl AsRef<[u8]>) -> Self { StrictVal::Bytes(Blob::from(s.as_ref().to_vec())) }
    pub fn newtype(inner: impl Into<StrictVal>) -> Self { StrictVal::Tuple(vec![inner.into()]) }
    pub fn tuple(fields: impl IntoIterator<Item = impl Into<StrictVal>>) -> Self {
        StrictVal::Tuple(fields.into_iter().map(|v| v.into()).collect())
    }
    pub fn struc(fields: impl IntoIterator<Item = (&'static str, impl Into<StrictVal>)>) -> Self {
        StrictVal::Struct(fields.into_iter().map(|(n, v)| (fname!(n), v.into())).collect())
    }
    pub fn enumer(tag: impl Into<EnumTag>) -> Self { StrictVal::Enum(tag.into()) }
    pub fn bool(v: bool) -> Self { StrictVal::enumer(v as u8) }
    pub fn union(tag: impl Into<EnumTag>, val: impl Into<StrictVal>) -> Self {
        StrictVal::Union(tag.into(), Box::new(val.into()))
    }
    pub fn none() -> Self { StrictVal::union("none", ()) }
    pub fn some(val: impl Into<StrictVal>) -> Self { StrictVal::union("some", val) }
    pub fn list(items: impl IntoIterator<Item = impl Into<StrictVal>>) -> Self {
        StrictVal::List(items.into_iter().map(|v| v.into()).collect())
    }
    pub fn set(items: impl IntoIterator<Item = impl Into<StrictVal>>) -> Self {
        StrictVal::Set(items.into_iter().map(|v| v.into()).collect())
    }
    pub fn map(
        items: impl IntoIterator<Item = (impl Into<StrictVal>, impl Into<StrictVal>)>,
    ) -> Self {
        StrictVal::Map(items.into_iter().map(|(n, v)| (n.into(), v.into())).collect())
    }

    pub fn skip_wrapper(&self) -> &StrictVal {
        let mut me = self;
        loop {
            match me {
                StrictVal::Tuple(fields) if fields.len() == 1 => me = &fields[0],
                _ => return me,
            }
        }
    }

    pub fn unwrap_option(&self) -> Option<&StrictVal> {
        let StrictVal::Union(tag, content) = self.skip_wrapper() else {
            panic!("StrictVal expected to be a number but holds non-numeric value `{self}`");
        };
        match tag {
            EnumTag::Name(name)
                if name.as_str() == "none" && content.as_ref() == &StrictVal::Unit =>
            {
                None
            }
            EnumTag::Ord(0) if content.as_ref() == &StrictVal::Unit => None,
            EnumTag::Name(name) if name.as_str() == "some" => Some(content.as_ref()),
            EnumTag::Ord(1) => Some(content.as_ref()),
            _ => panic!("StrictVal expected to be an optional, but it is not: {self}"),
        }
    }

    pub fn unwrap_num(&self) -> &StrictNum {
        let StrictVal::Number(v) = self.skip_wrapper() else {
            panic!("StrictVal expected to be a number but holds non-numeric value `{self}`");
        };
        v
    }

    pub fn unwrap_uint<N: TryFrom<u64>>(&self) -> N
    where N::Error: Debug {
        self.unwrap_num().unwrap_uint()
    }

    pub fn unwrap_string(&self) -> String {
        match self.skip_wrapper() {
            StrictVal::String(v) => v.clone(),
            StrictVal::Bytes(v) => {
                String::from_utf8(v.to_inner()).expect("non-Unicode and non-ASCII string")
            }
            StrictVal::List(v) if v.is_empty() => s!(""),
            // Here we process strings made of restricted character sets
            StrictVal::List(v) if v.iter().all(|c| matches!(c, StrictVal::Enum(_))) => {
                let bytes = v
                    .iter()
                    .map(StrictVal::unwrap_enum_tag)
                    .map(EnumTag::unwrap_ord)
                    .collect::<Vec<_>>();
                String::from_utf8(bytes).expect("non-Unicode and non-ASCII string")
            }
            _ => panic!("StrictVal expected to be a string but holds non-string value `{self}`"),
        }
    }

    pub fn unwrap_bytes(&self) -> &[u8] {
        let StrictVal::Bytes(v) = self.skip_wrapper() else {
            panic!("StrictVal expected to be a byte string but holds different value `{self}`");
        };
        v
    }

    pub fn unwrap_tuple(&self, no: u16) -> &StrictVal {
        let StrictVal::Tuple(v) = self.skip_wrapper() else {
            panic!("StrictVal expected to be a tuple but holds different value `{self}`");
        };
        v.get(no as usize)
            .unwrap_or_else(|| panic!("StrictVal tuple doesn't have field at index {no}"))
    }

    pub fn unwrap_struct(&self, field: &'static str) -> &StrictVal {
        let StrictVal::Struct(v) = self.skip_wrapper() else {
            panic!("StrictVal expected to be a string but holds different value `{self}`");
        };
        v.get::<FieldName>(&fname!(field))
            .unwrap_or_else(|| panic!("StrictVal struct doesn't have field named {field}"))
    }

    pub fn unwrap_enum_tag(&self) -> &EnumTag {
        let StrictVal::Enum(tag) = self.skip_wrapper() else {
            panic!("StrictVal expected to be an enum but holds different value `{self}`");
        };
        tag
    }

    pub fn unwrap_enum<E: StrictEnum>(&self) -> E
    where u8: From<E> {
        match self.unwrap_enum_tag() {
            EnumTag::Name(name) => E::from_variant_name(name).unwrap_or_else(|_| {
                panic!(
                    "enum {} doesn't have variant matching tag {name}",
                    E::strict_name().unwrap_or(tn!("unnamed"))
                )
            }),
            EnumTag::Ord(ord) => E::try_from(*ord).unwrap_or_else(|_| {
                panic!(
                    "enum {} doesn't have variant matching tag {ord}",
                    E::strict_name().unwrap_or(tn!("unnamed"))
                )
            }),
        }
    }

    pub fn unwrap_union(&self) -> (&EnumTag, &StrictVal) {
        let StrictVal::Union(tag, content) = self.skip_wrapper() else {
            panic!("StrictVal expected to be an enum but holds different value `{self}`");
        };
        (tag, content.as_ref())
    }

    pub fn unwrap_pos(&self, no: usize) -> &StrictVal {
        if let StrictVal::Set(v) | StrictVal::List(v) = self.skip_wrapper() {
            v.get(no)
                .unwrap_or_else(|| panic!("StrictVal list or set doesn't have item at index {no}"))
        } else {
            panic!("StrictVal expected to be a list or a set but holds different value `{self}`");
        }
    }

    pub fn unwrap_key(&self, key: impl Into<StrictVal>) -> &StrictVal {
        let StrictVal::Map(v) = self.skip_wrapper() else {
            panic!("StrictVal expected to be a map or a set but holds different value `{self}`");
        };
        let key = key.into();
        v.iter()
            .find(|(k, _)| k == &key)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("StrictVal map doesn't have key {key}"))
    }
}

impl<T: Into<StrictVal>> From<Option<T>> for StrictVal {
    fn from(value: Option<T>) -> Self {
        match value {
            None => StrictVal::none(),
            Some(val) => StrictVal::some(val),
        }
    }
}

#[cfg(test)]
mod test {
    // use super::*;

    #[test]
    fn construct() {
        svnum!(1u8);
        svstr!("some");
        svnone!();
        svsome!("val");
        ston!(1, "some", "val");
        svlist!([1, 2, 3]);
        svlist!(["a", "b", "c"]);
        let strct = ston!(name "Some name", ticker "TICK", precision 8u8);
        assert_eq!(
            format!("{strct:?}"),
            r#"Struct({FieldName("name"): String("Some name"), FieldName("ticker"): String("TICK"), FieldName("precision"): Number(Uint(8))})"#
        )
    }
}
