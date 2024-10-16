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

//! Strict type object notation (STON): JSON-like text serialization for stric types.

use std::fmt::{self, Display, Formatter};

use amplify::hex::ToHex;

use super::StrictVal;
use crate::value::EnumTag;

impl StrictVal {
    pub fn is_composite(&self) -> bool {
        match self {
            StrictVal::Struct(_) | StrictVal::List(_) | StrictVal::Set(_) | StrictVal::Map(_) => {
                true
            }
            StrictVal::Tuple(fields) if fields.iter().any(StrictVal::is_composite) => true,
            _ => false,
        }
    }

    fn needs_parenthesis(&self) -> bool {
        match self {
            StrictVal::Unit
            | StrictVal::Number(_)
            | StrictVal::String(_)
            | StrictVal::Bytes(_)
            | StrictVal::Enum(_)
            | StrictVal::Union(_, _)
            | StrictVal::List(_)
            | StrictVal::Set(_)
            | StrictVal::Map(_) => false,
            StrictVal::Tuple(fields) if fields.len() <= 1 => false,
            StrictVal::Tuple(fields) if fields.iter().any(StrictVal::is_composite) => true,
            StrictVal::Struct(fields) if fields.values().any(StrictVal::is_composite) => true,
            StrictVal::Tuple(_) | StrictVal::Struct(_) => false,
        }
    }
}

impl Display for StrictVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: Add nested formatting with `#` and `width`
        match self {
            StrictVal::Unit => f.write_str("(,)"),
            StrictVal::Number(n) => Display::fmt(n, f),
            StrictVal::String(s) => write!(f, r#""{s}""#),
            StrictVal::Bytes(blob) => write!(f, "0x{}", blob.to_hex()),
            StrictVal::Tuple(fields) => {
                let mut iter = fields.iter();
                let last = iter.next_back();
                if self.needs_parenthesis() {
                    f.write_str("(")?;
                }
                for field in iter {
                    Display::fmt(field, f)?;
                    f.write_str(", ")?;
                }
                if let Some(field) = last {
                    Display::fmt(field, f)?;
                }
                if self.needs_parenthesis() {
                    f.write_str(")")?;
                }
                Ok(())
            }
            StrictVal::Struct(fields) => {
                let mut iter = fields.iter();
                let last = iter.next_back();
                if self.needs_parenthesis() {
                    f.write_str("(")?;
                }
                for (fname, fval) in iter {
                    write!(f, "{fname}=")?;
                    Display::fmt(fval, f)?;
                    f.write_str(", ")?;
                }
                if let Some((fname, fval)) = last {
                    write!(f, "{fname}=")?;
                    Display::fmt(fval, f)?;
                }
                if self.needs_parenthesis() {
                    f.write_str(")")?;
                }
                Ok(())
            }
            StrictVal::Enum(tag) => Display::fmt(tag, f),
            StrictVal::Union(tag, val)
                if (*tag == EnumTag::Ord(0) || *tag == EnumTag::Name(vname!("none")))
                    && **val == StrictVal::Unit =>
            {
                f.write_str("~")
            }
            StrictVal::Union(tag, val) => {
                if val.needs_parenthesis() {
                    f.write_str("(")?;
                }
                Display::fmt(val, f)?;
                if val.needs_parenthesis() {
                    f.write_str(")")?;
                }
                f.write_str(".")?;
                Display::fmt(tag, f)?;
                Ok(())
            }
            StrictVal::List(items) => {
                let mut iter = items.iter();
                let last = iter.next_back();
                f.write_str("[")?;
                for item in iter {
                    Display::fmt(item, f)?;
                    f.write_str(", ")?;
                }
                if let Some(item) = last {
                    Display::fmt(item, f)?;
                }
                f.write_str("]")
            }
            StrictVal::Set(items) => {
                let mut iter = items.iter();
                let last = iter.next_back();
                f.write_str("{")?;
                for item in iter {
                    Display::fmt(item, f)?;
                    f.write_str(", ")?;
                }
                if let Some(item) = last {
                    Display::fmt(item, f)?;
                }
                f.write_str("}")
            }
            StrictVal::Map(items) => {
                let mut iter = items.iter();
                let last = iter.next_back();
                f.write_str("{")?;
                for (fname, fval) in iter {
                    write!(f, "{fname} -> ")?;
                    Display::fmt(fval, f)?;
                    f.write_str(", ")?;
                }
                if let Some((fname, fval)) = last {
                    write!(f, "{fname} -> ")?;
                    Display::fmt(fval, f)?;
                }
                f.write_str("}")
            }
        }
    }
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn simple() {
        let data = sv!(10u8);
        assert_eq!(format!("{data}"), "10");

        let data = sv!("string");
        assert_eq!(format!("{data}"), r#""string""#);

        let data = svenum!(1);
        assert_eq!(format!("{data}"), "1");

        let data = svenum!(tag);
        assert_eq!(format!("{data}"), "tag");

        let data = svnone!();
        assert_eq!(format!("{data}"), "~"); // shorthand for `(,).none`

        let data = svsome!("value");
        assert_eq!(format!("{data}"), r#""value".some"#);
    }

    #[test]
    fn complex() {
        let strct = svstruct!(name => "Some name", ticker => "TICK", precision => svenum!(8));
        assert_eq!(format!("{strct}"), r#"name="Some name", ticker="TICK", precision=8"#)
    }
}
