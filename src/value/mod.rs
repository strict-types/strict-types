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
mod test_helpers {
    use amplify::confinement::Confined;
    use encoding::{Ident, StrictDeserialize, StrictSerialize};

    use crate::stl::{std_stl, strict_types_stl};
    use crate::typesys::{SymbolicSys, SystemBuilder};
    use crate::LibBuilder;

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
        pub ticker: Ident,
        pub name: Confined<String, 1, 32>,
        pub precision: Precision,
    }

    impl StrictSerialize for Nominal {}
    impl StrictDeserialize for Nominal {}

    impl Nominal {
        pub fn with(ticker: &'static str, name: &'static str, precision: u8) -> Self {
            Nominal {
                ticker: Ident::try_from(ticker.to_owned()).unwrap(),
                name: Confined::try_from(name.to_owned()).unwrap(),
                precision: Precision::try_from(precision).unwrap(),
            }
        }
    }

    pub fn test_system() -> SymbolicSys {
        let std = std_stl();
        let st = strict_types_stl();
        let lib = LibBuilder::new("TestLib", [std.to_dependency(), st.to_dependency()])
            .transpile::<Nominal>()
            .compile()
            .unwrap();
        SystemBuilder::new()
            .import(lib)
            .unwrap()
            .import(std)
            .unwrap()
            .import(st)
            .unwrap()
            .finalize()
            .unwrap()
    }
}
