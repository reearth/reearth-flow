//! Dynamic integer type selection for 3D Tiles metadata encoding.
//!
//! Selects the smallest integer type that can represent all values while
//! reserving MIN (signed) or MAX (unsigned) as noData sentinel values.
//!
//! # Usage
//! ```ignore
//! let mut collector = SignedIntCollector::new();
//! collector.push(42);
//! collector.push(-10);
//! collector.push_no_data();  // noData sentinel
//! let finalized = collector.finalize();
//! finalized.encode_all(&mut buffer);
//! ```

use nusamai_gltf::nusamai_gltf_json::extensions::gltf::ext_structural_metadata::ClassPropertyComponentType;

// Signed integer noData values - use MIN to avoid collision with valid data
const INT8_NO_DATA: i8 = i8::MIN;
const INT16_NO_DATA: i16 = i16::MIN;
const INT32_NO_DATA: i32 = i32::MIN;
const INT64_NO_DATA: i64 = i64::MIN;

// Unsigned integer noData values - use MAX to avoid collision with valid data
const UINT8_NO_DATA: u8 = u8::MAX;
const UINT16_NO_DATA: u16 = u16::MAX;
const UINT32_NO_DATA: u32 = u32::MAX;
const UINT64_NO_DATA: u64 = u64::MAX;

/// Collects signed integer values and tracks min/max for optimal type selection.
#[derive(Debug, Clone)]
pub struct SignedIntCollector {
    values: Vec<i64>,
    min_signed: i64,
    max_unsigned: u64,
}

impl Default for SignedIntCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignedIntCollector {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            min_signed: i64::MAX,
            max_unsigned: 0,
        }
    }

    /// Push a value, updating min/max tracking.
    pub fn push(&mut self, val: i64) {
        self.values.push(val);
        self.min_signed = self.min_signed.min(val);
        if val >= 0 {
            self.max_unsigned = self.max_unsigned.max(val as u64);
        }
    }

    /// Push a noData sentinel without affecting min/max.
    pub fn push_no_data(&mut self) {
        self.values.push(INT64_NO_DATA);
    }

    /// Number of values collected.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Finalize and select the optimal integer type.
    pub fn finalize(self) -> FinalizedSignedInt {
        let variant = select_signed_variant(self.min_signed, self.max_unsigned);
        FinalizedSignedInt {
            values: self.values,
            variant,
        }
    }
}

/// Collects unsigned integer values and tracks max for optimal type selection.
#[derive(Debug, Clone)]
pub struct UnsignedIntCollector {
    values: Vec<u64>,
    max_unsigned: u64,
}

impl Default for UnsignedIntCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsignedIntCollector {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            max_unsigned: 0,
        }
    }

    /// Push a value, updating max tracking.
    pub fn push(&mut self, val: u64) {
        self.values.push(val);
        self.max_unsigned = self.max_unsigned.max(val);
    }

    /// Push a noData sentinel without affecting max.
    pub fn push_no_data(&mut self) {
        self.values.push(UINT64_NO_DATA);
    }

    /// Number of values collected.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Finalize and select the optimal integer type.
    pub fn finalize(self) -> FinalizedUnsignedInt {
        let variant = select_unsigned_variant(self.max_unsigned);
        FinalizedUnsignedInt {
            values: self.values,
            variant,
        }
    }
}

/// Finalized signed integer encoder with selected type.
#[derive(Debug)]
pub struct FinalizedSignedInt {
    values: Vec<i64>,
    variant: SignedVariant,
}

impl FinalizedSignedInt {
    pub fn component_type(&self) -> ClassPropertyComponentType {
        match self.variant {
            SignedVariant::Int8 => ClassPropertyComponentType::Int8,
            SignedVariant::Int16 => ClassPropertyComponentType::Int16,
            SignedVariant::Int32 => ClassPropertyComponentType::Int32,
            SignedVariant::Int64 => ClassPropertyComponentType::Int64,
        }
    }

