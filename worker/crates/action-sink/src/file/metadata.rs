use indexmap::{IndexMap, IndexSet};
use nusamai_citygml::schema::{Attribute, FeatureTypeDef};
use nusamai_gltf::nusamai_gltf_json::{
    extensions::gltf::ext_structural_metadata::{
        self, ClassPropertyComponentType, ClassPropertyType, Enum, EnumValue, EnumValueType,
        ExtStructuralMetadata, PropertyTable, PropertyTableProperty,
    },
    BufferView,
};
use std::collections::HashMap;

const ENUM_NO_DATA_NAME: &str = "";
const FLOAT_NO_DATA: f64 = f64::MAX;
const INT64_NO_DATA: i64 = i64::MIN;
const UINT64_NO_DATA: u64 = u64::MAX;

pub struct MetadataEncoder {
    classes: IndexMap<String, Class>,
    enum_set: IndexSet<String>,
}

impl MetadataEncoder {
    pub fn new() -> Self {
        let mut enum_set: IndexSet<String> = Default::default();
        enum_set.insert(ENUM_NO_DATA_NAME.to_string());

        Self {
            classes: Default::default(),
            enum_set,
        }
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
                    values.push(EnumValue {
                        value: idx as i32,
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
        // id
        properties.insert("id".to_string(), Property::new(PropertyType::String, false));
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
    fn make_metadata(
        self,
        class_name: &str,
        buffer: &mut Vec<u8>,
        buffer_views: &mut Vec<BufferView>,
    ) -> (
        ext_structural_metadata::Class,
        ext_structural_metadata::PropertyTable,
    ) {
        let mut class_properties = HashMap::new();
        let mut pt_properties: HashMap<String, PropertyTableProperty> = Default::default();

        for (name, prop) in self.properties {
            // Skip unused properties
            if !prop.used {
                continue;
            }

            class_properties.insert(
                name.to_string(),
                ext_structural_metadata::ClassProperty {
                    type_: match prop.type_ {
                        PropertyType::Int64 => ClassPropertyType::Scalar,
                        PropertyType::Uint64 => ClassPropertyType::Scalar,
                        PropertyType::Float64 => ClassPropertyType::Scalar,
                        PropertyType::String => ClassPropertyType::String,
                        // PropertyType::Boolean => ClassPropertyType::Boolean,
                        PropertyType::Enum => ClassPropertyType::Enum,
                    },
                    component_type: match prop.type_ {
                        PropertyType::Int64 => Some(ClassPropertyComponentType::Int64),
                        PropertyType::Uint64 => Some(ClassPropertyComponentType::Uint64),
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
                        (PropertyType::Int64, false) => Some(serde_json::Value::Number(
                            serde_json::Number::from(INT64_NO_DATA),
                        )),
                        (PropertyType::Uint64, false) => Some(serde_json::Value::Number(
                            serde_json::Number::from(UINT64_NO_DATA),
                        )),
                    },
                    ..Default::default()
                },
            );

            // values
            let start = buffer.len();
            buffer.extend(prop.value_buffer);
            buffer_views.push(BufferView {
                name: Some("prop_values".to_string()),
                byte_offset: start as u32,
                byte_length: (buffer.len() - start) as u32,
                ..Default::default()
            });
            let values_view_idx = buffer_views.len() as u32 - 1;
            add_padding(buffer, 4);

            // arrayOffsets
            let array_offsets_idx = if prop.is_array {
                let start = buffer.len();
                for offset in prop.array_offsets {
                    buffer.extend(offset.to_le_bytes());
                }
                buffer_views.push(BufferView {
                    name: Some("prop_array_offsets".to_string()),
                    byte_offset: start as u32,
                    byte_length: (buffer.len() - start) as u32,
                    ..Default::default()
                });
                Some(buffer_views.len() as u32 - 1)
            } else {
                None
            };

            // stringOffsets
            let string_offsets_idx = if prop.type_ == PropertyType::String {
                let start = buffer.len();
                for offset in prop.string_offsets {
                    buffer.extend(offset.to_le_bytes());
                }
                buffer_views.push(BufferView {
                    name: Some("prop_string_offsets".to_string()),
                    byte_offset: start as u32,
                    byte_length: (buffer.len() - start) as u32,
                    ..Default::default()
                });
                Some(buffer_views.len() as u32 - 1)
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

        let property_table = PropertyTable {
            class: class_name.to_string(),
            count: self.feature_count as u32,
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

#[derive(Debug)]
struct Property {
    type_: PropertyType,
    value_buffer: Vec<u8>,
    is_array: bool,
    /// Whether the property is used at least once.
    used: bool,
    array_offsets: Vec<u32>,
    string_offsets: Vec<u32>,
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
            value_buffer: Default::default(),
            is_array,
            used: false,
            string_offsets,
            array_offsets,
        }
    }
}

impl From<&Attribute> for Property {
    fn from(attr: &Attribute) -> Self {
        use nusamai_citygml::schema::TypeRef;
        let type_ = match attr.type_ref {
            TypeRef::String => PropertyType::String,
            TypeRef::Code => PropertyType::Enum,
            TypeRef::Integer => PropertyType::Int64,
            TypeRef::NonNegativeInteger => PropertyType::Uint64,
            TypeRef::Double => PropertyType::Float64,
            TypeRef::Boolean => PropertyType::Int64, // TODO: Boolean bitstream
            TypeRef::JsonString(_) => PropertyType::String,
            TypeRef::URI => PropertyType::String,
            TypeRef::Date => PropertyType::String,
            TypeRef::DateTime => PropertyType::String,
            TypeRef::Measure => PropertyType::Float64,
            TypeRef::Point => PropertyType::String, // TODO: VEC3<f64>
            TypeRef::Named(_) => unreachable!(),
            TypeRef::Unknown => unreachable!(),
        };
        let is_array = attr.max_occurs != Some(1);
        Property::new(type_, is_array)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyType {
    Int64,
    Uint64,
    Float64,
    String,
    // Boolean,
    Enum,
}

pub fn add_padding(buf: &mut Vec<u8>, align: usize) {
    let len = buf.len();
    let pad = (align - (len % align)) % align;
    buf.resize(len + pad, 0);
}
