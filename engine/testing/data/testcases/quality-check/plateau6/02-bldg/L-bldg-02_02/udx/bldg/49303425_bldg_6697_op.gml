<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>34.73480000 135.50200000 0</gml:lowerCorner>
			<gml:upperCorner>34.73510000 135.50240000 10.0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- L-bldg-02 error detection (CityGML 3.0): partial connectivity mix.
	     One Building with three LOD2 BuildingParts. Parts A and B are placed so that
	     A's north wall (Y=135.50210) and B's south wall have identical coordinates
	     (reverse order), so they are joined into one connected component of size 2 ->
	     status="partial" (connected but not all 3 parts). Part C is placed away from
	     both A and B and shares no surface -> status="alone". All three parts are
	     reported as L-bldg-02 errors (3 rows).
	     CityGML 2.0 -> 3.0 changes vs the plateau4 L-bldg-02_02 fixture:
	     bldg:consistsOfBuildingPart -> bldg:buildingPart, bldg:lod2Solid ->
	     core:lod2Solid, gml 3.1.1 -> gml 3.2, i-UR 3.1 -> i-UR 4.0. Coordinates are
	     ported verbatim so the union-find result is identical. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l02partial-0001">
			<gml:name>テスト建物部分接続</gml:name>
			<!-- Part A: X[34.73480,34.73490] Y[135.50200,135.50210]. Shares its north wall with B. -->
			<bldg:buildingPart>
				<bldg:BuildingPart gml:id="bpart_l02partial-A">
					<gml:name>テスト建物部分A</gml:name>
					<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">10.0</con:value></con:Height></con:height>
					<core:lod2Solid>
						<gml:Solid>
							<gml:exterior>
								<gml:CompositeSurface>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50200000 0 34.73490000 135.50200000 0 34.73490000 135.50210000 0 34.73480000 135.50210000 0 34.73480000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50200000 0 34.73480000 135.50210000 0 34.73480000 135.50210000 10.0 34.73480000 135.50200000 10.0 34.73480000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<!-- SHARED WITH B (A north wall == B south wall, reverse order) -->
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50210000 0 34.73490000 135.50210000 0 34.73490000 135.50210000 10.0 34.73480000 135.50210000 10.0 34.73480000 135.50210000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73490000 135.50210000 0 34.73490000 135.50200000 0 34.73490000 135.50200000 10.0 34.73490000 135.50210000 10.0 34.73490000 135.50210000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73490000 135.50200000 0 34.73480000 135.50200000 0 34.73480000 135.50200000 10.0 34.73490000 135.50200000 10.0 34.73490000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50200000 10.0 34.73480000 135.50210000 10.0 34.73490000 135.50210000 10.0 34.73490000 135.50200000 10.0 34.73480000 135.50200000 10.0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
								</gml:CompositeSurface>
							</gml:exterior>
						</gml:Solid>
					</core:lod2Solid>
				</bldg:BuildingPart>
			</bldg:buildingPart>
			<!-- Part B: X[34.73480,34.73490] Y[135.50210,135.50220]. Shares its south wall with A. -->
			<bldg:buildingPart>
				<bldg:BuildingPart gml:id="bpart_l02partial-B">
					<gml:name>テスト建物部分B</gml:name>
					<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">10.0</con:value></con:Height></con:height>
					<core:lod2Solid>
						<gml:Solid>
							<gml:exterior>
								<gml:CompositeSurface>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50210000 0 34.73490000 135.50210000 0 34.73490000 135.50220000 0 34.73480000 135.50220000 0 34.73480000 135.50210000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50210000 0 34.73480000 135.50220000 0 34.73480000 135.50220000 10.0 34.73480000 135.50210000 10.0 34.73480000 135.50210000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50220000 0 34.73490000 135.50220000 0 34.73490000 135.50220000 10.0 34.73480000 135.50220000 10.0 34.73480000 135.50220000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73490000 135.50220000 0 34.73490000 135.50210000 0 34.73490000 135.50210000 10.0 34.73490000 135.50220000 10.0 34.73490000 135.50220000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<!-- SHARED WITH A (B south wall == A north wall, reverse order) -->
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73490000 135.50210000 0 34.73480000 135.50210000 0 34.73480000 135.50210000 10.0 34.73490000 135.50210000 10.0 34.73490000 135.50210000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50210000 10.0 34.73480000 135.50220000 10.0 34.73490000 135.50220000 10.0 34.73490000 135.50210000 10.0 34.73480000 135.50210000 10.0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
								</gml:CompositeSurface>
							</gml:exterior>
						</gml:Solid>
					</core:lod2Solid>
				</bldg:BuildingPart>
			</bldg:buildingPart>
			<!-- Part C: X[34.73500,34.73510] Y[135.50230,135.50240]. Isolated (shares no surface). -->
			<bldg:buildingPart>
				<bldg:BuildingPart gml:id="bpart_l02partial-C">
					<gml:name>テスト建物部分C</gml:name>
					<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">10.0</con:value></con:Height></con:height>
					<core:lod2Solid>
						<gml:Solid>
							<gml:exterior>
								<gml:CompositeSurface>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73500000 135.50230000 0 34.73510000 135.50230000 0 34.73510000 135.50240000 0 34.73500000 135.50240000 0 34.73500000 135.50230000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73500000 135.50230000 0 34.73500000 135.50240000 0 34.73500000 135.50240000 10.0 34.73500000 135.50230000 10.0 34.73500000 135.50230000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73500000 135.50240000 0 34.73510000 135.50240000 0 34.73510000 135.50240000 10.0 34.73500000 135.50240000 10.0 34.73500000 135.50240000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73510000 135.50240000 0 34.73510000 135.50230000 0 34.73510000 135.50230000 10.0 34.73510000 135.50240000 10.0 34.73510000 135.50240000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73510000 135.50230000 0 34.73500000 135.50230000 0 34.73500000 135.50230000 10.0 34.73510000 135.50230000 10.0 34.73510000 135.50230000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73500000 135.50230000 10.0 34.73500000 135.50240000 10.0 34.73510000 135.50240000 10.0 34.73510000 135.50230000 10.0 34.73500000 135.50230000 10.0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
								</gml:CompositeSurface>
							</gml:exterior>
						</gml:Solid>
					</core:lod2Solid>
				</bldg:BuildingPart>
			</bldg:buildingPart>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