    pub fn no_data_json(&self) -> serde_json::Value {
        let n = match self.variant {
            SignedVariant::Int8 => serde_json::Number::from(INT8_NO_DATA),
            SignedVariant::Int16 => serde_json::Number::from(INT16_NO_DATA),
            SignedVariant::Int32 => serde_json::Number::from(INT32_NO_DATA),
            SignedVariant::Int64 => serde_json::Number::from(INT64_NO_DATA),
        };
        serde_json::Value::Number(n)
    }

    pub fn byte_size(&self) -> usize {
        match self.variant {
            SignedVariant::Int8 => 1,
            SignedVariant::Int16 => 2,
            SignedVariant::Int32 => 4,
            SignedVariant::Int64 => 8,
        }
    }

    /// Encode all collected values to the buffer.
    pub fn encode_all(&self, buf: &mut Vec<u8>) {
        for &val in &self.values {
            match self.variant {
                SignedVariant::Int8 => {
                    let v = if val == INT64_NO_DATA {
                        INT8_NO_DATA
                    } else {
                        val as i8
                    };
                    buf.extend(v.to_le_bytes());
                }
                SignedVariant::Int16 => {
                    let v = if val == INT64_NO_DATA {
                        INT16_NO_DATA
                    } else {
                        val as i16
                    };
                    buf.extend(v.to_le_bytes());
                }
                SignedVariant::Int32 => {
                    let v = if val == INT64_NO_DATA {
                        INT32_NO_DATA
                    } else {
                        val as i32
                    };
                    buf.extend(v.to_le_bytes());
                }
                SignedVariant::Int64 => {
                    buf.extend(val.to_le_bytes());
                }
            }
        }
    }
}

/// Finalized unsigned integer encoder with selected type.
#[derive(Debug)]
pub struct FinalizedUnsignedInt {
    values: Vec<u64>,
    variant: UnsignedVariant,
}

impl FinalizedUnsignedInt {
    pub fn component_type(&self) -> ClassPropertyComponentType {
        match self.variant {
            UnsignedVariant::Uint8 => ClassPropertyComponentType::Uint8,
            UnsignedVariant::Uint16 => ClassPropertyComponentType::Uint16,
            UnsignedVariant::Uint32 => ClassPropertyComponentType::Uint32,
            UnsignedVariant::Uint64 => ClassPropertyComponentType::Uint64,
        }
    }

    pub fn no_data_json(&self) -> serde_json::Value {
        let n = match self.variant {
            UnsignedVariant::Uint8 => serde_json::Number::from(UINT8_NO_DATA),
            UnsignedVariant::Uint16 => serde_json::Number::from(UINT16_NO_DATA),
            UnsignedVariant::Uint32 => serde_json::Number::from(UINT32_NO_DATA),
            UnsignedVariant::Uint64 => serde_json::Number::from(UINT64_NO_DATA),
        };
        serde_json::Value::Number(n)
    }

    pub fn byte_size(&self) -> usize {
        match self.variant {
            UnsignedVariant::Uint8 => 1,
            UnsignedVariant::Uint16 => 2,
            UnsignedVariant::Uint32 => 4,
            UnsignedVariant::Uint64 => 8,
        }
    }

    /// Encode all collected values to the buffer.
    pub fn encode_all(&self, buf: &mut Vec<u8>) {
        for &val in &self.values {
            match self.variant {
                UnsignedVariant::Uint8 => {
                    let v = if val == UINT64_NO_DATA {
                        UINT8_NO_DATA
                    } else {
                        val as u8
                    };
                    buf.extend(v.to_le_bytes());
                }
                UnsignedVariant::Uint16 => {
                    let v = if val == UINT64_NO_DATA {
                        UINT16_NO_DATA
                    } else {
                        val as u16
                    };
                    buf.extend(v.to_le_bytes());
                }
                UnsignedVariant::Uint32 => {
                    let v = if val == UINT64_NO_DATA {
                        UINT32_NO_DATA
                    } else {
                        val as u32
                    };
                    buf.extend(v.to_le_bytes());
                }
                UnsignedVariant::Uint64 => {
                    buf.extend(val.to_le_bytes());
                }
            }
        }
    }
}

// Internal types

#[derive(Debug, Clone, Copy)]
enum SignedVariant {
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Debug, Clone, Copy)]
enum UnsignedVariant {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
}

