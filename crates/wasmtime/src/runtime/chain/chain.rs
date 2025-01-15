// Copyright 2024 Colin Rozzi
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

//use crate::chain::SerializableVal;
use crate::component::__internal::{
    CanonicalAbiInfo, InstanceType, InterfaceType, LiftContext, LowerContext,
};
use crate::component::{ComponentType, Lift, Lower};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::vec::Vec;

// If you need error handling
use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaEvent {
    hash: u64,
    event: Event,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Event {
    type_: String,
    parent: Option<u64>,
    data: Vec<u8>,
}

impl Event {
    pub fn new(type_: String, data: Vec<u8>) -> Self {
        Event {
            type_,
            parent: None,
            data,
        }
    }

    fn calculate_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chain {
    events: Vec<MetaEvent>,
}

impl Chain {
    pub fn new() -> Self {
        Chain { events: Vec::new() }
    }

    pub fn add(&mut self, mut event: Event) -> u64 {
        let hash = event.calculate_hash();
        let parent_hash = self.events.last().map(|last| last.hash);
        event.parent = parent_hash;

        let node = MetaEvent { event, hash };

        self.events.push(node);
        hash
    }

    pub fn get_event_by_hash(&self, hash: u64) -> Option<&MetaEvent> {
        self.events.iter().find(|node| node.hash == hash)
    }

    pub fn get_parent(&self, hash: u64) -> Option<&MetaEvent> {
        self.events
            .iter()
            .find(|node| node.hash == hash)
            .and_then(|node| node.event.parent)
            .and_then(|parent_hash| self.get_event_by_hash(parent_hash))
    }

    pub fn head(&self) -> Option<u64> {
        self.events.last().map(|node| node.hash)
    }
}
unsafe impl ComponentType for Chain {
    type Lower = <String as ComponentType>::Lower; // Use String instead of str

    const ABI: CanonicalAbiInfo = CanonicalAbiInfo::POINTER_PAIR;

    fn typecheck(ty: &InterfaceType, _types: &InstanceType<'_>) -> Result<()> {
        match ty {
            InterfaceType::String => Ok(()),
            other => bail!("expected string found {:?}", other),
        }
    }
}

unsafe impl Lower for Chain {
    fn lower<T>(
        &self,
        cx: &mut LowerContext<'_, T>,
        ty: InterfaceType,
        dst: &mut MaybeUninit<Self::Lower>,
    ) -> Result<()> {
        // Convert Chain to JSON string
        let json = serde_json::to_string(self)?;
        // Use existing string lowering
        <String as Lower>::lower(&json, cx, ty, dst)
    }

    fn store<T>(
        &self,
        cx: &mut LowerContext<'_, T>,
        ty: InterfaceType,
        offset: usize,
    ) -> Result<()> {
        let json = serde_json::to_string(self)?;
        <String as Lower>::store(&json, cx, ty, offset)
    }
}

unsafe impl Lift for Chain {
    fn lift(cx: &mut LiftContext<'_>, ty: InterfaceType, src: &Self::Lower) -> Result<Self> {
        // Get the string using existing string lifting
        let json = <String as Lift>::lift(cx, ty, src)?;
        // Parse JSON back to Chain
        Ok(serde_json::from_str(&json)?)
    }

    fn load(cx: &mut LiftContext<'_>, ty: InterfaceType, bytes: &[u8]) -> Result<Self> {
        let json = <String as Lift>::load(cx, ty, bytes)?;
        Ok(serde_json::from_str(&json)?)
    }
}
