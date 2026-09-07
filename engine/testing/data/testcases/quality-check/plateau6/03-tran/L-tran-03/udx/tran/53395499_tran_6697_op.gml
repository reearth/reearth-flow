<?xml version="1.0" encoding="UTF-8"?>
<!--
  Error fixture for plateau6 03-tran L-tran-03 (xlink reference completeness), CityGML 3.0 + i-UR 4.0.
  L-tran-03 detects, per LOD, boundary surfaces of a tran:Road whose gml:id is NOT referenced by the
  Road's own aggregate lodMultiSurface (a gml:CompositeSurface made of gml:surfaceMember xlink:href).

  A single tran:Road within saitama-shi mesh 53395499 (EPSG:6697):
    - One tran:TrafficSpace boundary tran:TrafficArea carries a LOD3 MultiSurface with two polygons:
      poly_4a0c6d25... (referenced by the Road aggregate) and poly_5b1d7e36... (NOT referenced -> error).
    - One tran:AuxiliaryTrafficSpace boundary tran:AuxiliaryTrafficArea carries a LOD3 MultiSurface with
      two polygons: poly_8e4a0b69... (referenced) and poly_9f5b1c70... (NOT referenced -> error).
    - The Road core:lod3MultiSurface aggregates the boundary polygons through a gml:CompositeSurface that
      references only one polygon per boundary. The two surfaceMember lines for the remaining polygons are
      intentionally commented out, so poly_5b1d7e36... and poly_9f5b1c70... become the two expected errors.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a CityGML 3.0 /
  i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:tran="http://www.opengis.net/citygml/transportation/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd
urn:oasis:names:tc:ciq:xal:3 ../../schemas/citygml/xAL/3.0/xAL.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.83100 139.61770 0</gml:lowerCorner>
			<gml:upperCorner>35.83110 139.61810 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1d7f3a92-4b6c-4e58-9a10-2c5e7b8d0f34">
			<gml:name>11100-tran-1</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod3MultiSurface>
				<gml:MultiSurface gml:id="ms_road_2c3d4e5f-6071-4283-8495-061728394051">
					<gml:surfaceMember>
						<gml:CompositeSurface gml:id="cs_road_3d4e5f60-7182-4394-8506-172839405162">
							<!-- tran:TrafficArea -->
							<gml:surfaceMember xlink:href="#poly_4a0c6d25-7e9f-4b81-8d43-5f8b0e1a3c67"></gml:surfaceMember>
							<!-- The next line is commented out, which should result in a reference error -->
							<!-- <gml:surfaceMember xlink:href="#poly_5b1d7e36-8f0a-4c92-9e54-6a9c1f2b4d78"></gml:surfaceMember> -->

							<!-- tran:AuxiliaryTrafficArea -->
							<gml:surfaceMember xlink:href="#poly_8e4a0b69-1c3d-4f25-8b87-9d2f4c5e7a01"></gml:surfaceMember>
							<!-- The next line is commented out, which should result in a reference error -->
							<!-- <gml:surfaceMember xlink:href="#poly_9f5b1c70-2d4e-4a36-9c98-0e3a5d6f8b12"></gml:surfaceMember> -->
						</gml:CompositeSurface>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
			<tran:class codeSpace="../../codelists/Road_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:section>
				<tran:Section gml:id="tran_d0a4cf6d-f268-4b16-9999-85b6295a3c3b">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_2e8a4b03-5c7d-4f69-8b21-3d6f8c9e1a45">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_3f9b5c14-6d8e-4a70-9c32-4e7a9d0f2b56">
									<core:lod3MultiSurface>
										<gml:MultiSurface gml:id="ms_traffic_0a1b2c3d-4e5f-4061-8273-849506172839">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_4a0c6d25-7e9f-4b81-8d43-5f8b0e1a3c67">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61770 0 35.83110 139.61770 0 35.83110 139.61780 0 35.83100 139.61780 0 35.83100 139.61770 0</gml:posList>
														</gml:LinearRing>
													</gml:exterior>
												</gml:Polygon>
											</gml:surfaceMember>
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_5b1d7e36-8f0a-4c92-9e54-6a9c1f2b4d78">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61780 0 35.83110 139.61780 0 35.83110 139.61790 0 35.83100 139.61790 0 35.83100 139.61780 0</gml:posList>
														</gml:LinearRing>
													</gml:exterior>
												</gml:Polygon>
											</gml:surfaceMember>
										</gml:MultiSurface>
									</core:lod3MultiSurface>
									<tran:function codeSpace="../../codelists/TrafficArea_function.xml">1000</tran:function>
								</tran:TrafficArea>
							</core:boundary>
							<tran:function codeSpace="../../codelists/TrafficSpace_function.xml">1000</tran:function>
							<tran:granularity>way</tran:granularity>
						</tran:TrafficSpace>
					</tran:trafficSpace>
					<tran:auxiliaryTrafficSpace>
						<tran:AuxiliaryTrafficSpace gml:id="tran_6c2e8f47-9a1b-4d03-8f65-7b0d2a3c5e89">
							<core:boundary>
								<tran:AuxiliaryTrafficArea gml:id="tran_7d3f9a58-0b2c-4e14-9a76-8c1e3b4d6f90">
									<core:lod3MultiSurface>
										<gml:MultiSurface gml:id="ms_aux_1b2c3d4e-5f60-4172-8384-950617283940">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_8e4a0b69-1c3d-4f25-8b87-9d2f4c5e7a01">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61790 0 35.83110 139.61790 0 35.83110 139.61800 0 35.83100 139.61800 0 35.83100 139.61790 0</gml:posList>
														</gml:LinearRing>
													</gml:exterior>
												</gml:Polygon>
											</gml:surfaceMember>
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_9f5b1c70-2d4e-4a36-9c98-0e3a5d6f8b12">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61800 0 35.83110 139.61800 0 35.83110 139.61810 0 35.83100 139.61810 0 35.83100 139.61800 0</gml:posList>
														</gml:LinearRing>
													</gml:exterior>
												</gml:Polygon>
											</gml:surfaceMember>
										</gml:MultiSurface>
									</core:lod3MultiSurface>
									<tran:function codeSpace="../../codelists/AuxiliaryTrafficArea_function.xml">1000</tran:function>
								</tran:AuxiliaryTrafficArea>
							</core:boundary>
							<tran:function codeSpace="../../codelists/AuxiliaryTrafficSpace_function.xml">1000</tran:function>
							<tran:granularity>way</tran:granularity>
						</tran:AuxiliaryTrafficSpace>
					</tran:auxiliaryTrafficSpace>
				</tran:Section>
			</tran:section>
		</tran:Road>
	</core:cityObjectMember>
</core:CityModel>
