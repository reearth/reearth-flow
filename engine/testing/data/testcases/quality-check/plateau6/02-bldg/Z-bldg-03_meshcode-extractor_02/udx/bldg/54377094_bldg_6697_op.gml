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
		Z-bldg-03 (図郭不正 / meshcode) plateau6 テストデータ _02
		plateau4 の Z-bldg-03_meshcode-extractor_02 を CityGML 3.0 に移植（座標・gml:id・ファイル名は逐語流用）。
		ファイル名のメッシュコードは 54377094。LOD0 フットプリントは隣接 2 メッシュ 54377095/54377096 にまたがる（meshcount=2）。
		メッシュ境界をまたぐため図郭不正 1 件。支配メッシュ(面積最大)は 54377095。
	-->
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.650 137.060 0</gml:lowerCorner>
			<gml:upperCorner>36.670 137.080 50.0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cross_mesh_test_building">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>461</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">15.0</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_bldg_cross_mesh_test_building">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_bldg_cross_mesh_test_building">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.658333333 137.0650 0 36.665833333 137.0650 0 36.665833333 137.0750 0 36.658333333 137.0750 0 36.658333333 137.0650 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
