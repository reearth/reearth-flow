<?xml version="1.0" encoding="UTF-8"?>
<!--
  Collapsed triangle fixture for plateau6 06-fld, CityGML 3.0 + i-UR 4.0.

  One wtr:WaterBody holding a single triangle whose second and third corners are
  4.5 mm apart:

      A (36.64710000, 137.05280000)
      B (36.64710000, 137.05283000)
      B'(36.64710004, 137.05283000)   4.5 mm north of B

  Below the 1 cm at which two corners count as the same point, so the triangle
  collapses to a line and is reported as linear or point-like. Everything else
  about it is correct: four coordinates, closed, and counter-clockwise when read
  as (easting, northing), with an area that is tiny but not zero. Its three
  edges are the boundary of the surface, so no unshared edge is reported.
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
			<gml:upperCorner>36.647101 137.052830 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_2a90c471-8f36-4de2-b05c-71e8340af9b6">
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
				<wtr:WaterSurface gml:id="wtrs_c37e5b28-1a4f-4907-86d3-b20fe9c61d75">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9e04a1fb-63d2-4c58-b719-0a5d38e7c6f1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64710000 137.05280000 0 36.64710000 137.05283000 0 36.64710004 137.05283000 0 36.64710000 137.05280000 0</gml:posList>
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
