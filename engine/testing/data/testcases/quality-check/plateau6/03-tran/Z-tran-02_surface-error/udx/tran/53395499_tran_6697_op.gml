<?xml version="1.0" encoding="UTF-8"?>
<!--
  Error fixture for plateau6 03-tran surface errors, CityGML 3.0 + i-UR 4.0.
  The surface check validates each face on its own and reports one row per failed check.

  A single tran:Road within saitama-shi mesh 53395499 (EPSG:6697) carrying two LOD1 boundaries:
    - tran:TrafficArea holds a quadrilateral whose fourth corner sits 1 m above the plane of the
      other three, so the face is not planar. Planarity is only evaluated in a linear-unit frame,
      which makes this face a regression guard for the reprojection that precedes the checks.
    - tran:AuxiliaryTrafficArea holds a bow-tie ring whose two halves cross, so the face
      self-intersects.

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
			<gml:upperCorner>35.83110 139.61805 1</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4a1b7c30-6e9d-4f52-8a04-1b3d5f7a9c26">
			<gml:name>11100-tran-1</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<tran:trafficSpace>
				<tran:TrafficSpace gml:id="tran_5b2c8d41-7f0e-4a63-9b15-2c4e6a8b0d37">
					<core:boundary>
						<tran:TrafficArea gml:id="tran_6c3d9e52-8a1f-4b74-8c26-3d5f7b9c1e48">
							<core:lod1MultiSurface>
								<gml:MultiSurface gml:id="ms_7d4e0f63-9b2a-4c85-9d37-4e6a8c0d2f59">
									<gml:surfaceMember>
										<!-- The first corner is 1 m above the plane of the other three. -->
										<gml:Polygon gml:id="poly_8e5f1a74-0c3b-4d96-8e48-5f7b9d1e3a60">
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.83100 139.61785 1 35.83110 139.61785 0 35.83110 139.61770 0 35.83100 139.61770 0 35.83100 139.61785 1</gml:posList>
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
				<tran:AuxiliaryTrafficSpace gml:id="tran_9f6a2b85-1d4c-4e07-9f59-6a8c0e2b4d71">
					<core:boundary>
						<tran:AuxiliaryTrafficArea gml:id="tran_0a7b3c96-2e5d-4f18-8a6a-7b9d1f3c5e82">
							<core:lod1MultiSurface>
								<gml:MultiSurface gml:id="ms_1b8c4d07-3f6e-4a29-9b7b-8c0e2a4d6f93">
									<gml:surfaceMember>
										<!-- The second and fourth vertices cross the ring, making a bow tie. -->
										<gml:Polygon gml:id="poly_2c9d5e18-4a7f-4b30-8c8c-9d1f3b5e7a04">
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.83100 139.61790 0 35.83110 139.61805 0 35.83110 139.61790 0 35.83100 139.61805 0 35.83100 139.61790 0</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:MultiSurface>
							</core:lod1MultiSurface>
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
