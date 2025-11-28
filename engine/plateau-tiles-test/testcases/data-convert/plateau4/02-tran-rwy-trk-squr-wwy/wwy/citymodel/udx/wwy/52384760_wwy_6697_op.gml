<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.054357805637395 138.87906299890074 0</gml:lowerCorner>
			<gml:upperCorner>35.0584266045027 138.8829785393228 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_807aa125-cda5-498e-b2c6-8268c409e6d0">
			<gml:name>Null</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="区分">
				<gen:value>人工</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="面積">
				<gen:value>28800</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="漁港施設の名称">
				<gen:value>馬込泊地</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="取得の価格">
				<gen:value>140,000</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.0584266045027 138.8799465437886 0 35.05742643903865 138.87906299890074 0 35.05685940882028 138.87998220757504 0 35.05611785002322 138.88065227493337 0 35.05629778769166 138.8810247489965 0 35.05657058919889 138.8815894604818 0 35.05734579725642 138.88123474690698 0 35.05782252721488 138.88066654352895 0 35.0584266045027 138.8799465437886 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityAttribute>
				<uro:FishingPortFacility>
					<uro:facilityId>W-1-3</uro:facilityId>
					<uro:facilityDetailsType codeSpace="../../codelists/FishingPortFacilityAttribute_facilityDetailsType.xml">23</uro:facilityDetailsType>
					<uro:portName>静浦漁港</uro:portName>
					<uro:portType codeSpace="../../codelists/FishingPortFacilityAttribute_portType.xml">2</uro:portType>
					<uro:address>馬込</uro:address>
					<uro:designatedArea>不明</uro:designatedArea>
					<uro:referenceNumber>W-1-3</uro:referenceNumber>
					<uro:facilityManager>静岡県</uro:facilityManager>
					<uro:dateOfConstructionOrAcquisition>2002-03-31</uro:dateOfConstructionOrAcquisition>
					<uro:note>Ｈ12改修事業,（うち16ｍは取合岸壁-3.0ｍ）</uro:note>
				</uro:FishingPortFacility>
			</uro:tranFacilityAttribute>
			<uro:tranFacilityAttribute>
				<uro:FishingPortCapacity>
					<uro:facilityId>W-1-3</uro:facilityId>
					<uro:waterDepth3-6m uom="m2">16300</uro:waterDepth3-6m>
					<uro:waterDepth6-m uom="m2">12500</uro:waterDepth6-m>
				</uro:FishingPortCapacity>
			</uro:tranFacilityAttribute>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>W-1-3</uro:id>
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
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_1f1a017f-5381-4a39-82ed-b45951e46d4e">
			<gml:name>Null</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="区分">
				<gen:value>人工</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="面積">
				<gen:value>16900</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="漁港施設の名称">
				<gen:value>獅子浜泊地</gen:value>
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
									<gml:posList>35.054650063959876 138.8829785393228 0 35.055417117444975 138.88266289753443 0 35.05587350712265 138.88233277219769 0 35.05533558767435 138.8811872728213 0 35.054357805637395 138.88175963511736 0 35.054650063959876 138.8829785393228 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityAttribute>
				<uro:FishingPortFacility>
					<uro:facilityId>W-1-2</uro:facilityId>
					<uro:facilityDetailsType codeSpace="../../codelists/FishingPortFacilityAttribute_facilityDetailsType.xml">23</uro:facilityDetailsType>
					<uro:portName>静浦漁港</uro:portName>
					<uro:portType codeSpace="../../codelists/FishingPortFacilityAttribute_portType.xml">2</uro:portType>
					<uro:address>獅子浜</uro:address>
					<uro:designatedArea>不明</uro:designatedArea>
					<uro:referenceNumber>W-1-2</uro:referenceNumber>
					<uro:facilityManager>静岡県</uro:facilityManager>
					<uro:dateOfConstructionOrAcquisition>1986-03-31</uro:dateOfConstructionOrAcquisition>
					<uro:note>獅子浜１号防波堤</uro:note>
				</uro:FishingPortFacility>
			</uro:tranFacilityAttribute>
			<uro:tranFacilityAttribute>
				<uro:FishingPortCapacity>
					<uro:facilityId>W-1-2</uro:facilityId>
					<uro:waterDepth3-6m uom="m2">16900</uro:waterDepth3-6m>
				</uro:FishingPortCapacity>
			</uro:tranFacilityAttribute>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>W-1-2</uro:id>
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
