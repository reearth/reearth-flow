//! PLATEAU 4 (CityGML 2.0) objectlist reader: the single `A.3.1_取得項目一覧`
//! sheet.

use bytes::Bytes;

use crate::common::object_list_extractor::ObjectListReader;
use crate::object_list::{self, FeatureTypes, ObjectListMap};

#[derive(Debug)]
pub(crate) struct Plateau4ObjectListReader;

impl ObjectListReader for Plateau4ObjectListReader {
    fn read(&self, bytes: Bytes) -> object_list::Result<(FeatureTypes, ObjectListMap)> {
        object_list::parse(bytes)
    }
}
