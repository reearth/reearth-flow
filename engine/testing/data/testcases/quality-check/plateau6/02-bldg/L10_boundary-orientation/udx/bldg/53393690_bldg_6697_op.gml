<?xml version="1.0" encoding="UTF-8"?>
<!-- L10: misoriented polygon boundaries, one error per boundary.

     Each building has a hole in two faces:
       - LOD0 face: judged from above.
       - LOD2 GroundSurface: judged against the face's own normal, so its
         exterior is not judged. Its errors are boundary surface errors.

     Buildings:
       - bldg_997bb0a6-aa5a-4852-ba11-8b9c6b5f2d0f: every boundary wound correctly.
           No error.
       - bldg_435ed9e6-9785-4297-a36a-6a5979cf6fd0: exterior reversed in both faces.
           LOD0: exterior error.
           GroundSurface: no error.
       - bldg_a5d3629d-b115-418f-9068-e100b4bdeac5: hole wound the same way as the
         exterior in both faces.
           LOD0: interior error.
           GroundSurface: interior error.
       - bldg_468427ae-fc81-4d0c-9b18-d0b3ff589159: exterior reversed, and hole wound
         the same way as the reversed exterior, in both faces.
           LOD0: exterior and interior errors.
           GroundSurface: interior error.

     The holes in the GroundSurfaces leave the LOD2 solids not closed. -->

