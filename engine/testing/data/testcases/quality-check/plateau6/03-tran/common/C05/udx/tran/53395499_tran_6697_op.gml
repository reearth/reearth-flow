<?xml version="1.0" encoding="UTF-8"?>
<!--
  C05 (required attribute missing) data for plateau6 03-tran (CityGML 3.0 + i-UR 4.0).
  Same single tran:Road as the no-error reference, except that core:creationDate is
  omitted. The acquisition item list marks core:creationDate as required for tran:Road,
  so exactly one C05 error is expected.
  Coordinates are within saitama-shi mesh 53395499 (EPSG:6697).
  The acquisition item list is the PLATEAU5 (i-UR 3.x) one adjusted for CityGML 3.0;
  see the README next to it.
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
			<gml:lowerCorner>35.8310 139.6177 0</gml:lowerCorner>
			<gml:upperCorner>35.8311 139.6178 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_422a4508-0e87-427a-a3db-a5f1e5de84c0">
			<gml:name>11100-tran-1</gml:name>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<tran:class codeSpace="../../codelists/Road_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:section>
				<tran:Section gml:id="tran_1bae6eef-d0bb-4823-8307-1a3322043735">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_56f5a4ba-6e3e-4830-a5e6-c5e39331039b">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_50d704fa-a987-48ab-9e30-0fdc40feaa02">
									<core:lod1MultiSurface>
										<gml:MultiSurface gml:id="ms_50d704fa-a987-48ab-9e30-0fdc40feaa02">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_50d704fa-a987-48ab-9e30-0fdc40feaa02">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.8310 139.6178 0 35.8311 139.6178 0 35.8311 139.6177 0 35.8310 139.6177 0 35.8310 139.6178 0</gml:posList>
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
