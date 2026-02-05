//! Encode feature attributes into EXT_structural_metadata format

use std::collections::HashMap;

use indexmap::{IndexMap, IndexSet};
use nusamai_citygml::schema::{Attribute, FeatureTypeDef, Schema, TypeDef};
use nusamai_gltf::nusamai_gltf_json::{
    extensions::gltf::ext_structural_metadata::{
        self, ClassPropertyComponentType, ClassPropertyType, Enum, EnumValue, EnumValueType,
        ExtStructuralMetadata, PropertyTable, PropertyTableProperty,
    },
    BufferView,
};
use reearth_flow_types::AttributeValue;
use tracing::warn;

use super::{
    ENUM_NO_DATA, ENUM_NO_DATA_NAME, FLOAT_NO_DATA, INT16_NO_DATA, INT32_NO_DATA, INT64_NO_DATA,
    INT8_NO_DATA, UINT16_NO_DATA, UINT32_NO_DATA, UINT64_NO_DATA, UINT8_NO_DATA,
};

pub struct MetadataEncoder<'a> {
    /// The original city model schema
    original_schema: &'a Schema,
    /// typename -> Class
    classes: IndexMap<String, Class>,
    // Represents Code values as enum names?
    enum_set: IndexSet<String>,
}

impl<'a> MetadataEncoder<'a> {
    pub fn new(original_schema: &'a Schema) -> Self {
        // Use the first enum value as noData
        let mut enum_set: IndexSet<String> = Default::default();
        enum_set.insert(ENUM_NO_DATA_NAME.to_string());

        Self {
            original_schema,
            classes: Default::default(),
            enum_set,
        }
    }

    // Add a feature and return the assigned feature ID.
    pub fn add_feature(
        &mut self,
        typename: &str,
        attributes: &HashMap<String, AttributeValue>,
    ) -> crate::errors::Result<usize> {
        let Some(TypeDef::Feature(feature_def)) = self.original_schema.types.get(typename) else {
            return Err(crate::errors::Error::metadata(format!(
                "Feature type not found: {typename}"
            )));
        };

        let typename = typename.replace(':', "_");

        let class = self
            .classes
            .entry(typename)
            .or_insert_with(|| Class::from(feature_def));

        class.add_feature(attributes, &mut self.enum_set)
    }

    pub fn into_metadata(
        self,
        buffer: &mut Vec<u8>,
        buffer_views: &mut Vec<BufferView>,
    ) -> Option<ExtStructuralMetadata> {
        let (schema, property_tables) = {
            let enums = {
                let mut enums: HashMap<String, Enum> = HashMap::new();
                let mut values = vec![];

                for (idx, name) in self.enum_set.into_iter().enumerate() {
                    let Some(value) = i32::try_from(idx).ok() else {
                        warn!(
                            "Skipping enum value '{}': index {} exceeds i32::MAX",
                            name, idx
                        );
                        continue;
                    };
                    values.push(EnumValue {
                        value,
                        name,
                        ..Default::default()
                    });
                }

                enums.insert(
                    "Enum01".to_string(),
                    Enum {
                        value_type: EnumValueType::Uint32,
                        values,
                        ..Default::default()
                    },
                );
                enums
            };

            let (classes, property_tables) = {
                let mut classes = HashMap::new();
                let mut property_tables = Vec::new();
                for (typename, cls) in self.classes {
                    let (class, property_table) =
                        cls.make_metadata(&typename, buffer, buffer_views);
                    classes.insert(typename, class);
                    property_tables.push(property_table);
                }
                (classes, property_tables)
            };

            let schema = ext_structural_metadata::Schema {
                id: "Schema".to_string(),
                classes,
                enums,
                ..Default::default()
            };

            (schema, property_tables)
        };

        Some(ExtStructuralMetadata {
            schema: Some(schema),
            property_tables: Some(property_tables),
            ..Default::default()
        })
    }
}

#[derive(Default, Debug)]
struct Class {
    /// Counter for assigning feature IDs.
    feature_count: usize,
    /// properties
    properties: IndexMap<String, Property>,
}

impl From<&FeatureTypeDef> for Class {
    fn from(feature_def: &FeatureTypeDef) -> Self {
        let mut properties = IndexMap::new();
        // attributes
        for (name, attr) in &feature_def.attributes {
            properties.insert(name.to_string(), Property::from(attr));
        }
        Self {
            feature_count: 0,
            properties,
        }
    }
}

