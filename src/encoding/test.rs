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

use std::fmt::Debug;
use std::io;

use crate::{Decode, Encode, Writer};

#[allow(dead_code)]
pub fn encoding_roundtrip(val: &(impl Encode + Decode + Debug + Eq)) {
    let mut buf = Writer::in_memory();
    val.encode(&mut buf).unwrap();
    let val2 = Decode::decode(&mut io::Cursor::new(buf.unbox())).unwrap();
    assert_eq!(val, &val2);
}

pub fn encoding(val: &(impl Encode + Decode + Debug + Eq), expect: impl AsRef<[u8]>) {
    let mut buf = Writer::in_memory();
    val.encode(&mut buf).unwrap();
    let buf = buf.unbox();
    assert_eq!(&buf[..], expect.as_ref());
    let val2 = Decode::decode(&mut io::Cursor::new(buf)).unwrap();
    assert_eq!(val, &val2);
}
