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

#[macro_export]
macro_rules! fields {
    { $($value:expr),+ $(,)? } => {
        {
            let mut c = 0u8;
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert($crate::ast::Field::unnamed(c), $value.into()).is_none(), "repeated field");
                #[allow(unused_assignments)] {
                    c += 1;
                }
            )+
            amplify::confinement::Confined::try_from(m).expect("too many fields").into()
        }
    };
    { unnamed $($ord:literal => $value:expr),+ $(,)? } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert($crate::ast::Field::unnamed($ord), $value.into()).is_none(), "repeated field");
            )+
            amplify::confinement::Confined::try_from(m).expect("too many fields").into()
        }
    };
    { $($key:expr => $ord:literal => $value:expr),+ $(,)? } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert($crate::ast::Field::named(fname!($key), $ord), $value.into()).is_none(), "repeated field");
            )+
            amplify::confinement::Confined::try_from(m).expect("too many fields").into()
        }
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut c = 0u8;
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert($crate::ast::Field::named(fname!($key), c), $value.into()).is_none(), "repeated field");
                #[allow(unused_assignments)] {
                    c += 1;
                }
            )+
            amplify::confinement::Confined::try_from(m).expect("too many fields").into()
        }
    }
}

#[macro_export]
macro_rules! variants {
    { $from:literal..=$to:literal } => {
        {
            let mut m = ::std::collections::BTreeSet::new();
            for i in $from..=$to {
                assert!(m.insert($crate::ast::Field::unnamed(i)), "repeated enum variant");
            }
            amplify::confinement::Confined::try_from(m).expect("too many enum variants").into()
        }
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut m = ::std::collections::BTreeSet::new();
            $(
                assert!(m.insert($crate::ast::Field::named(tn!($key), $value)), "repeated enum variant");
            )+
            amplify::confinement::Confined::try_from(m).expect("too many enum variants").into()
        }
    };
}
