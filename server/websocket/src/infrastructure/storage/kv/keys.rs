use smallvec::{smallvec, SmallVec};
use std::io::Write;
use std::ops::Deref;

pub const V1: u8 = 0;

pub const KEYSPACE_OID: u8 = 0;

pub const KEYSPACE_DOC: u8 = 1;

pub const SUB_DOC: u8 = 0;

pub const SUB_STATE_VEC: u8 = 1;

pub const SUB_UPDATE: u8 = 2;

pub const SUB_META: u8 = 3;

pub const TERMINATOR: u8 = 0;
pub const TERMINATOR_HI_WATERMARK: u8 = 255;

pub type OID = u32;

pub fn key_oid(doc_name: &[u8]) -> Result<Key<20>, std::io::Error> {
    let mut v: SmallVec<[u8; 20]> = smallvec![V1, KEYSPACE_OID];
    v.write_all(doc_name)?;
    v.push(TERMINATOR);
    Ok(Key(v))
}

pub fn key_doc(oid: OID) -> Result<Key<8>, std::io::Error> {
    let mut v: SmallVec<[u8; 8]> = smallvec![V1, KEYSPACE_DOC];
    v.write_all(&oid.to_be_bytes())?;
    v.push(SUB_DOC);
    Ok(Key(v))
}

pub fn key_doc_start(oid: OID) -> Result<Key<8>, std::io::Error> {
    key_doc(oid)
}

pub fn key_doc_end(oid: OID) -> Result<Key<8>, std::io::Error> {
    let mut v: SmallVec<[u8; 8]> = smallvec![V1, KEYSPACE_DOC];
    v.write_all(&oid.to_be_bytes())?;
    v.push(TERMINATOR_HI_WATERMARK);
    Ok(Key(v))
}

pub fn key_state_vector(oid: OID) -> Result<Key<8>, std::io::Error> {
    let mut v: SmallVec<[u8; 8]> = smallvec![V1, KEYSPACE_DOC];
    v.write_all(&oid.to_be_bytes())?;
    v.push(SUB_STATE_VEC);
    Ok(Key(v))
}

pub fn key_update(oid: OID, clock: u32) -> Result<Key<12>, std::io::Error> {
    let mut v: SmallVec<[u8; 12]> = smallvec![V1, KEYSPACE_DOC];
    v.write_all(&oid.to_be_bytes())?;
    v.push(SUB_UPDATE);
    v.write_all(&clock.to_be_bytes())?;
    v.push(TERMINATOR);
    Ok(Key(v))
}

pub fn doc_meta_name(key: &[u8]) -> &[u8] {
    if key.len() < 7 {
        return &[];
    }
    &key[7..(key.len() - 1)]
}

pub fn doc_oid_name(key: &[u8]) -> &[u8] {
    if key.len() < 2 {
        return &[];
    }
    &key[2..(key.len() - 1)]
}

pub fn key_meta(oid: OID, name: &[u8]) -> Result<Key<20>, std::io::Error> {
    let mut v: SmallVec<[u8; 20]> = smallvec![V1, KEYSPACE_DOC];
    v.write_all(&oid.to_be_bytes())?;
    v.push(SUB_META);
    v.write_all(name)?;
    v.push(TERMINATOR);
    Ok(Key(v))
}

pub fn key_meta_start(oid: OID) -> Result<Key<8>, std::io::Error> {
    let mut v: SmallVec<[u8; 8]> = smallvec![V1, KEYSPACE_DOC];
    v.write_all(&oid.to_be_bytes())?;
    v.push(SUB_META);
    v.push(TERMINATOR);
    Ok(Key(v))
}

pub fn key_meta_end(oid: OID) -> Result<Key<8>, std::io::Error> {
    let mut v: SmallVec<[u8; 8]> = smallvec![V1, KEYSPACE_DOC];
    v.write_all(&oid.to_be_bytes())?;
    v.push(SUB_META + 1);
    Ok(Key(v))
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key<const N: usize>(SmallVec<[u8; N]>);

impl<const N: usize> Key<N> {
    pub const fn from_const(src: [u8; N]) -> Self {
        Key(SmallVec::from_const(src))
    }
}

impl<const N: usize> Deref for Key<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<const N: usize> AsRef<[u8]> for Key<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<const N: usize> AsMut<[u8]> for Key<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl<const N: usize> From<Key<N>> for Vec<u8> {
    fn from(val: Key<N>) -> Self {
        val.0.to_vec()
    }
}
