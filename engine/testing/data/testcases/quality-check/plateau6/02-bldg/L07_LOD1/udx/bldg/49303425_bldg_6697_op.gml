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
			<gml:lowerCorner>34.6833952578084 137.31255089625097 -0.1</gml:lowerCorner>
			<gml:upperCorner>34.69172867904967 137.32508280776253 31.708999999999996</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l07lod1-0001">
			<gml:name>23201-bldg-L07-LOD1</gml:name>
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>422</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">9.7</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<!-- L07 LOD1 check target: the second lod1Solid face has two near-duplicate
			     consecutive points (within tolerance) -> one DuplicateConsecutivePoints
			     error under "LOD1 境界面のエラー". The first face is valid. -->
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<!-- Passes -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.68339525781 137.325082 0.0 34.6834 137.325082 0.0 34.6834 137.325083 0.0 34.68339525781 137.325083 0.0 34.68339525781 137.325082 0.0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<!-- Fails: first two points are duplicates within tolerance -->
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.68339525781 137.325082 0.0 34.68339525782 137.325082 0.0 34.6834 137.325082 0.0 34.6834 137.325083 0.0 34.68339525781 137.325083 0.0 34.68339525781 137.325082 0.0</gml:posList>
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
