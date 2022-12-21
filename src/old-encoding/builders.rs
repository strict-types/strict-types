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

enum Example {
    Init(u8),
    Ping,
    Connect { host: Option<Vec<u8>> },
}

impl Encode for Example {
    fn sten_dumb() -> Self { Example::Ping }

    fn encode(&self, writer: impl TypedWrite) -> Result<(), Error> {
        let union = writer
            .write_union("Test", Some("Example"))
            .define_type("init", u8::sten_dumb())
            .define_unit("ping")
            .define_struct("connect")
            .define_field("host", Some(Vec::<u8>::sten_dumb()))
            .complete();
        match self {
            Example::Init(val) => union.write_value("init", val),
            Example::Ping => union.write_unit("ping"),
            Example::Connect { host } => union.write_struct("connect").write_field(host).complete(),
        }
        Ok(());

        reader.read_union("Test", Some("Example"), |field, r| match field {
            f!(0u8, "init") => Example::Init(r.read_type()),
            f!(2u8, "connect") => Example::Connect {
                host: r.read_struct().read_field("host").complete(),
            },
        })
    }
}
