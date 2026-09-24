<?xml version="1.0" encoding="UTF-8"?>
<!--
  No-error fixture for plateau6 06-fld, inland flooding package, CityGML 3.0 + i-UR 4.0.

  The same square split into two correctly wound triangles as the fld no-error
  fixture. What this case is really for is the output file name: every package
  writes its result sheet under its own prefix, and a package missing from that
  table has to stop the run rather than share another package's file.

      D --- C
      | \   |        A (36.647100, 137.052800)
      |  \  |        B (36.647100, 137.052830)
      |   \ |        C (36.647130, 137.052830)
      A --- B        D (36.647130, 137.052800)
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647100 137.052800 0</gml:lowerCorner>
			<gml:upperCorner>36.647130 137.052830 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="ifld_31f44920-a71a-49f8-9cbd-b1060e761756">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:InlandFloodingRiskAttribute>
					<urc:description codeSpace="../../../codelists/InlandFloodingRiskAttribute_description.xml">1</urc:description>
					<urc:rank codeSpace="../../../codelists/InlandFloodingRiskAttribute_rank.xml">1</urc:rank>
				</urc:InlandFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_6a66c238-b8de-48dd-a721-a978f248058d">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ccb999a0-1fbb-47ed-b1d6-39086ca6d0f1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647100 137.052800 0 36.647100 137.052830 0 36.647130 137.052830 0 36.647100 137.052800 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_7bb37245-3e74-4b04-8acc-a16fce0297b1">
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
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
</core:CityModel>
