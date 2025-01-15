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

use crate::component::ResourceAny;
use crate::component::Val;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableVal {
    Bool(bool),
    S8(i8),
    U8(u8),
    S16(i16),
    U16(u16),
    S32(i32),
    U32(u32),
    S64(i64),
    U64(u64),
    Float32(f32),
    Float64(f64),
    Char(char),
    String(String),
    List(Vec<SerializableVal>),
    Record(Vec<(String, SerializableVal)>),
    Tuple(Vec<SerializableVal>),
    Variant(String, Option<Box<SerializableVal>>),
    Enum(String),
    Option(Option<Box<SerializableVal>>),
    Result(Result<Option<Box<SerializableVal>>, Option<Box<SerializableVal>>>),
    Flags(Vec<String>),
    Resource(#[serde(with = "fthat")] ResourceAny), // lets get rid of dynamic typing who even
                                                    // wants that its lame and stupid
}

impl SerializableVal {
    pub fn from_val(val: &Val) -> Result<SerializableVal> {
        Ok(match val {
            Val::Bool(b) => SerializableVal::Bool(*b),
            Val::S8(n) => SerializableVal::S8(*n),
            Val::U8(n) => SerializableVal::U8(*n),
            Val::S16(n) => SerializableVal::S16(*n),
            Val::U16(n) => SerializableVal::U16(*n),
            Val::S32(n) => SerializableVal::S32(*n),
            Val::U32(n) => SerializableVal::U32(*n),
            Val::S64(n) => SerializableVal::S64(*n),
            Val::U64(n) => SerializableVal::U64(*n),
            Val::Float32(n) => SerializableVal::Float32(*n),
            Val::Float64(n) => SerializableVal::Float64(*n),
            Val::Char(c) => SerializableVal::Char(*c),
            Val::String(s) => SerializableVal::String(s.clone()),
            Val::List(l) => SerializableVal::List(
                l.iter()
                    .map(SerializableVal::from_val)
                    .collect::<Result<Vec<_>>>()?,
            ),
            Val::Record(r) => SerializableVal::Record(
                r.iter()
                    .map(|(k, v)| Ok((k.clone(), SerializableVal::from_val(v)?)))
                    .collect::<Result<Vec<_>>>()?,
            ),
            Val::Tuple(t) => SerializableVal::Tuple(
                t.iter()
                    .map(SerializableVal::from_val)
                    .collect::<Result<Vec<_>>>()?,
            ),
            Val::Variant(name, val) => SerializableVal::Variant(
                name.clone(),
                val.as_ref()
                    .map(|v| -> Result<Box<SerializableVal>> {
                        Ok(Box::new(SerializableVal::from_val(v)?))
                    })
                    .transpose()?,
            ),
            Val::Enum(e) => SerializableVal::Enum(e.clone()),
            Val::Option(o) => SerializableVal::Option(
                o.as_ref()
                    .map(|v| -> Result<Box<SerializableVal>> {
                        Ok(Box::new(SerializableVal::from_val(v)?))
                    })
                    .transpose()?,
            ),
            Val::Result(r) => SerializableVal::Result(match r {
                Ok(v) => Ok(v
                    .as_ref()
                    .map(|v| -> Result<Box<SerializableVal>> {
                        Ok(Box::new(SerializableVal::from_val(v)?))
                    })
                    .transpose()?),
                Err(v) => Err(v
                    .as_ref()
                    .map(|v| -> Result<Box<SerializableVal>> {
                        Ok(Box::new(SerializableVal::from_val(v)?))
                    })
                    .transpose()?),
            }),
            Val::Flags(f) => SerializableVal::Flags(f.clone()),
            Val::Resource(_r) => {
                panic!("AHHHHHH: Resource serialization not yet implemented")
            }
        })
    }

    pub fn from_vals(vals: &[Val]) -> Result<Vec<SerializableVal>> {
        vals.iter().map(SerializableVal::from_val).collect()
    }
}

impl std::hash::Hash for SerializableVal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // First hash the discriminant to differentiate between variants
        std::mem::discriminant(self).hash(state);

        match self {
            // For floats, we need special handling to match potential PartialEq implementation
            // where NaN == NaN and -0.0 == 0.0
            Self::Float32(f) => {
                if f.is_nan() {
                    // Hash all NaNs the same
                    state.write_u32(u32::MAX);
                } else if *f == 0.0 {
                    // Hash -0.0 and 0.0 the same
                    state.write_u32(0);
                } else {
                    f.to_bits().hash(state);
                }
            }
            Self::Float64(f) => {
                if f.is_nan() {
                    // Hash all NaNs the same
                    state.write_u64(u64::MAX);
                } else if *f == 0.0 {
                    // Hash -0.0 and 0.0 the same
                    state.write_u64(0);
                } else {
                    f.to_bits().hash(state);
                }
            }
            // For all other variants, just hash their contents directly
            Self::Bool(v) => v.hash(state),
            Self::S8(v) => v.hash(state),
            Self::U8(v) => v.hash(state),
            Self::S16(v) => v.hash(state),
            Self::U16(v) => v.hash(state),
            Self::S32(v) => v.hash(state),
            Self::U32(v) => v.hash(state),
            Self::S64(v) => v.hash(state),
            Self::U64(v) => v.hash(state),
            Self::Char(v) => v.hash(state),
            Self::String(v) => v.hash(state),
            Self::List(v) => v.hash(state),
            Self::Record(v) => v.hash(state),
            Self::Tuple(v) => v.hash(state),
            Self::Variant(name, val) => {
                name.hash(state);
                val.hash(state);
            }
            Self::Enum(v) => v.hash(state),
            Self::Option(v) => v.hash(state),
            Self::Result(v) => v.hash(state),
            Self::Flags(v) => v.hash(state),
            Self::Resource(_) => panic!("AHHHHHH: Resource serialization not yet implemented"),
        }
    }
}
mod fthat {

    use crate::component::ResourceAny;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(_resource: &ResourceAny, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unimplemented!("AHHHHHH: Resource serialization not yet implemented")
    }

    pub fn deserialize<'de, D>(_deserializer: D) -> Result<ResourceAny, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!("AHHHHHH: Resource deserialization not yet implemented")
    }
}
