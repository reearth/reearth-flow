<?xml version="1.0" encoding="UTF-8"?>
<!--
  Instance count fixture for plateau6 05-luse-urf-area, CityGML 3.0 + i-UR 4.0.
  The count is taken per file and per feature type, so this archive holds two
  files with a different number of city objects each.

  This file carries three luse:LandUse within imizu-shi mesh 54377074
  (EPSG:6697), each a planar rectangle wound counter-clockwise in the inspection
  frame so that no face check reports anything.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:luse="http://www.opengis.net/citygml/landuse/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647100 137.052800 0</gml:lowerCorner>
			<gml:upperCorner>36.647330 137.052830 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<luse:LandUse gml:id="luse_c758efa4-0795-482e-ad9d-73c79d5d7e0c">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_5b7f069e-a5d5-4f3d-bc4a-d8d15f86a102">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647100 137.052800 0 36.647100 137.052830 0 36.647130 137.052830 0 36.647130 137.052800 0 36.647100 137.052800 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<luse:class codeSpace="../../codelists/Common_landUseType.xml">202</luse:class>
		</luse:LandUse>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<luse:LandUse gml:id="luse_423bf08a-8698-4874-8190-07cfcd1fbe1b">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_d4d02a81-4b64-469b-9b72-f5527f192c12">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647200 137.052800 0 36.647200 137.052830 0 36.647230 137.052830 0 36.647230 137.052800 0 36.647200 137.052800 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<luse:class codeSpace="../../codelists/Common_landUseType.xml">204</luse:class>
		</luse:LandUse>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<luse:LandUse gml:id="luse_bcc49c20-eb9a-4d7d-bf25-07f3cab3d82a">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_d089cf14-524d-4631-9529-c9f7cb596436">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647300 137.052800 0 36.647300 137.052830 0 36.647330 137.052830 0 36.647330 137.052800 0 36.647300 137.052800 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<luse:class codeSpace="../../codelists/Common_landUseType.xml">205</luse:class>
		</luse:LandUse>
	</core:cityObjectMember>
</core:CityModel>
