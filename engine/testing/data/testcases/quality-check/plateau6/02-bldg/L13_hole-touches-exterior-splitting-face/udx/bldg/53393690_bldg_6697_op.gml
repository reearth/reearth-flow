<?xml version="1.0" encoding="UTF-8"?>
<!-- L13: a hole that touches the exterior at two points and splits the face in two.
     Two buildings cut out of the 13997 Tokyo sample (v3) 53393690_bldg_6697_op.gml.
     bldg_002b2ae9-d6ee-4810-9e15-c8d23215d0f4 has a hole added inside its LOD0 face and
     has no errors.
     bldg_9cafd036-8dbb-476d-88a0-8e7452944e4e has a hole added to its LOD0 face and to
     its GroundSurface that runs between the two ends of the first exterior edge, using
     the same coordinates, so the face is left as two parts joined only at those points.
     The hole in the GroundSurface leaves the LOD2 solid not closed. -->
<core:CityModel xmlns:urf="https://www.geospatial.jp/iur/urf/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xAL="urn:oasis:names:tc:ciq:xal:3" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:luse="http://www.opengis.net/citygml/landuse/3.0" xmlns:ct="urn:oasis:names:tc:ciq:ct:3" xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:pcl="http://www.opengis.net/citygml/pointcloud/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:dyn="http://www.opengis.net/citygml/dynamizer/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:vers="http://www.opengis.net/citygml/versioning/3.0" xmlns:brid="http://www.opengis.net/citygml/bridge/3.0" xmlns:dem="http://www.opengis.net/citygml/relief/3.0" xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:app="http://www.opengis.net/citygml/appearance/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:tran="http://www.opengis.net/citygml/transportation/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xsi:schemaLocation="http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd http://www.opengis.net/citygml/bridge/3.0 http://schemas.opengis.net/citygml/bridge/3.0/bridge.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd http://www.opengis.net/citygml/cityobjectgroup/3.0 http://schemas.opengis.net/citygml/cityobjectgroup/3.0/cityObjectGroup.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd http://www.opengis.net/citygml/dynamizer/3.0 http://schemas.opengis.net/citygml/dynamizer/3.0/dynamizer.xsd http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd http://www.opengis.net/citygml/pointcloud/3.0 http://schemas.opengis.net/citygml/pointcloud/3.0/pointCloud.xsd http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd http://www.opengis.net/citygml/versioning/3.0 http://schemas.opengis.net/citygml/versioning/3.0/versioning.xsd http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd https://www.geospatial.jp/iur/urf/4.0 ../../schemas/iur/urf/4.0/urbanFunction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd  http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.65898204183953 139.75205284312733 0.0</gml:lowerCorner>
			<gml:upperCorner>35.66035316576649 139.75465324315746 26.4</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_9cafd036-8dbb-476d-88a0-8e7452944e4e">
			<core:creationDate>2024-03-15T00:00:00Z</core:creationDate>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>延べ面積換算係数</gen:name>
					<gen:value>1</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>説明注記</gen:name>
					<gen:value> </gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>大字・町コード</gen:name>
					<gen:value>12</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>町・丁目コード</gen:name>
					<gen:value>1</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>13+区市町村コード+大字・町コード+町・丁目コード</gen:name>
					<gen:value>13103012001</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>建物構造コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">11</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>延べ面積換算コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">1</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>変化フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">2</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>過年度修正フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">0</gen:value>
				</gen:CodeAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>調整フラグ</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">0</gen:value>
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
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">14</urc:description>
					<urc:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:depth uom="m">0.259</urc:depth>
					<urc:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">34</urc:description>
					<urc:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:depth uom="m">0.26</urc:depth>
					<urc:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../codelists/HighTideRiskAttribute_rank.xml">1</urc:rank>
					<urc:depth uom="m">0.44</urc:depth>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
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
				<con:GroundSurface gml:id="surface-e9da001d-bfcb-46bf-b60b-54cfd02da504">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-e9da001d-bfcb-46bf-b60b-54cfd02da504">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-e9da001d-bfcb-46bf-b60b-54cfd02da504">
											<gml:posList>35.66033230639801 139.75205284312733 2.94 35.66035316576649 139.75205515245844 2.94 35.66032487140601 139.75213501349094 2.94 35.66033230639801 139.75205284312733 2.94</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
									<gml:interior>
										<gml:LinearRing>
											<gml:posList>35.66033230639801 139.75205284312733 2.94 35.660337676148600 139.752086635005213 2.94 35.66035316576649 139.75205515245844 2.94 35.660341992583049 139.752062214829238 2.94 35.66033230639801 139.75205284312733 2.94</gml:posList>
										</gml:LinearRing>
									</gml:interior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-4936712e-6eac-4276-94d6-f3468673da60">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-4936712e-6eac-4276-94d6-f3468673da60">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-4936712e-6eac-4276-94d6-f3468673da60">
											<gml:posList>35.66032487140601 139.75213501349094 8.7 35.66035316576649 139.75205515245844 8.7 35.66033230639801 139.75205284312733 8.7 35.66032487140601 139.75213501349094 8.7</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-389d0853-e2be-483e-b07b-3d434bea3d7b">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-389d0853-e2be-483e-b07b-3d434bea3d7b">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-389d0853-e2be-483e-b07b-3d434bea3d7b">
											<gml:posList>35.66032487140601 139.75213501349094 8.7 35.66033230639801 139.75205284312733 8.7 35.66033230639801 139.75205284312733 2.94 35.66032487140601 139.75213501349094 2.94 35.66032487140601 139.75213501349094 8.7</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-03105de1-6341-4c3a-87bf-32916ff17d77">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-03105de1-6341-4c3a-87bf-32916ff17d77">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-03105de1-6341-4c3a-87bf-32916ff17d77">
											<gml:posList>35.66033230639801 139.75205284312733 8.7 35.66035316576649 139.75205515245844 8.7 35.66035316576649 139.75205515245844 2.94 35.66033230639801 139.75205284312733 2.94 35.66033230639801 139.75205284312733 8.7</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-481244ab-2c34-43e3-936a-97da678e2394">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-481244ab-2c34-43e3-936a-97da678e2394">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-481244ab-2c34-43e3-936a-97da678e2394">
											<gml:posList>35.66035316576649 139.75205515245844 8.7 35.66032487140601 139.75213501349094 8.7 35.66032487140601 139.75213501349094 2.94 35.66035316576649 139.75205515245844 2.94 35.66035316576649 139.75205515245844 8.7</gml:posList>
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
									<gml:posList>35.66033230639801 139.75205284312733 0 35.660324871406004 139.75213501349094 0 35.66035316576647 139.75205515245844 0 35.66033230639801 139.75205284312733 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>35.66033230639801 139.75205284312733 0 35.660337676148593 139.752086635005213 0 35.660324871406004 139.75213501349094 0 35.660330674838853 139.752094159242233 0 35.66033230639801 139.75205284312733 0</gml:posList>
								</gml:LinearRing>
							</gml:interior>
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
											<gml:posList>35.66033230639801 139.75205284312733 2.9 35.66035316576647 139.75205515245844 2.9 35.660324871406004 139.75213501349094 2.9 35.66033230639801 139.75205284312733 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66033230639801 139.75205284312733 2.9 35.660324871406004 139.75213501349094 2.9 35.660324871406004 139.75213501349094 8.2 35.66033230639801 139.75205284312733 8.2 35.66033230639801 139.75205284312733 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.660324871406004 139.75213501349094 2.9 35.66035316576647 139.75205515245844 2.9 35.66035316576647 139.75205515245844 8.2 35.660324871406004 139.75213501349094 8.2 35.660324871406004 139.75213501349094 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66035316576647 139.75205515245844 2.9 35.66033230639801 139.75205284312733 2.9 35.66033230639801 139.75205284312733 8.2 35.66035316576647 139.75205515245844 8.2 35.66035316576647 139.75205515245844 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66033230639801 139.75205284312733 8.2 35.660324871406004 139.75213501349094 8.2 35.66035316576647 139.75205515245844 8.2 35.66033230639801 139.75205284312733 8.2</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-e9da001d-bfcb-46bf-b60b-54cfd02da504"/>
							<gml:surfaceMember xlink:href="#poly-4936712e-6eac-4276-94d6-f3468673da60"/>
							<gml:surfaceMember xlink:href="#poly-481244ab-2c34-43e3-936a-97da678e2394"/>
							<gml:surfaceMember xlink:href="#poly-03105de1-6341-4c3a-87bf-32916ff17d77"/>
							<gml:surfaceMember xlink:href="#poly-389d0853-e2be-483e-b07b-3d434bea3d7b"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">5.3</con:value>
				</con:Height>
			</con:height>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3002</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">401</bldg:usage>
			<bldg:storeysAboveGround>22</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>2</bldg:storeysBelowGround>
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:Address>
							<xAL:Country>
								<xAL:NameElement xAL:NameType="Name">日本</xAL:NameElement>
							</xAL:Country>
							<xAL:Locality>
								<xAL:NameElement xAL:NameType="Name">東京都港区芝公園一丁目</xAL:NameElement>
							</xAL:Locality>
						</xAL:Address>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">1731.12175</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1001</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:areaClassificationType codeSpace="../../codelists/Common_areaClassificationType.xml">22</uro:areaClassificationType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">10</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">212</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1210</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>600</uro:specifiedFloorAreaRate>
					<uro:surveyYear>2021-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-457</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_002b2ae9-d6ee-4810-9e15-c8d23215d0f4">
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
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">10</urc:description>
					<urc:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:depth uom="m">0.44</urc:depth>
					<urc:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">14</urc:description>
					<urc:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">2</urc:rank>
					<urc:depth uom="m">1.244</urc:depth>
					<urc:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">34</urc:description>
					<urc:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">2</urc:rank>
					<urc:depth uom="m">1.24</urc:depth>
					<urc:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../codelists/HighTideRiskAttribute_rank.xml">2</urc:rank>
					<urc:depth uom="m">0.74</urc:depth>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
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
				<con:GroundSurface gml:id="surface-f3c148bf-7bcf-446b-90df-e1a1ac990e85">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-f3c148bf-7bcf-446b-90df-e1a1ac990e85">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-f3c148bf-7bcf-446b-90df-e1a1ac990e85">
											<gml:posList>35.65899827710627 139.75465324315746 2.652 35.658982041839536 139.75465093969774 2.652 35.658985174845355 139.75461757061206 2.652 35.65900164448447 139.75461989592452 2.652 35.65899827710627 139.75465324315746 2.652</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-8a474c0b-61fa-4624-ad24-59ac8036928d">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-8a474c0b-61fa-4624-ad24-59ac8036928d">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-8a474c0b-61fa-4624-ad24-59ac8036928d">
											<gml:posList>35.658982041839536 139.75465093969774 6.313 35.65899827710627 139.75465324315746 6.313 35.65900164448447 139.75461989592452 6.313 35.658985174845355 139.75461757061206 6.313 35.658982041839536 139.75465093969774 6.313</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-96645067-74b6-4468-91fb-512da8ad5a65">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-96645067-74b6-4468-91fb-512da8ad5a65">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-96645067-74b6-4468-91fb-512da8ad5a65">
											<gml:posList>35.65899827710627 139.75465324315746 6.313 35.658982041839536 139.75465093969774 6.313 35.658982041839536 139.75465093969774 2.652 35.65899827710627 139.75465324315746 2.652 35.65899827710627 139.75465324315746 6.313</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-fcf04278-7def-4f19-9568-811b00b72860">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-fcf04278-7def-4f19-9568-811b00b72860">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-fcf04278-7def-4f19-9568-811b00b72860">
											<gml:posList>35.658985174845355 139.75461757061206 6.313 35.65900164448447 139.75461989592452 6.313 35.65900164448447 139.75461989592452 2.652 35.658985174845355 139.75461757061206 2.652 35.658985174845355 139.75461757061206 6.313</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-2b045129-5bca-4672-8d4d-3585c0c2bb8e">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-2b045129-5bca-4672-8d4d-3585c0c2bb8e">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-2b045129-5bca-4672-8d4d-3585c0c2bb8e">
											<gml:posList>35.65900164448447 139.75461989592452 6.313 35.65899827710627 139.75465324315746 6.313 35.65899827710627 139.75465324315746 2.652 35.65900164448447 139.75461989592452 2.652 35.65900164448447 139.75461989592452 6.313</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-41d98551-365e-4611-8371-dfe26e3cf0ea">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-41d98551-365e-4611-8371-dfe26e3cf0ea">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-41d98551-365e-4611-8371-dfe26e3cf0ea">
											<gml:posList>35.658982041839536 139.75465093969774 6.313 35.658985174845355 139.75461757061206 6.313 35.658985174845355 139.75461757061206 2.652 35.658982041839536 139.75465093969774 2.652 35.658982041839536 139.75465093969774 6.313</gml:posList>
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
									<gml:posList>35.65899827710627 139.75465324315746 0 35.65900164358309 139.7546198959254 0 35.65898517484536 139.75461757061206 0 35.65898204183953 139.75465093969774 0 35.65899827710627 139.75465324315746 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>35.658988861592356 139.754640070553023 0 35.658989801494101 139.754630059827321 0 35.658994742115418 139.754630757421324 0 35.658993732172377 139.754640761590963 0 35.658988861592356 139.754640070553023 0</gml:posList>
								</gml:LinearRing>
							</gml:interior>
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
											<gml:posList>35.65899827710627 139.75465324315746 2.7 35.65898204183953 139.75465093969774 2.7 35.65898517484536 139.75461757061206 2.7 35.65900164358309 139.7546198959254 2.7 35.65899827710627 139.75465324315746 2.7</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.65899827710627 139.75465324315746 2.7 35.65900164358309 139.7546198959254 2.7 35.65900164358309 139.7546198959254 26.4 35.65899827710627 139.75465324315746 26.4 35.65899827710627 139.75465324315746 2.7</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.65900164358309 139.7546198959254 2.7 35.65898517484536 139.75461757061206 2.7 35.65898517484536 139.75461757061206 26.4 35.65900164358309 139.7546198959254 26.4 35.65900164358309 139.7546198959254 2.7</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.65898517484536 139.75461757061206 2.7 35.65898204183953 139.75465093969774 2.7 35.65898204183953 139.75465093969774 26.4 35.65898517484536 139.75461757061206 26.4 35.65898517484536 139.75461757061206 2.7</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.65898204183953 139.75465093969774 2.7 35.65899827710627 139.75465324315746 2.7 35.65899827710627 139.75465324315746 26.4 35.65898204183953 139.75465093969774 26.4 35.65898204183953 139.75465093969774 2.7</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.65899827710627 139.75465324315746 26.4 35.65900164358309 139.7546198959254 26.4 35.65898517484536 139.75461757061206 26.4 35.65898204183953 139.75465093969774 26.4 35.65899827710627 139.75465324315746 26.4</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-f3c148bf-7bcf-446b-90df-e1a1ac990e85"/>
							<gml:surfaceMember xlink:href="#poly-8a474c0b-61fa-4624-ad24-59ac8036928d"/>
							<gml:surfaceMember xlink:href="#poly-41d98551-365e-4611-8371-dfe26e3cf0ea"/>
							<gml:surfaceMember xlink:href="#poly-96645067-74b6-4468-91fb-512da8ad5a65"/>
							<gml:surfaceMember xlink:href="#poly-fcf04278-7def-4f19-9568-811b00b72860"/>
							<gml:surfaceMember xlink:href="#poly-2b045129-5bca-4672-8d4d-3585c0c2bb8e"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">23.7</con:value>
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
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">212</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>700</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-1473</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
