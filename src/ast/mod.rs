// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
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

mod ty;
mod id;
mod serialize;
mod translate;
mod path;

pub use id::TyId;
pub use path::{Path, PathError, Step, TyIter};
pub use translate::{Translate, TranslateError};
pub use ty::{Field, FieldName, Fields, KeyTy, NestedRef, SubTy, Ty, TyInner, TypeRef, Variants};
