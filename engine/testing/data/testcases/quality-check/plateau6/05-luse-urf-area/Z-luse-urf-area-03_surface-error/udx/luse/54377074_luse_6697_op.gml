<?xml version="1.0" encoding="UTF-8"?>
<!--
  Surface error fixture for plateau6 05-luse-urf-area, CityGML 3.0 + i-UR 4.0.
  The surface check validates each face on its own and reports one row per failed check.

  Two luse:LandUse within imizu-shi mesh 54377074 (EPSG:6697), each with a
  core:lod1MultiSurface holding one broken face:
    - the first is a quadrilateral whose fourth corner sits 1 m above the plane of the
      other three, so the face is not planar. Planarity is only evaluated in a
      linear-unit frame, which makes this face a regression guard for the reprojection
      that precedes the checks.
    - the second is a bow-tie ring whose two halves cross, so the face self-intersects.
      Its two lobes have different areas, so the ring still encloses a non-zero area
      and the degeneracy check stays clean.

  Both rings are wound counter-clockwise in the inspection frame, so neither also
  trips the orientation check.

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
			<gml:upperCorner>36.647240 137.052940 1</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<luse:LandUse gml:id="luse_8922cd7d-5fcb-4a91-a687-17486c290189">
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
						<gml:Polygon gml:id="poly_080970d4-d6d4-486a-ae2f-0b623366ca51">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647100 137.052800 1 36.647100 137.052830 0 36.647130 137.052830 0 36.647130 137.052800 0 36.647100 137.052800 1</gml:posList>
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
		<luse:LandUse gml:id="luse_1c9f2425-578f-422c-89ff-61ea82e6e4cd">
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
						<gml:Polygon gml:id="poly_badd87f6-d6fb-4571-8e97-f08c4bf7ee10">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647200 137.052900 0 36.647215 137.052900 0 36.647200 137.052940 0 36.647240 137.052940 0 36.647200 137.052900 0</gml:posList>
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
