<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.05977243067618 138.87734357774562 0</gml:lowerCorner>
			<gml:upperCorner>35.06144625090657 138.87881513518371 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_0af7ecd0-f6f8-4bed-8903-199eebf853d4">
			<gml:name>Null</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="区分">
				<gen:value>人工</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="面積">
				<gen:value>17800</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="漁港施設の名称">
				<gen:value>志下泊地</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="取得の価格">
				<gen:value>不明</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.05980692751216 138.87881513518371 0 35.06144625090657 138.87830129150237 0 35.061265130813226 138.87734357774562 0 35.05977243067618 138.87774591471248 0 35.05980692751216 138.87881513518371 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityAttribute>
				<uro:FishingPortFacility>
					<uro:facilityId>W-1-1</uro:facilityId>
					<uro:facilityDetailsType codeSpace="../../codelists/FishingPortFacilityAttribute_facilityDetailsType.xml">23</uro:facilityDetailsType>
					<uro:portName>静浦漁港</uro:portName>
					<uro:portType codeSpace="../../codelists/FishingPortFacilityAttribute_portType.xml">2</uro:portType>
					<uro:address>志下</uro:address>
					<uro:designatedArea>不明</uro:designatedArea>
					<uro:referenceNumber>W-1-1</uro:referenceNumber>
					<uro:facilityManager>静岡県</uro:facilityManager>
					<uro:dateOfConstructionOrAcquisition>1975-03-31</uro:dateOfConstructionOrAcquisition>
					<uro:note>小島防波堤</uro:note>
				</uro:FishingPortFacility>
			</uro:tranFacilityAttribute>
			<uro:tranFacilityAttribute>
				<uro:FishingPortCapacity>
					<uro:facilityId>W-1-1</uro:facilityId>
					<uro:waterDepth3-6m uom="m2">17800</uro:waterDepth3-6m>
				</uro:FishingPortCapacity>
			</uro:tranFacilityAttribute>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>W-1-1</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityTypeAttribute>
				<uro:FacilityTypeAttribute>
					<uro:class codeSpace="../../codelists/FacilityTypeAttribute_class.xml">06</uro:class>
					<uro:function codeSpace="../../codelists/FacilityTypeAttribute_function.xml">0801</uro:function>
				</uro:FacilityTypeAttribute>
			</uro:tranFacilityTypeAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
</core:CityModel>
