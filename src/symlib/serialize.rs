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

use std::fmt;
use std::fmt::{Display, Formatter};

use encoding::{StrictDeserialize, StrictSerialize};

use crate::SymbolicLib;

impl StrictSerialize for SymbolicLib {}
impl StrictDeserialize for SymbolicLib {}

impl Display for SymbolicLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typelib {}", self.name())?;
        writeln!(f)?;
        for dep in self.dependencies() {
            writeln!(f, "{dep} as {}", dep.name)?;
        }
        if self.dependencies().is_empty() {
            f.write_str("-- no dependencies")?;
        }
        writeln!(f)?;
        writeln!(f)?;
        for (name, ty) in self.types() {
            writeln!(f, "data {name:16} :: {ty}")?;
        }
        Ok(())
    }
}
