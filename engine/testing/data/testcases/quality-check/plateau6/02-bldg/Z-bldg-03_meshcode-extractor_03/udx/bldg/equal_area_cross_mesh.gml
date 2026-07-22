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
		<bldg:Building gml:id="bldg_equal_area_cross_mesh">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>422</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">12.0</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_bldg_equal_area_cross_mesh">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_bldg_equal_area_cross_mesh">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.66625 137.0745 0 36.66625 137.0755 0 36.66750 137.0755 0 36.66750 137.0745 0 36.66625 137.0745 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</bldg:Building>
	</core:cityObjectMember>

	<!-- Circular building precisely centered on mesh boundary for perfect equal division -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_circular_equal_area">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>431</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.0</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_bldg_circular_equal_area">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_bldg_circular_equal_area">
							<gml:exterior>
								<gml:LinearRing>
									<!-- Approximated circle centered exactly on longitude boundary 137.075 -->
									<gml:posList>36.660 137.0745 0 36.66025 137.07465 0 36.66041421 137.07480 0 36.66050 137.075 0 36.66041421 137.0752 0 36.66025 137.07535 0 36.660 137.0755 0 36.65975 137.07535 0 36.65958579 137.0752 0 36.6595 137.075 0 36.65958579 137.07480 0 36.65975 137.07465 0 36.660 137.0745 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</bldg:Building>
	</core:cityObjectMember>

	<!-- Rectangular building crossing latitude mesh boundary -->
	<!-- Mesh boundary at 36.666666667 latitude (multiple of 0.008333333 degree intervals) -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_lat_boundary_equal">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class>3002</bldg:class>
			<bldg:usage>452</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">10.5</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_bldg_lat_boundary_equal">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_bldg_lat_boundary_equal">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.66625 137.0751 0 36.66708333333 137.0751 0 36.66708333333 137.0754 0 36.66625 137.0754 0 36.66625 137.0751 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</bldg:Building>
	</core:cityObjectMember>

</core:CityModel>
