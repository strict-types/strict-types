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

//! Strict values: schema-less representation of strict types. The module includes:
//! - [`path`]: path accessors/introspects into strict values;
//! - [STON][ston]: strict type object notation, a JSON-like representation of strict types;
//! - [`decode`]: conversion between strict encoding and strict values;
//! - [`typify`]: checks of strict values against strict type schema;
//! - [`convert`]: conversion between strict values and other text representations (JSON, YAML,
//!   TOML, etc).

#[macro_use]
mod val;
mod path;
pub mod ston;
pub mod typify;
pub mod decode;
#[cfg(feature = "serde")]
pub mod convert;
mod encode;

pub use path::{KeyStep, Path, PathError, Step};
pub use val::{EnumTag, StrictNum, StrictVal};

#[cfg(test)]
pub(self) mod test_helpers {
    use amplify::ascii::AsciiString;
    use amplify::confinement::{Confined, TinyAscii};
    use encoding::{StrictDeserialize, StrictSerialize};

    use crate::typeobj::LibBuilder;
    use crate::typesys::{SymbolicTypes, SystemBuilder};

    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
    #[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
    #[strict_type(lib = "TestLib", tags = repr, into_u8, try_from_u8)]
    #[repr(u8)]
    pub enum Precision {
        #[strict_type(dumb)]
        NoDecimals = 0,
        OneDecimal = 1,
        TwoDecimals = 2,
    }

    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    #[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
    #[strict_type(lib = "TestLib", dumb = { Nominal::with("DUMB", "Dumb", strict_dumb!()) })]
    pub struct Nominal {
        pub ticker: Confined<String, 1, 8>,
        pub name: TinyAscii,
        pub precision: Precision,
    }

    impl StrictSerialize for Nominal {}
    impl StrictDeserialize for Nominal {}

    impl Nominal {
        pub fn with(ticker: &'static str, name: &'static str, precision: u8) -> Self {
            Nominal {
                ticker: Confined::try_from(ticker.to_owned()).unwrap(),
                name: Confined::try_from(AsciiString::from_ascii(name).unwrap()).unwrap(),
                precision: Precision::try_from(precision).unwrap(),
            }
        }
    }

    pub fn test_system() -> SymbolicTypes {
        let lib = LibBuilder::new("TestLib", None).transpile::<Nominal>().compile().unwrap();
        SystemBuilder::new().import(lib).unwrap().finalize().unwrap()
    }
}
