// Strict encoding schema library, implementing validation and parsing of strict encoded data
// against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2019-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2024-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2022-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

//! Converts strict values from/to non-STON value serialization formats (JSON, YAML, TOML etc).

use crate::StrictVal;

impl From<serde_json::Value> for StrictVal {
    fn from(json: serde_json::Value) -> Self {
        use serde_json::Value;

        match json {
            Value::Null => StrictVal::Unit,
            Value::Bool(v) => StrictVal::bool(v),
            Value::Number(no) if no.is_u64() => StrictVal::num(no.as_u64().unwrap()),
            Value::Number(no) if no.is_i64() => StrictVal::num(no.as_i64().unwrap()),
            Value::Number(no) if no.is_f64() => todo!(),
            Value::Number(_) => {
                unreachable!()
            }
            Value::String(s) => StrictVal::String(s),
            Value::Array(vec) => StrictVal::list(vec.into_iter().map(StrictVal::from)),
            Value::Object(map) => {
                StrictVal::map(map.into_iter().map(|(k, v)| (k, StrictVal::from(v))))
            }
        }
    }
}

impl From<serde_yaml::Value> for StrictVal {
    fn from(yaml: serde_yaml::Value) -> Self {
        use serde_yaml::Value;

        match yaml {
            Value::Null => StrictVal::Unit,
            Value::Bool(v) => StrictVal::bool(v),
            Value::Number(no) if no.is_u64() => StrictVal::num(no.as_u64().unwrap()),
            Value::Number(no) if no.is_i64() => StrictVal::num(no.as_i64().unwrap()),
            Value::Number(no) if no.is_f64() => todo!(),
            Value::Number(_) => {
                unreachable!()
            }
            Value::String(s) => StrictVal::String(s),
            Value::Sequence(vec) => StrictVal::list(vec.into_iter().map(StrictVal::from)),
            Value::Mapping(map) => {
                StrictVal::map(map.into_iter().map(|(k, v)| (k, StrictVal::from(v))))
            }
            Value::Tagged(tagged) => StrictVal::from(tagged.value),
        }
    }
}

impl From<toml::Value> for StrictVal {
    fn from(toml: toml::Value) -> Self {
        use toml::Value;

        match toml {
            Value::Integer(no) => StrictVal::num(no),
            Value::Float(_f) => todo!(),
            Value::Boolean(v) => StrictVal::bool(v),
            Value::String(s) => StrictVal::String(s),
            Value::Array(vec) => StrictVal::list(vec.into_iter().map(StrictVal::from)),
            Value::Table(map) => {
                StrictVal::map(map.into_iter().map(|(k, v)| (k, StrictVal::from(v))))
            }
            Value::Datetime(_dt) => todo!(),
        }
    }
}
