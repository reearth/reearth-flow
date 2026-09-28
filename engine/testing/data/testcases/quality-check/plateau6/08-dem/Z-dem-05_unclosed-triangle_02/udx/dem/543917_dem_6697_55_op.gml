<?xml version="1.0" encoding="UTF-8"?>
<!--
  plateau6 08-dem test data (CityGML 3.0 + i-UR 4.0).
  Coordinates are in EPSG:6697, within Japan Plane Rectangular CS zone IX.

  One dem:ReliefFeature with one dem:TINRelief holding three separate
  triangles, each written with four positions whose last position differs
  slightly from the first. Positions are compared after rounding latitude
  and longitude to 12 decimal places and height to 4.

  1. The last height is 15.90004 against 15.9. Rounded, they match, so the
     triangle is closed.
  2. The last latitude is 36.1290944446001 against 36.1290944446. Rounded,
     they match, so the triangle is closed.
  3. The last height is 15.9001 against 15.9. Rounded, they still differ, so
     the triangle is not closed.

  One unclosed triangle is expected.
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
		<dem:ReliefFeature gml:id="dem_85090469-949f-4187-b9d3-c2c45fc6f5a8">
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
				<dem:TINRelief gml:id="dem_c76a952c-7015-45bd-8c8e-c1761893b738">
					<gml:name>54391759</gml:name>
					<core:creationDate>0001-01-01T00:00:00</core:creationDate>
					<dem:lod>1</dem:lod>
					<dem:tin>
						<gml:TriangulatedSurface gml:id="tin_e8a0e6e3-8898-44ec-936f-516c2f3c5572">
							<gml:patches>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.1291944446 139.9938055554 15.9 36.1291944446 139.9937499999 15.5 36.129138889 139.9937499999 15.8 36.1291944446 139.9938055554 15.90004</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.1290944446 139.9938055554 15.9 36.1290944446 139.9937499999 15.5 36.129038889 139.9937499999 15.8 36.1290944446001 139.9938055554 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.1289944446 139.9938055554 15.9 36.1289944446 139.9937499999 15.5 36.128938889 139.9937499999 15.8 36.1289944446 139.9938055554 15.9001</gml:posList>
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