<core:CityModel xmlns:urf="https://www.geospatial.jp/iur/urf/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xAL="urn:oasis:names:tc:ciq:xal:3" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:luse="http://www.opengis.net/citygml/landuse/3.0" xmlns:ct="urn:oasis:names:tc:ciq:ct:3" xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:pcl="http://www.opengis.net/citygml/pointcloud/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:dyn="http://www.opengis.net/citygml/dynamizer/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:vers="http://www.opengis.net/citygml/versioning/3.0" xmlns:brid="http://www.opengis.net/citygml/bridge/3.0" xmlns:dem="http://www.opengis.net/citygml/relief/3.0" xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:app="http://www.opengis.net/citygml/appearance/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:tran="http://www.opengis.net/citygml/transportation/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xsi:schemaLocation="http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd http://www.opengis.net/citygml/bridge/3.0 http://schemas.opengis.net/citygml/bridge/3.0/bridge.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd http://www.opengis.net/citygml/cityobjectgroup/3.0 http://schemas.opengis.net/citygml/cityobjectgroup/3.0/cityObjectGroup.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd http://www.opengis.net/citygml/dynamizer/3.0 http://schemas.opengis.net/citygml/dynamizer/3.0/dynamizer.xsd http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd http://www.opengis.net/citygml/pointcloud/3.0 http://schemas.opengis.net/citygml/pointcloud/3.0/pointCloud.xsd http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd http://www.opengis.net/citygml/versioning/3.0 http://schemas.opengis.net/citygml/versioning/3.0/versioning.xsd http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd https://www.geospatial.jp/iur/urf/4.0 ../../schemas/iur/urf/4.0/urbanFunction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd  http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.66121273763733 139.75028497296097 0</gml:lowerCorner>
			<gml:upperCorner>35.66598585640399 139.7543726851032 9.3</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
		<core:cityObjectMember>
		<bldg:Building gml:id="bldg_997bb0a6-aa5a-4852-ba11-8b9c6b5f2d0f">
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
					<gen:value>倉庫</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>大字・町コード</gen:name>
					<gen:value>20</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>町・丁目コード</gen:name>
					<gen:value>3</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>13+区市町村コード+大字・町コード+町・丁目コード</gen:name>
					<gen:value>13103020003</gen:value>
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
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">0</gen:value>
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
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">3</gen:value>
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
				<con:GroundSurface gml:id="surface-a7d35266-16fa-4de8-84ba-40fde96def94">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-a7d35266-16fa-4de8-84ba-40fde96def94">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-a7d35266-16fa-4de8-84ba-40fde96def94">
											<gml:posList>35.66255358832537 139.75121242833345 4.167 35.66252998417766 139.75120338441064 4.167 35.66253742763903 139.751174184377 4.167 35.66256104079511 139.7511832172376 4.167 35.66255358832537 139.75121242833345 4.167</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
									<gml:interior>
										<gml:LinearRing>
											<gml:posList>35.662547933661621 139.751199041012796 4.167 35.662550169402543 139.751190277684060 4.167 35.662543085455717 139.751187567825866 4.167 35.662540852417308 139.751196327835970 4.167 35.662547933661621 139.751199041012796 4.167</gml:posList>
										</gml:LinearRing>
									</gml:interior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-97332978-240a-4d7b-be03-3b2e0408b702">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-97332978-240a-4d7b-be03-3b2e0408b702">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-97332978-240a-4d7b-be03-3b2e0408b702">
											<gml:posList>35.66252998417766 139.75120338441064 7.163 35.66255358832537 139.75121242833345 7.163 35.66256104079511 139.7511832172376 7.163 35.66253742763903 139.751174184377 7.163 35.66252998417766 139.75120338441064 7.163</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-1d490dd3-9e97-478a-b224-dc5dbd1f9b17">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-1d490dd3-9e97-478a-b224-dc5dbd1f9b17">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-1d490dd3-9e97-478a-b224-dc5dbd1f9b17">
											<gml:posList>35.66255358832537 139.75121242833345 4.167 35.66256104079511 139.7511832172376 4.167 35.66256104079511 139.7511832172376 7.163 35.66255358832537 139.75121242833345 7.163 35.66255358832537 139.75121242833345 4.167</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-e67209d4-1e5c-4ac8-a929-6a963fe48fac">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-e67209d4-1e5c-4ac8-a929-6a963fe48fac">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-e67209d4-1e5c-4ac8-a929-6a963fe48fac">
											<gml:posList>35.66256104079511 139.7511832172376 4.167 35.66253742763903 139.751174184377 4.167 35.66253742763903 139.751174184377 7.163 35.66256104079511 139.7511832172376 7.163 35.66256104079511 139.7511832172376 4.167</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-66716eb7-4a6c-4c51-bc22-b79f57dcefa1">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-66716eb7-4a6c-4c51-bc22-b79f57dcefa1">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-66716eb7-4a6c-4c51-bc22-b79f57dcefa1">
											<gml:posList>35.66253742763903 139.751174184377 4.167 35.66252998417766 139.75120338441064 4.167 35.66252998417766 139.75120338441064 7.163 35.66253742763903 139.751174184377 7.163 35.66253742763903 139.751174184377 4.167</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-60c5bb6d-7aba-432d-acf6-ce5fe7bed7c7">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-60c5bb6d-7aba-432d-acf6-ce5fe7bed7c7">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-60c5bb6d-7aba-432d-acf6-ce5fe7bed7c7">
											<gml:posList>35.66252998417766 139.75120338441064 4.167 35.66255358832537 139.75121242833345 4.167 35.66255358832537 139.75121242833345 7.163 35.66252998417766 139.75120338441064 7.163 35.66252998417766 139.75120338441064 4.167</gml:posList>
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
									<gml:posList>35.66255358832537 139.75121242833345 0 35.66256104079512 139.75118321723764 0 35.66253742763904 139.751174184377 0 35.66252998417766 139.75120338441064 0 35.66255358832537 139.75121242833345 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>35.662547933661621 139.751199041012796 0 35.662540852417308 139.751196327835970 0 35.662543085455717 139.751187567825866 0 35.662550169402543 139.751190277684060 0 35.662547933661621 139.751199041012796 0</gml:posList>
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
											<gml:posList>35.66255358832537 139.75121242833345 4.2 35.66252998417766 139.75120338441064 4.2 35.66253742763904 139.751174184377 4.2 35.66256104079512 139.75118321723764 4.2 35.66255358832537 139.75121242833345 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66255358832537 139.75121242833345 4.2 35.66256104079512 139.75118321723764 4.2 35.66256104079512 139.75118321723764 7.2 35.66255358832537 139.75121242833345 7.2 35.66255358832537 139.75121242833345 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66256104079512 139.75118321723764 4.2 35.66253742763904 139.751174184377 4.2 35.66253742763904 139.751174184377 7.2 35.66256104079512 139.75118321723764 7.2 35.66256104079512 139.75118321723764 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66253742763904 139.751174184377 4.2 35.66252998417766 139.75120338441064 4.2 35.66252998417766 139.75120338441064 7.2 35.66253742763904 139.751174184377 7.2 35.66253742763904 139.751174184377 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66252998417766 139.75120338441064 4.2 35.66255358832537 139.75121242833345 4.2 35.66255358832537 139.75121242833345 7.2 35.66252998417766 139.75120338441064 7.2 35.66252998417766 139.75120338441064 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66255358832537 139.75121242833345 7.2 35.66256104079512 139.75118321723764 7.2 35.66253742763904 139.751174184377 7.2 35.66252998417766 139.75120338441064 7.2 35.66255358832537 139.75121242833345 7.2</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-a7d35266-16fa-4de8-84ba-40fde96def94"/>
							<gml:surfaceMember xlink:href="#poly-97332978-240a-4d7b-be03-3b2e0408b702"/>
							<gml:surfaceMember xlink:href="#poly-e67209d4-1e5c-4ac8-a929-6a963fe48fac"/>
							<gml:surfaceMember xlink:href="#poly-1d490dd3-9e97-478a-b224-dc5dbd1f9b17"/>
							<gml:surfaceMember xlink:href="#poly-66716eb7-4a6c-4c51-bc22-b79f57dcefa1"/>
							<gml:surfaceMember xlink:href="#poly-60c5bb6d-7aba-432d-acf6-ce5fe7bed7c7"/>
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
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">422</bldg:usage>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:Address>
							<xAL:Country>
								<xAL:NameElement xAL:NameType="Name">日本</xAL:NameElement>
							</xAL:Country>
							<xAL:Locality>
								<xAL:NameElement xAL:NameType="Name">東京都港区西新橋三丁目</xAL:NameElement>
							</xAL:Locality>
						</xAL:Address>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">7.6953</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1001</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:areaClassificationType codeSpace="../../codelists/Common_areaClassificationType.xml">22</uro:areaClassificationType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">10</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1131</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>500</uro:specifiedFloorAreaRate>
					<uro:surveyYear>2021-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-191</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_435ed9e6-9785-4297-a36a-6a5979cf6fd0">
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
					<gen:value>20</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>町・丁目コード</gen:name>
					<gen:value>3</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>13+区市町村コード+大字・町コード+町・丁目コード</gen:name>
					<gen:value>13103020003</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>建物構造コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">12</gen:value>
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
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">0</gen:value>
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
					<urc:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">10</urc:description>
					<urc:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:depth uom="m">0.01</urc:depth>
					<urc:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
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
				<con:GroundSurface gml:id="surface-60e29cbb-b090-4c90-9b30-dd5e69c29777">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-60e29cbb-b090-4c90-9b30-dd5e69c29777">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-60e29cbb-b090-4c90-9b30-dd5e69c29777">
											<gml:posList>35.664565912670632 139.751725815143146 4.22 35.664618025076578 139.751744981224789 4.22 35.664631418559097 139.751690402983655 4.22 35.664579216007276 139.751671237026102 4.22 35.664565912670632 139.751725815143146 4.22</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
									<gml:interior>
										<gml:LinearRing>
											<gml:posList>35.664588823956066 139.751713420909027 4.22 35.664592814957061 139.751697047473925 4.22 35.664608475722602 139.751702797261174 4.22 35.664604457677846 139.751719170733509 4.22 35.664588823956066 139.751713420909027 4.22</gml:posList>
										</gml:LinearRing>
									</gml:interior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-461f357c-1d0a-4af2-823f-427d13d76fa6">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-461f357c-1d0a-4af2-823f-427d13d76fa6">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-461f357c-1d0a-4af2-823f-427d13d76fa6">
											<gml:posList>35.664579216007276 139.7516712370261 8.5 35.66456591267063 139.75172581514315 8.5 35.66461802507658 139.7517449812248 8.5 35.6646314185591 139.75169040298366 8.5 35.664579216007276 139.7516712370261 8.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-c2370532-df87-4760-90f1-1716244338fa">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-c2370532-df87-4760-90f1-1716244338fa">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-c2370532-df87-4760-90f1-1716244338fa">
											<gml:posList>35.664579216007276 139.7516712370261 4.22 35.66456591267063 139.75172581514315 4.22 35.66456591267063 139.75172581514315 8.5 35.664579216007276 139.7516712370261 8.5 35.664579216007276 139.7516712370261 4.22</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-d7e153cd-7b35-4bc5-87ec-77dfcfa4e537">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-d7e153cd-7b35-4bc5-87ec-77dfcfa4e537">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-d7e153cd-7b35-4bc5-87ec-77dfcfa4e537">
											<gml:posList>35.66461802507658 139.7517449812248 4.22 35.6646314185591 139.75169040298366 4.22 35.6646314185591 139.75169040298366 8.5 35.66461802507658 139.7517449812248 8.5 35.66461802507658 139.7517449812248 4.22</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-014cb551-8d7e-477e-b2ce-c3b3089a671f">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-014cb551-8d7e-477e-b2ce-c3b3089a671f">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-014cb551-8d7e-477e-b2ce-c3b3089a671f">
											<gml:posList>35.66456591267063 139.75172581514315 4.22 35.66461802507658 139.7517449812248 4.22 35.66461802507658 139.7517449812248 8.5 35.66456591267063 139.75172581514315 8.5 35.66456591267063 139.75172581514315 4.22</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-839556ed-ab6e-4db9-a9d5-5c579283692c">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-839556ed-ab6e-4db9-a9d5-5c579283692c">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-839556ed-ab6e-4db9-a9d5-5c579283692c">
											<gml:posList>35.6646314185591 139.75169040298366 4.22 35.664579216007276 139.7516712370261 4.22 35.664579216007276 139.7516712370261 8.5 35.6646314185591 139.75169040298366 8.5 35.6646314185591 139.75169040298366 4.22</gml:posList>
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
									<gml:posList>35.664565912670632 139.751725815143146 0 35.664579216007283 139.751671237026102 0 35.664631418559097 139.751690402983655 0 35.664618025076578 139.751744981224789 0 35.664565912670632 139.751725815143146 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>35.664588823956066 139.751713420909027 0 35.664592814957061 139.751697047473925 0 35.664608475722609 139.751702797261174 0 35.664604457677854 139.751719170733509 0 35.664588823956066 139.751713420909027 0</gml:posList>
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
											<gml:posList>35.66456591267063 139.75172581514315 4.2 35.66457921600728 139.7516712370261 4.2 35.6646314185591 139.75169040298366 4.2 35.66461802507658 139.7517449812248 4.2 35.66456591267063 139.75172581514315 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66456591267063 139.75172581514315 4.2 35.66461802507658 139.7517449812248 4.2 35.66461802507658 139.7517449812248 8.2 35.66456591267063 139.75172581514315 8.2 35.66456591267063 139.75172581514315 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66461802507658 139.7517449812248 4.2 35.6646314185591 139.75169040298366 4.2 35.6646314185591 139.75169040298366 8.2 35.66461802507658 139.7517449812248 8.2 35.66461802507658 139.7517449812248 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6646314185591 139.75169040298366 4.2 35.66457921600728 139.7516712370261 4.2 35.66457921600728 139.7516712370261 8.2 35.6646314185591 139.75169040298366 8.2 35.6646314185591 139.75169040298366 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66457921600728 139.7516712370261 4.2 35.66456591267063 139.75172581514315 4.2 35.66456591267063 139.75172581514315 8.2 35.66457921600728 139.7516712370261 8.2 35.66457921600728 139.7516712370261 4.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66456591267063 139.75172581514315 8.2 35.66461802507658 139.7517449812248 8.2 35.6646314185591 139.75169040298366 8.2 35.66457921600728 139.7516712370261 8.2 35.66456591267063 139.75172581514315 8.2</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-60e29cbb-b090-4c90-9b30-dd5e69c29777"/>
							<gml:surfaceMember xlink:href="#poly-461f357c-1d0a-4af2-823f-427d13d76fa6"/>
							<gml:surfaceMember xlink:href="#poly-839556ed-ab6e-4db9-a9d5-5c579283692c"/>
							<gml:surfaceMember xlink:href="#poly-c2370532-df87-4760-90f1-1716244338fa"/>
							<gml:surfaceMember xlink:href="#poly-d7e153cd-7b35-4bc5-87ec-77dfcfa4e537"/>
							<gml:surfaceMember xlink:href="#poly-014cb551-8d7e-477e-b2ce-c3b3089a671f"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">4</con:value>
				</con:Height>
			</con:height>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:Address>
							<xAL:Country>
								<xAL:NameElement xAL:NameType="Name">日本</xAL:NameElement>
							</xAL:Country>
							<xAL:Locality>
								<xAL:NameElement xAL:NameType="Name">東京都港区西新橋三丁目</xAL:NameElement>
							</xAL:Locality>
						</xAL:Address>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">31.1618</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1002</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:areaClassificationType codeSpace="../../codelists/Common_areaClassificationType.xml">22</uro:areaClassificationType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">10</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1141</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>600</uro:specifiedFloorAreaRate>
					<uro:surveyYear>2021-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-6179</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_a5d3629d-b115-418f-9068-e100b4bdeac5">
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
					<gen:value>16</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>町・丁目コード</gen:name>
					<gen:value>3</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:StringAttribute>
					<gen:name>13+区市町村コード+大字・町コード+町・丁目コード</gen:name>
					<gen:value>13103016003</gen:value>
				</gen:StringAttribute>
			</core:genericAttribute>
			<core:genericAttribute>
				<gen:CodeAttribute>
					<gen:name>建物構造コード</gen:name>
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">21</gen:value>
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
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">0</gen:value>
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
					<urc:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">10</urc:description>
					<urc:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:depth uom="m">0.33</urc:depth>
					<urc:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../codelists/HighTideRiskAttribute_rank.xml">1</urc:rank>
					<urc:depth uom="m">0.31</urc:depth>
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
				<con:GroundSurface gml:id="surface-3b9da6b2-6223-440f-83b4-3e8fb31c3633">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-3b9da6b2-6223-440f-83b4-3e8fb31c3633">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-3b9da6b2-6223-440f-83b4-3e8fb31c3633">
											<gml:posList>35.66591273277406 139.7542737794629 2.87 35.66598585640399 139.7543076175327 2.87 35.6659658435193 139.7543726851032 2.87 35.66589262984973 139.75433896857322 2.87 35.66591273277406 139.7542737794629 2.87</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
									<gml:interior>
										<gml:LinearRing>
											<gml:posList>35.665931305777960 139.754308417706483 2.87 35.665953242866941 139.754318569127406 2.87 35.665947239001532 139.754338089398573 2.87 35.665925274900658 139.754327974439576 2.87 35.665931305777960 139.754308417706483 2.87</gml:posList>
										</gml:LinearRing>
									</gml:interior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-2a784c15-072a-482c-91df-00ebf6b91420">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-2a784c15-072a-482c-91df-00ebf6b91420">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-2a784c15-072a-482c-91df-00ebf6b91420">
											<gml:posList>35.6659658435193 139.7543726851032 8.2 35.66598585640399 139.7543076175327 8.2 35.66591273277406 139.7542737794629 8.2 35.66589262984973 139.75433896857322 8.2 35.6659658435193 139.7543726851032 8.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-a3ae6d3a-c283-47d5-8592-bcff4788cba6">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-a3ae6d3a-c283-47d5-8592-bcff4788cba6">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-a3ae6d3a-c283-47d5-8592-bcff4788cba6">
											<gml:posList>35.66589262984973 139.75433896857322 8.2 35.66591273277406 139.7542737794629 8.2 35.66591273277406 139.7542737794629 2.87 35.66589262984973 139.75433896857322 2.87 35.66589262984973 139.75433896857322 8.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-3204963c-0465-4268-ae08-12d0bf4ebb8d">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-3204963c-0465-4268-ae08-12d0bf4ebb8d">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-3204963c-0465-4268-ae08-12d0bf4ebb8d">
											<gml:posList>35.66598585640399 139.7543076175327 8.2 35.6659658435193 139.7543726851032 8.2 35.6659658435193 139.7543726851032 2.87 35.66598585640399 139.7543076175327 2.87 35.66598585640399 139.7543076175327 8.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-fe0c10db-be74-4c90-ac4e-4658d79b873b">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-fe0c10db-be74-4c90-ac4e-4658d79b873b">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-fe0c10db-be74-4c90-ac4e-4658d79b873b">
											<gml:posList>35.66591273277406 139.7542737794629 8.2 35.66598585640399 139.7543076175327 8.2 35.66598585640399 139.7543076175327 2.87 35.66591273277406 139.7542737794629 2.87 35.66591273277406 139.7542737794629 8.2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-408a91be-6472-4b59-9a3b-927a73be0a04">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-408a91be-6472-4b59-9a3b-927a73be0a04">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-408a91be-6472-4b59-9a3b-927a73be0a04">
											<gml:posList>35.6659658435193 139.7543726851032 8.2 35.66589262984973 139.75433896857322 8.2 35.66589262984973 139.75433896857322 2.87 35.6659658435193 139.7543726851032 2.87 35.6659658435193 139.7543726851032 8.2</gml:posList>
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
									<gml:posList>35.665912732774075 139.7542737794629 0 35.66589262984972 139.75433896857322 0 35.66596584442068 139.75437268510228 0 35.66598585640399 139.7543076175327 0 35.665912732774075 139.7542737794629 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>35.665931305935707 139.754308417706312 0 35.665925275058399 139.754327974439406 0 35.665947239429691 139.754338089398118 0 35.665953243024681 139.754318569127264 0 35.665931305935707 139.754308417706312 0</gml:posList>
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
											<gml:posList>35.665912732774075 139.7542737794629 2.9 35.66598585640399 139.7543076175327 2.9 35.66596584442068 139.75437268510228 2.9 35.66589262984972 139.75433896857322 2.9 35.665912732774075 139.7542737794629 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665912732774075 139.7542737794629 2.9 35.66589262984972 139.75433896857322 2.9 35.66589262984972 139.75433896857322 9.3 35.665912732774075 139.7542737794629 9.3 35.665912732774075 139.7542737794629 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66589262984972 139.75433896857322 2.9 35.66596584442068 139.75437268510228 2.9 35.66596584442068 139.75437268510228 9.3 35.66589262984972 139.75433896857322 9.3 35.66589262984972 139.75433896857322 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66596584442068 139.75437268510228 2.9 35.66598585640399 139.7543076175327 2.9 35.66598585640399 139.7543076175327 9.3 35.66596584442068 139.75437268510228 9.3 35.66596584442068 139.75437268510228 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66598585640399 139.7543076175327 2.9 35.665912732774075 139.7542737794629 2.9 35.665912732774075 139.7542737794629 9.3 35.66598585640399 139.7543076175327 9.3 35.66598585640399 139.7543076175327 2.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665912732774075 139.7542737794629 9.3 35.66589262984972 139.75433896857322 9.3 35.66596584442068 139.75437268510228 9.3 35.66598585640399 139.7543076175327 9.3 35.665912732774075 139.7542737794629 9.3</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-3b9da6b2-6223-440f-83b4-3e8fb31c3633"/>
							<gml:surfaceMember xlink:href="#poly-2a784c15-072a-482c-91df-00ebf6b91420"/>
							<gml:surfaceMember xlink:href="#poly-408a91be-6472-4b59-9a3b-927a73be0a04"/>
							<gml:surfaceMember xlink:href="#poly-a3ae6d3a-c283-47d5-8592-bcff4788cba6"/>
							<gml:surfaceMember xlink:href="#poly-fe0c10db-be74-4c90-ac4e-4658d79b873b"/>
							<gml:surfaceMember xlink:href="#poly-3204963c-0465-4268-ae08-12d0bf4ebb8d"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">6.4</con:value>
				</con:Height>
			</con:height>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:Address>
							<xAL:Country>
								<xAL:NameElement xAL:NameType="Name">日本</xAL:NameElement>
							</xAL:Country>
							<xAL:Locality>
								<xAL:NameElement xAL:NameType="Name">東京都港区新橋三丁目</xAL:NameElement>
							</xAL:Locality>
						</xAL:Address>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">54.5764</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:areaClassificationType codeSpace="../../codelists/Common_areaClassificationType.xml">22</uro:areaClassificationType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">10</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1310</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>80</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>700</uro:specifiedFloorAreaRate>
					<uro:surveyYear>2021-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-4901</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_468427ae-fc81-4d0c-9b18-d0b3ff589159">
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
					<gen:value codeSpace="../../codelists/KeyValuePairAttribute_key106.xml">20</gen:value>
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
				<con:GroundSurface gml:id="surface-035bda12-505f-433f-a9ef-3d8f5a8e83da">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-035bda12-505f-433f-a9ef-3d8f5a8e83da">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-035bda12-505f-433f-a9ef-3d8f5a8e83da">
											<gml:posList>35.661212737637328 139.750315356949471 3.931 35.661236902603839 139.750327150087912 3.931 35.661246706639567 139.750296766185016 3.931 35.661222631807512 139.750284972960969 3.931 35.661212737637328 139.750315356949471 3.931</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
									<gml:interior>
										<gml:LinearRing>
											<gml:posList>35.661224642561642 139.750308850166931 3.931 35.661231892051596 139.750312388108455 3.931 35.661234833262313 139.750303272937600 3.931 35.661227610812695 139.750299734970383 3.931 35.661224642561642 139.750308850166931 3.931</gml:posList>
										</gml:LinearRing>
									</gml:interior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="surface-bd99fbb9-eb14-4bef-815c-38de8ba6c9fa">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-bd99fbb9-eb14-4bef-815c-38de8ba6c9fa">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-bd99fbb9-eb14-4bef-815c-38de8ba6c9fa">
											<gml:posList>35.66122263180751 139.75028497296097 8.1 35.66121273763733 139.75031535694947 8.1 35.66123690260384 139.7503271500879 8.1 35.66124670663957 139.75029676618502 8.1 35.66122263180751 139.75028497296097 8.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-d303048b-53ef-4f97-860a-226014400280">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-d303048b-53ef-4f97-860a-226014400280">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-d303048b-53ef-4f97-860a-226014400280">
											<gml:posList>35.66122263180751 139.75028497296097 3.931 35.66121273763733 139.75031535694947 3.931 35.66121273763733 139.75031535694947 8.1 35.66122263180751 139.75028497296097 8.1 35.66122263180751 139.75028497296097 3.931</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-c895976a-2fe1-4a01-b2e7-88478ca1bb8d">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-c895976a-2fe1-4a01-b2e7-88478ca1bb8d">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-c895976a-2fe1-4a01-b2e7-88478ca1bb8d">
											<gml:posList>35.66124670663957 139.75029676618502 3.931 35.66122263180751 139.75028497296097 3.931 35.66122263180751 139.75028497296097 8.1 35.66124670663957 139.75029676618502 8.1 35.66124670663957 139.75029676618502 3.931</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-b60bcff2-0521-44b1-a7ce-8762be44e15b">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-b60bcff2-0521-44b1-a7ce-8762be44e15b">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-b60bcff2-0521-44b1-a7ce-8762be44e15b">
											<gml:posList>35.66121273763733 139.75031535694947 3.931 35.66123690260384 139.7503271500879 3.931 35.66123690260384 139.7503271500879 8.1 35.66121273763733 139.75031535694947 8.1 35.66121273763733 139.75031535694947 3.931</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="surface-70dd0ee7-c2f4-42ae-983f-344999e88b60">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-70dd0ee7-c2f4-42ae-983f-344999e88b60">
									<gml:exterior>
										<gml:LinearRing gml:id="linr-70dd0ee7-c2f4-42ae-983f-344999e88b60">
											<gml:posList>35.66123690260384 139.7503271500879 3.931 35.66124670663957 139.75029676618502 3.931 35.66124670663957 139.75029676618502 8.1 35.66123690260384 139.7503271500879 8.1 35.66123690260384 139.7503271500879 3.931</gml:posList>
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
									<gml:posList>35.661212737637328 139.750315356949471 0 35.661222631807505 139.750284972960969 0 35.661246706639560 139.750296766185045 0 35.661236902603832 139.750327150087912 0 35.661212737637328 139.750315356949471 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>35.661224642561635 139.750308850166931 0 35.661231892051589 139.750312388108455 0 35.661234833262306 139.750303272937600 0 35.661227610812688 139.750299734970383 0 35.661224642561635 139.750308850166931 0</gml:posList>
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
											<gml:posList>35.66121273763733 139.75031535694947 3.9 35.661222631807505 139.75028497296097 3.9 35.66124670663956 139.75029676618504 3.9 35.66123690260383 139.7503271500879 3.9 35.66121273763733 139.75031535694947 3.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66121273763733 139.75031535694947 3.9 35.66123690260383 139.7503271500879 3.9 35.66123690260383 139.7503271500879 7.5 35.66121273763733 139.75031535694947 7.5 35.66121273763733 139.75031535694947 3.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66123690260383 139.7503271500879 3.9 35.66124670663956 139.75029676618504 3.9 35.66124670663956 139.75029676618504 7.5 35.66123690260383 139.7503271500879 7.5 35.66123690260383 139.7503271500879 3.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66124670663956 139.75029676618504 3.9 35.661222631807505 139.75028497296097 3.9 35.661222631807505 139.75028497296097 7.5 35.66124670663956 139.75029676618504 7.5 35.66124670663956 139.75029676618504 3.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.661222631807505 139.75028497296097 3.9 35.66121273763733 139.75031535694947 3.9 35.66121273763733 139.75031535694947 7.5 35.661222631807505 139.75028497296097 7.5 35.661222631807505 139.75028497296097 3.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66121273763733 139.75031535694947 7.5 35.66123690260383 139.7503271500879 7.5 35.66124670663956 139.75029676618504 7.5 35.661222631807505 139.75028497296097 7.5 35.66121273763733 139.75031535694947 7.5</gml:posList>
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
							<gml:surfaceMember xlink:href="#poly-035bda12-505f-433f-a9ef-3d8f5a8e83da"/>
							<gml:surfaceMember xlink:href="#poly-bd99fbb9-eb14-4bef-815c-38de8ba6c9fa"/>
							<gml:surfaceMember xlink:href="#poly-d303048b-53ef-4f97-860a-226014400280"/>
							<gml:surfaceMember xlink:href="#poly-b60bcff2-0521-44b1-a7ce-8762be44e15b"/>
							<gml:surfaceMember xlink:href="#poly-c895976a-2fe1-4a01-b2e7-88478ca1bb8d"/>
							<gml:surfaceMember xlink:href="#poly-70dd0ee7-c2f4-42ae-983f-344999e88b60"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<con:height>
				<con:Height>
					<con:highReference codeSpace="../../codelists/Elevation_elevationReference.xml">2</con:highReference>
					<con:lowReference codeSpace="../../codelists/Elevation_elevationReference.xml">6</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">3.6</con:value>
				</con:Height>
			</con:height>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:storeysAboveGround>-9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>-9999</bldg:storeysBelowGround>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">-9999</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:areaClassificationType codeSpace="../../codelists/Common_areaClassificationType.xml">22</uro:areaClassificationType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">5</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:specifiedBuildingCoverageRate>60</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>200</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001-01-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13103-bldg-7730</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13103</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
