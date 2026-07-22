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
			<gml:lowerCorner>34.6890 137.3176 0.0</gml:lowerCorner>
			<gml:upperCorner>34.6902 137.3200 8.0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- L09 LOD1 SelfIntersection: another building variant of L09_LOD1_01.
	     Ground + roof lod1Solid faces have a self-touching exterior LinearRing
	     (5th vertex lies outside the quad), reported as two SelfIntersection
	     errors under "LOD1 境界面のエラー". Coordinates ported verbatim from the
	     plateau4 L09_LOD1_02 fixture. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_c87254fa-0ebc-449f-bb29-e0e21d2a7016">
			<gml:name>16211-bldg-L09-LOD1-02</gml:name>
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
							<!-- Ground surface - self-touching -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.6890 137.3176 0.0 34.6900 137.3176 0.0 34.6900 137.3200 0.0 34.6890 137.3200 0.0 34.6902 137.3188 0.0 34.6890 137.3176 0.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- Roof surface - self-touching -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.6890 137.3176 8.0 34.6900 137.3176 8.0 34.6900 137.3200 8.0 34.6890 137.3200 8.0 34.6902 137.3188 8.0 34.6890 137.3176 8.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- Wall surface - valid -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.6890 137.3176 0.0 34.6900 137.3176 0.0 34.6900 137.3176 8.0 34.6890 137.3176 8.0 34.6890 137.3176 0.0</gml:posList>
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
