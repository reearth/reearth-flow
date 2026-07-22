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
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647 137.053 0.0</gml:lowerCorner>
			<gml:upperCorner>36.648 137.054 8.0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- L09 LOD1 DuplicateConsecutivePoints: ground + roof lod1Solid faces have an
	     exterior LinearRing with an exact duplicated consecutive vertex (ground: 4th
	     and 5th vertices identical; roof: 2nd and 3rd vertices identical), reported as
	     two DuplicateConsecutivePoints errors under "LOD1 境界面のエラー". Coordinates
	     ported verbatim from the plateau4 L09_LOD1_03 fixture. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_95f14b97-8bea-4c54-a326-74819dfc09a7">
			<gml:name>16211-bldg-L09-LOD1-03</gml:name>
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>411</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.0</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<!-- Ground surface - duplicate consecutive points (4th == 5th) -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647 137.053 0 36.647 137.054 0 36.648 137.054 0 36.648 137.053 0 36.648 137.053 0 36.647 137.053 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- Roof surface - duplicate consecutive points (2nd == 3rd) -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647 137.053 8.0 36.648 137.053 8.0 36.648 137.053 8.0 36.648 137.054 8.0 36.647 137.054 8.0 36.647 137.053 8.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- Wall surface 1 - valid -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647 137.053 0 36.647 137.054 0 36.647 137.054 8.0 36.647 137.053 8.0 36.647 137.053 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- Wall surface 2 - valid -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647 137.054 0 36.648 137.054 0 36.648 137.054 8.0 36.647 137.054 8.0 36.647 137.054 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- Wall surface 3 - valid -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.648 137.054 0 36.648 137.053 0 36.648 137.053 8.0 36.648 137.054 8.0 36.648 137.054 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- Wall surface 4 - valid -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.648 137.053 0 36.647 137.053 0 36.647 137.053 8.0 36.648 137.053 8.0 36.648 137.053 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
