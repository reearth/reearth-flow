<?xml version="1.0" encoding="UTF-8"?>
<!--
  plateau6 08-dem test data (CityGML 3.0 + i-UR 4.0).
  Coordinates are in EPSG:6697, within Japan Plane Rectangular CS zone IX.

  One dem:ReliefFeature whose only dem:reliefComponent is empty, so it has
  no triangle to check. No error is expected.
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
		<dem:ReliefFeature gml:id="dem_6e16e75e-3df6-49c9-95b5-ca7ad1c43cd2">
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
			<dem:reliefComponent/>
		</dem:ReliefFeature>
	</core:cityObjectMember>
</core:CityModel>
