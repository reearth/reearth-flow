<?xml version="1.0" encoding="UTF-8"?>
<!--
  C01 (duplicated gml:id) data for plateau6 03-tran (CityGML 3.0 + i-UR 4.0), file 2 of 2.
  The tran:Road here carries the same gml:id as the tran:Road in 53395499_tran_6697_op.gml,
  so the pair is expected to be reported as a duplicate across the two files. Every other
  gml:id is unique, and the two roads sit in different meshes so that they do not touch and
  no adjacent-surface overlap is reported alongside the duplicate.
  Coordinates are within saitama-shi mesh 53395590 (EPSG:6697).
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:tran="http://www.opengis.net/citygml/transportation/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd
urn:oasis:names:tc:ciq:xal:3 ../../schemas/citygml/xAL/3.0/xAL.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.8310 139.6260 0</gml:lowerCorner>
			<gml:upperCorner>35.8311 139.6261 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9b3c5d7e-1f24-4a86-b0c9-2e5f7a1b3d68">
			<gml:name>11100-tran-2</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<tran:class codeSpace="../../codelists/Road_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:section>
				<tran:Section gml:id="tran_449dea01-209f-4002-9a89-083cc1db86e5">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_5e6f7081-92a3-4b4c-9d5e-6f708293a4b5">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_6f708192-a3b4-4c5d-8e6f-708293a4b5c6">
									<core:lod1MultiSurface>
										<gml:MultiSurface gml:id="ms_708192a3-b4c5-4d6e-9f70-8293a4b5c6d7">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_8192a3b4-c5d6-4e7f-8081-93a4b5c6d7e8">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.8310 139.6261 0 35.8311 139.6261 0 35.8311 139.6260 0 35.8310 139.6260 0 35.8310 139.6261 0</gml:posList>
														</gml:LinearRing>
													</gml:exterior>
												</gml:Polygon>
											</gml:surfaceMember>
										</gml:MultiSurface>
									</core:lod1MultiSurface>
									<tran:function codeSpace="../../codelists/TrafficArea_function.xml">1000</tran:function>
								</tran:TrafficArea>
							</core:boundary>
							<tran:function codeSpace="../../codelists/TrafficSpace_function.xml">1000</tran:function>
							<tran:granularity>way</tran:granularity>
						</tran:TrafficSpace>
					</tran:trafficSpace>
				</tran:Section>
			</tran:section>
		</tran:Road>
	</core:cityObjectMember>
</core:CityModel>
