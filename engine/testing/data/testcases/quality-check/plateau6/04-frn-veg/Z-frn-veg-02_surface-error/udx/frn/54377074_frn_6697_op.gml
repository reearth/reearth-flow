<?xml version="1.0" encoding="UTF-8"?>
<!--
  Surface error fixture for plateau6 04-frn-veg, CityGML 3.0 + i-UR 4.0.
  The surface check validates each face on its own and reports one row per failed check.

  Two frn:CityFurniture within imizu-shi mesh 54377074 (EPSG:6697), each with a
  core:lod3MultiSurface holding one broken face:
    - the first is a quadrilateral whose fourth corner sits 1 m above the plane of the
      other three, so the face is not planar. Planarity is only evaluated in a
      linear-unit frame, which makes this face a regression guard for the reprojection
      that precedes the checks.
    - the second is a bow-tie ring whose two halves cross, so the face self-intersects.
      Its two lobes have different areas, so the ring still encloses a non-zero area
      and the degeneracy check stays clean.

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
			<gml:lowerCorner>36.647100 137.052800 0</gml:lowerCorner>
			<gml:upperCorner>36.647240 137.052940 1</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="frn_1c6f4a82-9d30-4b75-8e21-5a7c3f09b6d4">
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
						<gml:Polygon gml:id="poly_4d8b2f10-7c65-4a93-9f02-6e1a5b3c8d74">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647100 137.052800 1 36.647100 137.052830 0 36.647130 137.052830 0 36.647130 137.052800 0 36.647100 137.052800 1</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</frn:CityFurniture>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="frn_0b93e5d7-46a1-4c28-8d5f-2f7b9a0c1e63">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod3>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<frn:class codeSpace="../../codelists/CityFurniture_class.xml">1030</frn:class>
			<frn:function codeSpace="../../codelists/CityFurniture_function.xml">7200</frn:function>
			<core:lod3MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_9a2c7e34-1b58-4f60-8c9d-3e0f6a4b7d15">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647200 137.052900 0 36.647240 137.052940 0 36.647200 137.052940 0 36.647215 137.052900 0 36.647200 137.052900 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</frn:CityFurniture>
	</core:cityObjectMember>
</core:CityModel>
