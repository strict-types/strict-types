// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 UBIDECO Institute
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

// use amplify::num::apfloat::ieee;
use amplify::num::{i1024, u1024};
use encoding::{FieldName, VariantName};
use indexmap::IndexMap;

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
macro_rules! svtuple {
    ($val:expr) => {
        $crate::StrictVal::tuple($val)
    };
}

#[macro_export]
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
#[non_exhaustive]
pub enum StrictNum {
    #[from(u8)]
    #[from(u16)]
    #[from(u32)]
    #[from(u64)]
    #[from]
    Uint(u128),

    // TODO: Do conversion of number types in to amplify_num
    //#[from(u256)]
    //#[from(u512)]
    #[from]
    BigUint(u1024),

    #[from(i8)]
    #[from(i16)]
    #[from(i32)]
    #[from(i64)]
    #[from]
    Int(i128),

    // TODO: Do conversion of number types in to amplify_num
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

/// A tag specifying enum or union variant used in strict value representation.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[display(inner)]
pub enum EnumTag {
    #[from]
    Name(VariantName),
    #[from]
    Ord(u8),
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

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum StrictVal {
    #[from(())]
    Unit,

    #[from(u8)]
    #[from(u16)]
    //#[from(u24)]
    #[from(u32)]
    #[from(u64)]
    #[from(u128)]
    //#[from(u256)]
    //#[from(u512)]
    //#[from(u1024)]
    #[from(i8)]
    #[from(i16)]
    #[from(i32)]
    #[from(i64)]
    #[from(i128)]
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

    // Covers unicode & ascii strings and characters
    #[from]
    String(String),

    // Covers byte strings and fixed-size byte arrays
    #[from]
    Bytes(Vec<u8>),

    // TODO: Use confined collection
    Tuple(Vec<StrictVal>),

    // TODO: Use confined collection
    Struct(IndexMap<FieldName, StrictVal>),

    #[from]
    Enum(EnumTag),

    Union(EnumTag, Box<StrictVal>),

    // Covers both variable- and fixed-size non-byte and non-unicode arrays.
    // May be used for representing tuples.
    List(Vec<StrictVal>),

    Set(Vec<StrictVal>),

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
    pub fn bytes(s: impl AsRef<[u8]>) -> Self { StrictVal::Bytes(s.as_ref().to_vec()) }
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
    pub fn none() -> Self { StrictVal::union(0, ()) }
    pub fn some(val: impl Into<StrictVal>) -> Self { StrictVal::union(1, val) }
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

    pub fn as_num(&self) -> &StrictNum {
        if let StrictVal::Number(v) = self {
            v
        } else {
            panic!("StrictVal expected to be a number holds non-numeric value ({self})");
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            StrictVal::String(v) => v.clone(),
            StrictVal::Bytes(v) => {
                String::from_utf8(v.clone()).expect("non-Unicode and non-ASCII string")
            }
            StrictVal::List(v) if v.is_empty() => s!(""),
            // Here we process strings made of restricted character sets
            StrictVal::List(v) if v.iter().all(|c| matches!(c, StrictVal::Enum(_))) => {
                let bytes =
                    v.iter().map(StrictVal::as_enum).map(EnumTag::unwrap_ord).collect::<Vec<_>>();
                String::from_utf8(bytes).expect("non-Unicode and non-ASCII string")
            }
            _ => panic!("StrictVal expected to be a string holds non-string value ({self})"),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        if let StrictVal::Bytes(v) = self {
            v
        } else {
            panic!("StrictVal expected to be a byte string holds different value ({self})");
        }
    }

    pub fn tuple_field(&self, no: u16) -> &StrictVal {
        if let StrictVal::Tuple(v) = self {
            v.get(no as usize)
                .unwrap_or_else(|| panic!("StrictVal tuple doesn't have field at index {no}"))
        } else {
            panic!("StrictVal expected to be a tuple holds different value ({self})");
        }
    }

    pub fn struct_field(&self, field: &'static str) -> &StrictVal {
        if let StrictVal::Struct(v) = self {
            v.get::<FieldName>(&fname!(field))
                .unwrap_or_else(|| panic!("StrictVal struct doesn't have field named {field}"))
        } else {
            panic!("StrictVal expected to be a string holds different value ({self})");
        }
    }

    pub fn as_enum(&self) -> &EnumTag {
        if let StrictVal::Enum(tag) = self {
            tag
        } else {
            panic!("StrictVal expected to be an enum holds different value ({self})");
        }
    }

    pub fn as_union(&self) -> (&EnumTag, &StrictVal) {
        if let StrictVal::Union(tag, v) = self {
            (tag, v.as_ref())
        } else {
            panic!("StrictVal expected to be an enum holds different value ({self})");
        }
    }

    pub fn get_at_pos(&self, no: usize) -> &StrictVal {
        if let StrictVal::Set(v) | StrictVal::List(v) = self {
            v.get(no)
                .unwrap_or_else(|| panic!("StrictVal list or set doesn't have item at index {no}"))
        } else {
            panic!("StrictVal expected to be a list or a set holds different value ({self})");
        }
    }

    pub fn get_at_key(&self, key: impl Into<StrictVal>) -> &StrictVal {
        if let StrictVal::Map(v) = self {
            let key = key.into();
            v.iter()
                .find(|(k, _)| k == &key)
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("StrictVal map doesn't have key {key}"))
        } else {
            panic!("StrictVal expected to be a map or a set holds different value ({self})");
        }
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
        svtuple!([sv!(1), sv!("some"), svsome!("val")]);
        svlist!([1, 2, 3]);
        svlist!(["a", "b", "c"]);
        let strct = svstruct!(name => "Some name", ticker => "TICK", precision => 8u8);
        assert_eq!(
            format!("{strct:?}"),
            r#"Struct({FieldName("name"): String("Some name"), FieldName("ticker"): String("TICK"), FieldName("precision"): Number(Uint(8))})"#
        )
    }
}
