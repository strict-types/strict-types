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

//! Path accessors into strict values.

use std::fmt::{self, Display, Formatter};

use amplify::confinement::{SmallVec, TinyBlob, TinyString};
use encoding::{FieldName, Primitive, STRICT_TYPES_LIB};

// TODO: Convert into `StrictKey` and use in `StrictVal::Map` for key value repr.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = Self::Primitive(strict_dumb!()))]
#[non_exhaustive]
pub enum KeyStep {
    #[from]
    Primitive(Primitive),

    #[from]
    TinyBlob(TinyBlob),

    #[from]
    TinyString(TinyString),
}

impl Display for KeyStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            KeyStep::Primitive(prim) => Display::fmt(prim, f),
            KeyStep::TinyBlob(data) => {
                f.write_str("0h")?;
                for byte in data {
                    write!(f, "{byte:02X}")?;
                }
                Ok(())
            }
            KeyStep::TinyString(s) => {
                let s = s.replace("\"", "\\\"");
                f.write_str(&s)
            }
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = Self::UnnamedField(strict_dumb!()))]
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
