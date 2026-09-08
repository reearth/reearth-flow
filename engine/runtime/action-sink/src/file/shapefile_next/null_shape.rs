//! The `.shp` and `.shx` of a shapefile whose every record is a null shape,
//! which `shapefile::Writer` cannot write.

use std::io::{Result, Write};

use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

/// Bytes of the header both files start with.
const HEADER_BYTES: usize = 100;
/// The file code every header opens with.
const FILE_CODE: u32 = 9994;
/// The format version every header states.
const VERSION: u32 = 1000;
/// The null shape type.
const NULL_SHAPE_TYPE: u32 = 0;
/// The number of the first record.
const FIRST_RECORD_NUMBER: u32 = 1;
/// Bytes of one `.shp` null record: its header and shape type.
const SHP_RECORD_BYTES: usize = 12;
/// Bytes of one `.shx` index record.
const SHX_RECORD_BYTES: usize = 8;
/// A null record's content length in 16-bit words.
const NULL_CONTENT_WORDS: i32 = 2;

/// Write a `.shp` holding `feature_count` null records.
pub(super) fn write_shp(mut writer: impl Write, feature_count: usize) -> Result<()> {
    write_header(&mut writer, HEADER_BYTES + feature_count * SHP_RECORD_BYTES)?;
    for i in 0..feature_count {
        writer.write_u32::<BigEndian>(FIRST_RECORD_NUMBER + i as u32)?;
        writer.write_i32::<BigEndian>(NULL_CONTENT_WORDS)?;
        writer.write_u32::<LittleEndian>(NULL_SHAPE_TYPE)?;
    }
    Ok(())
}

/// Write the `.shx` indexing the `.shp` of [`write_shp`] for the same count.
pub(super) fn write_shx(mut writer: impl Write, feature_count: usize) -> Result<()> {
    write_header(&mut writer, HEADER_BYTES + feature_count * SHX_RECORD_BYTES)?;
    for i in 0..feature_count {
        let offset = HEADER_BYTES + i * SHP_RECORD_BYTES;
        writer.write_i32::<BigEndian>(words(offset))?;
        writer.write_i32::<BigEndian>(NULL_CONTENT_WORDS)?;
    }
    Ok(())
}

/// Write the header of a file of `file_bytes`: file code, five unused words,
/// length, version, shape type, then a zero bounding box and Z and M ranges.
fn write_header(mut writer: impl Write, file_bytes: usize) -> Result<()> {
    writer.write_u32::<BigEndian>(FILE_CODE)?;
    for _ in 0..5 {
        writer.write_u32::<BigEndian>(0)?;
    }
    writer.write_i32::<BigEndian>(words(file_bytes))?;
    writer.write_u32::<LittleEndian>(VERSION)?;
    writer.write_u32::<LittleEndian>(NULL_SHAPE_TYPE)?;
    for _ in 0..8 {
        writer.write_f64::<LittleEndian>(0.0)?;
    }
    Ok(())
}

/// A byte count in 16-bit words.
fn words(bytes: usize) -> i32 {
    (bytes / 2) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn files(feature_count: usize) -> (Vec<u8>, Vec<u8>) {
        let mut shp = Vec::new();
        write_shp(&mut shp, feature_count).unwrap();
        let mut shx = Vec::new();
        write_shx(&mut shx, feature_count).unwrap();
        (shp, shx)
    }

    /// A header declares its own file's length in 16-bit words.
    #[test]
    fn each_header_declares_the_length_of_its_own_file() {
        for count in [0, 1, 5] {
            let (shp, shx) = files(count);
            assert_eq!(
                i32::from_be_bytes(shp[24..28].try_into().unwrap()) as usize * 2,
                shp.len(),
                "shp length for {count} records"
            );
            assert_eq!(
                i32::from_be_bytes(shx[24..28].try_into().unwrap()) as usize * 2,
                shx.len(),
                "shx length for {count} records"
            );
        }
    }

    /// The files read back as null shapes.
    #[test]
    fn the_records_read_back_as_null_shapes() {
        let (shp, shx) = files(3);
        let reader =
            shapefile::ShapeReader::with_shx(std::io::Cursor::new(shp), std::io::Cursor::new(shx))
                .expect("the written files are expected to be readable");
        let shapes: Vec<shapefile::Shape> = reader
            .read()
            .expect("the records are expected to be readable");
        assert_eq!(shapes.len(), 3);
        assert!(shapes
            .iter()
            .all(|s| matches!(s, shapefile::Shape::NullShape)));
    }
}
