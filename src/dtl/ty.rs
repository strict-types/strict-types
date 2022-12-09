// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Project.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use amplify::ascii::AsciiString;
use amplify::confinement::{Confined, SmallOrdMap};

use crate::ast::{FieldName, Variants};
use crate::primitive::Primitive;
use crate::util::Sizing;

pub type TypeName = Confined<AsciiString, 1, 32>;
pub type TypeRef = TypeName;

pub struct TypeLib {
    pub types: SmallOrdMap<TypeName, TypeDef>,
}
