<?xml version="1.0" encoding="UTF-8"?>
<!--
  Tunnel with every kind of nested object for plateau6 09-brid-tun-ubld, CityGML 3.0 + i-UR 4.0.
  One tun:Tunnel in tsukuba-shi (EPSG:6697): a box shaped body with a LOD1 solid
  and a LOD2 solid built from its six boundary surfaces (roof, ground, two walls
  and two closure surfaces), a tun:TunnelPart to the north, an outside
  tun:TunnelInstallation, a con:DoorSurface filling the east wall, and a
  tun:HollowSpace inside the body with a floor, a ceiling, an interior wall and
  a tun:TunnelFurniture.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.08600 140.11099 10</gml:lowerCorner>
			<gml:upperCorner>36.08614 140.11107 14</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tun:Tunnel gml:id="tun_cd59d3c1-1890-44f2-815c-cfb4172a90fc">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod2>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<con:RoofSurface gml:id="tun_fceac424-487a-433e-8f0e-e6e08129c764">
					<core:lod2MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_9c2115b4-c95d-45d9-a83b-9a6146df4b34">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08600 140.11100 14 36.08600 140.11106 14 36.08609 140.11106 14 36.08609 140.11100 14 36.08600 140.11100 14</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:GroundSurface gml:id="tun_ea738896-1398-4f13-8957-127ac6610508">
					<core:lod2MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_daadc06d-69f7-47bf-9865-0cdd54abb40b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08600 140.11100 10 36.08609 140.11100 10 36.08609 140.11106 10 36.08600 140.11106 10 36.08600 140.11100 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="tun_0ffdee28-93d7-483c-8cbc-d4d17ca76681">
					<core:lod2MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_85a19ce0-946e-412c-b623-5db3d74c536e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08609 140.11100 10 36.08600 140.11100 10 36.08600 140.11100 14 36.08609 140.11100 14 36.08609 140.11100 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="tun_953ba517-8dc6-44b8-90d5-0e47c936dcfc">
					<core:lod2MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_678da3db-3fc2-4d1c-8e73-bb9c1eb88e81">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08600 140.11106 10 36.08609 140.11106 10 36.08609 140.11106 14 36.08600 140.11106 14 36.08600 140.11106 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
					<con:fillingSurface>
						<con:DoorSurface gml:id="tun_d396f46e-beb9-4433-82bf-7225eaf37100">
							<core:lod3MultiSurface>
								<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08605 140.11106 10 36.08604 140.11106 10 36.08604 140.11106 12 36.08605 140.11106 12 36.08605 140.11106 10</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:MultiSurface>
							</core:lod3MultiSurface>
						</con:DoorSurface>
					</con:fillingSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<core:ClosureSurface gml:id="tun_a6260435-3fbc-403f-83dc-ccbbaf9695cb">
					<core:lod2MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_ffc3a5bf-8945-4eb1-9d0d-e078ea780ec9">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08600 140.11100 10 36.08600 140.11106 10 36.08600 140.11106 14 36.08600 140.11100 14 36.08600 140.11100 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</core:ClosureSurface>
			</core:boundary>
			<core:boundary>
				<core:ClosureSurface gml:id="tun_22d7ab89-4734-456e-953b-9dc8dcb51721">
					<core:lod2MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d9b6e081-2089-4c58-b6e2-430f71fb0259">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08609 140.11106 10 36.08609 140.11100 10 36.08609 140.11100 14 36.08609 140.11106 14 36.08609 140.11106 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</core:ClosureSurface>
			</core:boundary>
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08600 140.11100 10 36.08609 140.11100 10 36.08609 140.11106 10 36.08600 140.11106 10 36.08600 140.11100 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08600 140.11100 14 36.08600 140.11106 14 36.08609 140.11106 14 36.08609 140.11100 14 36.08600 140.11100 14</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08600 140.11100 10 36.08600 140.11106 10 36.08600 140.11106 14 36.08600 140.11100 14 36.08600 140.11100 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08609 140.11106 10 36.08609 140.11100 10 36.08609 140.11100 14 36.08609 140.11106 14 36.08609 140.11106 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08609 140.11100 10 36.08600 140.11100 10 36.08600 140.11100 14 36.08609 140.11100 14 36.08609 140.11100 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08600 140.11106 10 36.08609 140.11106 10 36.08609 140.11106 14 36.08600 140.11106 14 36.08600 140.11106 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<core:lod2Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember xlink:href="#pol_daadc06d-69f7-47bf-9865-0cdd54abb40b"/>
							<gml:surfaceMember xlink:href="#pol_9c2115b4-c95d-45d9-a83b-9a6146df4b34"/>
							<gml:surfaceMember xlink:href="#pol_ffc3a5bf-8945-4eb1-9d0d-e078ea780ec9"/>
							<gml:surfaceMember xlink:href="#pol_d9b6e081-2089-4c58-b6e2-430f71fb0259"/>
							<gml:surfaceMember xlink:href="#pol_85a19ce0-946e-412c-b623-5db3d74c536e"/>
							<gml:surfaceMember xlink:href="#pol_678da3db-3fc2-4d1c-8e73-bb9c1eb88e81"/>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<tun:tunnelInstallation>
				<tun:TunnelInstallation gml:id="tun_5774cc50-76d1-422e-b42d-f0e21fd2d891">
					<core:lod2MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.08604 140.11099 11 36.08602 140.11099 11 36.08602 140.11099 12 36.08604 140.11099 12 36.08604 140.11099 11</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
					<con:relationToConstruction>outside</con:relationToConstruction>
				</tun:TunnelInstallation>
			</tun:tunnelInstallation>
			<tun:hollowSpace>
				<tun:HollowSpace gml:id="tun_7cf0b3ad-5d61-43f1-bd27-ebfcb59dadd9">
					<core:boundary>
						<con:FloorSurface gml:id="tun_15e4d7a8-7153-4654-863a-bf2a4a27430c">
							<core:lod3MultiSurface>
								<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08601 140.11101 10.5 36.08601 140.11105 10.5 36.08608 140.11105 10.5 36.08608 140.11101 10.5 36.08601 140.11101 10.5</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:MultiSurface>
							</core:lod3MultiSurface>
						</con:FloorSurface>
					</core:boundary>
					<core:boundary>
						<con:CeilingSurface gml:id="tun_8759e3e0-94b8-41a7-a43b-916d38738296">
							<core:lod3MultiSurface>
								<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08601 140.11101 13.5 36.08608 140.11101 13.5 36.08608 140.11105 13.5 36.08601 140.11105 13.5 36.08601 140.11101 13.5</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:MultiSurface>
							</core:lod3MultiSurface>
						</con:CeilingSurface>
					</core:boundary>
					<core:boundary>
						<con:InteriorWallSurface gml:id="tun_d61cc36a-4faf-404d-802c-a5ab12d6ba25">
							<core:lod3MultiSurface>
								<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08608 140.11101 10.5 36.08608 140.11101 13.5 36.08601 140.11101 13.5 36.08601 140.11101 10.5 36.08608 140.11101 10.5</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:MultiSurface>
							</core:lod3MultiSurface>
						</con:InteriorWallSurface>
					</core:boundary>
					<tun:tunnelFurniture>
						<tun:TunnelFurniture gml:id="tun_251f6d96-75d6-4552-875d-528c945245a7">
							<core:lod3MultiSurface>
								<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08603 140.11102 11 36.08603 140.11103 11 36.08604 140.11103 11 36.08604 140.11102 11 36.08603 140.11102 11</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:MultiSurface>
							</core:lod3MultiSurface>
						</tun:TunnelFurniture>
					</tun:tunnelFurniture>
				</tun:HollowSpace>
			</tun:hollowSpace>
			<tun:tunnelPart>
				<tun:TunnelPart gml:id="tun_7cf0f6db-21fe-407d-8f92-f847819796ff">
					<core:creationDate>2024-03-19T00:00:00</core:creationDate>
					<core:lod1Solid>
						<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:exterior>
								<gml:Shell>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08609 140.11100 10 36.08614 140.11100 10 36.08614 140.11106 10 36.08609 140.11106 10 36.08609 140.11100 10</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08609 140.11100 14 36.08609 140.11106 14 36.08614 140.11106 14 36.08614 140.11100 14 36.08609 140.11100 14</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08609 140.11100 10 36.08609 140.11106 10 36.08609 140.11106 14 36.08609 140.11100 14 36.08609 140.11100 10</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08614 140.11106 10 36.08614 140.11100 10 36.08614 140.11100 14 36.08614 140.11106 14 36.08614 140.11106 10</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08614 140.11100 10 36.08609 140.11100 10 36.08609 140.11100 14 36.08614 140.11100 14 36.08614 140.11100 10</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>36.08609 140.11106 10 36.08614 140.11106 10 36.08614 140.11106 14 36.08609 140.11106 14 36.08609 140.11106 10</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:Shell>
							</gml:exterior>
						</gml:Solid>
					</core:lod1Solid>
				</tun:TunnelPart>
			</tun:tunnelPart>
		</tun:Tunnel>
	</core:cityObjectMember>
</core:CityModel>
