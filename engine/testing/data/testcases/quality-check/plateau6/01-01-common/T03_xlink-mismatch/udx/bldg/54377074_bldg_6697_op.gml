<?xml version="1.0" encoding="UTF-8"?>
<!-- T03: xlink:hrefs whose referent is not of the type the association allows.

     One building of 射水市. Its LOD2 solid refers by xlink:href to the polygons
     of its boundary surfaces, and its two building units refer by xlink:href to
     its building room.

     T03 errors (3):
     - A gml:surfaceMember of the LOD2 solid refers to the building itself,
       bldg_f036798c-4381-4af2-97fc-e78d8fe34001, instead of the roof polygon. A surface member must be a
       surface.
     - A gml:surfaceMember of the LOD2 solid refers to the gml:LinearRing
       ring_8d2f4b61-0c7e-4a93-b5d8-1e6f2a9c4b07, instead of its polygon. A ring is not a surface.
     - The bldg:buildingRoom of the building unit unit_9c4e1b7a-3f2d-4a6e-8b5c-0d1e7f3a6b92
       refers to the other building unit, unit_2e9d7a15-4c3b-4e8f-b6a1-7f0c9d2e5a83. A building room must be a
       bldg:BuildingRoom.
     Xlink with no referent (1):
     - A gml:surfaceMember of the LOD2 solid refers to
       poly_ffaa06df-721a-40df-8b75-d67675907578, which does not exist.

     Correct:
     - The other 4 gml:surfaceMember of the LOD2 solid each refer to a
       gml:Polygon of a boundary surface.
     - The bldg:buildingRoom of the building unit unit_2e9d7a15-4c3b-4e8f-b6a1-7f0c9d2e5a83
       refers to the building room room_6b0f3c2e-8d41-4f7a-9a25-1c7e5d3b9f40. -->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
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
		<bldg:Building gml:id="bldg_f036798c-4381-4af2-97fc-e78d8fe34001">
			<gml:name>16211-bldg-78</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0>000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1>000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2>000</urc:geometrySrcDescLod2>
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
			<core:boundary>
				<con:GroundSurface gml:id="surface_3a7c1e52-9b04-4d6f-8e21-5c9a0b7d3f14">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3a7c1e52-9b04-4d6f-8e21-5c9a0b7d3f14">
									<gml:exterior>
										<gml:LinearRing gml:id="ring_3a7c1e52-9b04-4d6f-8e21-5c9a0b7d3f14">
											<gml:posList>36.6477 137.0537 0 36.6477 137.0536 0 36.6478 137.0536 0 36.6478 137.0537 0 36.6477 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface_8d2f4b61-0c7e-4a93-b5d8-1e6f2a9c4b07">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8d2f4b61-0c7e-4a93-b5d8-1e6f2a9c4b07">
									<gml:exterior>
										<gml:LinearRing gml:id="ring_8d2f4b61-0c7e-4a93-b5d8-1e6f2a9c4b07">
											<gml:posList>36.6477 137.0537 0 36.6478 137.0537 0 36.6478 137.0537 9 36.6477 137.0537 9 36.6477 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface_5e9b3c70-2d8a-4f14-a6c2-9b0e7d1f3a58">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5e9b3c70-2d8a-4f14-a6c2-9b0e7d1f3a58">
									<gml:exterior>
										<gml:LinearRing gml:id="ring_5e9b3c70-2d8a-4f14-a6c2-9b0e7d1f3a58">
											<gml:posList>36.6478 137.0537 0 36.6478 137.0536 0 36.6478 137.0536 9 36.6478 137.0537 9 36.6478 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface_c14a7e93-6f2b-4d05-8a1c-3e7b9d0f2c66">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c14a7e93-6f2b-4d05-8a1c-3e7b9d0f2c66">
									<gml:exterior>
										<gml:LinearRing gml:id="ring_c14a7e93-6f2b-4d05-8a1c-3e7b9d0f2c66">
											<gml:posList>36.6478 137.0536 0 36.6477 137.0536 0 36.6477 137.0536 9 36.6478 137.0536 9 36.6478 137.0536 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface_71f0d2a8-4b6c-4e39-9d57-a2c8e1b3f095">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_71f0d2a8-4b6c-4e39-9d57-a2c8e1b3f095">
									<gml:exterior>
										<gml:LinearRing gml:id="ring_71f0d2a8-4b6c-4e39-9d57-a2c8e1b3f095">
											<gml:posList>36.6477 137.0536 0 36.6477 137.0537 0 36.6477 137.0537 9 36.6477 137.0536 9 36.6477 137.0536 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface_e6b38f1d-7a05-4c92-b1e4-0d9f6a2c5b83">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_e6b38f1d-7a05-4c92-b1e4-0d9f6a2c5b83">
									<gml:exterior>
										<gml:LinearRing gml:id="ring_e6b38f1d-7a05-4c92-b1e4-0d9f6a2c5b83">
											<gml:posList>36.6477 137.0537 9 36.6478 137.0537 9 36.6478 137.0536 9 36.6477 137.0536 9 36.6477 137.0537 9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_f036798c-4381-4af2-97fc-e78d8fe34001">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_f036798c-4381-4af2-97fc-e78d8fe34001">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6477 137.0537 0 36.6478 137.0537 0 36.6478 137.0536 0 36.6477 137.0536 0 36.6477 137.0537 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<core:lod1Solid>
				<gml:Solid gml:id="solid_lod1_f036798c-4381-4af2-97fc-e78d8fe34001">
					<gml:exterior>
						<gml:Shell>
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
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<core:lod2Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember xlink:href="#poly_3a7c1e52-9b04-4d6f-8e21-5c9a0b7d3f14"/>
							<gml:surfaceMember xlink:href="#ring_8d2f4b61-0c7e-4a93-b5d8-1e6f2a9c4b07"/>
							<gml:surfaceMember xlink:href="#poly_5e9b3c70-2d8a-4f14-a6c2-9b0e7d1f3a58"/>
							<gml:surfaceMember xlink:href="#poly_c14a7e93-6f2b-4d05-8a1c-3e7b9d0f2c66"/>
							<gml:surfaceMember xlink:href="#poly_71f0d2a8-4b6c-4e39-9d57-a2c8e1b3f095"/>
							<gml:surfaceMember xlink:href="#bldg_f036798c-4381-4af2-97fc-e78d8fe34001"/>
							<gml:surfaceMember xlink:href="#poly_ffaa06df-721a-40df-8b75-d67675907578"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
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
			<bldg:buildingRoom>
				<bldg:BuildingRoom gml:id="room_6b0f3c2e-8d41-4f7a-9a25-1c7e5d3b9f40"/>
			</bldg:buildingRoom>
			<bldg:buildingSubdivision>
				<bldg:BuildingUnit gml:id="unit_2e9d7a15-4c3b-4e8f-b6a1-7f0c9d2e5a83">
					<bldg:buildingRoom xlink:href="#room_6b0f3c2e-8d41-4f7a-9a25-1c7e5d3b9f40"/>
				</bldg:BuildingUnit>
			</bldg:buildingSubdivision>
			<bldg:buildingSubdivision>
				<bldg:BuildingUnit gml:id="unit_9c4e1b7a-3f2d-4a6e-8b5c-0d1e7f3a6b92">
					<bldg:buildingRoom xlink:href="#unit_2e9d7a15-4c3b-4e8f-b6a1-7f0c9d2e5a83"/>
				</bldg:BuildingUnit>
			</bldg:buildingSubdivision>
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
</core:CityModel>
