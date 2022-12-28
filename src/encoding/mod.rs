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

#[macro_use]
mod macros;
mod types;
mod traits;
mod error;
mod read;
mod write;
mod base;
#[cfg(test)]
pub(crate) mod test;

pub use error::{DecodeError, DeserializeError, SerializeError};
pub use read::StrictReader;
pub use traits::*;
pub use types::*;
pub use write::{SplitParent, StrictParent, StrictWriter, StructWriter, UnionWriter};
