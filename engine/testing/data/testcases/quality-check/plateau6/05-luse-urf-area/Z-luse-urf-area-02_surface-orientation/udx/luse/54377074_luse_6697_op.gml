<?xml version="1.0" encoding="UTF-8"?>
<!--
  Error fixture for plateau6 05-luse-urf-area surface orientation, CityGML 3.0 + i-UR 4.0.
  The orientation check reports every face whose exterior ring is wound against the
  coordinate frame once the face is flattened to 2D.

  Two luse:LandUse within imizu-shi mesh 54377074 (EPSG:6697) holding the same
  rectangle shape at different places:
    - luse_a40a18f2... is wound counter-clockwise in the inspection frame, which is correct.
    - luse_0a024e99... is wound clockwise, so exactly this one is expected to be reported.

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
			<gml:upperCorner>36.647230 137.052830 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<luse:LandUse gml:id="luse_a40a18f2-cdd7-4274-bf56-3f9a6092c004">
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
						<gml:Polygon gml:id="poly_ed3fd0d3-c81b-4506-9d84-a97bec363353">
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
		<luse:LandUse gml:id="luse_0a024e99-52af-4476-afdb-ae6ab52711f5">
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
						<!-- Same rectangle shape as above, reversed winding. -->
						<gml:Polygon gml:id="poly_6f270260-8fcd-49bf-a0a4-3349714cfa7a">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647200 137.052800 0 36.647230 137.052800 0 36.647230 137.052830 0 36.647200 137.052830 0 36.647200 137.052800 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<luse:class codeSpace="../../codelists/Common_landUseType.xml">204</luse:class>
		</luse:LandUse>
	</core:cityObjectMember>
</core:CityModel>
