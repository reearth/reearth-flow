<?xml version="1.0" encoding="UTF-8"?>
<!--
  Vegetation fixture for plateau6 04-frn-veg, CityGML 3.0 + i-UR 4.0. It covers the
  veg package, whose results go to the 04-2_植生 files, with one of each vegetation
  feature type:
    - veg:SolitaryVegetationObject carries a core:lod3MultiSurface whose single
      quadrilateral has one corner 1 m above the plane of the other three, so the
      face is not planar.
    - veg:PlantCover carries a core:lod1Solid built from a tetrahedron with the base
      face left out, so the shell is not closed and the three edges around the
      opening are its reported positions.

  CityGML 3.0 gives vegetation no lod1MultiSurface and no boundary surfaces, so LOD1
  here has to be a solid. Both objects sit in imizu-shi mesh 54377074 (EPSG:6697).

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647500 137.053000 0</gml:lowerCorner>
			<gml:upperCorner>36.647640 137.053040 3</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<veg:SolitaryVegetationObject gml:id="veg_6a3f8d21-4c97-4b05-8e63-1d7a2f0c9b54">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod3>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<veg:class codeSpace="../../codelists/SolitaryVegetationObject_class.xml">1</veg:class>
			<veg:function codeSpace="../../codelists/SolitaryVegetationObject_function.xml">11</veg:function>
			<core:lod3MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_7c1b4e93-0a58-4d26-9f74-3b6e2a0d5c81">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647500 137.053000 1 36.647500 137.053030 0 36.647530 137.053030 0 36.647530 137.053000 0 36.647500 137.053000 1</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</veg:SolitaryVegetationObject>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<veg:PlantCover gml:id="veg_0e5c9a76-3d12-4f80-8b47-6a1f3c8d2e95">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<veg:class codeSpace="../../codelists/PlantCover_class.xml">1</veg:class>
			<veg:averageHeight uom="m">3.0</veg:averageHeight>
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_1f8a3c60-5e29-4b74-9d03-7c2b6a0e4f18">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647600 137.053040 0 36.647640 137.053000 0 36.647600 137.053000 3 36.647600 137.053040 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5b0d2f84-9c16-4a37-8e50-1d7f4b3a6c29">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647600 137.053000 0 36.647600 137.053000 3 36.647640 137.053000 0 36.647600 137.053000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9d4e7b12-0f63-4c85-8a29-3e6b1d5f0a74">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647600 137.053000 0 36.647600 137.053040 0 36.647600 137.053000 3 36.647600 137.053000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
		</veg:PlantCover>
	</core:cityObjectMember>
</core:CityModel>