impl Class {
    fn add_feature(
        &mut self,
        attributes: &HashMap<String, AttributeValue>,
        enum_set: &mut IndexSet<String>,
    ) -> crate::errors::Result<usize> {
        // Encode attributes
        for key in &self.properties.keys().cloned().collect::<Vec<_>>() {
            let Some(prop) = self.properties.get_mut(&key.to_string()) else {
                continue;
            };
            let Some(value) = attributes.get(key) else {
                continue;
            };
            if prop.is_array {
                encode_array_value(value, prop, enum_set);
            } else {
                encode_value(value, prop, enum_set);
            }
            prop.used = true;
        }
        // Fill in the default values for the properties that don't occur in the input
        for (key, prop) in &mut self.properties {
            if attributes.contains_key(key) {
                continue;
            }

            if prop.is_array {
                match prop.type_ {
                    PropertyType::String => {
                        let len = prop.string_offsets.len();
                        if len == 0 {
                            warn!("Skipping default array offset for property '{}': string_offsets is unexpectedly empty", key);
                            continue;
                        }
                        let Some(offset) = u32::try_from(len - 1).ok() else {
                            warn!(
                                "Skipping default array offset for property '{}': string_offsets length {} exceeds u32::MAX",
                                key, len
                            );
                            continue;
                        };
                        prop.array_offsets.push(offset);
                    }
                    // PropertyType::Boolean => todo!(), // TODO
                    _ => {
                        prop.array_offsets.push(prop.count);
                    }
                }
            } else {
                match prop.type_ {
                    PropertyType::SignedInt => {
                        // Push sentinel value for noData (don't update min_signed)
                        prop.int_values.push(INT64_NO_DATA);
                    }
                    PropertyType::UnsignedInt => {
                        // Push sentinel value for noData (don't update max_unsigned)
                        prop.uint_values.push(UINT64_NO_DATA);
                    }
                    PropertyType::Float64 => prop.value_buffer.extend(FLOAT_NO_DATA.to_le_bytes()),
                    PropertyType::String => {
                        let Some(offset) = u32::try_from(prop.value_buffer.len()).ok() else {
                            warn!(
                                "Skipping default string offset for property '{}': value_buffer length {} exceeds u32::MAX",
                                key, prop.value_buffer.len()
                            );
                            continue;
                        };
                        prop.string_offsets.push(offset);
                    }
                    PropertyType::Enum => prop.value_buffer.extend(ENUM_NO_DATA.to_le_bytes()),
                    // PropertyType::Boolean => todo!(),
                };
            }
        }

        // Return the assigned feature ID
        let feature_id = self.feature_count;
        self.feature_count += 1;
        Ok(feature_id)
    }

