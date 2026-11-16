<?xml version="1.0" encoding="UTF-8"?>
<!-- L06: three buildings of 53393690, one inside the envelope and two that fail the extents check.
     bldg_0cd4b5d2-adea-49b6-ba38-1619f9431106: inside the envelope, the pair of both errors.
     bldg_3635e238-46d9-43d3-a189-abe6a02f22b5: the elevation of the second vertex of its first posList is 999, above the envelope.
     bldg_aabdf030-7e6e-401b-b12d-ac52821c1e83: the elevation of the second vertex of its first posList is NaN. -->
<core:CityModel xmlns:urf="https://www.geospatial.jp/iur/urf/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xAL="urn:oasis:names:tc:ciq:xal:3" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:luse="http://www.opengis.net/citygml/landuse/3.0" xmlns:ct="urn:oasis:names:tc:ciq:ct:3" xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:pcl="http://www.opengis.net/citygml/pointcloud/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:dyn="http://www.opengis.net/citygml/dynamizer/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:vers="http://www.opengis.net/citygml/versioning/3.0" xmlns:brid="http://www.opengis.net/citygml/bridge/3.0" xmlns:dem="http://www.opengis.net/citygml/relief/3.0" xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:app="http://www.opengis.net/citygml/appearance/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:tran="http://www.opengis.net/citygml/transportation/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xsi:schemaLocation="http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd http://www.opengis.net/citygml/bridge/3.0 http://schemas.opengis.net/citygml/bridge/3.0/bridge.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd http://www.opengis.net/citygml/cityobjectgroup/3.0 http://schemas.opengis.net/citygml/cityobjectgroup/3.0/cityObjectGroup.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd http://www.opengis.net/citygml/dynamizer/3.0 http://schemas.opengis.net/citygml/dynamizer/3.0/dynamizer.xsd http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd http://www.opengis.net/citygml/pointcloud/3.0 http://schemas.opengis.net/citygml/pointcloud/3.0/pointCloud.xsd http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd http://www.opengis.net/citygml/versioning/3.0 http://schemas.opengis.net/citygml/versioning/3.0/versioning.xsd http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd https://www.geospatial.jp/iur/urf/4.0 ../../schemas/iur/urf/4.0/urbanFunction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd  http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.662397275099174 139.750079421742 0</gml:lowerCorner>
			<gml:upperCorner>35.66596302730524 139.7587499949588 8.4</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_0cd4b5d2-adea-49b6-ba38-1619f9431106">
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
					<urc:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</urc:lod1HeightType>
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
				<con:GroundSurface gml:id="surface-feb9ffd9-9534-434b-8397-f0e1f7334f57">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-feb9ffd9-9534-434b-8397-f0e1f7334f57">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-feb9ffd9-9534-434b-8397-f0e1f7334f57">
											<gml:posList>35.66380432470524 139.7517406398241 4.82 35.66379072843751 139.7517354071144 4.82 35.66380084213435 139.75169592072814 4.82 35.66381443840376 139.7517011534318 4.82 35.66380432470524 139.7517406398241 4.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-9daa501f-edb3-432d-bf17-0a663b458fdd">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-9daa501f-edb3-432d-bf17-0a663b458fdd">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-9daa501f-edb3-432d-bf17-0a663b458fdd">
											<gml:posList>35.66379072843751 139.7517354071144 6.8 35.66380432470524 139.7517406398241 6.8 35.66381443840376 139.7517011534318 6.8 35.66380084213435 139.75169592072814 6.8 35.66379072843751 139.7517354071144 6.8</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-cd2ac1b9-1c5d-4855-8384-bdcfadfcadd7">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-cd2ac1b9-1c5d-4855-8384-bdcfadfcadd7">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-cd2ac1b9-1c5d-4855-8384-bdcfadfcadd7">
											<gml:posList>35.66380084213435 139.75169592072814 4.82 35.66379072843751 139.7517354071144 4.82 35.66379072843751 139.7517354071144 6.8 35.66380084213435 139.75169592072814 6.8 35.66380084213435 139.75169592072814 4.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-d945ddf6-266c-4630-bfb0-80cc229b3b2c">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-d945ddf6-266c-4630-bfb0-80cc229b3b2c">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-d945ddf6-266c-4630-bfb0-80cc229b3b2c">
											<gml:posList>35.66380432470524 139.7517406398241 4.82 35.66381443840376 139.7517011534318 4.82 35.66381443840376 139.7517011534318 6.8 35.66380432470524 139.7517406398241 6.8 35.66380432470524 139.7517406398241 4.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-2a60820d-d5d0-4234-9d93-cc6de8a40c0a">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-2a60820d-d5d0-4234-9d93-cc6de8a40c0a">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-2a60820d-d5d0-4234-9d93-cc6de8a40c0a">
											<gml:posList>35.66379072843751 139.7517354071144 4.82 35.66380432470524 139.7517406398241 4.82 35.66380432470524 139.7517406398241 6.8 35.66379072843751 139.7517354071144 6.8 35.66379072843751 139.7517354071144 4.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-45674a9d-18d6-4558-ad35-5adfe2ddf248">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-45674a9d-18d6-4558-ad35-5adfe2ddf248">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-45674a9d-18d6-4558-ad35-5adfe2ddf248">
											<gml:posList>35.66381443840376 139.7517011534318 4.82 35.66380084213435 139.75169592072814 4.82 35.66380084213435 139.75169592072814 6.8 35.66381443840376 139.7517011534318 6.8 35.66381443840376 139.7517011534318 4.82</gml:posList>
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
									<gml:posList>35.66380432470523 139.7517406398241 0 35.66381443840376 139.75170115343178 0 35.66380084213435 139.75169592072814 0 35.66379072843751 139.7517354071144 0 35.66380432470523 139.7517406398241 0</gml:posList>
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
											<gml:posList>35.66380432470523 139.7517406398241 4.8 35.66379072843751 139.7517354071144 4.8 35.66380084213435 139.75169592072814 4.8 35.66381443840376 139.75170115343178 4.8 35.66380432470523 139.7517406398241 4.8</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66380432470523 139.7517406398241 4.8 35.66381443840376 139.75170115343178 4.8 35.66381443840376 139.75170115343178 7.8 35.66380432470523 139.7517406398241 7.8 35.66380432470523 139.7517406398241 4.8</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66381443840376 139.75170115343178 4.8 35.66380084213435 139.75169592072814 4.8 35.66380084213435 139.75169592072814 7.8 35.66381443840376 139.75170115343178 7.8 35.66381443840376 139.75170115343178 4.8</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66380084213435 139.75169592072814 4.8 35.66379072843751 139.7517354071144 4.8 35.66379072843751 139.7517354071144 7.8 35.66380084213435 139.75169592072814 7.8 35.66380084213435 139.75169592072814 4.8</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66379072843751 139.7517354071144 4.8 35.66380432470523 139.7517406398241 4.8 35.66380432470523 139.7517406398241 7.8 35.66379072843751 139.7517354071144 7.8 35.66379072843751 139.7517354071144 4.8</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66380432470523 139.7517406398241 7.8 35.66381443840376 139.75170115343178 7.8 35.66380084213435 139.75169592072814 7.8 35.66379072843751 139.7517354071144 7.8 35.66380432470523 139.7517406398241 7.8</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-feb9ffd9-9534-434b-8397-f0e1f7334f57"/>
							<gml:surfaceMember xlink:href="#poly-9daa501f-edb3-432d-bf17-0a663b458fdd"/>
							<gml:surfaceMember xlink:href="#poly-cd2ac1b9-1c5d-4855-8384-bdcfadfcadd7"/>
							<gml:surfaceMember xlink:href="#poly-45674a9d-18d6-4558-ad35-5adfe2ddf248"/>
							<gml:surfaceMember xlink:href="#poly-2a60820d-d5d0-4234-9d93-cc6de8a40c0a"/>
							<gml:surfaceMember xlink:href="#poly-d945ddf6-266c-4630-bfb0-80cc229b3b2c"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">-9999</con:value>
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
					<uro:buildingID>13103-bldg-5940</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_3635e238-46d9-43d3-a189-abe6a02f22b5">
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
					<urc:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</urc:lod1HeightType>
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
				<con:GroundSurface gml:id="surface-f13e951c-6652-4dc6-b897-83e011c35d93">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-f13e951c-6652-4dc6-b897-83e011c35d93">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-f13e951c-6652-4dc6-b897-83e011c35d93">
											<gml:posList>35.66596189172181 139.7587059867453 3.645 35.66596302730524 139.75874939510322 999 35.66594750601866 139.7587499949588 3.645 35.66594637043544 139.75870658660924 3.645 35.66596189172181 139.7587059867453 3.645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-3c902585-306f-4313-b59a-6975e841f123">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-3c902585-306f-4313-b59a-6975e841f123">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-3c902585-306f-4313-b59a-6975e841f123">
											<gml:posList>35.66596302730524 139.75874939510322 6.5 35.66596189172181 139.7587059867453 6.5 35.66594637043544 139.75870658660924 6.5 35.66594750601866 139.7587499949588 6.5 35.66596302730524 139.75874939510322 6.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-dc697fa8-e61e-466f-bc3c-501c47269821">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-dc697fa8-e61e-466f-bc3c-501c47269821">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-dc697fa8-e61e-466f-bc3c-501c47269821">
											<gml:posList>35.66596302730524 139.75874939510322 6.5 35.66594750601866 139.7587499949588 6.5 35.66594750601866 139.7587499949588 3.645 35.66596302730524 139.75874939510322 3.645 35.66596302730524 139.75874939510322 6.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-895f87cb-b31d-43f5-94e7-4b8c9304ddf7">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-895f87cb-b31d-43f5-94e7-4b8c9304ddf7">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-895f87cb-b31d-43f5-94e7-4b8c9304ddf7">
											<gml:posList>35.66596189172181 139.7587059867453 6.5 35.66596302730524 139.75874939510322 6.5 35.66596302730524 139.75874939510322 3.645 35.66596189172181 139.7587059867453 3.645 35.66596189172181 139.7587059867453 6.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-2f8a4097-3bf5-421e-beab-496ab57758f3">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-2f8a4097-3bf5-421e-beab-496ab57758f3">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-2f8a4097-3bf5-421e-beab-496ab57758f3">
											<gml:posList>35.66594750601866 139.7587499949588 6.5 35.66594637043544 139.75870658660924 6.5 35.66594637043544 139.75870658660924 3.645 35.66594750601866 139.7587499949588 3.645 35.66594750601866 139.7587499949588 6.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-c9e0cb77-540a-4fab-b26f-2730df69564e">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-c9e0cb77-540a-4fab-b26f-2730df69564e">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-c9e0cb77-540a-4fab-b26f-2730df69564e">
											<gml:posList>35.66594637043544 139.75870658660924 6.5 35.66596189172181 139.7587059867453 6.5 35.66596189172181 139.7587059867453 3.645 35.66594637043544 139.75870658660924 3.645 35.66594637043544 139.75870658660924 6.5</gml:posList>
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
									<gml:posList>35.6659618917218 139.7587059867453 0 35.66594637043545 139.75870658660924 0 35.66594750601866 139.7587499949588 0 35.66596302730523 139.75874939510322 0 35.6659618917218 139.7587059867453 0</gml:posList>
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
											<gml:posList>35.6659618917218 139.7587059867453 3.6 35.66596302730523 139.75874939510322 3.6 35.66594750601866 139.7587499949588 3.6 35.66594637043545 139.75870658660924 3.6 35.6659618917218 139.7587059867453 3.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6659618917218 139.7587059867453 3.6 35.66594637043545 139.75870658660924 3.6 35.66594637043545 139.75870658660924 6.6 35.6659618917218 139.7587059867453 6.6 35.6659618917218 139.7587059867453 3.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66594637043545 139.75870658660924 3.6 35.66594750601866 139.7587499949588 3.6 35.66594750601866 139.7587499949588 6.6 35.66594637043545 139.75870658660924 6.6 35.66594637043545 139.75870658660924 3.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66594750601866 139.7587499949588 3.6 35.66596302730523 139.75874939510322 3.6 35.66596302730523 139.75874939510322 6.6 35.66594750601866 139.7587499949588 6.6 35.66594750601866 139.7587499949588 3.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66596302730523 139.75874939510322 3.6 35.6659618917218 139.7587059867453 3.6 35.6659618917218 139.7587059867453 6.6 35.66596302730523 139.75874939510322 6.6 35.66596302730523 139.75874939510322 3.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6659618917218 139.7587059867453 6.6 35.66594637043545 139.75870658660924 6.6 35.66594750601866 139.7587499949588 6.6 35.66596302730523 139.75874939510322 6.6 35.6659618917218 139.7587059867453 6.6</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-f13e951c-6652-4dc6-b897-83e011c35d93"/>
							<gml:surfaceMember xlink:href="#poly-3c902585-306f-4313-b59a-6975e841f123"/>
							<gml:surfaceMember xlink:href="#poly-c9e0cb77-540a-4fab-b26f-2730df69564e"/>
							<gml:surfaceMember xlink:href="#poly-2f8a4097-3bf5-421e-beab-496ab57758f3"/>
							<gml:surfaceMember xlink:href="#poly-dc697fa8-e61e-466f-bc3c-501c47269821"/>
							<gml:surfaceMember xlink:href="#poly-895f87cb-b31d-43f5-94e7-4b8c9304ddf7"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">-9999</con:value>
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
					<uro:buildingID>13103-bldg-2534</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_aabdf030-7e6e-401b-b12d-ac52821c1e83">
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
				<con:GroundSurface gml:id="surface-24496e9e-5ecf-47ce-b594-89585a7b0bb4">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-24496e9e-5ecf-47ce-b594-89585a7b0bb4">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-24496e9e-5ecf-47ce-b594-89585a7b0bb4">
											<gml:posList>35.662397275099174 139.75011402564965 4.97 35.6624073555876 139.750079421742 NaN 35.66242458606932 139.75008696979333 4.97 35.66241450557114 139.75012156266234 4.97 35.662397275099174 139.75011402564965 4.97</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-be9e99a4-7e21-4800-8163-b475c9be7f8f">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-be9e99a4-7e21-4800-8163-b475c9be7f8f">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-be9e99a4-7e21-4800-8163-b475c9be7f8f">
											<gml:posList>35.6624073555876 139.750079421742 8.296 35.662397275099174 139.75011402564965 8.296 35.66241450557114 139.75012156266234 8.296 35.66242458606932 139.75008696979333 8.296 35.6624073555876 139.750079421742 8.296</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-152d1ee4-714a-42f2-83c2-9732f9376aec">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-152d1ee4-714a-42f2-83c2-9732f9376aec">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-152d1ee4-714a-42f2-83c2-9732f9376aec">
											<gml:posList>35.662397275099174 139.75011402564965 4.97 35.66241450557114 139.75012156266234 4.97 35.66241450557114 139.75012156266234 8.296 35.662397275099174 139.75011402564965 8.296 35.662397275099174 139.75011402564965 4.97</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-2e883165-21a3-4311-9f6d-770b6bee5246">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-2e883165-21a3-4311-9f6d-770b6bee5246">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-2e883165-21a3-4311-9f6d-770b6bee5246">
											<gml:posList>35.66241450557114 139.75012156266234 4.97 35.66242458606932 139.75008696979333 4.97 35.66242458606932 139.75008696979333 8.296 35.66241450557114 139.75012156266234 8.296 35.66241450557114 139.75012156266234 4.97</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-f4183dc9-6eca-4164-b343-f2f8471504d4">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-f4183dc9-6eca-4164-b343-f2f8471504d4">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-f4183dc9-6eca-4164-b343-f2f8471504d4">
											<gml:posList>35.66242458606932 139.75008696979333 4.97 35.6624073555876 139.750079421742 4.97 35.6624073555876 139.750079421742 8.296 35.66242458606932 139.75008696979333 8.296 35.66242458606932 139.75008696979333 4.97</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-c3162834-2a70-495c-ab08-62d1a46d9efb">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-c3162834-2a70-495c-ab08-62d1a46d9efb">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-c3162834-2a70-495c-ab08-62d1a46d9efb">
											<gml:posList>35.6624073555876 139.750079421742 4.97 35.662397275099174 139.75011402564965 4.97 35.662397275099174 139.75011402564965 8.296 35.6624073555876 139.750079421742 8.296 35.6624073555876 139.750079421742 4.97</gml:posList>
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
									<gml:posList>35.662397275099174 139.75011402564965 0 35.66241450557114 139.75012156266234 0 35.66242458606932 139.75008696979333 0 35.662407355587604 139.750079421742 0 35.662397275099174 139.75011402564965 0</gml:posList>
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
											<gml:posList>35.662397275099174 139.75011402564965 5 35.662407355587604 139.750079421742 5 35.66242458606932 139.75008696979333 5 35.66241450557114 139.75012156266234 5 35.662397275099174 139.75011402564965 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.662397275099174 139.75011402564965 5 35.66241450557114 139.75012156266234 5 35.66241450557114 139.75012156266234 8.4 35.662397275099174 139.75011402564965 8.4 35.662397275099174 139.75011402564965 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66241450557114 139.75012156266234 5 35.66242458606932 139.75008696979333 5 35.66242458606932 139.75008696979333 8.4 35.66241450557114 139.75012156266234 8.4 35.66241450557114 139.75012156266234 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66242458606932 139.75008696979333 5 35.662407355587604 139.750079421742 5 35.662407355587604 139.750079421742 8.4 35.66242458606932 139.75008696979333 8.4 35.66242458606932 139.75008696979333 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.662407355587604 139.750079421742 5 35.662397275099174 139.75011402564965 5 35.662397275099174 139.75011402564965 8.4 35.662407355587604 139.750079421742 8.4 35.662407355587604 139.750079421742 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.662397275099174 139.75011402564965 8.4 35.66241450557114 139.75012156266234 8.4 35.66242458606932 139.75008696979333 8.4 35.662407355587604 139.750079421742 8.4 35.662397275099174 139.75011402564965 8.4</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-24496e9e-5ecf-47ce-b594-89585a7b0bb4"/>
							<gml:surfaceMember xlink:href="#poly-be9e99a4-7e21-4800-8163-b475c9be7f8f"/>
							<gml:surfaceMember xlink:href="#poly-f4183dc9-6eca-4164-b343-f2f8471504d4"/>
							<gml:surfaceMember xlink:href="#poly-c3162834-2a70-495c-ab08-62d1a46d9efb"/>
							<gml:surfaceMember xlink:href="#poly-2e883165-21a3-4311-9f6d-770b6bee5246"/>
							<gml:surfaceMember xlink:href="#poly-152d1ee4-714a-42f2-83c2-9732f9376aec"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">3.4</con:value>
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
					<uro:specifiedFloorAreaRate>500</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-11518</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
