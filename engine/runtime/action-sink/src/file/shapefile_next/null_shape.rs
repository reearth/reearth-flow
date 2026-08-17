//! The `.shp` and `.shx` of a shapefile whose every record is a null shape.
//!
//! `shapefile::Writer` cannot write a null record, so a file of features carrying
//! no geometry is assembled here instead.

use std::io::{Result, Write};

use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

/// Bytes of the header both files start with.
const HEADER_BYTES: usize = 100;
/// Bytes of one `.shp` record: an 8-byte record header, then a 4-byte shape type
/// and no content beyond it.
const SHP_RECORD_BYTES: usize = 12;
/// Bytes of one `.shx` index record: a file offset and a content length.
const SHX_RECORD_BYTES: usize = 8;
/// A null record's content length, in 16-bit words: the shape type alone.
const NULL_CONTENT_WORDS: i32 = 2;

/// Write a `.shp` holding `feature_count` null records.
pub(super) fn write_shp(mut writer: impl Write, feature_count: usize) -> Result<()> {
    write_header(&mut writer, HEADER_BYTES + feature_count * SHP_RECORD_BYTES)?;
    for i in 0..feature_count {
        // Record numbers start at 1.
        writer.write_u32::<BigEndian>(i as u32 + 1)?;
        writer.write_i32::<BigEndian>(NULL_CONTENT_WORDS)?;
        writer.write_u32::<LittleEndian>(0)?;
    }
    Ok(())
}

/// Write the `.shx` indexing the `.shp` [`write_shp`] produces for the same count.
pub(super) fn write_shx(mut writer: impl Write, feature_count: usize) -> Result<()> {
    write_header(&mut writer, HEADER_BYTES + feature_count * SHX_RECORD_BYTES)?;
    for i in 0..feature_count {
        let offset = HEADER_BYTES + i * SHP_RECORD_BYTES;
        writer.write_i32::<BigEndian>(words(offset))?;
        writer.write_i32::<BigEndian>(NULL_CONTENT_WORDS)?;
    }
    Ok(())
}

/// The 100-byte header both files share. `file_bytes` is the length of the file
/// the header belongs to, which the two files do not share.
fn write_header(mut writer: impl Write, file_bytes: usize) -> Result<()> {
    writer.write_u32::<BigEndian>(9994)?;
    // Bytes 4..24 are unused.
    for _ in 0..5 {
        writer.write_u32::<BigEndian>(0)?;
    }
    writer.write_i32::<BigEndian>(words(file_bytes))?;
    writer.write_u32::<LittleEndian>(1000)?;
    // Shape type: null.
    writer.write_u32::<LittleEndian>(0)?;

    // Bounding box, then the Z and M ranges: a file of null records bounds nothing.
    for _ in 0..8 {
        writer.write_f64::<LittleEndian>(0.0)?;
    }
    Ok(())
}

/// A byte count as the 16-bit word count the format records lengths and offsets in.
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

    /// The length a header declares is the length of its own file, in 16-bit words.
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

    /// The files are well-formed enough for a reader to get the records back.
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
