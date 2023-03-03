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

//! Strict type object notation (STON): JSON-like text serialization for stric types.

use std::fmt::{self, Display, Formatter};

use super::StrictVal;

impl StrictVal {
    fn needs_braces(&self) -> bool {
        matches!(
            self,
            StrictVal::Number(_) | StrictVal::String(_) | StrictVal::Enum(_) | StrictVal::Union(..)
        )
    }
}

impl Display for StrictVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: Add nested formatting with `#` and `width`
        match self {
            StrictVal::Unit => f.write_str("()"),
            StrictVal::Number(n) => Display::fmt(n, f),
            StrictVal::String(s) => write!(f, r#""{s}""#),
            StrictVal::Tuple(fields) => {
                let mut iter = fields.iter();
                let last = iter.next_back();
                f.write_str("(")?;
                for field in iter {
                    Display::fmt(field, f)?;
                    f.write_str(", ")?;
                }
                if let Some(field) = last {
                    Display::fmt(field, f)?;
                }
                f.write_str(")")
            }
            StrictVal::Struct(fields) => {
                let mut iter = fields.iter();
                let last = iter.next_back();
                f.write_str("(")?;
                for (fname, fval) in iter {
                    write!(f, "{fname}=")?;
                    Display::fmt(fval, f)?;
                    f.write_str(", ")?;
                }
                if let Some((fname, fval)) = last {
                    write!(f, "{fname}=")?;
                    Display::fmt(fval, f)?;
                }
                f.write_str(")")
            }
            StrictVal::Enum(tag) => Display::fmt(tag, f),
            StrictVal::Union(tag, val) => {
                Display::fmt(tag, f)?;
                if val.needs_braces() {
                    f.write_str("(")?;
                }
                Display::fmt(val, f)?;
                if val.needs_braces() {
                    f.write_str("(")?;
                }
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
            StrictVal::Table(items) => {
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
    fn serialize() {
        let strct = svstruct!(name => "Some name", ticker => "TICK", precision => svenum!(8));
        assert_eq!(format!("{strct}"), r#"(name="Some name", ticker="TICK", precision=8)"#)
    }
}
