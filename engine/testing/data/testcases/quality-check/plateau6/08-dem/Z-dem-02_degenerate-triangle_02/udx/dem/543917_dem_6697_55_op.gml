<?xml version="1.0" encoding="UTF-8"?>
<!--
  Triangles with no or almost no area on the ground, to compare the linear or
  point-like and orientation checks with the original implementation.
  Coordinates are in EPSG:6697. Orientation is read as (longitude, latitude).

  1. Three collinear corners. The area is exactly zero.
  2. Three almost collinear corners, wound clockwise. The third corner is
     1e-7 degrees off the line through the other two.
  3. Three almost collinear corners, wound counter-clockwise, 1e-7 degrees
     off the line.
  4. Two corners at the same ground position with heights 15.9 and 16.9.
  5. A ring of five positions A B C B A, which goes out and back and
     encloses no area.

  Expected: 1, 4 and 5 are linear or point-like, 2 is incorrectly oriented,
  and 5 also has an incorrect vertex count. The three edges of 4 have no
  shared edge. The original implementation does not report 1.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:dem="http://www.opengis.net/citygml/relief/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.1249999998 139.9874999996 13.2</gml:lowerCorner>
			<gml:upperCorner>36.1416671635 140 21.7</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<dem:ReliefFeature gml:id="dem_ac090e78-9554-41e1-9c08-22e5d0acbea7">
			<gml:name>54391759</gml:name>
			<core:creationDate>0001-01-01T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">700</urc:thematicSrcDesc>
					<urc:publicSurveyDataQualityAttribute>
						<urc:PublicSurveyDataQualityAttribute>
							<urc:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</urc:srcScaleLod1>
							<urc:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">022</urc:publicSurveySrcDescLod1>
						</urc:PublicSurveyDataQualityAttribute>
					</urc:publicSurveyDataQualityAttribute>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<dem:lod>1</dem:lod>
			<dem:reliefComponent>
				<dem:TINRelief gml:id="dem_50928ba4-b24e-435b-8cd2-949821d147a5">
					<gml:name>54391759</gml:name>
					<core:creationDate>0001-01-01T00:00:00</core:creationDate>
					<dem:lod>1</dem:lod>
					<dem:tin>
						<gml:TriangulatedSurface gml:id="tin_e5698532-7e56-4c88-8bec-942b4fe0bf81">
							<gml:patches>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.12918 139.9938 15.9 36.12914 139.99374 15.9 36.12916 139.99377 15.9 36.12918 139.9938 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.1285 139.993 15.9 36.1285 139.9935 15.9 36.1284999 139.99325 15.9 36.1285 139.993 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.128 139.993 15.9 36.128 139.9935 15.9 36.1280001 139.99325 15.9 36.128 139.993 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.1275 139.993 15.9 36.1275 139.9935 15.9 36.1275 139.9935 16.9 36.1275 139.993 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.127 139.993 15.9 36.127 139.9935 15.9 36.1272 139.99325 15.9 36.127 139.9935 15.9 36.127 139.993 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
							</gml:patches>
						</gml:TriangulatedSurface>
					</dem:tin>
				</dem:TINRelief>
			</dem:reliefComponent>
		</dem:ReliefFeature>
	</core:cityObjectMember>
</core:CityModel>
