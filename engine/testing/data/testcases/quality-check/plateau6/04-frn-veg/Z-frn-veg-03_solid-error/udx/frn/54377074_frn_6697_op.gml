<?xml version="1.0" encoding="UTF-8"?>
<!--
  Solid error fixture for plateau6 04-frn-veg, CityGML 3.0 + i-UR 4.0.
  The shell-level checks split into two reports: a shell that is not a closed
  2-manifold is a 非水密立体, every other shell failure is a 立体のエラー.

  Two frn:CityFurniture within imizu-shi mesh 54377074 (EPSG:6697), each with a
  core:lod1Solid built from the same tetrahedron (one corner at the origin, the
  others one step east, north and up):
    - the first keeps only three of the four faces, so the base is missing and the
      three edges around it are left with a single face each. That is a
      ShellManifold failure, and those three edges are its reported positions.
    - the second is closed but every face is wound the other way, so the shell's
      normals point inward. ShellManifold and Orientation both pass, which is what
      lets ShellOrientation run and fail.

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
			<gml:lowerCorner>36.647300 137.053000 0</gml:lowerCorner>
			<gml:upperCorner>36.647440 137.053040 3</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="frn_7e2a4c15-3b60-4d89-9a17-8c0f5e2d6b34">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<frn:class codeSpace="../../codelists/CityFurniture_class.xml">1000</frn:class>
			<frn:function codeSpace="../../codelists/CityFurniture_function.xml">8150</frn:function>
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3f8d1a56-2c74-4e91-8b05-7d6a9c2f0e18">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647300 137.053040 0 36.647340 137.053000 0 36.647300 137.053000 3 36.647300 137.053040 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6b0e9d23-5a17-4c48-9f36-1e8b4d7a2c95">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647300 137.053000 0 36.647300 137.053000 3 36.647340 137.053000 0 36.647300 137.053000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c14a7f80-9d62-4b35-8e07-2a5c3f9b6d41">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647300 137.053000 0 36.647300 137.053040 0 36.647300 137.053000 3 36.647300 137.053000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
		</frn:CityFurniture>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="frn_5d9b0e47-8c23-4a16-9f58-3e7a1d4c6b02">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<frn:class codeSpace="../../codelists/CityFurniture_class.xml">1030</frn:class>
			<frn:function codeSpace="../../codelists/CityFurniture_function.xml">7200</frn:function>
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_2a6c8b39-4f01-4d57-8e92-5b3d7a0f1c64">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647400 137.053040 0 36.647400 137.053000 3 36.647440 137.053000 0 36.647400 137.053040 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9e3f5d12-7b48-4c60-8a25-6d1c0b4f3e97">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647400 137.053000 0 36.647440 137.053000 0 36.647400 137.053000 3 36.647400 137.053000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_4c7b1e05-6a93-4f28-9d31-0e5a8c2b7d46">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647400 137.053000 0 36.647400 137.053000 3 36.647400 137.053040 0 36.647400 137.053000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8d2a6f74-1c50-4b93-8e46-7f9b3d0a5c21">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647400 137.053000 0 36.647400 137.053040 0 36.647440 137.053000 0 36.647400 137.053000 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
		</frn:CityFurniture>
	</core:cityObjectMember>
</core:CityModel>
