<?xml version="1.0" encoding="UTF-8"?>
<!--
  A solid of a generic city object that does not close (EPSG:6697).
  - GenericOccupiedSpace: a LOD1 box solid whose top face is missing, so the
    four upper edges of the walls are shared by no other face.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:gen="http://www.opengis.net/citygml/generics/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>33.7175 130.481 0</gml:lowerCorner>
			<gml:upperCorner>33.7185 130.482 10</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<gen:GenericOccupiedSpace gml:id="gen_a2b3c4d5-e6f7-4081-9a2b-3c4d5e6f7081">
			<gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
			<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7175 130.481 0 33.7185 130.481 0 33.7185 130.482 0 33.7175 130.482 0 33.7175 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7175 130.481 0 33.7175 130.482 0 33.7175 130.482 10 33.7175 130.481 10 33.7175 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7175 130.482 0 33.7185 130.482 0 33.7185 130.482 10 33.7175 130.482 10 33.7175 130.482 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7185 130.482 0 33.7185 130.481 0 33.7185 130.481 10 33.7185 130.482 10 33.7185 130.482 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7185 130.481 0 33.7175 130.481 0 33.7175 130.481 10 33.7185 130.481 10 33.7185 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
		</gen:GenericOccupiedSpace>
	</core:cityObjectMember>
</core:CityModel>
