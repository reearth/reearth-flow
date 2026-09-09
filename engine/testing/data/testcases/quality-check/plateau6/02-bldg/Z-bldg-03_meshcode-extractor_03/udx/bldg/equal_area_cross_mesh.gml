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
	<!--
		Z-bldg-03 (図郭不正 / meshcode) plateau6 テストデータ _03
		plateau4 の Z-bldg-03_meshcode-extractor_03 を CityGML 3.0 に移植（座標・gml:id は逐語流用）。
		メッシュ境界に跨り面積がほぼ等しい建物のタイブレーク（同面積時はメッシュコードが小さい方を採用）を検証。
		3 建物すべてメッシュ跨ぎで図郭不正、計 3 件。ファイル名 equal_area_cross_mesh.gml にメッシュコードは含まれない。
	-->
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.658 137.074 0</gml:lowerCorner>
			<gml:upperCorner>36.668 137.076 25.0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>

	<!-- Building with exactly equal areas in two adjacent 1km meshes -->
	<!-- Mesh boundary at 137.075 longitude (multiple of 0.0125 degree intervals) -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_5a56023c-0e19-4333-b840-ac0647de7a9b">
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>422</bldg:usage>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">12.0</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_bldg_5a56023c-0e19-4333-b840-ac0647de7a9b">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_bldg_5a56023c-0e19-4333-b840-ac0647de7a9b">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.66625 137.0745 0 36.66625 137.0755 0 36.66750 137.0755 0 36.66750 137.0745 0 36.66625 137.0745 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-1</uro:buildingID>
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

	<!-- Circular building precisely centered on mesh boundary for perfect equal division -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_0f775683-636e-4948-8aa8-bd8db6921f0b">
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>431</bldg:usage>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.0</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_bldg_0f775683-636e-4948-8aa8-bd8db6921f0b">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_bldg_0f775683-636e-4948-8aa8-bd8db6921f0b">
							<gml:exterior>
								<gml:LinearRing>
									<!-- Approximated circle centered exactly on longitude boundary 137.075 -->
									<gml:posList>36.660 137.0745 0 36.65975 137.07465 0 36.65958579 137.07480 0 36.6595 137.075 0 36.65958579 137.0752 0 36.65975 137.07535 0 36.660 137.0755 0 36.66025 137.07535 0 36.66041421 137.0752 0 36.66050 137.075 0 36.66041421 137.07480 0 36.66025 137.07465 0 36.660 137.0745 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-2</uro:buildingID>
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

	<!-- Rectangular building crossing latitude mesh boundary -->
	<!-- Mesh boundary at 36.666666667 latitude (multiple of 0.008333333 degree intervals) -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_3682cf30-a7cd-40f5-ba9f-7146698e1590">
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3002</bldg:class>
			<bldg:usage>452</bldg:usage>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">10.5</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_bldg_3682cf30-a7cd-40f5-ba9f-7146698e1590">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_bldg_3682cf30-a7cd-40f5-ba9f-7146698e1590">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.66625 137.0757 0 36.66625 137.0760 0 36.66708333333 137.0760 0 36.66708333333 137.0757 0 36.66625 137.0757 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-3</uro:buildingID>
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
