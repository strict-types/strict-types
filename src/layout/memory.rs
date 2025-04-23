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

use amplify::confinement::LargeVec;
use strict_encoding::STRICT_TYPES_LIB;

#[cfg(feature = "vesper")]
use super::vesper::TypeVesper;
use crate::typesys::{TypeInfo, TypeTree};

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct MemoryLayout {
    items: LargeVec<TypeInfo>,
}

impl From<TypeTree<'_>> for MemoryLayout {
    fn from(tree: TypeTree) -> Self {
        let mut layout = MemoryLayout::new();
        layout.items.extend(&tree).expect("type layout exceeds billions of fields");
        layout
    }
}

impl<'a> From<&'a TypeTree<'_>> for MemoryLayout {
    fn from(tree: &'a TypeTree) -> Self {
        let mut layout = MemoryLayout::new();
        layout.items.extend(tree).expect("type layout exceeds billions of fields");
        layout
    }
}

#[cfg(feature = "vesper")]
impl std::fmt::Display for MemoryLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.to_vesper().display(), f)
    }
}

impl MemoryLayout {
    fn new() -> Self { Self { items: empty!() } }

    #[cfg(feature = "vesper")]
    pub fn to_vesper(&self) -> TypeVesper {
        let mut root = None;
        let mut path: Vec<usize> = vec![];
        for item in &self.items {
            let expr = item.to_vesper();
            let depth = item.depth;

            if path.is_empty() && depth == 0 {
                debug_assert_eq!(root, None);
                root = Some(expr);
                continue;
            }

            debug_assert!(depth > 0);
            // if the stack top is the same depth or deeper:
            // - remove everything down from the depth
            // - take the remaining top and add the item as a new child
            // - create new item and push it to stack
            if path.len() >= depth as usize {
                let _ = path.split_off(depth as usize - 1);
            }
            // if the stack top is one level up
            // - create new item and add it as a child to the stack top item
            // - push the newly created item to stack
            let mut head = root.as_mut().expect("already set");
            for el in &path {
                head = head.content.get_mut(*el).expect("algorithm inconsistency");
            }
            path.push(head.content.len());
            head.content
                .push(Box::new(expr))
                .expect("invalid type layout containing too much items");
        }
        root.expect("invalid type layout with zero items")
    }
}
