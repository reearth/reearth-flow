<?xml version="1.0" encoding="UTF-8"?>
<!--
  Error fixture for plateau6 03-tran L-tran-02 (adjacent surface overlap), CityGML 3.0 + i-UR 4.0.
  L-tran-02 detects, per LOD and package, 2D footprint overlaps between DIFFERENT tran:Road instances.

  Four tran:Road features within saitama-shi mesh 53395499 (EPSG:6697):
    - Road 1 and Road 2 carry LOD3 TrafficArea rectangles that share a collinear edge AND overlap in
      area (same latitude band, partially overlapping longitude range). They are the expected error pair.
    - Road 3 and Road 4 carry LOD1 TrafficArea rectangles that share only a single edge (zero overlap
      area). They are a control pair: valid adjacency that must NOT be reported.

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
			<gml:upperCorner>35.83110 139.61840 5.5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<!-- Road 1: LOD3 rectangle, overlaps Road 2 in area with a collinear shared edge -> reported. -->
		<tran:Road gml:id="tran_2f8e1c0a-3b6d-4e21-9c14-5a7b9d0e1f23">
			<gml:name>11100-tran-1</gml:name>
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
				<tran:Section gml:id="tran_851c2747-262f-49d0-8855-a9502c8094e6">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_7d3a9b21-6c48-4f15-8e92-1a4b7c0d3e56">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_b6e04c19-2f7a-4d83-9b51-6c8e0a2f4d17">
									<core:lod3MultiSurface>
										<gml:MultiSurface gml:id="ms_3c1f8a26-9d40-4b72-8a63-5e9c1d0f7b28">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_9a2d7e14-5b83-4c60-9f27-8d1a3b6e0c49">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61785 5.5 35.83110 139.61785 5.5 35.83110 139.61770 5.5 35.83100 139.61770 5.5 35.83100 139.61785 5.5</gml:posList>
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
				</tran:Section>
			</tran:section>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<!-- Road 2: LOD3 rectangle, overlaps Road 1 in area with a collinear shared edge -> reported. -->
		<tran:Road gml:id="tran_3a9f2d1b-4c7e-4f32-8d25-6b8c0e1f2a34">
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
				<tran:Section gml:id="tran_504fa514-f26d-4fd6-9ba9-1bbd6c220e79">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_8e4b0c32-7d59-4a26-9f83-2b5c8d1e4f67">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_c7f15d2a-3e8b-4c94-8a62-7d9f0b3e5a18">
									<core:lod3MultiSurface>
										<gml:MultiSurface gml:id="ms_4d2a9b37-1c50-4e83-9b74-6f0d2e1a8c39">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_a1b3e825-6c94-4d70-8e38-9f2b4c7d0e5a">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61795 3.25 35.83110 139.61795 3.25 35.83110 139.61780 3.25 35.83100 139.61780 3.25 35.83100 139.61795 3.25</gml:posList>
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
				</tran:Section>
			</tran:section>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<!-- Road 3: LOD1 rectangle, shares only one edge with Road 4 (zero overlap area) -> not reported. -->
		<tran:Road gml:id="tran_4b0a3e2c-5d8f-4a43-9e36-7c9d1f2a3b45">
			<gml:name>11100-tran-3</gml:name>
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
				<tran:Section gml:id="tran_668506ca-7963-4351-b80e-cd742a2819b5">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_9f5c1d43-8e60-4b37-8a94-3c6d9e2f5a78">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_d8025e3b-4f9c-4da5-9b73-8e0a1c4f6b29">
									<core:lod1MultiSurface>
										<gml:MultiSurface gml:id="ms_5e3b0c48-2d61-4f94-8a85-7a1e3f2b9d40">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_b2c4f936-7d05-4e81-9f49-0a3c5d8e1f6b">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61825 0 35.83110 139.61825 0 35.83110 139.61810 0 35.83100 139.61810 0 35.83100 139.61825 0</gml:posList>
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
	<core:cityObjectMember>
		<!-- Road 4: LOD1 rectangle, shares only one edge with Road 3 (zero overlap area) -> not reported. -->
		<tran:Road gml:id="tran_5c1b4f3d-6e90-4b54-8f47-8d0e2a3b4c56">
			<gml:name>11100-tran-4</gml:name>
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
				<tran:Section gml:id="tran_77900a37-4534-42c6-ad05-2ddfdde698f4">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_a06d2e54-9f71-4c48-8b05-4d7e0f3a6b89">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_e9136f4c-5a0d-4eb6-9c84-9f1b2d5a7c30">
									<core:lod1MultiSurface>
										<gml:MultiSurface gml:id="ms_6f4c1d59-3e72-4a05-8b96-8b2f4a3c0e51">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_c3d5024a-8e16-4f92-9a5a-1b4d6e9f2a7c">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61840 0 35.83110 139.61840 0 35.83110 139.61825 0 35.83100 139.61825 0 35.83100 139.61840 0</gml:posList>
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
