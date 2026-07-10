<?xml version="1.0" encoding="UTF-8"?>
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
		<bldg:Building gml:id="bldg_1922674b-cc85-4dc4-a0df-83e2b77cddc1">
			<gml:name>16211-bldg-L07</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3003</bldg:class>
			<bldg:usage>411</bldg:usage>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.6</con:value>
				</con:Height>
			</con:height>
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
			<!-- L07 base check target: LOD0 exterior LinearRing has two near-duplicate
			     consecutive points (within tolerance) -> one DuplicateConsecutivePoints error -->
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_1922674b-cc85-4dc4-a0df-83e2b77cddc1">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_1922674b-cc85-4dc4-a0df-83e2b77cddc1">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6477 137.0537 0 36.64770000001 137.0537 0 36.6478 137.0537 0 36.6478 137.0536 0 36.6477 137.0536 0 36.6477 137.0537 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<core:lod1Solid>
				<gml:Solid gml:id="solid_lod1_1922674b-cc85-4dc4-a0df-83e2b77cddc1">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0537 0 36.6477 137.0536 0 36.6478 137.0536 0 36.6478 137.0537 0 36.6477 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0537 0 36.6478 137.0537 0 36.6478 137.0537 9 36.6477 137.0537 9 36.6477 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6478 137.0537 0 36.6478 137.0536 0 36.6478 137.0536 9 36.6478 137.0537 9 36.6478 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6478 137.0536 0 36.6477 137.0536 0 36.6477 137.0536 9 36.6478 137.0536 9 36.6478 137.0536 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0536 0 36.6477 137.0537 0 36.6477 137.0537 9 36.6477 137.0536 9 36.6477 137.0536 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0537 9 36.6478 137.0537 9 36.6478 137.0536 9 36.6477 137.0536 9 36.6477 137.0537 9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-L07</uro:buildingID>
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
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
