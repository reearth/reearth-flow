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
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.64710 137.05290 0</gml:lowerCorner>
			<gml:upperCorner>36.64722 137.05302 5.0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>

	<!-- Error building: LOD0 footprint exterior ring is CLOCKWISE (not
	     counter-clockwise), so it is reported under 'LOD0 面の向きエラー'. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_a1b2c3d4-0001-11f0-a3af-18ece7a5508c">
			<gml:name>16211-bldg-cw</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>461</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">5.0</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_a1b2c3d4-0001-11f0-a3af-18ece7a5508c">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_orientation_cw">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.64720 137.05300 0 36.64722 137.05300 0 36.64722 137.05302 0 36.64720 137.05302 0 36.64720 137.05300 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-200</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>

	<!-- Control building (no error): LOD0 footprint exterior ring is
	     counter-clockwise, so it must NOT be reported. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_a1b2c3d4-0002-11f0-810e-18ece7a5508c">
			<gml:name>16211-bldg-ccw</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>461</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">5.0</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_a1b2c3d4-0002-11f0-810e-18ece7a5508c">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_orientation_ccw">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.64710 137.05290 0 36.64710 137.05292 0 36.64712 137.05292 0 36.64712 137.05290 0 36.64710 137.05290 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-201</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
