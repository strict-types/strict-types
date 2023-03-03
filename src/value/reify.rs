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

//! Reification module: reads & writes strict values from binary strict encodings.

use std::io;

use crate::typify::TypedVal;
use crate::{SemId, TypeSystem};

impl TypeSystem {
    pub fn reify(&self, id: SemId, obj: TypedVal, e: impl io::Write) -> Result<(), Error> {
        todo!()
    }
    pub fn dereify(&self, id: SemId, d: impl io::Read) -> Result<TypedVal, Error> { todo!() }
}

pub enum Error {}