    fn make_metadata(
        self,
        class_name: &str,
        buffer: &mut Vec<u8>,
        buffer_views: &mut Vec<BufferView>,
    ) -> (
        ext_structural_metadata::Class,
        ext_structural_metadata::PropertyTable,
    ) {
        let mut class_properties = IndexMap::new();
        let mut pt_properties: IndexMap<String, PropertyTableProperty> = Default::default();

        for (name, prop) in self.properties {
            // Skip unused properties
            if !prop.used {
                continue;
            }

            // Select optimal integer types based on actual value ranges
            let selected_signed = select_signed_type(prop.min_signed, prop.max_unsigned);
            let selected_unsigned = select_unsigned_type(prop.max_unsigned);

            class_properties.insert(
                name.to_string(),
                ext_structural_metadata::ClassProperty {
                    type_: match prop.type_ {
                        PropertyType::SignedInt => ClassPropertyType::Scalar,
                        PropertyType::UnsignedInt => ClassPropertyType::Scalar,
                        PropertyType::Float64 => ClassPropertyType::Scalar,
                        PropertyType::String => ClassPropertyType::String,
                        // PropertyType::Boolean => ClassPropertyType::Boolean,
                        PropertyType::Enum => ClassPropertyType::Enum,
                    },
                    component_type: match prop.type_ {
                        PropertyType::SignedInt => Some(selected_signed.component_type()),
                        PropertyType::UnsignedInt => Some(selected_unsigned.component_type()),
                        PropertyType::Float64 => Some(ClassPropertyComponentType::Float64),
                        PropertyType::String => None,
                        PropertyType::Enum => None,
                        //PropertyType::Boolean => None,
                    },
                    enum_type: match prop.type_ {
                        PropertyType::Enum => Some("Enum01".to_string()),
                        _ => None,
                    },
                    array: prop.is_array,
                    no_data: match (prop.type_, prop.is_array) {
                        (_, true) => Some(serde_json::Value::Array(vec![])),
                        (PropertyType::Enum, false) => {
                            Some(serde_json::Value::String(ENUM_NO_DATA_NAME.to_string()))
                        }
                        (PropertyType::String, false) => {
                            Some(serde_json::Value::String("".to_string()))
                        }
                        (PropertyType::Float64, false) => Some(serde_json::Value::Number(
                            serde_json::Number::from_f64(FLOAT_NO_DATA).unwrap(),
                        )),
                        (PropertyType::SignedInt, false) => Some(selected_signed.no_data_json()),
                        (PropertyType::UnsignedInt, false) => {
                            Some(selected_unsigned.no_data_json())
                        }
                    },
                    ..Default::default()
                },
            );

            // values
            // Align based on property type and selected integer size
            let alignment = match prop.type_ {
                PropertyType::SignedInt => selected_signed.byte_size(),
                PropertyType::UnsignedInt => selected_unsigned.byte_size(),
                PropertyType::Float64 => 8,
                PropertyType::Enum => 4,   // Enum uses u32
                PropertyType::String => 1, // String values are raw bytes
            };
            add_padding(buffer, alignment);

            let start = buffer.len();

            // Encode values based on property type
            match prop.type_ {
                PropertyType::SignedInt => {
                    for val in &prop.int_values {
                        selected_signed.encode_value(*val, buffer);
                    }
                }
                PropertyType::UnsignedInt => {
                    for val in &prop.uint_values {
                        selected_unsigned.encode_value(*val, buffer);
                    }
                }
                _ => {
                    buffer.extend(prop.value_buffer);
                }
            }

            // Check for overflow when creating buffer view
            let Some(byte_offset) = u32::try_from(start).ok() else {
                warn!(
                    "Skipping property '{}': buffer offset {} exceeds u32::MAX",
                    name, start
                );
                class_properties.swap_remove(&name);
                continue;
            };
            let Some(byte_length) = u32::try_from(buffer.len() - start).ok() else {
                warn!(
                    "Skipping property '{}': buffer length {} exceeds u32::MAX",
                    name,
                    buffer.len() - start
                );
                class_properties.swap_remove(&name);
                continue;
            };
            buffer_views.push(BufferView {
                name: Some("prop_values".to_string()),
                byte_offset,
                byte_length,
                ..Default::default()
            });

            // Check for overflow when getting buffer view index
            if buffer_views.is_empty() {
                warn!(
                    "Skipping property '{}': buffer_views is unexpectedly empty",
                    name
                );
                class_properties.swap_remove(&name);
                continue;
            }
            let Some(values_view_idx) = u32::try_from(buffer_views.len() - 1).ok() else {
                warn!(
                    "Skipping property '{}': buffer_views index {} exceeds u32::MAX",
                    name,
                    buffer_views.len() - 1
                );
                class_properties.swap_remove(&name);
                continue;
            };

            // arrayOffsets (u32 values require 4-byte alignment)
            let array_offsets_idx = if prop.is_array {
                add_padding(buffer, 4);
                let start = buffer.len();
                for offset in prop.array_offsets {
                    buffer.extend(offset.to_le_bytes());
                }

                let Some(byte_offset) = u32::try_from(start).ok() else {
                    warn!(
                        "Skipping property '{}' array offsets: buffer offset {} exceeds u32::MAX",
                        name, start
                    );
                    class_properties.swap_remove(&name);
                    continue;
                };
                let Some(byte_length) = u32::try_from(buffer.len() - start).ok() else {
                    warn!(
                        "Skipping property '{}' array offsets: buffer length {} exceeds u32::MAX",
                        name,
                        buffer.len() - start
                    );
                    class_properties.swap_remove(&name);
                    continue;
                };
                buffer_views.push(BufferView {
                    name: Some("prop_array_offsets".to_string()),
                    byte_offset,
                    byte_length,
                    ..Default::default()
                });

                if buffer_views.is_empty() {
                    warn!(
                        "Skipping property '{}': buffer_views is unexpectedly empty after array offsets",
                        name
                    );
                    class_properties.swap_remove(&name);
                    continue;
                }
                let Some(idx) = u32::try_from(buffer_views.len() - 1).ok() else {
                    warn!(
                        "Skipping property '{}': array offsets buffer_views index {} exceeds u32::MAX",
                        name,
                        buffer_views.len() - 1
                    );
                    class_properties.swap_remove(&name);
                    continue;
                };
                Some(idx)
            } else {
                None
            };

            // stringOffsets (u32 values require 4-byte alignment)
            let string_offsets_idx = if prop.type_ == PropertyType::String {
                add_padding(buffer, 4);
                let start = buffer.len();
                for offset in prop.string_offsets {
                    buffer.extend(offset.to_le_bytes());
                }

                let Some(byte_offset) = u32::try_from(start).ok() else {
                    warn!(
                        "Skipping property '{}' string offsets: buffer offset {} exceeds u32::MAX",
                        name, start
                    );
                    class_properties.swap_remove(&name);
                    continue;
                };
                let Some(byte_length) = u32::try_from(buffer.len() - start).ok() else {
                    warn!(
                        "Skipping property '{}' string offsets: buffer length {} exceeds u32::MAX",
                        name,
                        buffer.len() - start
                    );
                    class_properties.swap_remove(&name);
                    continue;
                };
                buffer_views.push(BufferView {
                    name: Some("prop_string_offsets".to_string()),
                    byte_offset,
                    byte_length,
                    ..Default::default()
                });

                if buffer_views.is_empty() {
                    warn!(
                        "Skipping property '{}': buffer_views is unexpectedly empty after string offsets",
                        name
                    );
                    class_properties.swap_remove(&name);
                    continue;
                }
                let Some(idx) = u32::try_from(buffer_views.len() - 1).ok() else {
                    warn!(
                        "Skipping property '{}': string offsets buffer_views index {} exceeds u32::MAX",
                        name,
                        buffer_views.len() - 1
                    );
                    class_properties.swap_remove(&name);
                    continue;
                };
                Some(idx)
            } else {
                None
            };

            pt_properties.insert(
                name,
                PropertyTableProperty {
                    values: values_view_idx,
                    array_offsets: array_offsets_idx,
                    string_offsets: string_offsets_idx,
                    ..Default::default()
                },
            );
        }

        let feature_count = match u32::try_from(self.feature_count) {
            Ok(count) => count,
            Err(_) => {
                warn!(
                    "Feature count {} exceeds u32::MAX, capping at u32::MAX",
                    self.feature_count
                );
                u32::MAX
            }
        };

        let property_table = PropertyTable {
            class: class_name.to_string(),
            count: feature_count,
            properties: pt_properties,
            ..Default::default()
        };

        let class = ext_structural_metadata::Class {
            properties: class_properties,
            ..Default::default()
        };

        (class, property_table)
    }
}

