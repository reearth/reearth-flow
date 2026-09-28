<?xml version="1.0" encoding="UTF-8"?>
<!--
  Instance count fixture for plateau6 04-frn-veg, CityGML 3.0 + i-UR 4.0. The count
  is per feature type per file, and one city object counts once however many LODs it
  carries. This file holds two frn:CityFurniture, so its expected count is 2:
    - frn_459936be-... carries both a core:lod1Solid and a core:lod3MultiSurface.
      The LOD splitter emits it twice, and only the duplicate filter keeps it from
      being counted twice.
    - frn_302db804-... carries a core:lod3MultiSurface alone.

  Its neighbour 54377075_frn_6697_op.gml holds one more, which is what makes the
  count group per file rather than per archive.

  Every geometry here is valid: a finding would belong to another test, not this one.
  The objects sit in imizu-shi mesh 54377074 (EPSG:6697).

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
			<gml:lowerCorner>36.647300 137.052700 0</gml:lowerCorner>
			<gml:upperCorner>36.647440 137.052840 2.5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="frn_459936be-8559-4fe2-80f3-94a6361cb40c">
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
			<core:lod1Solid>
				<gml:Solid gml:id="solid_5f6cd78c-2f2b-4455-8207-149e2d23954f" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647300 137.052740 0 36.647300 137.052700 0 36.647340 137.052700 0 36.647340 137.052740 0 36.647300 137.052740 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647300 137.052740 0 36.647340 137.052740 0 36.647340 137.052740 2.5 36.647300 137.052740 2.5 36.647300 137.052740 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647340 137.052740 0 36.647340 137.052700 0 36.647340 137.052700 2.5 36.647340 137.052740 2.5 36.647340 137.052740 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647340 137.052700 0 36.647300 137.052700 0 36.647300 137.052700 2.5 36.647340 137.052700 2.5 36.647340 137.052700 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647300 137.052700 0 36.647300 137.052740 0 36.647300 137.052740 2.5 36.647300 137.052700 2.5 36.647300 137.052700 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647300 137.052740 2.5 36.647340 137.052740 2.5 36.647340 137.052700 2.5 36.647300 137.052700 2.5 36.647300 137.052740 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<core:lod3MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_f753a6c0-2fd9-49ac-93f4-024c45d481e2">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647300 137.052700 2.5 36.647300 137.052740 2.5 36.647340 137.052740 2.5 36.647300 137.052700 2.5</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_0d40c4b2-72a3-40b6-8806-be0e37da6f56">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647300 137.052700 2.5 36.647340 137.052740 2.5 36.647340 137.052700 2.5 36.647300 137.052700 2.5</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</frn:CityFurniture>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="frn_302db804-aa7b-49fe-97ea-cb1ffe417477">
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
						<gml:Polygon gml:id="poly_7b55e268-29e3-417d-a02c-b3d8fa158857">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647400 137.052800 2 36.647400 137.052840 2 36.647440 137.052840 2 36.647400 137.052800 2</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_a753c887-d0d5-47fd-b342-7bf1b6d4df26">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647400 137.052800 2 36.647440 137.052840 2 36.647440 137.052800 2 36.647400 137.052800 2</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</frn:CityFurniture>
	</core:cityObjectMember>
</core:CityModel>
