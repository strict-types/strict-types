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

use std::any;
use std::collections::BTreeSet;
use std::fmt::Display;

use crate::{FieldName, LibName, TypeName};

#[derive(Clone, Eq, PartialEq, Debug, Display, Error)]
#[display("unexpected variant {1} for enum or union {0}")]
pub struct VariantError<V: Display>(TypeName, V);

pub trait StrictDumb {
    type Dumb: StrictInfo = Self;
    fn strict_dumb() -> Self::Dumb;
}

pub trait StrictType: StrictDumb {
    const STRICT_LIB_NAME: &'static str;
    fn strict_name() -> Option<String> {
        fn get_ident(path: &str) -> &str { path.rsplit_once("::").map(|(_, n)| n).unwrap_or(path) }

        let name = any::type_name::<Self>();
        let (base, generics) = name.split_once("<").unwrap_or((name, ""));
        let generics = generics.trim_end_matches('>');
        let mut ident = get_ident(base).to_owned();
        for arg in generics.split(',') {
            ident.push('_');
            ident.extend(get_ident(arg));
        }
        Some(ident)
    }
}

impl<T: StrictType> StrictType for &T {
    const STRICT_LIB_NAME: &'static str = T::STRICT_LIB_NAME;
}

pub trait StrictProduct: StrictType {}
pub trait StrictTuple: StrictProduct {
    const ALL_FIELDS: &'static [u8];
    fn strict_check_fields() {
        let set = BTreeSet::from(Self::ALL_FIELDS);
        assert_eq!(
            set.len(),
            Self::ALL_FIELDS.len(),
            "tuple type {} contains repeated field ids",
            Self::strict_name()
        );
    }
}
pub trait StrictStruct: StrictProduct {
    const ALL_FIELDS: &'static [(u8, &'static str)];

    fn strict_check_fields() {
        let (ords, names): (BTreeSet<_>, BTreeSet<_>) = Self::ALL_FIELDS.iter().unzip();
        assert_eq!(
            ords.len(),
            Self::ALL_FIELDS.len(),
            "struct type {} contains repeated field ids",
            Self::strict_name()
        );
        assert_eq!(
            names.len(),
            Self::ALL_FIELDS.len(),
            "struct type {} contains repeated field names",
            Self::strict_name()
        );
    }
}

pub trait StrictSum: StrictType {
    const ALL_VARIANTS: &'static [(u8, &'static str)];

    fn strict_check_variants() {
        let (ords, names): (BTreeSet<_>, BTreeSet<_>) = Self::ALL_VARIANTS.iter().unzip();
        assert_eq!(
            ords.len(),
            Self::ALL_FIELDS.len(),
            "type {} contains repeated variant ids",
            Self::strict_name()
        );
        assert_eq!(
            names.len(),
            Self::ALL_FIELDS.len(),
            "type {} contains repeated variant names",
            Self::strict_name()
        );
    }

    fn variant_ord(&self) -> u8 {
        let variant = self.variant_name();
        for (ord, name) in Self::ALL_VARIANTS {
            if name == variant {
                return *ord;
            }
        }
        unreachable!(
            "not all variants are enumerated for {} enum in StrictUnion::all_variants \
             implementation",
            any::type_name::<Self>()
        )
    }
    fn variant_name(&self) -> &'static str;
}

pub trait StrictUnion: StrictSum {}

pub trait StrictEnum
where
    Self: StrictSum + Copy + TryFrom<u8, Error = VariantError<u8>>,
    u8: From<Self>,
{
    fn from_variant_name(name: &FieldName) -> Result<Self, VariantError<&FieldName>>;
}

impl<T: StrictDumb> StrictDumb for &T {
    type Dumb = T;
    fn strict_dumb() -> Self::Dumb { T::strict_dumb() }
}

impl<T> StrictDumb for T
where T: StrictProduct + Default
{
    type Dumb = Self;
    fn strict_dumb() -> Self::Dumb { T::default() }
}

impl<T> StrictDumb for T
where T: StrictUnion + Default
{
    type Dumb = Self;
    fn strict_dumb() -> Self::Dumb { T::default() }
}

impl<T> StrictDumb for T
where T: StrictEnum
{
    type Dumb = Self;
    fn strict_dumb() -> Self::Dumb {
        T::all_variants().first().expect("enum must have at least one variant").0.into()
    }
}

pub enum TypeClass {
    Embedded,
    Enum(&'static [(u8, &'static str)]),
    Union(&'static [(u8, &'static str)]),
    Tuple(&'static [u8]),
    Struct(&'static [(u8, &'static str)]),
}

pub struct TypeInfo<T: StrictInfo> {
    lib: LibName,
    name: Option<TypeName>,
    cls: TypeClass,
    dumb: T,
}

pub trait StrictInfo: StrictType {
    fn strict_type_info() -> TypeInfo<Self::Dumb>;
}

impl<T: StrictInfo> StrictInfo for &T {
    fn strict_type_info() -> TypeInfo<T> { T::strict_type_info() }
}

impl<T> StrictInfo for T
where T: StrictUnion
{
    fn strict_type_info() -> TypeInfo<Self::Dumb> {
        T::strict_check_variants();
        TypeInfo {
            lib: libname!(T::STRICT_LIB_NAME),
            name: T::strict_name().map(|name| tn!(name)),
            cls: TypeClass::Union(T::ALL_VARIANTS),
            dumb: T::strict_dumb(),
        }
    }
}

impl<T> StrictInfo for T
where T: StrictEnum
{
    fn strict_type_info() -> TypeInfo<Self::Dumb> {
        T::strict_check_variants();
        TypeInfo {
            lib: libname!(T::STRICT_LIB_NAME),
            name: T::strict_name().map(|name| tn!(name)),
            cls: TypeClass::Enum(T::ALL_VARIANTS),
            dumb: T::strict_dumb(),
        }
    }
}

impl<T> StrictInfo for T
where T: StrictStruct
{
    fn strict_type_info() -> TypeInfo<Self::Dumb> {
        T::strict_check_fields();
        TypeInfo {
            lib: libname!(T::STRICT_LIB_NAME),
            name: T::strict_name().map(|name| tn!(name)),
            cls: TypeClass::Struct(T::ALL_FIELDS),
            dumb: T::strict_dumb(),
        }
    }
}

impl<T> StrictInfo for T
where T: StrictTuple
{
    fn strict_type_info() -> TypeInfo<Self::Dumb> {
        T::strict_check_fields();
        TypeInfo {
            lib: libname!(T::STRICT_LIB_NAME),
            name: T::strict_name().map(|name| tn!(name)),
            cls: TypeClass::Tuple(T::ALL_FIELDS),
            dumb: T::strict_dumb(),
        }
    }
}