fn encode_value(value: &AttributeValue, prop: &mut Property, enum_set: &mut IndexSet<String>) {
    // Encode based on the target property type, converting values as needed
    match prop.type_ {
        PropertyType::String => {
            let s = match value {
                AttributeValue::String(s) => s.clone(),
                AttributeValue::DateTime(d) => d.to_string(),
                AttributeValue::Number(n) => n.to_string(),
                AttributeValue::Bool(b) => b.to_string(),
                _ => return,
            };
            prop.value_buffer.extend_from_slice(s.as_bytes());
            let Some(offset) = u32::try_from(prop.value_buffer.len()).ok() else {
                warn!(
                    "Skipping string encoding: value_buffer length {} exceeds u32::MAX",
                    prop.value_buffer.len()
                );
                return;
            };
            prop.string_offsets.push(offset);
            let Some(new_count) = prop.count.checked_add(1) else {
                warn!("Skipping string encoding: property count would overflow u32");
                return;
            };
            prop.count = new_count;
        }
        PropertyType::SignedInt => {
            // Store value temporarily, track min/max for type selection later
            let val: i64 = match value {
                AttributeValue::Number(n) => n.as_i64().unwrap_or(0),
                AttributeValue::String(s) => s.parse().unwrap_or(0),
                AttributeValue::Bool(b) => *b as i64,
                _ => 0,
            };
            prop.int_values.push(val);
            prop.min_signed = prop.min_signed.min(val);
            // Track positive values using max_unsigned for upper bound check
            if val >= 0 {
                prop.max_unsigned = prop.max_unsigned.max(val as u64);
            }
            let Some(new_count) = prop.count.checked_add(1) else {
                warn!("Skipping SignedInt encoding: property count would overflow u32");
                return;
            };
            prop.count = new_count;
        }
        PropertyType::UnsignedInt => {
            // Store value temporarily, track max for type selection later
            let val: u64 = match value {
                AttributeValue::Number(n) => n.as_u64().unwrap_or(0),
                AttributeValue::String(s) => s.parse().unwrap_or(0),
                AttributeValue::Bool(b) => *b as u64,
                _ => 0,
            };
            prop.uint_values.push(val);
            prop.max_unsigned = prop.max_unsigned.max(val);
            let Some(new_count) = prop.count.checked_add(1) else {
                warn!("Skipping UnsignedInt encoding: property count would overflow u32");
                return;
            };
            prop.count = new_count;
        }
        PropertyType::Float64 => {
            let val: f64 = match value {
                AttributeValue::Number(n) => n.as_f64().unwrap_or(FLOAT_NO_DATA),
                AttributeValue::String(s) => s.parse().unwrap_or(FLOAT_NO_DATA),
                AttributeValue::Bool(b) => {
                    if *b {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => FLOAT_NO_DATA,
            };
            prop.value_buffer.extend(val.to_le_bytes());
            let Some(new_count) = prop.count.checked_add(1) else {
                warn!("Skipping Float64 encoding: property count would overflow u32");
                return;
            };
            prop.count = new_count;
        }
        PropertyType::Enum => {
            // Enum values are stored as u32, looked up by name in enum_set
            let val: u32 = match value {
                AttributeValue::String(s) => {
                    // Insert the enum value if not present, get its index
                    let (index, _) = enum_set.insert_full(s.clone());
                    let Some(idx) = u32::try_from(index).ok() else {
                        warn!(
                            "Skipping enum encoding: enum index {} exceeds u32::MAX",
                            index
                        );
                        return;
                    };
                    idx
                }
                AttributeValue::Number(n) => {
                    let num = n.as_u64().unwrap_or(ENUM_NO_DATA as u64);
                    let Some(idx) = u32::try_from(num).ok() else {
                        warn!(
                            "Skipping enum encoding: numeric value {} exceeds u32::MAX",
                            num
                        );
                        return;
                    };
                    idx
                }
                _ => ENUM_NO_DATA,
            };
            prop.value_buffer.extend(val.to_le_bytes());
            let Some(new_count) = prop.count.checked_add(1) else {
                warn!("Skipping enum encoding: property count would overflow u32");
                return;
            };
            prop.count = new_count;
        }
    }
}

fn encode_array_value(
    value: &AttributeValue,
    prop: &mut Property,
    enum_set: &mut IndexSet<String>,
) {
    match value {
        AttributeValue::Array(arr) => {
            for v in arr {
                encode_value(v, prop, enum_set);
            }

            if !push_array_offset(prop) {
                warn!(
                    "Skipping array offset: string_offsets length {} exceeds u32::MAX",
                    prop.string_offsets.len()
                );
            }
        }
        AttributeValue::Map(map) => {
            for v in map.values() {
                encode_value(v, prop, enum_set);
            }

            if !push_array_offset(prop) {
                warn!(
                    "Skipping array offset: string_offsets length {} exceeds u32::MAX",
                    prop.string_offsets.len()
                );
            }
        }
        _ => {
            // Single value in array context - this may indicate a schema mismatch
            warn!(
                "Single value provided for array property (type: {:?}), wrapping as single-element array",
                prop.type_
            );
            encode_value(value, prop, enum_set);
            if !push_array_offset(prop) {
                warn!(
                    "Skipping array offset: string_offsets length {} exceeds u32::MAX",
                    prop.string_offsets.len()
                );
            }
        }
    }
}

/// Helper function to push array offset, returns false if overflow would occur
fn push_array_offset(prop: &mut Property) -> bool {
    match prop.type_ {
        PropertyType::String => {
            // string_offsets.len() - 1: safe because string_offsets is initialized with [0]
            let len = prop.string_offsets.len();
            if len == 0 {
                warn!("Skipping array offset: string_offsets is unexpectedly empty");
                return false;
            }
            let Some(offset) = u32::try_from(len - 1).ok() else {
                return false;
            };
            prop.array_offsets.push(offset);
        }
        _ => {
            prop.array_offsets.push(prop.count);
        }
    }
    true
}

#[derive(Debug)]
struct Property {
    type_: PropertyType,
    value_buffer: Vec<u8>,
    count: u32,
    is_array: bool,
    /// Whether the property is used at least once.
    used: bool,
    array_offsets: Vec<u32>,
    string_offsets: Vec<u32>,
    /// Temporary storage for signed integer values (for dynamic type selection)
    int_values: Vec<i64>,
    /// Temporary storage for unsigned integer values (for dynamic type selection)
    uint_values: Vec<u64>,
    /// Track minimum value for signed integers
    min_signed: i64,
    /// Track maximum value for unsigned integers
    max_unsigned: u64,
}

impl Property {
    pub fn new(type_: PropertyType, is_array: bool) -> Self {
        let string_offsets = match type_ {
            PropertyType::String => vec![0],
            _ => vec![],
        };
        let array_offsets = match is_array {
            true => vec![0],
            false => vec![],
        };
        Property {
            type_,
            count: 0,
            value_buffer: Default::default(),
            is_array,
            used: false,
            string_offsets,
            array_offsets,
            int_values: Vec::new(),
            uint_values: Vec::new(),
            min_signed: i64::MAX,
            max_unsigned: 0,
        }
    }
}

impl From<&Attribute> for Property {
    fn from(attr: &Attribute) -> Self {
        use nusamai_citygml::schema::TypeRef;
        let type_ = match attr.type_ref {
            TypeRef::String => PropertyType::String,
            TypeRef::Code => PropertyType::Enum,
            TypeRef::Integer => PropertyType::SignedInt,
            TypeRef::NonNegativeInteger => PropertyType::UnsignedInt,
            TypeRef::Double => PropertyType::Float64,
            TypeRef::Boolean => PropertyType::SignedInt, // TODO: Boolean bitstream
            TypeRef::JsonString(_) => PropertyType::String,
            TypeRef::URI => PropertyType::String,
            TypeRef::Date => PropertyType::String,
            TypeRef::DateTime => PropertyType::String,
            TypeRef::Measure => PropertyType::Float64,
            TypeRef::Point => PropertyType::String, // TODO: VEC3<f64>
            TypeRef::Named(_) => PropertyType::String,
            TypeRef::Unknown => PropertyType::String,
        };
        let is_array = attr.max_occurs != Some(1);
        Property::new(type_, is_array)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyType {
    SignedInt,
    UnsignedInt,
    Float64,
    String,
    // Boolean,
    Enum,
}

/// Selected integer type with its noData value and byte size
#[derive(Debug, Clone, Copy)]
enum SelectedSignedType {
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Debug, Clone, Copy)]
enum SelectedUnsignedType {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
}

/// Select the smallest signed integer type that can hold all values.
/// Uses MIN values as noData, so we need min > type::MIN to use that type.
/// Also checks that max fits within the positive range of the type.
fn select_signed_type(min: i64, max: u64) -> SelectedSignedType {
    // i8: min > -128 and max <= 127
    if min > INT8_NO_DATA as i64 && max <= i8::MAX as u64 {
        SelectedSignedType::Int8
    // i16: min > -32768 and max <= 32767
    } else if min > INT16_NO_DATA as i64 && max <= i16::MAX as u64 {
        SelectedSignedType::Int16
    // i32: min > -2147483648 and max <= 2147483647
    } else if min > INT32_NO_DATA as i64 && max <= i32::MAX as u64 {
        SelectedSignedType::Int32
    } else {
        SelectedSignedType::Int64
    }
}

/// Select the smallest unsigned integer type that can hold all values.
/// Uses MAX values as noData, so we need max < type::MAX to use that type.
fn select_unsigned_type(max: u64) -> SelectedUnsignedType {
    if max < UINT8_NO_DATA as u64 {
        SelectedUnsignedType::Uint8
    } else if max < UINT16_NO_DATA as u64 {
        SelectedUnsignedType::Uint16
    } else if max < UINT32_NO_DATA as u64 {
        SelectedUnsignedType::Uint32
    } else {
        SelectedUnsignedType::Uint64
    }
}

impl SelectedSignedType {
    fn component_type(self) -> ClassPropertyComponentType {
        match self {
            SelectedSignedType::Int8 => ClassPropertyComponentType::Int8,
            SelectedSignedType::Int16 => ClassPropertyComponentType::Int16,
            SelectedSignedType::Int32 => ClassPropertyComponentType::Int32,
            SelectedSignedType::Int64 => ClassPropertyComponentType::Int64,
        }
    }

    fn no_data_json(self) -> serde_json::Value {
        match self {
            SelectedSignedType::Int8 => {
                serde_json::Value::Number(serde_json::Number::from(INT8_NO_DATA))
            }
            SelectedSignedType::Int16 => {
                serde_json::Value::Number(serde_json::Number::from(INT16_NO_DATA))
            }
            SelectedSignedType::Int32 => {
                serde_json::Value::Number(serde_json::Number::from(INT32_NO_DATA))
            }
            SelectedSignedType::Int64 => {
                serde_json::Value::Number(serde_json::Number::from(INT64_NO_DATA))
            }
        }
    }

    fn byte_size(self) -> usize {
        match self {
            SelectedSignedType::Int8 => 1,
            SelectedSignedType::Int16 => 2,
            SelectedSignedType::Int32 => 4,
            SelectedSignedType::Int64 => 8,
        }
    }

    /// Encode a value to bytes, converting INT64_NO_DATA sentinel to the appropriate noData
    fn encode_value(self, val: i64, buf: &mut Vec<u8>) {
        match self {
            SelectedSignedType::Int8 => {
                let v = if val == INT64_NO_DATA {
                    INT8_NO_DATA
                } else {
                    val as i8
                };
                buf.extend(v.to_le_bytes());
            }
            SelectedSignedType::Int16 => {
                let v = if val == INT64_NO_DATA {
                    INT16_NO_DATA
                } else {
                    val as i16
                };
                buf.extend(v.to_le_bytes());
            }
            SelectedSignedType::Int32 => {
                let v = if val == INT64_NO_DATA {
                    INT32_NO_DATA
                } else {
                    val as i32
                };
                buf.extend(v.to_le_bytes());
            }
            SelectedSignedType::Int64 => {
                buf.extend(val.to_le_bytes());
            }
        }
    }
}

impl SelectedUnsignedType {
    fn component_type(self) -> ClassPropertyComponentType {
        match self {
            SelectedUnsignedType::Uint8 => ClassPropertyComponentType::Uint8,
            SelectedUnsignedType::Uint16 => ClassPropertyComponentType::Uint16,
            SelectedUnsignedType::Uint32 => ClassPropertyComponentType::Uint32,
            SelectedUnsignedType::Uint64 => ClassPropertyComponentType::Uint64,
        }
    }

    fn no_data_json(self) -> serde_json::Value {
        match self {
            SelectedUnsignedType::Uint8 => {
                serde_json::Value::Number(serde_json::Number::from(UINT8_NO_DATA))
            }
            SelectedUnsignedType::Uint16 => {
                serde_json::Value::Number(serde_json::Number::from(UINT16_NO_DATA))
            }
            SelectedUnsignedType::Uint32 => {
                serde_json::Value::Number(serde_json::Number::from(UINT32_NO_DATA))
            }
            SelectedUnsignedType::Uint64 => {
                serde_json::Value::Number(serde_json::Number::from(UINT64_NO_DATA))
            }
        }
    }

    fn byte_size(self) -> usize {
        match self {
            SelectedUnsignedType::Uint8 => 1,
            SelectedUnsignedType::Uint16 => 2,
            SelectedUnsignedType::Uint32 => 4,
            SelectedUnsignedType::Uint64 => 8,
        }
    }

    /// Encode a value to bytes, converting UINT64_NO_DATA sentinel to the appropriate noData
    fn encode_value(self, val: u64, buf: &mut Vec<u8>) {
        match self {
            SelectedUnsignedType::Uint8 => {
                let v = if val == UINT64_NO_DATA {
                    UINT8_NO_DATA
                } else {
                    val as u8
                };
                buf.extend(v.to_le_bytes());
            }
            SelectedUnsignedType::Uint16 => {
                let v = if val == UINT64_NO_DATA {
                    UINT16_NO_DATA
                } else {
                    val as u16
                };
                buf.extend(v.to_le_bytes());
            }
            SelectedUnsignedType::Uint32 => {
                let v = if val == UINT64_NO_DATA {
                    UINT32_NO_DATA
                } else {
                    val as u32
                };
                buf.extend(v.to_le_bytes());
            }
            SelectedUnsignedType::Uint64 => {
                buf.extend(val.to_le_bytes());
            }
        }
    }
}

pub fn add_padding(buf: &mut Vec<u8>, align: usize) {
    let len = buf.len();
    let pad = (align - (len % align)) % align;
    buf.resize(len + pad, 0);
}
