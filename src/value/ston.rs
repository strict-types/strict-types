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
            StrictVal::Tuple(fields) if fields.len() == 1 => false,
            StrictVal::Tuple(_) | StrictVal::Struct(_) => true,
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
                while let Some(field) = iter.next() {
                    if field.needs_parenthesis() {
                        f.write_str("(")?;
                    }
                    Display::fmt(field, f)?;
                    if field.needs_parenthesis() {
                        f.write_str(")")?;
                    }
                    if iter.len() > 0 {
                        f.write_str(", ")?;
                    }
                }
                Ok(())
            }
            StrictVal::Struct(fields) => {
                let mut iter = fields.iter();
                while let Some((fname, fval)) = iter.next() {
                    write!(f, "{fname} ")?;
                    if fval.needs_parenthesis() {
                        f.write_str("(")?;
                    }
                    Display::fmt(fval, f)?;
                    if fval.needs_parenthesis() {
                        f.write_str(")")?;
                    }
                    if iter.len() > 0 {
                        f.write_str(", ")?;
                    }
                }
                Ok(())
            }
            StrictVal::Enum(tag) => Display::fmt(tag, f),
            StrictVal::Union(tag, content)
                if (*tag == EnumTag::Ord(0) || *tag == EnumTag::Name(vname!("none")))
                    && **content == StrictVal::Unit =>
            {
                f.write_str("~")
            }
            StrictVal::Union(tag, content) => {
                if content.needs_parenthesis() {
                    f.write_str("(")?;
                }
                Display::fmt(content, f)?;
                if content.needs_parenthesis() {
                    f.write_str(")")?;
                }
                f.write_str(".")?;
                Display::fmt(tag, f)?;
                Ok(())
            }
            StrictVal::List(items) => {
                let mut iter = items.iter();
                f.write_str("[")?;
                while let Some(item) = iter.next() {
                    if item.needs_parenthesis() {
                        f.write_str("(")?;
                    }
                    Display::fmt(item, f)?;
                    if item.needs_parenthesis() {
                        f.write_str(")")?;
                    }
                    if iter.len() > 0 {
                        f.write_str(", ")?;
                    }
                }
                f.write_str("]")
            }
            StrictVal::Set(items) => {
                let mut iter = items.iter();
                f.write_str("{")?;
                while let Some(item) = iter.next() {
                    if item.needs_parenthesis() {
                        f.write_str("(")?;
                    }
                    Display::fmt(item, f)?;
                    if item.needs_parenthesis() {
                        f.write_str(")")?;
                    }
                    if iter.len() > 0 {
                        f.write_str(", ")?;
                    }
                }
                f.write_str("}")
            }
            StrictVal::Map(items) => {
                let mut iter = items.iter();
                f.write_str("{")?;
                while let Some((fname, fval)) = iter.next() {
                    write!(f, "{fname} -> ")?;
                    if fval.needs_parenthesis() {
                        f.write_str("(")?;
                    }
                    Display::fmt(fval, f)?;
                    if fval.needs_parenthesis() {
                        f.write_str(")")?;
                    }
                    if iter.len() > 0 {
                        f.write_str(", ")?;
                    }
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
        let strct = ston!(name "Some name", ticker "TICK", precision svenum!(8), data svlist!([0u8, 1, 2, 3, 4]), tuple ston!(a 15u8, b "text"));
        assert_eq!(
            format!("{strct}"),
            r#"name "Some name", ticker "TICK", precision 8, data [0, 1, 2, 3, 4], tuple (a 15, b "text")"#
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde() {
        use std::iter;

        use amplify::confinement;
        use amplify::confinement::{SmallVec, TinyBlob, TinyString};
        use encoding::{Primitive, StrictDeserialize, StrictSerialize};

        use crate::{LibBuilder, SemId, StrictVal, SystemBuilder, Ty};

        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
        #[derive(StrictType, StrictEncode, StrictDecode)]
        #[strict_type(lib = "Test", try_from_u8, into_u8, tags = repr)]
        enum Enum {
            One = 1,
            #[default]
            Two = 2,
        }
        #[derive(Default)]
        #[derive(StrictType, StrictEncode, StrictDecode)]
        #[strict_type(lib = "Test")]
        struct Tuple(TinyString, i16, TinyString);
        #[derive(Default)]
        #[derive(StrictType, StrictEncode, StrictDecode)]
        #[strict_type(lib = "Test")]
        struct Test {
            name: TinyString,
            option: Option<u8>,
            precision: Enum,
            data: TinyBlob,
            list: SmallVec<u16>,
            tuple: Tuple,
        }
        impl StrictSerialize for Test {}
        impl StrictDeserialize for Test {}

        let lib =
            LibBuilder::new(libname!("Test"), iter::empty()).transpile::<Test>().compile().unwrap();
        let stl = lib.types.get(&tn!("Test")).unwrap();
        assert_eq!(
            stl.to_string(),
            "name [Unicode ^ ..0xff]
                       , option U8?
                       , precision \
             semid:3KnxzApb-9~6dpmO-YsJf16I-zNiMxSh-sNC3zIL-q~uHyeo#manila-compare-person
                       , data [Byte ^ ..0xff]
                       , list [U16]
                       , tuple \
             semid:61l54MmE-DY1FQ1b-jjkquKp-5UOyhe0-sBSjtr~-VPzs4fk#cabinet-vampire-change"
        );
        let sys = SystemBuilder::new().import(lib).unwrap().finalize().unwrap();
        eprintln!("{}", Ty::<SemId>::Primitive(Primitive::BYTE).is_byte());

        let strct = Test::default();
        let blob = strct.to_strict_serialized::<{ confinement::U16 }>().unwrap();
        let typed = sys.strict_deserialize_type("Test.Test", blob.as_slice()).unwrap();

        assert_eq!(
            format!("{}", typed.val),
            r#"name "", option ~, precision two, data 0x, list [], tuple ("", 0, "")"#
        );
        let typified = sys.typify(typed.val.clone(), "Test.Test").unwrap();
        assert_eq!(typified, typed);

        let ser = serde_yaml::to_string(&typed.val).unwrap();
        assert_eq!(
            ser,
            r#"name: ''
option:
- none
- null
precision: two
data: ''
list: []
tuple:
- ''
- 0
- ''
"#
        );
        let des: StrictVal = serde_yaml::from_str(&ser).unwrap();
        assert_eq!(
            des.to_string(),
            r#"name "", option ~, precision "two", data "", list [], tuple ["", 0, ""]"#
        );
        let typified = sys.typify(des, "Test.Test").unwrap();
        assert_eq!(typified, typed);
    }
}
