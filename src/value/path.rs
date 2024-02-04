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

//! Path accessors into strict values.

use std::fmt::{self, Display, Formatter};

use amplify::confinement::{SmallVec, TinyBlob, TinyString};
use encoding::{FieldName, STRICT_TYPES_LIB};

use crate::value::{EnumTag, StrictNum};
use crate::StrictVal;

// TODO: Convert into `StrictKey` and use in `StrictVal::Map` for key value repr.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = Self::Number(strict_dumb!()))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
#[non_exhaustive]
pub enum KeyStep {
    #[from]
    Number(u128),

    #[from]
    TinyBlob(TinyBlob),

    #[from]
    TinyString(TinyString),
}

impl KeyStep {
    pub fn has_match(&self, val: &StrictVal) -> bool {
        match (self, val) {
            (KeyStep::Number(no), StrictVal::Enum(EnumTag::Ord(tag))) if *tag as u128 == *no => {
                true
            }
            (KeyStep::Number(num1), StrictVal::Number(StrictNum::Uint(num2))) if num1 == num2 => {
                true
            }
            (KeyStep::TinyBlob(blob1), StrictVal::Bytes(blob2))
                if blob1.as_slice() == blob2.as_slice() =>
            {
                true
            }
            (KeyStep::TinyString(s1), StrictVal::String(s2)) if s1.as_str() == s2.as_str() => true,
            _ => false,
        }
    }
}

impl Display for KeyStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            KeyStep::Number(num) => Display::fmt(num, f),
            KeyStep::TinyBlob(data) => {
                f.write_str("0h")?;
                for byte in data {
                    write!(f, "{byte:02X}")?;
                }
                Ok(())
            }
            KeyStep::TinyString(s) => {
                let s = s.replace('"', "\\\"");
                f.write_str(&s)
            }
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = Self::UnnamedField(strict_dumb!()))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub enum Step {
    #[display(".{0}")]
    #[from]
    NamedField(FieldName),

    #[display(".{0}")]
    #[from]
    UnnamedField(u8),

    #[display("[{0}]")]
    Index(u32),

    #[display("{{{0}}}")]
    #[from]
    Key(KeyStep),
}

#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Path(SmallVec<Step>);

impl Path {
    pub fn new() -> Path { Path::default() }

    pub fn with(step: Step) -> Path { Path(small_vec!(step)) }

    pub fn iter(&self) -> std::slice::Iter<Step> { self.0.iter() }
}

impl<'path> IntoIterator for &'path Path {
    type Item = &'path Step;
    type IntoIter = std::slice::Iter<'path, Step>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for step in self {
            Display::fmt(step, f)?;
        }
        Ok(())
    }
}

// TODO: Implement FromStr for value Path expressions

#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display(doc_comments)]
pub enum PathError {
    /// collection has less items than requested in the path ({0} vs {1}).
    CollectionIndexOutOfBounds(u32, usize),
    /// tuple has less fields than requested in the path ({0} vs {1}).
    FieldNoOutOfBounds(u8, usize),
    /// struct doesn't have field named `{0}`.
    UnknownFieldName(FieldName),
    /// map doesn't have key named `{0}`.
    UnknownKey(KeyStep),
    /// path doesn't match value at step {0}.
    TypeMismatch(Step, StrictVal),
}

impl StrictVal {
    pub fn at_path<'p>(
        &self,
        path: impl IntoIterator<Item = &'p Step>,
    ) -> Result<&StrictVal, PathError> {
        let mut iter = path.into_iter();
        match (self, iter.next()) {
            (val, None) => Ok(val),
            (StrictVal::Tuple(fields), Some(Step::UnnamedField(no)))
                if *no as usize >= fields.len() =>
            {
                Err(PathError::FieldNoOutOfBounds(*no, fields.len()))
            }
            (StrictVal::Tuple(fields), Some(Step::UnnamedField(no))) => Ok(&fields[*no as usize]),
            (StrictVal::Struct(fields), Some(Step::NamedField(name))) => {
                fields.get(name).ok_or(PathError::UnknownFieldName(name.clone()))
            }
            (StrictVal::List(items) | StrictVal::Set(items), Some(Step::Index(idx)))
                if *idx as usize >= items.len() =>
            {
                Err(PathError::CollectionIndexOutOfBounds(*idx, items.len()))
            }
            (StrictVal::List(items) | StrictVal::Set(items), Some(Step::Index(idx))) => {
                Ok(&items[*idx as usize])
            }
            (StrictVal::Map(items), Some(Step::Key(idx))) => items
                .iter()
                .find(|(key, _)| idx.has_match(key))
                .map(|(_, val)| val)
                .ok_or(PathError::UnknownKey(idx.clone())),

            (_, Some(step)) => Err(PathError::TypeMismatch(step.clone(), self.clone())),
        }
    }
}
