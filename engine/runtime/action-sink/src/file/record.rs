//! Encoding of spilled records. Records are read back only by the process that
//! wrote them, so the encoding is native-endian and carries no versioning.

use std::io;
use std::str::FromStr;

use reearth_flow_types::datetime::DateTime;
use reearth_flow_types::{Attribute, AttributeValue, Attributes};

pub(crate) fn put_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

pub(crate) fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_ne_bytes());
}

pub(crate) fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    put_u32(out, bytes.len() as u32);
    out.extend_from_slice(bytes);
}

pub(crate) fn put_str(out: &mut Vec<u8>, s: &str) {
    put_bytes(out, s.as_bytes());
}

pub(crate) fn put_pods<T: bytemuck::Pod>(out: &mut Vec<u8>, values: &[T]) {
    put_u32(out, values.len() as u32);
    out.extend_from_slice(bytemuck::cast_slice(values));
}

const TAG_NULL: u8 = 0;
const TAG_BOOL: u8 = 1;
const TAG_NUMBER: u8 = 2;
const TAG_STRING: u8 = 3;
const TAG_DATETIME: u8 = 4;
const TAG_ARRAY: u8 = 5;
const TAG_MAP: u8 = 6;
const TAG_BYTES: u8 = 7;

pub(crate) fn encode_value(out: &mut Vec<u8>, value: &AttributeValue) {
    match value {
        AttributeValue::Null => put_u8(out, TAG_NULL),
        AttributeValue::Bool(b) => {
            put_u8(out, TAG_BOOL);
            put_u8(out, *b as u8);
        }
        AttributeValue::Number(n) => {
            put_u8(out, TAG_NUMBER);
            put_str(out, &n.to_string());
        }
        AttributeValue::String(s) => {
            put_u8(out, TAG_STRING);
            put_str(out, s);
        }
        AttributeValue::DateTime(dt) => {
            put_u8(out, TAG_DATETIME);
            put_str(out, &dt.to_raw());
        }
        AttributeValue::Array(values) => {
            put_u8(out, TAG_ARRAY);
            put_u32(out, values.len() as u32);
            for v in values {
                encode_value(out, v);
            }
        }
        AttributeValue::Map(map) => {
            put_u8(out, TAG_MAP);
            put_u32(out, map.len() as u32);
            for (k, v) in map {
                put_str(out, k);
                encode_value(out, v);
            }
        }
        AttributeValue::Bytes(bytes) => {
            put_u8(out, TAG_BYTES);
            put_bytes(out, bytes);
        }
    }
}

/// Writes `attributes` in their order.
pub(crate) fn encode_attributes(out: &mut Vec<u8>, attributes: &Attributes) {
    put_u32(out, attributes.len() as u32);
    for (key, value) in attributes {
        put_str(out, key.as_ref());
        encode_value(out, value);
    }
}

pub(crate) fn invalid(what: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("corrupt record: {what}"),
    )
}

pub(crate) struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if self.0.len() < n {
            return Err(invalid("truncated"));
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }

    pub(crate) fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub(crate) fn u32(&mut self) -> io::Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_ne_bytes(bytes.try_into().expect("took 4 bytes")))
    }

    pub(crate) fn bytes(&mut self) -> io::Result<&'a [u8]> {
        let len = self.u32()? as usize;
        self.take(len)
    }

    pub(crate) fn string(&mut self) -> io::Result<String> {
        let bytes = self.bytes()?;
        String::from_utf8(bytes.to_vec()).map_err(|_| invalid("non-UTF-8 string"))
    }

    pub(crate) fn pods<T: bytemuck::Pod>(&mut self) -> io::Result<Vec<T>> {
        let len = self.u32()? as usize;
        let bytes = self.take(len * std::mem::size_of::<T>())?;
        Ok(bytemuck::pod_collect_to_vec(bytes))
    }

    pub(crate) fn array<T: bytemuck::Pod, const N: usize>(&mut self) -> io::Result<[T; N]> {
        let values: Vec<T> = self.pods()?;
        values.try_into().map_err(|_| invalid("array length"))
    }
}

pub(crate) fn decode_value(r: &mut Reader<'_>) -> io::Result<AttributeValue> {
    Ok(match r.u8()? {
        TAG_NULL => AttributeValue::Null,
        TAG_BOOL => AttributeValue::Bool(r.u8()? != 0),
        TAG_NUMBER => AttributeValue::Number(
            serde_json::Number::from_str(&r.string()?).map_err(|_| invalid("number"))?,
        ),
        TAG_STRING => AttributeValue::String(r.string()?),
        TAG_DATETIME => AttributeValue::DateTime(
            DateTime::from_str(&r.string()?).map_err(|_| invalid("datetime"))?,
        ),
        TAG_ARRAY => {
            let len = r.u32()? as usize;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(decode_value(r)?);
            }
            AttributeValue::Array(values)
        }
        TAG_MAP => {
            let len = r.u32()? as usize;
            let mut map = std::collections::HashMap::with_capacity(len);
            for _ in 0..len {
                let key = r.string()?;
                map.insert(key, decode_value(r)?);
            }
            AttributeValue::Map(map)
        }
        TAG_BYTES => AttributeValue::Bytes(bytes::Bytes::copy_from_slice(r.bytes()?)),
        _ => return Err(invalid("attribute tag")),
    })
}

/// Reads attributes written by [`encode_attributes`].
pub(crate) fn decode_attributes(r: &mut Reader<'_>) -> io::Result<Attributes> {
    let len = r.u32()? as usize;
    let mut attributes = Attributes::with_capacity(len);
    for _ in 0..len {
        let key = Attribute::new(r.string()?);
        attributes.insert(key, decode_value(r)?);
    }
    Ok(attributes)
}
