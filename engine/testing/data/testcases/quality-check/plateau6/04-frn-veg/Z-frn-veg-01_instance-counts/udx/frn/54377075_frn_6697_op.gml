<?xml version="1.0" encoding="UTF-8"?>
<!--
  Second file of the instance count fixture for plateau6 04-frn-veg. It holds a single
  frn:CityFurniture carrying a core:lod1Solid, so its expected count is 1 while its
  neighbour 54377074_frn_6697_op.gml counts 2. A count that ignored the file would
  report 3 on one row instead of the two rows this test expects.

  The geometry is valid: a finding would belong to another test, not this one.

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
			<gml:lowerCorner>36.647500 137.052900 0</gml:lowerCorner>
			<gml:upperCorner>36.647540 137.052940 2.5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="frn_61249513-56b0-4a5e-b577-4b377171106b">
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
				<gml:Solid gml:id="solid_23cf7aeb-7d6d-4525-a51b-37f984a6ae82" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647500 137.052940 0 36.647500 137.052900 0 36.647540 137.052900 0 36.647540 137.052940 0 36.647500 137.052940 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647500 137.052940 0 36.647540 137.052940 0 36.647540 137.052940 2.5 36.647500 137.052940 2.5 36.647500 137.052940 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647540 137.052940 0 36.647540 137.052900 0 36.647540 137.052900 2.5 36.647540 137.052940 2.5 36.647540 137.052940 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647540 137.052900 0 36.647500 137.052900 0 36.647500 137.052900 2.5 36.647540 137.052900 2.5 36.647540 137.052900 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647500 137.052900 0 36.647500 137.052940 0 36.647500 137.052940 2.5 36.647500 137.052900 2.5 36.647500 137.052900 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647500 137.052940 2.5 36.647540 137.052940 2.5 36.647540 137.052900 2.5 36.647500 137.052900 2.5 36.647500 137.052940 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
		</frn:CityFurniture>
	</core:cityObjectMember>
</core:CityModel>
