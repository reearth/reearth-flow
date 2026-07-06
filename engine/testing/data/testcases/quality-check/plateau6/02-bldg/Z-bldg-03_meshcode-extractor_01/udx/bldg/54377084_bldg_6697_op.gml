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
		Z-bldg-03 (図郭不正 / meshcode) plateau6 テストデータ _01
		plateau4 の Z-bldg-03_meshcode-extractor_01 を CityGML 3.0 に移植（座標・gml:id・ファイル名は逐語流用）。
		ファイル名のメッシュコードは 54377084 だが、LOD0 フットプリントは隣接する 3 次メッシュ 54377085 に位置する。
		meshcount=1 だが filename のメッシュ(84)と実メッシュ(85)が不一致のため図郭不正 1 件。
	-->
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.65422397737467 137.0701714804155 0</gml:lowerCorner>
			<gml:upperCorner>36.65426289610968 137.07022204628032 99.63672</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078de-02e4-11f0-9961-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>461</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.4</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_bldg_b3f078de">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_bldg_b3f078de">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65423985231743 137.07022204628032 0 36.65426289610968 137.07018801464667 0 36.654247021162256 137.0701714804155 0 36.65422397737467 137.07020551204704 0 36.65423985231743 137.07022204628032 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
