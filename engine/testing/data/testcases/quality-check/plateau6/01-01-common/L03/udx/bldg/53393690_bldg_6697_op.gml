<?xml version="1.0" encoding="UTF-8"?>
<!-- L03: one building cut out of 53393690_bldg_6697_op.gml of the 13997 sample package,
     with five uses the objectlist does not allow:
     - a veg:PlantCover member, which the bldg package does not accept;
     - core:adeOfAbstractCityObject/uro:FacilityIdAttribute, not marked 作成対象;
     - a gen:GenericThematicSurface as core:boundary, a feature type of another package;
     - bldg:buildingPart, not marked 作成対象;
     - gml:name on a con:WallSurface, not marked 作成対象.
     The urf pair is left out of xsi:schemaLocation: no urf element is used. -->
<core:CityModel xmlns:urf="https://www.geospatial.jp/iur/urf/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xAL="urn:oasis:names:tc:ciq:xal:3" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:luse="http://www.opengis.net/citygml/landuse/3.0" xmlns:ct="urn:oasis:names:tc:ciq:ct:3" xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:pcl="http://www.opengis.net/citygml/pointcloud/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:dyn="http://www.opengis.net/citygml/dynamizer/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:vers="http://www.opengis.net/citygml/versioning/3.0" xmlns:brid="http://www.opengis.net/citygml/bridge/3.0" xmlns:dem="http://www.opengis.net/citygml/relief/3.0" xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:app="http://www.opengis.net/citygml/appearance/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:tran="http://www.opengis.net/citygml/transportation/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xsi:schemaLocation="http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd http://www.opengis.net/citygml/bridge/3.0 http://schemas.opengis.net/citygml/bridge/3.0/bridge.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd http://www.opengis.net/citygml/cityobjectgroup/3.0 http://schemas.opengis.net/citygml/cityobjectgroup/3.0/cityObjectGroup.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd http://www.opengis.net/citygml/dynamizer/3.0 http://schemas.opengis.net/citygml/dynamizer/3.0/dynamizer.xsd http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd http://www.opengis.net/citygml/pointcloud/3.0 http://schemas.opengis.net/citygml/pointcloud/3.0/pointCloud.xsd http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd http://www.opengis.net/citygml/versioning/3.0 http://schemas.opengis.net/citygml/versioning/3.0/versioning.xsd http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd  http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.660929102578 139.7594395550122 0</gml:lowerCorner>
			<gml:upperCorner>35.66104355460079 139.75950826924787 7.3</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<veg:PlantCover gml:id="bldg_af4c5b56-166a-4709-8585-39af3ba8d489"/>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_8b50c00e-8549-4e1e-89f9-2b6b82272a87">
			<core:creationDate>2024-03-15T00:00:00Z</core:creationDate>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>再開発等促進区を定める地区計画</gen:name>
					<gen:value>汐留地区</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
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
			<core:adeOfAbstractCityObject>
				<uro:FacilityIdAttribute/>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<gen:GenericThematicSurface gml:id="gen_12c4abfe-c861-407d-b092-e9d988bb2de2"/>
			</core:boundary>
			<core:boundary>
				<con:GroundSurface gml:id="surface-a351da51-99e2-4833-a11b-5e6b669c9f9d">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-a351da51-99e2-4833-a11b-5e6b669c9f9d">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-a351da51-99e2-4833-a11b-5e6b669c9f9d">
											<gml:posList>35.66092910257801 139.75949224425517 3.43 35.66093429823225 139.7594395550122 3.43 35.66104355460079 139.75945546956743 3.43 35.66103844914487 139.75950826924787 3.43 35.66092910257801 139.75949224425517 3.43</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-583afcb9-e826-4084-8267-70fcc2f6211d">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-583afcb9-e826-4084-8267-70fcc2f6211d">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-583afcb9-e826-4084-8267-70fcc2f6211d">
											<gml:posList>35.66093429823225 139.7594395550122 7.2 35.66092910257801 139.75949224425517 7.2 35.66103844914487 139.75950826924787 7.2 35.66104355460079 139.75945546956743 7.2 35.66093429823225 139.7594395550122 7.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-49570ac3-0aa8-4a28-a011-67416d37d682">
					<gml:name>wall</gml:name>
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-49570ac3-0aa8-4a28-a011-67416d37d682">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-49570ac3-0aa8-4a28-a011-67416d37d682">
											<gml:posList>35.66104355460079 139.75945546956743 7.2 35.66103844914487 139.75950826924787 7.2 35.66103844914487 139.75950826924787 3.43 35.66104355460079 139.75945546956743 3.43 35.66104355460079 139.75945546956743 7.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-607f1f71-cb0e-46af-8a6c-64fe0defb3ff">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-607f1f71-cb0e-46af-8a6c-64fe0defb3ff">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-607f1f71-cb0e-46af-8a6c-64fe0defb3ff">
											<gml:posList>35.66092910257801 139.75949224425517 7.2 35.66093429823225 139.7594395550122 7.2 35.66093429823225 139.7594395550122 3.43 35.66092910257801 139.75949224425517 3.43 35.66092910257801 139.75949224425517 7.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-2a777b8c-2ec1-4f84-9c5c-df2217af6fa3">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-2a777b8c-2ec1-4f84-9c5c-df2217af6fa3">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-2a777b8c-2ec1-4f84-9c5c-df2217af6fa3">
											<gml:posList>35.66103844914487 139.75950826924787 7.2 35.66092910257801 139.75949224425517 7.2 35.66092910257801 139.75949224425517 3.43 35.66103844914487 139.75950826924787 3.43 35.66103844914487 139.75950826924787 7.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-c1784802-823f-41a4-bc6e-471d5cee7e6c">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-c1784802-823f-41a4-bc6e-471d5cee7e6c">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-c1784802-823f-41a4-bc6e-471d5cee7e6c">
											<gml:posList>35.66093429823225 139.7594395550122 7.2 35.66104355460079 139.75945546956743 7.2 35.66104355460079 139.75945546956743 3.43 35.66093429823225 139.7594395550122 3.43 35.66093429823225 139.7594395550122 7.2</gml:posList>
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
									<gml:posList>35.660929102578 139.75949224425517 0 35.66103844914487 139.75950826924787 0 35.66104355460079 139.75945546956743 0 35.66093429823224 139.7594395550122 0 35.660929102578 139.75949224425517 0</gml:posList>
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
											<gml:posList>35.660929102578 139.75949224425517 3.4 35.66093429823224 139.7594395550122 3.4 35.66104355460079 139.75945546956743 3.4 35.66103844914487 139.75950826924787 3.4 35.660929102578 139.75949224425517 3.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.660929102578 139.75949224425517 3.4 35.66103844914487 139.75950826924787 3.4 35.66103844914487 139.75950826924787 7.3 35.660929102578 139.75949224425517 7.3 35.660929102578 139.75949224425517 3.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66103844914487 139.75950826924787 3.4 35.66104355460079 139.75945546956743 3.4 35.66104355460079 139.75945546956743 7.3 35.66103844914487 139.75950826924787 7.3 35.66103844914487 139.75950826924787 3.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66104355460079 139.75945546956743 3.4 35.66093429823224 139.7594395550122 3.4 35.66093429823224 139.7594395550122 7.3 35.66104355460079 139.75945546956743 7.3 35.66104355460079 139.75945546956743 3.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66093429823224 139.7594395550122 3.4 35.660929102578 139.75949224425517 3.4 35.660929102578 139.75949224425517 7.3 35.66093429823224 139.7594395550122 7.3 35.66093429823224 139.7594395550122 3.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.660929102578 139.75949224425517 7.3 35.66103844914487 139.75950826924787 7.3 35.66104355460079 139.75945546956743 7.3 35.66093429823224 139.7594395550122 7.3 35.660929102578 139.75949224425517 7.3</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-a351da51-99e2-4833-a11b-5e6b669c9f9d"/>
							<gml:surfaceMember xlink:href="#poly-583afcb9-e826-4084-8267-70fcc2f6211d"/>
							<gml:surfaceMember xlink:href="#poly-607f1f71-cb0e-46af-8a6c-64fe0defb3ff"/>
							<gml:surfaceMember xlink:href="#poly-2a777b8c-2ec1-4f84-9c5c-df2217af6fa3"/>
							<gml:surfaceMember xlink:href="#poly-c1784802-823f-41a4-bc6e-471d5cee7e6c"/>
							<gml:surfaceMember xlink:href="#poly-49570ac3-0aa8-4a28-a011-67416d37d682"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">3.9</con:value>
				</con:Height>
			</con:height>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
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
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>600</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-434</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:buildingPart/>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>


