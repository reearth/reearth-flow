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
			<gml:lowerCorner>34.64915716995996 137.34535335684816 0</gml:lowerCorner>
			<gml:upperCorner>36.64705502737237 137.0537094956814 8.592</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>

	<!-- #03: square exterior, two interior rings that overlap each other. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef114-02e4-11f0-a3af-18ece7a5508c">
			<gml:name>16211-bldg-78</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3003</bldg:class>
			<bldg:usage>411</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.6</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_b3eef114-02e4-11f0-a3af-18ece7a5508c">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_b3eef114-02e4-11f0-a3af-18ece7a5508c">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>0.0 0.0 0.0 0.0 1.0 0.0 1.0 1.0 0.0 1.0 0.0 0.0 0.0 0.0 0.0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>0.1 0.1 0.0 0.5 0.1 0.0 0.5 0.5 0.0 0.1 0.5 0.0 0.1 0.1 0.0</gml:posList>
								</gml:LinearRing>
							</gml:interior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>0.4 0.4 0.0 0.8 0.4 0.0 0.8 0.8 0.0 0.4 0.8 0.0 0.4 0.4 0.0</gml:posList>
								</gml:LinearRing>
							</gml:interior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-78</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>

	<!-- Valid building (no error): a simple, properly closed LOD0 footprint surface. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef115-02e4-11f0-810e-18ece7a5508c">
			<gml:name>16211-bldg-77</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>461</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.0</con:value>
				</con:Height>
			</con:height>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lod0_b3eef115-02e4-11f0-810e-18ece7a5508c">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_b3eef115-02e4-11f0-810e-18ece7a5508c">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.64705388584696 137.0527173474674 0 36.64705502737237 137.05268591841337 0 36.64700527700596 137.05268308385453 0 36.6470041354812 137.05271451288834 0 36.64705388584696 137.0527173474674 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-77</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
