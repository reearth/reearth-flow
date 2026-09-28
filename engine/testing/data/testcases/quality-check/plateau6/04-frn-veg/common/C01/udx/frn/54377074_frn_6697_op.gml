<?xml version="1.0" encoding="UTF-8"?>
<!--
  C01 fixture for plateau6 04-frn-veg, CityGML 3.0 + i-UR 4.0. This file and its
  neighbour in the archive carry the same gml:id on their frn:CityFurniture, which is
  a gml:id duplication across the city model. Both objects are otherwise valid, so the
  duplication is the only finding.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647100 137.052800 2</gml:lowerCorner>
			<gml:upperCorner>36.647230 137.052930 2</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="frn_29da0ec1-d3f2-4482-bca3-b6b0a57a3519">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod3>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<frn:class codeSpace="../../codelists/CityFurniture_class.xml">1000</frn:class>
			<frn:function codeSpace="../../codelists/CityFurniture_function.xml">8150</frn:function>
			<core:lod3MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_5e5a4a01-6a0c-4de5-9a4f-3f8a0d0a3c11">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647100 137.052800 2 36.647100 137.052830 2 36.647130 137.052830 2 36.647100 137.052800 2</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_2b1f0c77-63bd-4f39-9a05-0d5f2f7e9a42">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647100 137.052800 2 36.647130 137.052830 2 36.647130 137.052800 2 36.647100 137.052800 2</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</frn:CityFurniture>
	</core:cityObjectMember>
</core:CityModel>
