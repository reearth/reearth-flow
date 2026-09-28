<?xml version="1.0" encoding="UTF-8"?>
<!--
  Unshared edge fixture for plateau6 06-fld, CityGML 3.0 + i-UR 4.0, planning
  scale (L1). The surface is the one in Z-fld-05_unshared-edge_01.

  The L2 file next to this one covers only the left square, so its outline runs
  along B-C, right over the three unshared edges below. They must still be
  reported: an edge is only excused by the outline of its own scale.

  One wtr:WaterBody tiling a 9 m by 11 m rectangle with five triangles, meeting
  along B-C from one side and along B-M, M-C from the other — a T-junction, the
  classic way a triangulated surface stops being watertight. Vertices:

  A (36.647100, 137.052800)
  B (36.647100, 137.052900)
  C (36.647200, 137.052900)
  D (36.647200, 137.052800)
  E (36.647100, 137.053000)
  F (36.647200, 137.053000)
  M (36.647150, 137.052900)   midpoint of B-C

  Triangles: A-B-C, A-C-D, B-E-M, M-E-F, M-F-C. Each is written with four
  coordinates, closed, counter-clockwise when read as (easting, northing), and
  encloses a real area, so the other four checks report nothing.

  A-C, E-M and F-M are each used twice and cancel. What is left is the outline
  of the rectangle plus B-C, B-M and M-C: the left square sees one long edge
  where the right square sees two short ones, so none of the three finds a
  partner. All three run down the middle of the covered area rather than around
  its edge, so the outline test keeps them.

  An unshared edge is reported but is not counted as a failure, so the run still
  writes a qc_result_ok file.
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
			<gml:upperCorner>36.647200 137.053000 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_b3619d7e-52f0-4a84-8c27-de401b6a95f3">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">1</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_4e7c0b95-1d38-42fa-b6e1-057a9c38d216">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6f2b8134-0e5a-47c9-9d16-83b4a05e72cd">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647100 137.052800 0 36.647100 137.052900 0 36.647200 137.052900 0 36.647100 137.052800 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8d105c6a-39b7-4e21-a4f8-2b67e0913d54">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647100 137.052800 0 36.647200 137.052900 0 36.647200 137.052800 0 36.647100 137.052800 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_1a4e7f28-c063-4b95-80d7-5e39c2b6014f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647100 137.052900 0 36.647100 137.053000 0 36.647150 137.052900 0 36.647100 137.052900 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c92d6053-4a81-4f7e-b03c-17e5804a9b26">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647150 137.052900 0 36.647100 137.053000 0 36.647200 137.053000 0 36.647150 137.052900 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3b70e59c-8146-4d2a-9fb5-60c2a7134e08">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647150 137.052900 0 36.647200 137.053000 0 36.647200 137.052900 0 36.647150 137.052900 0</gml:posList>
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
