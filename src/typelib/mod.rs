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

mod id;
mod type_lib;
mod compile;
mod serialize;
mod builder;
mod symbolic;
mod translate;

pub use builder::LibBuilder;
pub(crate) use compile::NestedContext;
#[allow(deprecated)]
pub use compile::TranslateError;
pub use compile::{CompileError, TypeIndex};
pub use id::TypeLibId;
pub use symbolic::{ExternTypes, SymbolRef, SymbolicLib, TranspileError, TranspileRef};
use translate::SymbolContext;
pub use translate::SymbolError;
pub(crate) use type_lib::TypeMap;
pub use type_lib::{
    Dependency, ExternRef, InlineRef, InlineRef1, InlineRef2, LibRef, LibSubref, TypeLib,
};

#[deprecated(since = "1.3.0", note = "import from the crate root")]
pub use super::parse_args;
