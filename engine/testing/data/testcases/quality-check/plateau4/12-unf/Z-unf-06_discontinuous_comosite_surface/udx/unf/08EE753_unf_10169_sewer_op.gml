<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd  https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
			<gml:lowerCorner>156999.0 21599.0 109.0</gml:lowerCorner>
			<gml:upperCorner>157102.0 21602.0 112.0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- 不連続CompositeSurface: LOD3のCompositeSurfaceが2つの独立した面グループを含む -->
	<!-- グループ1: 位置Aの3面（底面+前面+左面、互いにエッジ共有） -->
	<!-- グループ2: 位置Bの3面（底面+前面+左面、互いにエッジ共有、グループ1とはエッジ非共有） -->
	<!-- LOD2は正常Solid、LOD3に不連続CompositeSurfaceを含む -->
	<core:cityObjectMember>
		<uro:SewerPipe gml:id="unf_test_discontinuous_cs_001">
			<gml:name>雨水管路</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<frn:lod2Geometry>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<!-- 底面 (z=110) -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-lod2-bottom-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000.0 21600.0 110.0 157000.0 21601.0 110.0 157001.0 21601.0 110.0 157001.0 21600.0 110.0 157000.0 21600.0 110.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 上面 (z=111) -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-lod2-top-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000.0 21600.0 111.0 157001.0 21600.0 111.0 157001.0 21601.0 111.0 157000.0 21601.0 111.0 157000.0 21600.0 111.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 前面 (y=21600) - 正常 -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-lod2-front-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000.0 21600.0 110.0 157001.0 21600.0 110.0 157001.0 21600.0 111.0 157000.0 21600.0 111.0 157000.0 21600.0 110.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 後面 (y=21601) -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-lod2-back-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000.0 21601.0 110.0 157000.0 21601.0 111.0 157001.0 21601.0 111.0 157001.0 21601.0 110.0 157000.0 21601.0 110.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 左面 (x=157000) -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-lod2-left-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000.0 21600.0 110.0 157000.0 21600.0 111.0 157000.0 21601.0 111.0 157000.0 21601.0 110.0 157000.0 21600.0 110.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- 右面 (x=157001) -->
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-lod2-right-001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001.0 21600.0 110.0 157001.0 21601.0 110.0 157001.0 21601.0 111.0 157001.0 21600.0 111.0 157001.0 21600.0 110.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</frn:lod2Geometry>
			<frn:lod3Geometry>
				<gml:CompositeSurface srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<!-- ===== グループ1: 位置A (x=157000-157001) の3面 ===== -->
					<!-- グループ1 底面 (z=109) -->
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly-lod3-grp1-bottom">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157000.0 21600.0 109.0 157000.0 21601.0 109.0 157001.0 21601.0 109.0 157001.0 21600.0 109.0 157000.0 21600.0 109.0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<!-- グループ1 前面 (y=21600) - 底面とエッジ共有: (157000,21600,109)→(157001,21600,109) -->
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly-lod3-grp1-front">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157000.0 21600.0 109.0 157001.0 21600.0 109.0 157001.0 21600.0 110.0 157000.0 21600.0 110.0 157000.0 21600.0 109.0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<!-- グループ1 左面 (x=157000) - 底面・前面とエッジ共有 -->
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly-lod3-grp1-left">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157000.0 21600.0 109.0 157000.0 21600.0 110.0 157000.0 21601.0 110.0 157000.0 21601.0 109.0 157000.0 21600.0 109.0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<!-- ===== グループ2: 位置B (x=157100-157101) の3面 - グループ1とエッジ非共有 ===== -->
					<!-- グループ2 底面 (z=109) -->
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly-lod3-grp2-bottom">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157100.0 21600.0 109.0 157100.0 21601.0 109.0 157101.0 21601.0 109.0 157101.0 21600.0 109.0 157100.0 21600.0 109.0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<!-- グループ2 前面 (y=21600) - グループ2底面とエッジ共有: (157100,21600,109)→(157101,21600,109) -->
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly-lod3-grp2-front">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157100.0 21600.0 109.0 157101.0 21600.0 109.0 157101.0 21600.0 110.0 157100.0 21600.0 110.0 157100.0 21600.0 109.0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<!-- グループ2 左面 (x=157100) - グループ2底面・前面とエッジ共有 -->
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly-lod3-grp2-left">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157100.0 21600.0 109.0 157100.0 21600.0 110.0 157100.0 21601.0 110.0 157100.0 21601.0 109.0 157100.0 21600.0 109.0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:CompositeSurface>
			</frn:lod3Geometry>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:year>2011</uro:year>
		</uro:SewerPipe>
	</core:cityObjectMember>
</core:CityModel>
