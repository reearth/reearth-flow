<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>33.71206052393341 130.4791323734121 0</gml:lowerCorner>
			<gml:upperCorner>33.719151919976895 130.4911331675937 100</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- 非水密立体エラー: 面が1つ欠けている立体（上面なし） -->
	<!-- SolidBoundaryValidatorのNotClosedポートで検出される -->
	<core:cityObjectMember>
		<gen:GenericCityObject gml:id="gen_surface_not_closed_001">
			<gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
			<core:creationDate>2025-03-31</core:creationDate>
			<gen:stringAttribute name="告示番号">
				<gen:value>福岡県告示第316号</gen:value>
			</gen:stringAttribute>
			<gen:dateAttribute name="告示年月日">
				<gen:value>2019-09-27</gen:value>
			</gen:dateAttribute>
			<gen:stringAttribute name="名称">
				<gen:value>テスト指定区域（非水密立体）</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="区域の所在">
				<gen:value>古賀市テスト町</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="区域の面積">
				<gen:value>1000</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="備考">
				<gen:value>非水密立体テスト用データ</gen:value>
			</gen:stringAttribute>
			<!-- LOD1 Solid: 6面の立方体から上面を削除して5面のみ（非水密） -->
			<gen:lod1Geometry>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<!-- 底面 (z=0) -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>
												33.71750000000000 130.48100000000000 0
												33.71750000000000 130.48200000000000 0
												33.71850000000000 130.48200000000000 0
												33.71850000000000 130.48100000000000 0
												33.71750000000000 130.48100000000000 0
											</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 南側壁面 -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>
												33.71750000000000 130.48100000000000 0
												33.71850000000000 130.48100000000000 0
												33.71850000000000 130.48100000000000 10
												33.71750000000000 130.48100000000000 10
												33.71750000000000 130.48100000000000 0
											</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 東側壁面 -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>
												33.71850000000000 130.48100000000000 0
												33.71850000000000 130.48200000000000 0
												33.71850000000000 130.48200000000000 10
												33.71850000000000 130.48100000000000 10
												33.71850000000000 130.48100000000000 0
											</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 北側壁面 -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>
												33.71850000000000 130.48200000000000 0
												33.71750000000000 130.48200000000000 0
												33.71750000000000 130.48200000000000 10
												33.71850000000000 130.48200000000000 10
												33.71850000000000 130.48200000000000 0
											</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 西側壁面 -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>
												33.71750000000000 130.48200000000000 0
												33.71750000000000 130.48100000000000 0
												33.71750000000000 130.48100000000000 10
												33.71750000000000 130.48200000000000 10
												33.71750000000000 130.48200000000000 0
											</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 上面 (z=10) が欠落 - これにより非水密立体となる -->
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</gen:lod1Geometry>
		</gen:GenericCityObject>
	</core:cityObjectMember>
</core:CityModel>
