<?xml version="1.0" encoding="UTF-8"?>
<!--
  plateau6 08-dem test data (CityGML 3.0 + i-UR 4.0).
  Coordinates are in EPSG:6697, within Japan Plane Rectangular CS zone IX.

  One dem:ReliefFeature with a dem:TINRelief holding a single valid triangle
  and a dem:BreaklineRelief whose line has a non-numeric token in its
  posList. Only triangles are checked, so no error is expected.
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
		<dem:ReliefFeature gml:id="dem_01cabc87-2292-489c-8259-c766bcc64f44">
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
				<dem:TINRelief gml:id="dem_53b46b12-a0ef-4e79-882f-889f84fcddf6">
					<gml:name>54391759</gml:name>
					<core:creationDate>0001-01-01T00:00:00</core:creationDate>
					<dem:lod>1</dem:lod>
					<dem:tin>
						<gml:TriangulatedSurface gml:id="tin_1088fcb3-61ff-4db5-8473-b62f51ca00ca">
							<gml:patches>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.1291944446 139.9938055554 15.9 36.1291944446 139.9937499999 15.5 36.129138889 139.9937499999 15.8 36.1291944446 139.9938055554 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
							</gml:patches>
						</gml:TriangulatedSurface>
					</dem:tin>
				</dem:TINRelief>
			</dem:reliefComponent>
			<dem:reliefComponent>
				<dem:BreaklineRelief gml:id="dem_7f3c2a1e-5b8d-4c6f-9e2a-3d4b5c6e7f80">
					<dem:lod>1</dem:lod>
					<dem:breaklines>
						<gml:MultiCurve>
							<gml:curveMember>
								<gml:LineString gml:id="line_2b9e4d6a-8c1f-4a3e-b5d7-9f0a1c2e3b4d">
									<gml:posList>36.1291944446 139.9938055554 15.9 36.129138889 x 15.8</gml:posList>
								</gml:LineString>
							</gml:curveMember>
						</gml:MultiCurve>
					</dem:breaklines>
				</dem:BreaklineRelief>
			</dem:reliefComponent>
		</dem:ReliefFeature>
	</core:cityObjectMember>
</core:CityModel>