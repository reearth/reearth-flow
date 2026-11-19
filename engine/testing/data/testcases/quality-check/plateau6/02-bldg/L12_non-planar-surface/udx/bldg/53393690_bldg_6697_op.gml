<?xml version="1.0" encoding="UTF-8"?>
<!-- L12: LOD2 faces off their plane by more than the 0.03 m thickness tolerance.

     In each changed building one corner of the LOD2 RoofSurface is raised. The
     corner is raised in every boundary surface that has it, so the walls stay
     planar and only the roof can be non-planar. The roof thickness is half the
     raise.

     Buildings:
       - bldg_eceb9e92-a5be-41db-bba2-d30bd61bd863: unchanged. No error.
       - bldg_cb214d58-e0de-4bc5-886a-c1e2f8f2ed06: corner raised by 0.04 m
         (thickness 0.02 m). No error.
       - bldg_79cb8bc6-73e6-43e7-9ba0-a8ada41e94e4: corner raised by 0.1 m
         (thickness 0.05 m). The roof is one non-planar surface error. -->

<core:CityModel xmlns:urf="https://www.geospatial.jp/iur/urf/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xAL="urn:oasis:names:tc:ciq:xal:3" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:luse="http://www.opengis.net/citygml/landuse/3.0" xmlns:ct="urn:oasis:names:tc:ciq:ct:3" xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:pcl="http://www.opengis.net/citygml/pointcloud/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:dyn="http://www.opengis.net/citygml/dynamizer/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:vers="http://www.opengis.net/citygml/versioning/3.0" xmlns:brid="http://www.opengis.net/citygml/bridge/3.0" xmlns:dem="http://www.opengis.net/citygml/relief/3.0" xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:app="http://www.opengis.net/citygml/appearance/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:tran="http://www.opengis.net/citygml/transportation/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xsi:schemaLocation="http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd http://www.opengis.net/citygml/bridge/3.0 http://schemas.opengis.net/citygml/bridge/3.0/bridge.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd http://www.opengis.net/citygml/cityobjectgroup/3.0 http://schemas.opengis.net/citygml/cityobjectgroup/3.0/cityObjectGroup.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd http://www.opengis.net/citygml/dynamizer/3.0 http://schemas.opengis.net/citygml/dynamizer/3.0/dynamizer.xsd http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd http://www.opengis.net/citygml/pointcloud/3.0 http://schemas.opengis.net/citygml/pointcloud/3.0/pointCloud.xsd http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd http://www.opengis.net/citygml/versioning/3.0 http://schemas.opengis.net/citygml/versioning/3.0/versioning.xsd http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd https://www.geospatial.jp/iur/urf/4.0 ../../schemas/iur/urf/4.0/urbanFunction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd  http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.66122464943641 139.75013227597438 0.0</gml:lowerCorner>
			<gml:upperCorner>35.66656941672891 139.7590422319645 16.8</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_eceb9e92-a5be-41db-bba2-d30bd61bd863">
			<core:creationDate>2024-03-15T00:00:00Z</core:creationDate>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>建物構造コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">99</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>延べ面積換算コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>変化フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>過年度修正フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>調整フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>高度地区</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key105.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>防火及び準防火地域</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key106.xml">10</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:adeOfAbstractCityObject>
				<urc:ExteriorDataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod2>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
					<urc:appearanceSrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">1</urc:appearanceSrcDescLod2>
					<urc:lodType codeSpace="../../codelists/Building_lodType.xml">2.2_exterior</urc:lodType>
					<urc:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</urc:lod1HeightType>
					<urc:granularity codeSpace="../../codelists/Common_granularity.xml">000</urc:granularity>
					<urc:publicSurveyDataQualityAttribute>
						<urc:PublicSurveyDataQualityAttribute>
							<urc:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod0>
							<urc:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod1>
							<urc:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod2>
							<urc:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</urc:publicSurveySrcDescLod0>
							<urc:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</urc:publicSurveySrcDescLod1>
							<urc:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">012</urc:publicSurveySrcDescLod1>
							<urc:publicSurveySrcDescLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">012</urc:publicSurveySrcDescLod2>
						</urc:PublicSurveyDataQualityAttribute>
					</urc:publicSurveyDataQualityAttribute>
				</urc:ExteriorDataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<con:GroundSurface gml:id="surface-179e6862-ef74-4e98-a4cd-4188c1434365">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-179e6862-ef74-4e98-a4cd-4188c1434365">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-179e6862-ef74-4e98-a4cd-4188c1434365">
											<gml:posList>35.661261292985984 139.7525124783774 2.85 35.66122464943641 139.7524950642033 2.85 35.66124891665526 139.75241783517464 2.85 35.66128556022322 139.7524352603637 2.85 35.661261292985984 139.7525124783774 2.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-918262f6-a279-48f4-9570-2ef4067f56d4">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-918262f6-a279-48f4-9570-2ef4067f56d4">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-918262f6-a279-48f4-9570-2ef4067f56d4">
											<gml:posList>35.66122464943641 139.7524950642033 5.611 35.661261292985984 139.7525124783774 5.611 35.66128556022322 139.7524352603637 5.611 35.66124891665526 139.75241783517464 5.611 35.66122464943641 139.7524950642033 5.611</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-b145626b-2fc9-4963-9239-4cc1d698d620">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-b145626b-2fc9-4963-9239-4cc1d698d620">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-b145626b-2fc9-4963-9239-4cc1d698d620">
											<gml:posList>35.66124891665526 139.75241783517464 5.611 35.66128556022322 139.7524352603637 5.611 35.66128556022322 139.7524352603637 2.85 35.66124891665526 139.75241783517464 2.85 35.66124891665526 139.75241783517464 5.611</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-f2928c53-b8e7-4959-8f4f-ed6b4c4f0aba">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-f2928c53-b8e7-4959-8f4f-ed6b4c4f0aba">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-f2928c53-b8e7-4959-8f4f-ed6b4c4f0aba">
											<gml:posList>35.66122464943641 139.7524950642033 5.611 35.66124891665526 139.75241783517464 5.611 35.66124891665526 139.75241783517464 2.85 35.66122464943641 139.7524950642033 2.85 35.66122464943641 139.7524950642033 5.611</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-8f50405c-a378-4668-92c4-fa462916d8fb">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-8f50405c-a378-4668-92c4-fa462916d8fb">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-8f50405c-a378-4668-92c4-fa462916d8fb">
											<gml:posList>35.66128556022322 139.7524352603637 5.611 35.661261292985984 139.7525124783774 5.611 35.661261292985984 139.7525124783774 2.85 35.66128556022322 139.7524352603637 2.85 35.66128556022322 139.7524352603637 5.611</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-3e98cd94-14da-4406-a942-f105b4de9b54">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-3e98cd94-14da-4406-a942-f105b4de9b54">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-3e98cd94-14da-4406-a942-f105b4de9b54">
											<gml:posList>35.661261292985984 139.7525124783774 5.611 35.66122464943641 139.7524950642033 5.611 35.66122464943641 139.7524950642033 2.85 35.661261292985984 139.7525124783774 2.85 35.661261292985984 139.7525124783774 5.611</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:lod0MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.661261292985984 139.7525124783774 0 35.66128556022322 139.75243526036368 0 35.66124891665526 139.75241783517464 0 35.66122464943641 139.7524950642033 0 35.661261292985984 139.7525124783774 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.661261292985984 139.7525124783774 2.9 35.66122464943641 139.7524950642033 2.9 35.66124891665526 139.75241783517464 2.9 35.66128556022322 139.75243526036368 2.9 35.661261292985984 139.7525124783774 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.661261292985984 139.7525124783774 2.9 35.66128556022322 139.75243526036368 2.9 35.66128556022322 139.75243526036368 9 35.661261292985984 139.7525124783774 9 35.661261292985984 139.7525124783774 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66128556022322 139.75243526036368 2.9 35.66124891665526 139.75241783517464 2.9 35.66124891665526 139.75241783517464 9 35.66128556022322 139.75243526036368 9 35.66128556022322 139.75243526036368 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66124891665526 139.75241783517464 2.9 35.66122464943641 139.7524950642033 2.9 35.66122464943641 139.7524950642033 9 35.66124891665526 139.75241783517464 9 35.66124891665526 139.75241783517464 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66122464943641 139.7524950642033 2.9 35.661261292985984 139.7525124783774 2.9 35.661261292985984 139.7525124783774 9 35.66122464943641 139.7524950642033 9 35.66122464943641 139.7524950642033 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.661261292985984 139.7525124783774 9 35.66128556022322 139.75243526036368 9 35.66124891665526 139.75241783517464 9 35.66122464943641 139.7524950642033 9 35.661261292985984 139.7525124783774 9</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-179e6862-ef74-4e98-a4cd-4188c1434365"/>
							<gml:surfaceMember xlink:href="#poly-918262f6-a279-48f4-9570-2ef4067f56d4"/>
							<gml:surfaceMember xlink:href="#poly-8f50405c-a378-4668-92c4-fa462916d8fb"/>
							<gml:surfaceMember xlink:href="#poly-b145626b-2fc9-4963-9239-4cc1d698d620"/>
							<gml:surfaceMember xlink:href="#poly-f2928c53-b8e7-4959-8f4f-ed6b4c4f0aba"/>
							<gml:surfaceMember xlink:href="#poly-3e98cd94-14da-4406-a942-f105b4de9b54"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">6.1</con:value>
				</con:Height>
			</con:height>
			<bldg:class codeSpace="../../codelists/Building_class.xml">9999</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:storeysAboveGround>-9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>-9999</bldg:storeysBelowGround>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">-9999</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:areaClassificationType codeSpace="../../codelists/Common_areaClassificationType.xml">22</uro:areaClassificationType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">10</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>600</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-6100</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cb214d58-e0de-4bc5-886a-c1e2f8f2ed06">
			<core:creationDate>2024-03-15T00:00:00Z</core:creationDate>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>建物構造コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">99</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>延べ面積換算コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>変化フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>過年度修正フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>調整フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>高度地区</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key105.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>防火及び準防火地域</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key106.xml">10</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:adeOfAbstractCityObject>
				<urc:ExteriorDataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod2>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
					<urc:appearanceSrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">1</urc:appearanceSrcDescLod2>
					<urc:lodType codeSpace="../../codelists/Building_lodType.xml">2.2_exterior</urc:lodType>
					<urc:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</urc:lod1HeightType>
					<urc:granularity codeSpace="../../codelists/Common_granularity.xml">000</urc:granularity>
					<urc:publicSurveyDataQualityAttribute>
						<urc:PublicSurveyDataQualityAttribute>
							<urc:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod0>
							<urc:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod1>
							<urc:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod2>
							<urc:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</urc:publicSurveySrcDescLod0>
							<urc:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</urc:publicSurveySrcDescLod1>
							<urc:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">012</urc:publicSurveySrcDescLod1>
							<urc:publicSurveySrcDescLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">012</urc:publicSurveySrcDescLod2>
						</urc:PublicSurveyDataQualityAttribute>
					</urc:publicSurveyDataQualityAttribute>
				</urc:ExteriorDataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<con:GroundSurface gml:id="surface-b2a4f488-a60a-414d-81e5-31f7ad96f164">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-b2a4f488-a60a-414d-81e5-31f7ad96f164">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-b2a4f488-a60a-414d-81e5-31f7ad96f164">
											<gml:posList>35.66387358547611 139.75013227597438 5.03 35.66390556655629 139.75014579546237 5.03 35.66390433464266 139.7501500933885 5.03 35.663872353562965 139.75013657389903 5.03 35.66387358547611 139.75013227597438 5.03</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-d41919ad-ea5d-4caf-bcbf-414fcad6106b">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-d41919ad-ea5d-4caf-bcbf-414fcad6106b">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-d41919ad-ea5d-4caf-bcbf-414fcad6106b">
											<gml:posList>35.66390556655629 139.75014579546237 9.088 35.66387358547611 139.75013227597438 9.048 35.663872353562965 139.75013657389903 9.048 35.66390433464266 139.7501500933885 9.048 35.66390556655629 139.75014579546237 9.088</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-14c1ecf9-7147-4041-b6fc-7871ab6667a8">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-14c1ecf9-7147-4041-b6fc-7871ab6667a8">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-14c1ecf9-7147-4041-b6fc-7871ab6667a8">
											<gml:posList>35.66390556655629 139.75014579546237 5.03 35.66387358547611 139.75013227597438 5.03 35.66387358547611 139.75013227597438 9.048 35.66390556655629 139.75014579546237 9.088 35.66390556655629 139.75014579546237 5.03</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-4e0cd52f-0ec5-4706-a741-673a710013f1">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-4e0cd52f-0ec5-4706-a741-673a710013f1">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-4e0cd52f-0ec5-4706-a741-673a710013f1">
											<gml:posList>35.66390433464266 139.7501500933885 5.03 35.66390556655629 139.75014579546237 5.03 35.66390556655629 139.75014579546237 9.088 35.66390433464266 139.7501500933885 9.048 35.66390433464266 139.7501500933885 5.03</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-35896a76-52fa-4f18-84b7-4763033a8503">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-35896a76-52fa-4f18-84b7-4763033a8503">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-35896a76-52fa-4f18-84b7-4763033a8503">
											<gml:posList>35.66387358547611 139.75013227597438 5.03 35.663872353562965 139.75013657389903 5.03 35.663872353562965 139.75013657389903 9.048 35.66387358547611 139.75013227597438 9.048 35.66387358547611 139.75013227597438 5.03</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-77a285b5-8e8f-46da-8d80-330797ca36f2">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-77a285b5-8e8f-46da-8d80-330797ca36f2">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-77a285b5-8e8f-46da-8d80-330797ca36f2">
											<gml:posList>35.663872353562965 139.75013657389903 5.03 35.66390433464266 139.7501500933885 5.03 35.66390433464266 139.7501500933885 9.048 35.663872353562965 139.75013657389903 9.048 35.663872353562965 139.75013657389903 5.03</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:lod0MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.66387358457473 139.75013227597532 0 35.66387235356296 139.75013657389903 0 35.66390433464266 139.7501500933885 0 35.66390556655628 139.75014579546237 0 35.66387358457473 139.75013227597532 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66387358457473 139.75013227597532 5 35.66390556655628 139.75014579546237 5 35.66390433464266 139.7501500933885 5 35.66387235356296 139.75013657389903 5 35.66387358457473 139.75013227597532 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66387358457473 139.75013227597532 5 35.66387235356296 139.75013657389903 5 35.66387235356296 139.75013657389903 16.8 35.66387358457473 139.75013227597532 16.8 35.66387358457473 139.75013227597532 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66387235356296 139.75013657389903 5 35.66390433464266 139.7501500933885 5 35.66390433464266 139.7501500933885 16.8 35.66387235356296 139.75013657389903 16.8 35.66387235356296 139.75013657389903 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66390433464266 139.7501500933885 5 35.66390556655628 139.75014579546237 5 35.66390556655628 139.75014579546237 16.8 35.66390433464266 139.7501500933885 16.8 35.66390433464266 139.7501500933885 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66390556655628 139.75014579546237 5 35.66387358457473 139.75013227597532 5 35.66387358457473 139.75013227597532 16.8 35.66390556655628 139.75014579546237 16.8 35.66390556655628 139.75014579546237 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66387358457473 139.75013227597532 16.8 35.66387235356296 139.75013657389903 16.8 35.66390433464266 139.7501500933885 16.8 35.66390556655628 139.75014579546237 16.8 35.66387358457473 139.75013227597532 16.8</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-b2a4f488-a60a-414d-81e5-31f7ad96f164"/>
							<gml:surfaceMember xlink:href="#poly-d41919ad-ea5d-4caf-bcbf-414fcad6106b"/>
							<gml:surfaceMember xlink:href="#poly-35896a76-52fa-4f18-84b7-4763033a8503"/>
							<gml:surfaceMember xlink:href="#poly-77a285b5-8e8f-46da-8d80-330797ca36f2"/>
							<gml:surfaceMember xlink:href="#poly-14c1ecf9-7147-4041-b6fc-7871ab6667a8"/>
							<gml:surfaceMember xlink:href="#poly-4e0cd52f-0ec5-4706-a741-673a710013f1"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">11.8</con:value>
				</con:Height>
			</con:height>
			<bldg:class codeSpace="../../codelists/Building_class.xml">9999</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:storeysAboveGround>-9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>-9999</bldg:storeysBelowGround>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">-9999</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:areaClassificationType codeSpace="../../codelists/Common_areaClassificationType.xml">22</uro:areaClassificationType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">10</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>600</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-9082</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_79cb8bc6-73e6-43e7-9ba0-a8ada41e94e4">
			<core:creationDate>2024-03-15T00:00:00Z</core:creationDate>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>建物構造コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">99</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>延べ面積換算コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>変化フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>過年度修正フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>調整フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>高度地区</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key105.xml">9</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>防火及び準防火地域</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key106.xml">10</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:adeOfAbstractCityObject>
				<urc:ExteriorDataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod2>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</urc:thematicSrcDesc>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
					<urc:appearanceSrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">1</urc:appearanceSrcDescLod2>
					<urc:lodType codeSpace="../../codelists/Building_lodType.xml">2.2_exterior</urc:lodType>
					<urc:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</urc:lod1HeightType>
					<urc:granularity codeSpace="../../codelists/Common_granularity.xml">000</urc:granularity>
					<urc:publicSurveyDataQualityAttribute>
						<urc:PublicSurveyDataQualityAttribute>
							<urc:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod0>
							<urc:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod1>
							<urc:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</urc:srcScaleLod2>
							<urc:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</urc:publicSurveySrcDescLod0>
							<urc:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</urc:publicSurveySrcDescLod1>
							<urc:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">012</urc:publicSurveySrcDescLod1>
							<urc:publicSurveySrcDescLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">012</urc:publicSurveySrcDescLod2>
						</urc:PublicSurveyDataQualityAttribute>
					</urc:publicSurveyDataQualityAttribute>
				</urc:ExteriorDataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<con:GroundSurface gml:id="surface-431c653f-da82-46af-bbf6-037311dfa5bc">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-431c653f-da82-46af-bbf6-037311dfa5bc">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-431c653f-da82-46af-bbf6-037311dfa5bc">
											<gml:posList>35.66655315749524 139.75904223196363 4.028 35.66654946676019 139.75900634779507 4.028 35.66656572599312 139.75900383638816 4.028 35.66656941672891 139.75903972056386 4.028 35.66655315749524 139.75904223196363 4.028</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-543b6ab5-c1b9-4829-a6df-32d9df07d689">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-543b6ab5-c1b9-4829-a6df-32d9df07d689">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-543b6ab5-c1b9-4829-a6df-32d9df07d689">
											<gml:posList>35.66654946676019 139.75900634779507 8.308 35.66655315749524 139.75904223196363 8.208 35.66656941672891 139.75903972056386 8.208 35.66656572599312 139.75900383638816 8.208 35.66654946676019 139.75900634779507 8.308</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-9e4c3084-321e-4efc-9fd9-f19f86ede051">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-9e4c3084-321e-4efc-9fd9-f19f86ede051">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-9e4c3084-321e-4efc-9fd9-f19f86ede051">
											<gml:posList>35.66655315749524 139.75904223196363 8.208 35.66654946676019 139.75900634779507 8.308 35.66654946676019 139.75900634779507 4.028 35.66655315749524 139.75904223196363 4.028 35.66655315749524 139.75904223196363 8.208</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-f075cc90-6e1f-44d5-9fe2-bf8b4a532345">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-f075cc90-6e1f-44d5-9fe2-bf8b4a532345">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-f075cc90-6e1f-44d5-9fe2-bf8b4a532345">
											<gml:posList>35.66656572599312 139.75900383638816 8.208 35.66656941672891 139.75903972056386 8.208 35.66656941672891 139.75903972056386 4.028 35.66656572599312 139.75900383638816 4.028 35.66656572599312 139.75900383638816 8.208</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-5ad60018-9a1b-4eb2-a595-4acf1c47542f">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-5ad60018-9a1b-4eb2-a595-4acf1c47542f">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-5ad60018-9a1b-4eb2-a595-4acf1c47542f">
											<gml:posList>35.66654946676019 139.75900634779507 8.308 35.66656572599312 139.75900383638816 8.208 35.66656572599312 139.75900383638816 4.028 35.66654946676019 139.75900634779507 4.028 35.66654946676019 139.75900634779507 8.308</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-91defbcd-1b9f-4556-8cb7-86ec676215f0">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-91defbcd-1b9f-4556-8cb7-86ec676215f0">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-91defbcd-1b9f-4556-8cb7-86ec676215f0">
											<gml:posList>35.66656941672891 139.75903972056386 8.208 35.66655315749524 139.75904223196363 8.208 35.66655315749524 139.75904223196363 4.028 35.66656941672891 139.75903972056386 4.028 35.66656941672891 139.75903972056386 8.208</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:lod0MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.666553156593864 139.7590422319645 0 35.66656941672891 139.75903972056386 0 35.66656572599312 139.75900383638816 0 35.66654946676019 139.75900634779507 0 35.666553156593864 139.7590422319645 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666553156593864 139.7590422319645 4 35.66654946676019 139.75900634779507 4 35.66656572599312 139.75900383638816 4 35.66656941672891 139.75903972056386 4 35.666553156593864 139.7590422319645 4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666553156593864 139.7590422319645 4 35.66656941672891 139.75903972056386 4 35.66656941672891 139.75903972056386 7.8 35.666553156593864 139.7590422319645 7.8 35.666553156593864 139.7590422319645 4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66656941672891 139.75903972056386 4 35.66656572599312 139.75900383638816 4 35.66656572599312 139.75900383638816 7.8 35.66656941672891 139.75903972056386 7.8 35.66656941672891 139.75903972056386 4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66656572599312 139.75900383638816 4 35.66654946676019 139.75900634779507 4 35.66654946676019 139.75900634779507 7.8 35.66656572599312 139.75900383638816 7.8 35.66656572599312 139.75900383638816 4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66654946676019 139.75900634779507 4 35.666553156593864 139.7590422319645 4 35.666553156593864 139.7590422319645 7.8 35.66654946676019 139.75900634779507 7.8 35.66654946676019 139.75900634779507 4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666553156593864 139.7590422319645 7.8 35.66656941672891 139.75903972056386 7.8 35.66656572599312 139.75900383638816 7.8 35.66654946676019 139.75900634779507 7.8 35.666553156593864 139.7590422319645 7.8</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-431c653f-da82-46af-bbf6-037311dfa5bc"/>
							<gml:surfaceMember xlink:href="#poly-543b6ab5-c1b9-4829-a6df-32d9df07d689"/>
							<gml:surfaceMember xlink:href="#poly-9e4c3084-321e-4efc-9fd9-f19f86ede051"/>
							<gml:surfaceMember xlink:href="#poly-5ad60018-9a1b-4eb2-a595-4acf1c47542f"/>
							<gml:surfaceMember xlink:href="#poly-91defbcd-1b9f-4556-8cb7-86ec676215f0"/>
							<gml:surfaceMember xlink:href="#poly-f075cc90-6e1f-44d5-9fe2-bf8b4a532345"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">3.8</con:value>
				</con:Height>
			</con:height>
			<bldg:class codeSpace="../../codelists/Building_class.xml">9999</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:storeysAboveGround>-9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>-9999</bldg:storeysBelowGround>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">-9999</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:areaClassificationType codeSpace="../../codelists/Common_areaClassificationType.xml">22</uro:areaClassificationType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">10</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">215</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>1000</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-36</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
