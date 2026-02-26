<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>33.71750000000000 130.48100000000000 0</gml:lowerCorner>
			<gml:upperCorner>33.71850000000000 130.48300000000000 5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- LOD2立体のエラー: Face Wrong Orientation (面の法線が内向き) -->
	<!-- 水密な立方体だが、上面の頂点順序が逆で法線が内向き（下向き）になっている -->
	<!-- FME SolidBoundaryValidatorのOtherIssuesポートで検出されるエラー -->
	<!-- 現在のワークフローでは立体のエラー検出は未実装のため、0エラーが期待される -->
	<core:cityObjectMember>
		<gen:GenericCityObject gml:id="gen_solid_face_wrong_orientation_001">
			<gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
			<core:creationDate>2025-03-31</core:creationDate>
			<gen:stringAttribute name="告示番号">
				<gen:value>福岡県告示第316号</gen:value>
			</gen:stringAttribute>
			<gen:dateAttribute name="告示年月日">
				<gen:value>2019-09-27</gen:value>
			</gen:dateAttribute>
			<gen:stringAttribute name="名称">
				<gen:value>テスト指定区域（面の向きエラー）</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="区域の所在">
				<gen:value>古賀市テスト町</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="区域の面積">
				<gen:value>1000</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="備考">
				<gen:value>テスト用データ - 立体のエラー：Face Wrong Orientation</gen:value>
			</gen:stringAttribute>
			<gen:lod2Geometry>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<!-- 底面 (z=0) - 正しい向き: 外側（下）から見て反時計回り -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-bottom-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.71750000000000 130.48100000000000 0 33.71750000000000 130.48300000000000 0 33.71850000000000 130.48300000000000 0 33.71850000000000 130.48100000000000 0 33.71750000000000 130.48100000000000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 上面 (z=5) - 誤った向き: 頂点順序を逆にして法線を内向き（下向き）にする -->
							<!-- 正しい順序: 外側（上）から見て反時計回り = 33.7175,130.481,5 → 33.7185,130.481,5 → 33.7185,130.483,5 → 33.7175,130.483,5 -->
							<!-- 誤った順序（逆回り）: 33.7175,130.481,5 → 33.7175,130.483,5 → 33.7185,130.483,5 → 33.7185,130.481,5 -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-top-001-wrong-orientation">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.71750000000000 130.48100000000000 5 33.71750000000000 130.48300000000000 5 33.71850000000000 130.48300000000000 5 33.71850000000000 130.48100000000000 5 33.71750000000000 130.48100000000000 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 前面 (lat=33.7175) - 正しい向き -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-front-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.71750000000000 130.48100000000000 0 33.71750000000000 130.48100000000000 5 33.71750000000000 130.48300000000000 5 33.71750000000000 130.48300000000000 0 33.71750000000000 130.48100000000000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 背面 (lat=33.7185) - 正しい向き -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-back-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.71850000000000 130.48100000000000 0 33.71850000000000 130.48300000000000 0 33.71850000000000 130.48300000000000 5 33.71850000000000 130.48100000000000 5 33.71850000000000 130.48100000000000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 左側面 (lon=130.481) - 正しい向き -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-left-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.71750000000000 130.48100000000000 0 33.71850000000000 130.48100000000000 0 33.71850000000000 130.48100000000000 5 33.71750000000000 130.48100000000000 5 33.71750000000000 130.48100000000000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 右側面 (lon=130.483) - 正しい向き -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-right-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.71750000000000 130.48300000000000 0 33.71750000000000 130.48300000000000 5 33.71850000000000 130.48300000000000 5 33.71850000000000 130.48300000000000 0 33.71750000000000 130.48300000000000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</gen:lod2Geometry>
		</gen:GenericCityObject>
	</core:cityObjectMember>
</core:CityModel>
