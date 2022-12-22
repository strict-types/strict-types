// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 Ubideco Project
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

use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;
use std::io;

use amplify::ascii::{AsciiChar, AsciiString};
use amplify::confinement::Confined;
use amplify::num::apfloat::{ieee, Float};
use amplify::num::{i1024, i256, i512, u1024, u24, u256, u512};

use crate::encoding::{DefineUnion, StrictEncode, TypedWrite, WriteUnion};

const STD_LIB: &'static str = "StdLib";

macro_rules! encode_num {
    ($ty:ty) => {
        impl StrictEncode for $ty {
            fn strict_encode_dumb() -> Self { <$ty>::MAX }
            fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
                unsafe { writer.write_raw_array(self.to_le_bytes()) }
            }
        }
    };
}

macro_rules! encode_float {
    ($ty:ty, $len:literal) => {
        impl StrictEncode for $ty {
            fn strict_encode_dumb() -> Self { <$ty>::SMALLEST }
            fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
                let mut be = [0u8; $len];
                be.copy_from_slice(&self.to_bits().to_le_bytes()[..$len]);
                unsafe { writer.write_raw_array(be) }
            }
        }
    };
}

encode_num!(u8);
encode_num!(u16);
encode_num!(u24);
encode_num!(u32);
encode_num!(u64);
encode_num!(u128);
encode_num!(u256);
encode_num!(u512);
encode_num!(u1024);

encode_num!(i8);
encode_num!(i16);
encode_num!(i32);
encode_num!(i64);
encode_num!(i128);
encode_num!(i256);
encode_num!(i512);
encode_num!(i1024);

encode_float!(ieee::Half, 2);
encode_float!(ieee::Single, 4);
encode_float!(ieee::Double, 8);
encode_float!(ieee::X87DoubleExtended, 10);
encode_float!(ieee::Quad, 16);
encode_float!(ieee::Oct, 32);

impl<T: StrictEncode<Dumb = T>> StrictEncode for Option<T> {
    fn strict_encode_dumb() -> Self { Some(T::strict_encode_dumb()) }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        let u = writer
            .define_union(None::<String>)
            .define_unit("none")
            .define_type::<T>("some")
            .complete();

        Ok(match self {
            None => u.write_unit("none"),
            Some(val) => u.write_type("some", val),
        }?
        .complete())
    }
}

impl<const LEN: usize> StrictEncode for [u8; LEN] {
    fn strict_encode_dumb() -> Self { [66u8; LEN] }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        unsafe { writer.write_raw_array(*self) }
    }
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> StrictEncode
    for Confined<String, MIN_LEN, MAX_LEN>
{
    fn strict_encode_dumb() -> Self {
        Self::try_from_iter(['a'; MIN_LEN]).expect("hardcoded literal")
    }
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        unsafe { writer.write_raw_bytes::<MAX_LEN>(self.as_bytes()) }
    }
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> StrictEncode
    for Confined<AsciiString, MIN_LEN, MAX_LEN>
{
    fn strict_encode_dumb() -> Self {
        Self::try_from_iter([AsciiChar::a; MIN_LEN]).expect("hardcoded literal")
    }
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        unsafe { writer.write_raw_bytes::<MAX_LEN>(self.as_bytes()) }
    }
}

impl<T: StrictEncode<Dumb = T>, const MIN_LEN: usize, const MAX_LEN: usize> StrictEncode
    for Confined<Vec<T>, MIN_LEN, MAX_LEN>
{
    fn strict_encode_dumb() -> Self {
        Self::try_from_iter(vec![T::strict_encode_dumb()]).expect("hardcoded literal")
    }
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        unsafe { writer.write_raw_collection::<Vec<T>, MIN_LEN, MAX_LEN>(self) }
    }
}

impl<T: StrictEncode<Dumb = T> + Ord, const MIN_LEN: usize, const MAX_LEN: usize> StrictEncode
    for Confined<BTreeSet<T>, MIN_LEN, MAX_LEN>
{
    fn strict_encode_dumb() -> Self {
        Self::try_from_iter(bset![T::strict_encode_dumb()]).expect("hardcoded literal")
    }
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        unsafe { writer.write_raw_collection::<BTreeSet<T>, MIN_LEN, MAX_LEN>(self) }
    }
}

impl<
        K: StrictEncode<Dumb = K> + Ord + Hash,
        const MIN_LEN: usize,
        V: StrictEncode<Dumb = V>,
        const MAX_LEN: usize,
    > StrictEncode for Confined<BTreeMap<K, V>, MIN_LEN, MAX_LEN>
{
    fn strict_encode_dumb() -> Self {
        Self::try_from_iter(bmap! { K::strict_encode_dumb() => V::strict_encode_dumb() })
            .expect("hardcoded literal")
    }
    fn strict_encode<W: TypedWrite>(&self, mut writer: W) -> io::Result<W> {
        unsafe {
            writer = writer.write_raw_len::<MAX_LEN>(self.len())?;
        }
        for (k, v) in self {
            writer = k.strict_encode(writer)?;
            writer = v.strict_encode(writer)?
        }
        Ok(writer)
    }
}
