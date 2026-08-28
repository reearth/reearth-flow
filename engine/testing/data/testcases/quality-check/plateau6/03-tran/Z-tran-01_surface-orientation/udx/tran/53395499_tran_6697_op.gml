<?xml version="1.0" encoding="UTF-8"?>
<!--
  Error fixture for plateau6 03-tran surface orientation, CityGML 3.0 + i-UR 4.0.
  The orientation check reports every face whose exterior ring is wound against the
  coordinate frame once the face is flattened to 2D.

  A single tran:Road within saitama-shi mesh 53395499 (EPSG:6697) carrying two boundaries,
  each holding a pair of polygons that are identical except for their winding:
    - tran:TrafficArea, LOD1: poly_0f3c... is wound correctly, poly_1a4d... is reversed.
    - tran:AuxiliaryTrafficArea, LOD3: poly_2b5e... is wound correctly, poly_3c6f... is reversed.
  So exactly one face per LOD is expected to be reported.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a CityGML 3.0 /
  i-UR 4.0 objectlist is not yet standardized.
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
			<gml:lowerCorner>35.83100 139.61770 0</gml:lowerCorner>
			<gml:upperCorner>35.83110 139.61805 2.5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5e2b8c41-7a09-4d63-8f15-2c4e6a8b0d37">
			<gml:name>11100-tran-1</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<tran:trafficSpace>
				<tran:TrafficSpace gml:id="tran_6f3c9d52-8b1a-4e74-9026-3d5f7b9c1e48">
					<core:boundary>
						<tran:TrafficArea gml:id="tran_7a4d0e63-9c2b-4f85-8137-4e6a8c0d2f59">
							<core:lod1MultiSurface>
								<gml:MultiSurface gml:id="ms_8b5e1f74-0d3c-4096-9248-5f7b9d1e3a60">
									<gml:surfaceMember>
										<gml:Polygon gml:id="poly_0f3c7a51-2d6b-4e84-9137-5a8c0e2b4d69">
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.83100 139.61785 0 35.83110 139.61785 0 35.83110 139.61770 0 35.83100 139.61770 0 35.83100 139.61785 0</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<!-- Same rectangle, reversed winding. -->
										<gml:Polygon gml:id="poly_1a4d8b62-3e7c-4f95-8248-6b9d1f3c5e70">
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.83100 139.61785 0 35.83100 139.61770 0 35.83110 139.61770 0 35.83110 139.61785 0 35.83100 139.61785 0</gml:posList>
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
			<tran:auxiliaryTrafficSpace>
				<tran:AuxiliaryTrafficSpace gml:id="tran_9c6f2a85-1e4d-4a07-8359-7c0e2b4d6f81">
					<core:boundary>
						<tran:AuxiliaryTrafficArea gml:id="tran_0d7a3b96-2f5e-4b18-946a-8d1f3c5e7a92">
							<core:lod3MultiSurface>
								<gml:MultiSurface gml:id="ms_1e8b4c07-3a6f-4c29-857b-9e2a4d6f8b03">
									<gml:surfaceMember>
										<gml:Polygon gml:id="poly_2b5e9c73-4f8a-4d06-9159-7c0e2a4b6d81">
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.83100 139.61805 2.5 35.83110 139.61805 2.5 35.83110 139.61790 2.5 35.83100 139.61790 2.5 35.83100 139.61805 2.5</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<!-- Same rectangle, reversed winding. -->
										<gml:Polygon gml:id="poly_3c6f0d84-5a9b-4e17-826a-8d1f3b5c7e92">
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.83100 139.61805 2.5 35.83100 139.61790 2.5 35.83110 139.61790 2.5 35.83110 139.61805 2.5 35.83100 139.61805 2.5</gml:posList>
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
			<tran:class codeSpace="../../codelists/Road_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
		</tran:Road>
	</core:cityObjectMember>
</core:CityModel>
