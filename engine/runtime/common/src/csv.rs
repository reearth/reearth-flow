use std::borrow::Cow;
use std::io::{BufRead, Cursor, Read};

use strum_macros::Display;

#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub enum Delimiter {
    Comma,
    Tab,
}

impl From<u8> for Delimiter {
    fn from(value: u8) -> Self {
        match value {
            b',' => Self::Comma,
            b'\t' => Self::Tab,
            _ => unreachable!(),
        }
    }
}

impl From<Delimiter> for u8 {
    fn from(value: Delimiter) -> Self {
        match value {
            Delimiter::Comma => b',',
            Delimiter::Tab => b'\t',
        }
    }
}

/// Decode bytes from the specified encoding to UTF-8.
///
/// If encoding is `None` or a UTF-8 variant, returns the original bytes
/// unchanged.  Otherwise, uses `encoding_rs` to transcode.
pub fn decode_to_utf8<'a>(
    content: &'a [u8],
    encoding: Option<&str>,
) -> Result<Cow<'a, [u8]>, String> {
    let encoding_name = match encoding {
        Some(name) if !name.is_empty() => name,
        _ => return Ok(Cow::Borrowed(content)),
    };

    let name_upper = encoding_name.to_uppercase();
    if matches!(name_upper.as_str(), "UTF-8" | "UTF8" | "UNICODE" | "UTF_8") {
        return Ok(Cow::Borrowed(content));
    }

    let enc = encoding_rs::Encoding::for_label(encoding_name.as_bytes())
        .ok_or_else(|| format!("Unsupported encoding: {encoding_name}"))?;

    let (decoded, _, had_errors) = enc.decode(content);
    if had_errors {
        tracing::warn!(
            "Encoding conversion from {} had unmappable characters (replaced with U+FFFD)",
            enc.name()
        );
    }
    Ok(Cow::Owned(decoded.into_owned().into_bytes()))
}

/// Build a CSV/TSV `csv::Reader` from raw bytes, applying encoding
/// conversion and skipping `offset` leading lines (via raw line reads so
/// that blank rows are counted correctly).
pub fn build_csv_reader(
    content: &[u8],
    encoding: Option<&str>,
    delimiter: Delimiter,
    offset: usize,
) -> Result<csv::Reader<Cursor<Vec<u8>>>, String> {
    let decoded = decode_to_utf8(content, encoding)?;

    // Copy into an owned Vec so the Cursor can own the data.
    let mut cursor = Cursor::new(decoded.into_owned());

    // Skip `offset` raw lines (BufRead-based) so blank rows are counted.
    for _ in 0..offset {
        let mut line = String::new();
        cursor
            .read_line(&mut line)
            .map_err(|e| format!("Failed to skip offset line: {e:?}"))?;
    }

    // Read the remaining bytes into a new Cursor so the csv crate starts
    // at the right position.
    let mut remaining = Vec::new();
    cursor
        .read_to_end(&mut remaining)
        .map_err(|e| format!("Failed to read CSV content: {e:?}"))?;

    let rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .trim(csv::Trim::All)
        .delimiter(delimiter.into())
        .from_reader(Cursor::new(remaining));
    Ok(rdr)
}

/// Read and merge `header_rows` rows from a csv::Reader into a single
/// header vector.
///
/// - `header_rows == 0`: returns an empty vec (auto-generate later).
/// - `header_rows == 1`: standard single header row.
/// - `header_rows > 1`: joins non-empty cell values across rows with `_`.
pub fn read_merged_header<R: std::io::Read>(
    rdr: &mut csv::Reader<R>,
    header_rows: usize,
) -> Result<Vec<String>, String> {
    if header_rows == 0 {
        return Ok(Vec::new());
    }

    let mut iter = rdr.deserialize::<Vec<String>>();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(header_rows);
    for _ in 0..header_rows {
        let row: Vec<String> = iter
            .next()
            .unwrap_or(Ok(Vec::new()))
            .map_err(|e| format!("Failed to read header row: {e:?}"))?;
        rows.push(row);
    }

    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let header = (0..max_cols)
        .map(|col_idx| {
            rows.iter()
                .filter_map(|row| row.get(col_idx).map(|s| s.trim()).filter(|s| !s.is_empty()))
                .collect::<Vec<_>>()
                .join("_")
        })
        .collect();
    Ok(header)
}

/// Generate auto-column names ("column1", "column2", …) for header-less
/// CSV files.  Call this on the first data row when `header_rows == 0`.
pub fn auto_generate_header(column_count: usize) -> Vec<String> {
    (0..column_count)
        .map(|i| format!("column{}", i + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delimiter_from_u8() {
        assert_eq!(Delimiter::from(b','), Delimiter::Comma);
        assert_eq!(Delimiter::from(b'\t'), Delimiter::Tab);
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn test_delimiter_from_u8_unreachable() {
        let _ = Delimiter::from(b'#');
    }

    #[test]
    fn test_delimiter_into_u8() {
        assert_eq!(u8::from(Delimiter::Comma), b',');
        assert_eq!(u8::from(Delimiter::Tab), b'\t');
    }

    #[test]
    fn test_decode_to_utf8_none() {
        let data = b"hello";
        let result = decode_to_utf8(data, None).unwrap();
        assert_eq!(&*result, b"hello");
    }

    #[test]
    fn test_decode_to_utf8_utf8() {
        let data = b"hello";
        let result = decode_to_utf8(data, Some("UTF-8")).unwrap();
        assert_eq!(&*result, b"hello");
    }

    #[test]
    fn test_decode_to_utf8_unsupported() {
        let data = b"hello";
        let result = decode_to_utf8(data, Some("UNSUPPORTED-ENCODING-XYZ"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_merged_header_single() {
        let data = b"a,b,c\n1,2,3\n";
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Cursor::new(data.to_vec()));
        let header = read_merged_header(&mut rdr, 1).unwrap();
        assert_eq!(header, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_read_merged_header_multi() {
        let data = b"cat,,dog\nX,Y,Z\n1,2,3\n";
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Cursor::new(data.to_vec()));
        let header = read_merged_header(&mut rdr, 2).unwrap();
        assert_eq!(header, vec!["cat_X", "Y", "dog_Z"]);
    }

    #[test]
    fn test_read_merged_header_zero() {
        let data = b"1,2,3\n";
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Cursor::new(data.to_vec()));
        let header = read_merged_header(&mut rdr, 0).unwrap();
        assert!(header.is_empty());
    }

    #[test]
    fn test_auto_generate_header() {
        let header = auto_generate_header(3);
        assert_eq!(header, vec!["column1", "column2", "column3"]);
    }
}
