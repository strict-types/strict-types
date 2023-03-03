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
    ($tag:expr) => {
        $crate::StrictVal::enumer($tag)
    };
}

#[macro_export]
macro_rules! svunion {
    ($tag:expr => $val:expr) => {
        $crate::StrictVal::union($tag, $val)
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
    BitInt(i1024),
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
    Name(String),
    #[from]
    Ord(u8),
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum StrictVal {
    #[from(())]
    Unit,

    #[from(u8)]
    #[from(u16)]
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

    #[from]
    String(String),

    // TODO: Use confined collection
    Tuple(Vec<StrictVal>),

    // TODO: Use confined collection
    Struct(IndexMap<String, StrictVal>),

    #[from]
    Enum(EnumTag),

    Union(EnumTag, Box<StrictVal>),

    List(Vec<StrictVal>),

    Table(IndexMap<String, StrictVal>),
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
    pub fn tuple(fields: impl IntoIterator<Item = impl Into<StrictVal>>) -> Self {
        StrictVal::Tuple(fields.into_iter().map(|v| v.into()).collect())
    }
    pub fn struc(fields: impl IntoIterator<Item = (&'static str, impl Into<StrictVal>)>) -> Self {
        StrictVal::Struct(fields.into_iter().map(|(n, v)| (n.to_string(), v.into())).collect())
    }
    pub fn enumer(tag: impl Into<EnumTag>) -> Self { StrictVal::Enum(tag.into()) }
    pub fn union(tag: impl Into<EnumTag>, val: impl Into<StrictVal>) -> Self {
        StrictVal::Union(tag.into(), Box::new(val.into()))
    }
    pub fn none() -> Self { StrictVal::union(0, ()) }
    pub fn some(val: impl Into<StrictVal>) -> Self { StrictVal::union(1, val) }
    pub fn list(items: impl IntoIterator<Item = impl Into<StrictVal>>) -> Self {
        StrictVal::List(items.into_iter().map(|v| v.into()).collect())
    }
    pub fn table(items: impl IntoIterator<Item = (impl ToString, impl Into<StrictVal>)>) -> Self {
        StrictVal::Table(items.into_iter().map(|(n, v)| (n.to_string(), v.into())).collect())
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
            r#"Struct({"name": String("Some name"), "ticker": String("TICK"), "precision": Number(Uint(8))})"#
        )
    }
}
