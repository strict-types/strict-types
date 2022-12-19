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

use amplify::confinement::Confined;
use amplify::num::apfloat::ieee;
use amplify::num::{i1024, i256, i512, u1024, u24, u256, u512};
use half::bf16;

use crate::util::Sizing;
use crate::{StenSchema, StenType, Ty};

macro_rules! st_impl {
    ($name:ident, $ty:ty) => {
        impl StenSchema for $ty {
            const STEN_TYPE_NAME: &'static str = stringify!($name);
            fn sten_ty() -> Ty<StenType> { Ty::$name }
        }
    };
}

st_impl!(U8, u8);
st_impl!(U16, u16);
st_impl!(U24, u24);
st_impl!(U32, u32);
st_impl!(U64, u64);
st_impl!(U128, u128);
st_impl!(U256, u256);
st_impl!(U512, u512);
st_impl!(U1024, u1024);

st_impl!(I8, i8);
st_impl!(I16, i16);
//st_impl!(I24, i24);
st_impl!(I32, i32);
st_impl!(I64, i64);
st_impl!(I128, i128);
st_impl!(I256, i256);
st_impl!(I512, i512);
st_impl!(I1024, i1024);

st_impl!(F16B, bf16);
st_impl!(F16, ieee::Half);
st_impl!(F32, ieee::Single);
st_impl!(F64, ieee::Double);
st_impl!(F80, ieee::X87DoubleExtended);
st_impl!(F128, ieee::Quad);
st_impl!(F256, ieee::Oct);

// We can't restrict max value for the const expression, however we will have a
// panic on `as u16` in the implementation, so the StenType for arrays longer
// than u16::MAX will not be resolvable.
impl<const LEN: usize> StenSchema for [u8; LEN] {
    const STEN_TYPE_NAME: &'static str = "";

    fn sten_ty() -> Ty<StenType> { Ty::<StenType>::byte_array(LEN as u16) }
}

impl StenSchema for () {
    const STEN_TYPE_NAME: &'static str = "";

    fn sten_ty() -> Ty<StenType> { Ty::UNIT }
}

impl<T> StenSchema for Option<T>
where T: StenSchema
{
    const STEN_TYPE_NAME: &'static str = "";

    fn sten_ty() -> Ty<StenType> { Ty::<StenType>::option(T::sten_type()) }
}

impl<T, const MIN: usize, const MAX: usize> StenSchema for Confined<Vec<T>, MIN, MAX>
where T: StenSchema
{
    const STEN_TYPE_NAME: &'static str = "";

    fn sten_ty() -> Ty<StenType> {
        Ty::<StenType>::list(T::sten_type(), Sizing::new(MIN as u16, MAX as u16))
    }
}

impl<T, const MIN: usize, const MAX: usize> StenSchema for Confined<BTreeSet<T>, MIN, MAX>
where T: StenSchema + Ord
{
    const STEN_TYPE_NAME: &'static str = "";

    fn sten_ty() -> Ty<StenType> {
        Ty::<StenType>::set(T::sten_type(), Sizing::new(MIN as u16, MAX as u16))
    }
}

impl<K, V, const MIN: usize, const MAX: usize> StenSchema for Confined<BTreeMap<K, V>, MIN, MAX>
where
    K: StenSchema + Ord + Hash,
    V: StenSchema,
{
    const STEN_TYPE_NAME: &'static str = "";

    fn sten_ty() -> Ty<StenType> {
        Ty::<StenType>::map(
            K::sten_type().try_to_key().expect("invalid key type"),
            V::sten_type(),
            Sizing::new(MIN as u16, MAX as u16),
        )
    }
}
