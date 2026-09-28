<?xml version="1.0" encoding="UTF-8"?>
<!--
  No-error fixture for plateau6 06-fld, CityGML 3.0 + i-UR 4.0.

  One wtr:WaterBody within imizu-shi mesh 54377074 (EPSG:6697) whose
  core:lod1MultiSurface holds a square split into two triangles:

      D --- C
      | \   |        A (36.647100, 137.052800)
      |  \  |        B (36.647100, 137.052830)
      |   \ |        C (36.647130, 137.052830)
      A --- B        D (36.647130, 137.052800)

  A-B-C and A-C-D, each written with four coordinates and wound
  counter-clockwise when read as (easting, northing) — the winding the
  inspection frame calls correct.

  The diagonal A-C is used by both triangles and so cancels. The four outer
  edges are used once each, but they are the boundary of the surface and are
  removed by the outline test, so no unshared edge is reported either.

  In CityGML 3.0 a WaterBody is an occupied space and carries its surface
  through core:boundary / wtr:WaterSurface rather than the wtr:lod1MultiSurface
  of 2.0.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647100 137.052800 0</gml:lowerCorner>
			<gml:upperCorner>36.647130 137.052830 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_1f2b6e04-9c3a-4d51-8b77-2e0a5c916d38">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">3</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_5a7c3b81-64de-4f0a-9d25-c8e13f70b4a9">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3c9e1d75-0a46-4b82-95f1-6d27e0ba8c34">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647100 137.052800 0 36.647100 137.052830 0 36.647130 137.052830 0 36.647100 137.052800 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8b41c6de-2f70-4a39-b5e8-91d0357ac26f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647100 137.052800 0 36.647130 137.052830 0 36.647130 137.052800 0 36.647100 137.052800 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
</core:CityModel>
