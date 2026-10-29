<?xml version="1.0" encoding="UTF-8"?>
<!--
  Faces the reader cannot read are read errors, and nothing else is reported (EPSG:6697).
  Each building has a LOD0 multi-surface of two faces. The broken ordinate is the
  last one in the building, so the extent check still reads every position in step.
  - The first building's second face has 14 ordinates in its posList, so the
    reader drops that face and keeps the first.
  - The second building's second face is written with gml:pos and ends with one
    more position of only two ordinates, so the reader drops that position and
    the ring stays closed.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd
urn:oasis:names:tc:ciq:xal:3 ../../schemas/citygml/xAL/3.0/xAL.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647 137.052 0</gml:lowerCorner>
			<gml:upperCorner>36.648 137.054 110</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_9982e37f-e5c0-4de5-8079-4a19ab5ffd9a">
			<gml:name>16211-bldg-78</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0>000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1>000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc>100</urc:thematicSrcDesc>
					<urc:thematicSrcDesc>201</urc:thematicSrcDesc>
					<urc:thematicSrcDesc>000</urc:thematicSrcDesc>
					<urc:lod1HeightType>2</urc:lod1HeightType>
					<urc:publicSurveyDataQualityAttribute>
						<urc:PublicSurveyDataQualityAttribute>
							<urc:srcScaleLod0>1</urc:srcScaleLod0>
							<urc:srcScaleLod1>1</urc:srcScaleLod1>
							<urc:publicSurveySrcDescLod0>023</urc:publicSurveySrcDescLod0>
							<urc:publicSurveySrcDescLod1>023</urc:publicSurveySrcDescLod1>
							<urc:publicSurveySrcDescLod1>003</urc:publicSurveySrcDescLod1>
						</urc:PublicSurveyDataQualityAttribute>
					</urc:publicSurveyDataQualityAttribute>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_9982e37f-e5c0-4de5-8079-4a19ab5ffd9a">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_9982e37f-e5c0-4de5-8079-4a19ab5ffd9a">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6477 137.0537 0 36.6478 137.0537 0 36.6478 137.0536 0 36.6477 137.0536 0 36.6477 137.0537 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_47a92ee8-c573-4ee9-9bed-f2c34fc32749">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6477 137.0535 0 36.6478 137.0535 0 36.6478 137.0534 0 36.6477 137.0534 0 36.6477 137.0535</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.6</con:value>
				</con:Height>
			</con:height>
			<bldg:class>3003</bldg:class>
			<bldg:usage>411</bldg:usage>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:Address xmlns:xAL="urn:oasis:names:tc:ciq:xal:3">
							<xAL:FreeTextAddress>
								<xAL:AddressLine>富山県射水市</xAL:AddressLine>
							</xAL:FreeTextAddress>
						</xAL:Address>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-78</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">76.2</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">76.2</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">58.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType>611</uro:buildingStructureType>
					<uro:fireproofStructureType>1011</uro:fireproofStructureType>
					<uro:landUseType>211</uro:landUseType>
					<uro:detailedUsage>4111</uro:detailedUsage>
					<uro:buildingHeight uom="m">8.6</uro:buildingHeight>
					<uro:surveyYear>2020-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_d5882c2b-cce8-412a-aa9b-82e499ea28f3">
			<gml:name>16211-bldg-79</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0>000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1>000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc>100</urc:thematicSrcDesc>
					<urc:thematicSrcDesc>201</urc:thematicSrcDesc>
					<urc:thematicSrcDesc>000</urc:thematicSrcDesc>
					<urc:lod1HeightType>2</urc:lod1HeightType>
					<urc:publicSurveyDataQualityAttribute>
						<urc:PublicSurveyDataQualityAttribute>
							<urc:srcScaleLod0>1</urc:srcScaleLod0>
							<urc:srcScaleLod1>1</urc:srcScaleLod1>
							<urc:publicSurveySrcDescLod0>023</urc:publicSurveySrcDescLod0>
							<urc:publicSurveySrcDescLod1>023</urc:publicSurveySrcDescLod1>
							<urc:publicSurveySrcDescLod1>003</urc:publicSurveySrcDescLod1>
						</urc:PublicSurveyDataQualityAttribute>
					</urc:publicSurveyDataQualityAttribute>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_d5882c2b-cce8-412a-aa9b-82e499ea28f3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_d5882c2b-cce8-412a-aa9b-82e499ea28f3">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6473 137.0537 0 36.6474 137.0537 0 36.6474 137.0536 0 36.6473 137.0536 0 36.6473 137.0537 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_f3d567fc-87b0-4705-b43f-fafa47e6e953">
							<gml:exterior>
								<gml:LinearRing>
									<gml:pos>36.6473 137.0535 0</gml:pos>
									<gml:pos>36.6474 137.0535 0</gml:pos>
									<gml:pos>36.6474 137.0534 0</gml:pos>
									<gml:pos>36.6473 137.0534 0</gml:pos>
									<gml:pos>36.6473 137.0535 0</gml:pos>
									<gml:pos>36.6473 137.0535</gml:pos>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.6</con:value>
				</con:Height>
			</con:height>
			<bldg:class>3003</bldg:class>
			<bldg:usage>411</bldg:usage>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:Address xmlns:xAL="urn:oasis:names:tc:ciq:xal:3">
							<xAL:FreeTextAddress>
								<xAL:AddressLine>富山県射水市</xAL:AddressLine>
							</xAL:FreeTextAddress>
						</xAL:Address>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-79</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">76.2</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">76.2</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">58.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType>611</uro:buildingStructureType>
					<uro:fireproofStructureType>1011</uro:fireproofStructureType>
					<uro:landUseType>211</uro:landUseType>
					<uro:detailedUsage>4111</uro:detailedUsage>
					<uro:buildingHeight uom="m">8.6</uro:buildingHeight>
					<uro:surveyYear>2020-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