fn select_signed_variant(min: i64, max: u64) -> SignedVariant {
    if min > INT8_NO_DATA as i64 && max <= i8::MAX as u64 {
        SignedVariant::Int8
    } else if min > INT16_NO_DATA as i64 && max <= i16::MAX as u64 {
        SignedVariant::Int16
    } else if min > INT32_NO_DATA as i64 && max <= i32::MAX as u64 {
        SignedVariant::Int32
    } else {
        SignedVariant::Int64
    }
}

fn select_unsigned_variant(max: u64) -> UnsignedVariant {
    if max < UINT8_NO_DATA as u64 {
        UnsignedVariant::Uint8
    } else if max < UINT16_NO_DATA as u64 {
        UnsignedVariant::Uint16
    } else if max < UINT32_NO_DATA as u64 {
        UnsignedVariant::Uint32
    } else {
        UnsignedVariant::Uint64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_type_selection() {
        // Empty defaults to i8
        assert!(matches!(
            SignedIntCollector::new().finalize().variant,
            SignedVariant::Int8
        ));

        // Fits in i8 range (avoiding MIN as noData)
        let mut c = SignedIntCollector::new();
        c.push(-127);
        c.push(127);
        let f = c.finalize();
        assert!(matches!(f.variant, SignedVariant::Int8));
        assert_eq!(f.byte_size(), 1);
        assert_eq!(f.component_type(), ClassPropertyComponentType::Int8);
        assert_eq!(f.no_data_json(), serde_json::json!(-128));

        // MIN value collision forces upgrade to i16
        let mut c = SignedIntCollector::new();
        c.push(-128);
        assert!(matches!(c.finalize().variant, SignedVariant::Int16));

        // Max overflow forces upgrade
        let mut c = SignedIntCollector::new();
        c.push(128);
        assert!(matches!(c.finalize().variant, SignedVariant::Int16));

        // i16 range
        let mut c = SignedIntCollector::new();
        c.push(-32767);
        c.push(32767);
        let f = c.finalize();
        assert!(matches!(f.variant, SignedVariant::Int16));
        assert_eq!(f.byte_size(), 2);
        assert_eq!(f.no_data_json(), serde_json::json!(-32768));

        // i32 range
        let mut c = SignedIntCollector::new();
        c.push(-2147483647);
        c.push(2147483647);
        let f = c.finalize();
        assert!(matches!(f.variant, SignedVariant::Int32));
        assert_eq!(f.byte_size(), 4);

        // i64 range
        let mut c = SignedIntCollector::new();
        c.push(i64::MIN + 1);
        let f = c.finalize();
        assert!(matches!(f.variant, SignedVariant::Int64));
        assert_eq!(f.byte_size(), 8);

        // noData doesn't affect type selection
        let mut c = SignedIntCollector::new();
        c.push(0);
        c.push_no_data();
        assert!(matches!(c.finalize().variant, SignedVariant::Int8));
    }

    #[test]
    fn test_unsigned_type_selection() {
        // Empty defaults to u8
        assert!(matches!(
            UnsignedIntCollector::new().finalize().variant,
            UnsignedVariant::Uint8
        ));

        // Fits in u8 range (avoiding MAX as noData)
        let mut c = UnsignedIntCollector::new();
        c.push(254);
        let f = c.finalize();
        assert!(matches!(f.variant, UnsignedVariant::Uint8));
        assert_eq!(f.byte_size(), 1);
        assert_eq!(f.component_type(), ClassPropertyComponentType::Uint8);
        assert_eq!(f.no_data_json(), serde_json::json!(255));

        // MAX value collision forces upgrade to u16
        let mut c = UnsignedIntCollector::new();
        c.push(255);
        assert!(matches!(c.finalize().variant, UnsignedVariant::Uint16));

        // u16 range
        let mut c = UnsignedIntCollector::new();
        c.push(65534);
        let f = c.finalize();
        assert!(matches!(f.variant, UnsignedVariant::Uint16));
        assert_eq!(f.byte_size(), 2);
        assert_eq!(f.no_data_json(), serde_json::json!(65535));

        // u32 range
        let mut c = UnsignedIntCollector::new();
        c.push(u32::MAX as u64 - 1);
        let f = c.finalize();
        assert!(matches!(f.variant, UnsignedVariant::Uint32));
        assert_eq!(f.byte_size(), 4);

        // u64 range
        let mut c = UnsignedIntCollector::new();
        c.push(u64::MAX);
        let f = c.finalize();
        assert!(matches!(f.variant, UnsignedVariant::Uint64));
        assert_eq!(f.byte_size(), 8);

        // noData doesn't affect type selection
        let mut c = UnsignedIntCollector::new();
        c.push(0);
        c.push_no_data();
        assert!(matches!(c.finalize().variant, UnsignedVariant::Uint8));
    }

    #[test]
    fn test_signed_encoding() {
        // i8 encoding with noData
        let mut c = SignedIntCollector::new();
        c.push(42);
        c.push(-10);
        c.push_no_data();
        let f = c.finalize();
        let mut buf = Vec::new();
        f.encode_all(&mut buf);
        assert_eq!(buf, vec![42u8, 246u8, INT8_NO_DATA as u8]);

        // i16 encoding with noData
        let mut c = SignedIntCollector::new();
        c.push(1000);
        c.push(-500);
        c.push_no_data();
        let f = c.finalize();
        let mut buf = Vec::new();
        f.encode_all(&mut buf);
        assert_eq!(&buf[0..2], &1000i16.to_le_bytes());
        assert_eq!(&buf[2..4], &(-500i16).to_le_bytes());
        assert_eq!(&buf[4..6], &INT16_NO_DATA.to_le_bytes());

        // i32 encoding
        let mut c = SignedIntCollector::new();
        c.push(100000);
        c.push_no_data();
        let f = c.finalize();
        let mut buf = Vec::new();
        f.encode_all(&mut buf);
        assert_eq!(&buf[0..4], &100000i32.to_le_bytes());
        assert_eq!(&buf[4..8], &INT32_NO_DATA.to_le_bytes());

        // i64 encoding
        let mut c = SignedIntCollector::new();
        c.push(i64::MIN + 1);
        c.push_no_data();
        let f = c.finalize();
        let mut buf = Vec::new();
        f.encode_all(&mut buf);
        assert_eq!(&buf[0..8], &(i64::MIN + 1).to_le_bytes());
        assert_eq!(&buf[8..16], &INT64_NO_DATA.to_le_bytes());
    }

    #[test]
    fn test_unsigned_encoding() {
        // u8 encoding with noData
        let mut c = UnsignedIntCollector::new();
        c.push(42);
        c.push(200);
        c.push_no_data();
        let f = c.finalize();
        let mut buf = Vec::new();
        f.encode_all(&mut buf);
        assert_eq!(buf, vec![42u8, 200u8, UINT8_NO_DATA]);

        // u16 encoding with noData
        let mut c = UnsignedIntCollector::new();
        c.push(1000);
        c.push(50000);
        c.push_no_data();
        let f = c.finalize();
        let mut buf = Vec::new();
        f.encode_all(&mut buf);
        assert_eq!(&buf[0..2], &1000u16.to_le_bytes());
        assert_eq!(&buf[2..4], &50000u16.to_le_bytes());
        assert_eq!(&buf[4..6], &UINT16_NO_DATA.to_le_bytes());

        // u32 encoding
        let mut c = UnsignedIntCollector::new();
        c.push(100000);
        c.push_no_data();
        let f = c.finalize();
        let mut buf = Vec::new();
        f.encode_all(&mut buf);
        assert_eq!(&buf[0..4], &100000u32.to_le_bytes());
        assert_eq!(&buf[4..8], &UINT32_NO_DATA.to_le_bytes());

        // u64 encoding
        let mut c = UnsignedIntCollector::new();
        c.push(u64::MAX - 1);
        c.push_no_data();
        let f = c.finalize();
        let mut buf = Vec::new();
        f.encode_all(&mut buf);
        assert_eq!(&buf[0..8], &(u64::MAX - 1).to_le_bytes());
        assert_eq!(&buf[8..16], &UINT64_NO_DATA.to_le_bytes());
    }
}
