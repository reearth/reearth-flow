<?xml version="1.0" encoding="UTF-8"?>
<!--
  Surface error fixture for the area package of plateau6 05-luse-urf-area,
  CityGML 3.0 + i-UR 4.0. The same check runs on all three packages; this file
  covers the area branch of the output file prefix.

  The area model describes every zone that has no dedicated subclass with
  urf:Zone itself, so both features here are urf:Zone. Two of them within
  imizu-shi mesh 54377074 (EPSG:6697), each with a core:lod1MultiSurface holding
  one face:
    - the river zone carries a planar rectangle wound counter-clockwise in the
      inspection frame, so it reports nothing.
    - the port zone carries a bow-tie ring whose two halves cross, so the face
      self-intersects. Its two lobes have different areas, so the ring still
      encloses a non-zero area and the degeneracy check stays clean, and it is
      wound counter-clockwise so it does not also trip the orientation check.

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
		<urf:Zone gml:id="area_91d67586-db4a-47df-bafc-91d824916535">
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
						<gml:Polygon gml:id="poly_e1104101-6b82-4982-9a7a-33f79d49a3ab">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647100 137.052800 0 36.647100 137.052830 0 36.647130 137.052830 0 36.647130 137.052800 0 36.647100 137.052800 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<urf:function codeSpace="../../codelists/Zone_function.xml">0101</urf:function>
			<urf:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</urf:prefecture>
			<urf:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</urf:city>
		</urf:Zone>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<urf:Zone gml:id="area_a5754443-f071-4430-88bb-822f47052dd3">
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
						<gml:Polygon gml:id="poly_485fc0ec-8dee-4d91-bd0d-51ad73ca5e3a">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647200 137.052900 0 36.647215 137.052900 0 36.647200 137.052940 0 36.647240 137.052940 0 36.647200 137.052900 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<urf:function codeSpace="../../codelists/Zone_function.xml">0201</urf:function>
			<urf:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</urf:prefecture>
			<urf:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</urf:city>
		</urf:Zone>
	</core:cityObjectMember>
</core:CityModel>
