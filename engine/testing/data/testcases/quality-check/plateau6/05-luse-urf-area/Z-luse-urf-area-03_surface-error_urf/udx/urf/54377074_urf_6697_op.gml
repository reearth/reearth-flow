<?xml version="1.0" encoding="UTF-8"?>
<!--
  Surface error fixture for the urf package of plateau6 05-luse-urf-area,
  CityGML 3.0 + i-UR 4.0. The same check runs on all three packages; this file
  covers the urf branch of the output file prefix.

  Two urban planning decision features within imizu-shi mesh 54377074
  (EPSG:6697), each with a core:lod1MultiSurface holding one face:
    - urf:AreaClassification carries a planar rectangle wound counter-clockwise
      in the inspection frame, so it reports nothing.
    - urf:UrbanPlanningArea carries a bow-tie ring whose two halves cross, so the
      face self-intersects. Its two lobes have different areas, so the ring still
      encloses a non-zero area and the degeneracy check stays clean, and it is
      wound counter-clockwise so it does not also trip the orientation check.

  In i-UR 4.0 every urf feature is a thematic surface, so its geometry hangs off
  core:lod1MultiSurface rather than the urf:lod1MultiSurface of 3.x.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:urf="https://www.geospatial.jp/iur/urf/4.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
https://www.geospatial.jp/iur/urf/4.0 ../../schemas/iur/urf/4.0/urbanFunction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647100 137.052800 0</gml:lowerCorner>
			<gml:upperCorner>36.647240 137.052940 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<urf:AreaClassification gml:id="urf_b408c1dd-a93c-40a1-8b3c-26c225eddb0a">
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
						<gml:Polygon gml:id="poly_7551dce1-1662-4b96-bcf8-f9b5444bc9d2">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647100 137.052800 0 36.647100 137.052830 0 36.647130 137.052830 0 36.647130 137.052800 0 36.647100 137.052800 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<urf:function codeSpace="../../codelists/Common_areaClassificationType.xml">22</urf:function>
			<urf:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</urf:prefecture>
			<urf:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</urf:city>
		</urf:AreaClassification>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<urf:UrbanPlanningArea gml:id="urf_fb7f1886-e859-4c9f-a5f2-9d7e02e4580b">
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
						<gml:Polygon gml:id="poly_1170eafe-a865-4764-9b1e-0dadb9a18c82">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647200 137.052900 0 36.647215 137.052900 0 36.647200 137.052940 0 36.647240 137.052940 0 36.647200 137.052900 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<urf:function codeSpace="../../codelists/Common_urbanPlanType.xml">21</urf:function>
			<urf:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</urf:prefecture>
			<urf:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</urf:city>
			<urf:areaClassification codeSpace="../../codelists/Common_availabilityType.xml">1</urf:areaClassification>
		</urf:UrbanPlanningArea>
	</core:cityObjectMember>
</core:CityModel>
