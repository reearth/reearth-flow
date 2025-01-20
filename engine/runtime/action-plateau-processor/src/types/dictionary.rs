use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Default)]
#[serde(rename = "Dictionary")]
pub(crate) struct Dictionary {
    pub(crate) name: Name,
    #[serde(rename = "dictionaryEntry")]
    pub(crate) entries: Vec<DictionaryEntry>,
}

#[derive(Deserialize, Debug, PartialEq, Default)]
#[serde(rename = "dictionaryEntry")]
pub(crate) struct DictionaryEntry {
    #[serde(rename = "Definition")]
    pub(crate) definition: Definition,
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
#[serde(default)]
pub(crate) struct Definition {
    #[serde(rename = "@id")]
    pub(crate) id: String,
    pub(crate) description: Description,
    pub(crate) name: Name,
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
#[serde(rename = "description")]
pub(crate) struct Description {
    #[serde(rename = "$text")]
    pub(crate) value: String,
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
#[serde(rename = "name")]
pub(crate) struct Name {
    #[serde(rename = "$text")]
    pub(crate) value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_XML: &str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<gml:Dictionary xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:gml="http://www.opengis.net/gml" xsi:schemaLocation="http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/profiles/SimpleDictionary/1.0.0/gmlSimpleDictionaryProfile.xsd" gml:id="cl_9d8ed669-98d2-430a-8e2c-c4158b869749">
	<gml:name>Common_localPublicAuthoritiesType</gml:name>
	<gml:dictionaryEntry>
		<gml:Definition gml:id="id1">
			<gml:description>北海道</gml:description>
			<gml:name>01</gml:name>
		</gml:Definition>
	</gml:dictionaryEntry>
	<gml:dictionaryEntry>
		<gml:Definition gml:id="id2">
			<gml:description>北海道札幌市</gml:description>
			<gml:name>01100</gml:name>
		</gml:Definition>
	</gml:dictionaryEntry>
</gml:Dictionary>
    "#;

    #[test]
    fn test_deserialize_dictionary_entry() {
        let dic: Dictionary = quick_xml::de::from_str(TEST_XML).unwrap();
        assert_eq!(dic.name.value, "Common_localPublicAuthoritiesType");
        assert_eq!(dic.entries.len(), 2);
        assert_eq!(dic.entries[0].definition.id, "id1");
        assert_eq!(dic.entries[0].definition.description.value, "北海道");
        assert_eq!(dic.entries[0].definition.name.value, "01");
        assert_eq!(dic.entries[1].definition.id, "id2");
        assert_eq!(dic.entries[1].definition.description.value, "北海道札幌市");
        assert_eq!(dic.entries[1].definition.name.value, "01100");
    }
}
