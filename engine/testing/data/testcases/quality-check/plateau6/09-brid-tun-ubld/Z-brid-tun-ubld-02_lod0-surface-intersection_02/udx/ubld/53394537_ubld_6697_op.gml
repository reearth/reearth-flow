<?xml version="1.0" encoding="UTF-8"?>
<!--
  LOD0 faces overlapping three ways.
  Three uro:UndergroundBuilding in shinjuku-ku (EPSG:6697) modelled at LOD0 only,
  one square face each, in this order in the file:
    - ubld_9539d7ac, 10 m by 4 m;
    - ubld_16e42fe1, overlapping the north-east corner of ubld_9539d7ac;
    - ubld_56a0b491, overlapping both, so one area is covered by all three faces
      and is reported once, as a pair of two of them.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.69103 139.69905 20</gml:lowerCorner>
			<gml:upperCorner>35.69112 139.69920 20</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:UndergroundBuilding gml:id="ubld_9539d7ac-7cb1-4d2d-b347-f4118e438c15">
			<core:creationDate>2024-03-15T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod0MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69103 139.69905 20 35.69103 139.69915 20 35.69107 139.69915 20 35.69107 139.69905 20 35.69103 139.69905 20</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</uro:UndergroundBuilding>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:UndergroundBuilding gml:id="ubld_16e42fe1-0823-44e4-bf90-c35fbbb5d746">
			<core:creationDate>2024-03-15T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod0MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69105 139.69913 20 35.69105 139.69920 20 35.69112 139.69920 20 35.69112 139.69913 20 35.69105 139.69913 20</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</uro:UndergroundBuilding>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:UndergroundBuilding gml:id="ubld_56a0b491-1477-479b-ae5d-fb7b9658e17e">
			<core:creationDate>2024-03-15T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod0MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69106 139.69914 20 35.69106 139.69918 20 35.69110 139.69918 20 35.69110 139.69914 20 35.69106 139.69914 20</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</uro:UndergroundBuilding>
	</core:cityObjectMember>
</core:CityModel>
