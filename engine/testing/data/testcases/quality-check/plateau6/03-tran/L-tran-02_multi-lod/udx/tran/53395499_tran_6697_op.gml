<?xml version="1.0" encoding="UTF-8"?>
<!--
  Error fixture for plateau6 03-tran L-tran-02 (adjacent surface overlap) across two LODs,
  CityGML 3.0 + i-UR 4.0.

  Two tran:Road features within saitama-shi mesh 53395499 (EPSG:6697). Each carries the same
  footprint at LOD1 and at LOD3, and the two footprints overlap in area. The pair therefore
  produces one finding per LOD, which pins the dedup key of the overlap report to include `lod`:
  keyed on the road id set alone the two findings collapse into one row.

  The test excludes the 隣接ID column: it is a running counter over the overlap regions, and the
  LOD1 and LOD3 regions reach the counter through separate branches of the graph, so which LOD
  gets which number varies between runs.

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
			<gml:upperCorner>35.83110 139.61795 5.5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<!-- Road 1: same footprint at LOD1 and LOD3, overlaps Road 2 at both LODs. -->
		<tran:Road gml:id="tran_2fde7a5f-9ad1-427a-923b-69f0ab7d82fc">
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
				<tran:Section gml:id="tran_55cb66db-11fd-409e-be06-9c1d8cc44b52">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_ccd936f0-c84e-40d2-894e-a65b616e8b8c">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_c6010901-3df2-4d6e-848c-821dc81035f4">
									<core:lod1MultiSurface>
										<gml:MultiSurface gml:id="ms_551d0f54-1128-483d-97a5-f49f9598aae6">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_1b7a36cd-890c-47e8-ac51-4b3f039452ff">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61785 0 35.83110 139.61785 0 35.83110 139.61770 0 35.83100 139.61770 0 35.83100 139.61785 0</gml:posList>
														</gml:LinearRing>
													</gml:exterior>
												</gml:Polygon>
											</gml:surfaceMember>
										</gml:MultiSurface>
									</core:lod1MultiSurface>
									<core:lod3MultiSurface>
										<gml:MultiSurface gml:id="ms_57da8b0e-9f55-4059-a814-99ed77eb2297">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_c9b1d993-0c58-4e18-a92b-a712e81a5427">
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
		<!-- Road 2: same footprint at LOD1 and LOD3, overlaps Road 1 at both LODs. -->
		<tran:Road gml:id="tran_69bcd697-4e2c-4b6e-a427-f8941b674a18">
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
				<tran:Section gml:id="tran_93fe14af-a1dd-4570-ba77-8314058c989c">
					<tran:trafficSpace>
						<tran:TrafficSpace gml:id="tran_a18c9629-451a-4ea7-8145-9108b5d9aab2">
							<core:boundary>
								<tran:TrafficArea gml:id="tran_d34b902f-9770-43d1-a724-d46ecf1d27f6">
									<core:lod1MultiSurface>
										<gml:MultiSurface gml:id="ms_3541aad6-6f5f-4ae0-929d-5f7cf89532ce">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_a78d2920-f409-44ab-ad38-effaadb88db0">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61795 0 35.83110 139.61795 0 35.83110 139.61780 0 35.83100 139.61780 0 35.83100 139.61795 0</gml:posList>
														</gml:LinearRing>
													</gml:exterior>
												</gml:Polygon>
											</gml:surfaceMember>
										</gml:MultiSurface>
									</core:lod1MultiSurface>
									<core:lod3MultiSurface>
										<gml:MultiSurface gml:id="ms_34c36881-bdca-454d-92af-0ceb3242d95b">
											<gml:surfaceMember>
												<gml:Polygon gml:id="poly_0af7875d-abf0-4317-a4a5-b13fc9341679">
													<gml:exterior>
														<gml:LinearRing>
															<gml:posList>35.83100 139.61795 5.5 35.83110 139.61795 5.5 35.83110 139.61780 5.5 35.83100 139.61780 5.5 35.83100 139.61795 5.5</gml:posList>
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
</core:CityModel>
