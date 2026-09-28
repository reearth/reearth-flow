<?xml version="1.0" encoding="UTF-8"?>
<!--
  No-error fixture for plateau6 05-luse-urf-area, CityGML 3.0 + i-UR 4.0.

  Two luse:LandUse within imizu-shi mesh 54377074 (EPSG:6697), each with a
  core:lod1MultiSurface holding one planar rectangle wound counter-clockwise in
  the inspection frame, so neither the face checks nor the orientation check
  reports anything.

  In CityGML 3.0 LandUse is a thematic surface, so its geometry hangs off
  core:lod1MultiSurface rather than the luse:lod1MultiSurface of 2.0.

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
			<gml:upperCorner>36.647230 137.052930 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<luse:LandUse gml:id="luse_d32a50b3-290a-4bbf-928c-a22e2ea2cf05">
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
						<gml:Polygon gml:id="poly_6aa89d60-0edb-439f-9b97-cc6e5e673827">
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
		<luse:LandUse gml:id="luse_78177ec6-ab35-486b-9857-b6477350a2d3">
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
						<gml:Polygon gml:id="poly_14f56a69-c7fd-4e20-9917-2365fd659f9f">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647200 137.052900 0 36.647200 137.052930 0 36.647230 137.052930 0 36.647230 137.052900 0 36.647200 137.052900 0</gml:posList>
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
