<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>34.77140 137.36340 0</gml:lowerCorner>
			<gml:upperCorner>34.77150 137.36390 10</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>

	<!-- Building 1: has BOTH an unmatched "from" and an unmatched "to".
	     - core:lod2Solid references poly_l06b1_missing which is NOT defined
	       -> unmatched 参照元 xlink:href (tag lod2Solid).
	     - con:WallSurface defines poly_l06b1_wall4 which is NOT referenced by
	       the solid -> unmatched 参照先 gml:id (tag gml:Polygon). -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l06lod2-0001">
			<gml:name>16211-bldg-L06-1</gml:name>
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>422</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">10.0</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<core:lod2Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember xlink:href="#poly_l06b1_gnd"/>
							<gml:surfaceMember xlink:href="#poly_l06b1_roof"/>
							<gml:surfaceMember xlink:href="#poly_l06b1_wall1"/>
							<gml:surfaceMember xlink:href="#poly_l06b1_wall2"/>
							<gml:surfaceMember xlink:href="#poly_l06b1_wall3"/>
							<gml:surfaceMember xlink:href="#poly_l06b1_missing"/>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<core:boundary>
				<con:GroundSurface gml:id="gnd_l06b1">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b1_gnd">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36340 0 34.77140 137.36350 0 34.77150 137.36350 0 34.77150 137.36340 0 34.77140 137.36340 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="roof_l06b1">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b1_roof">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36340 10 34.77150 137.36340 10 34.77150 137.36350 10 34.77140 137.36350 10 34.77140 137.36340 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall1_l06b1">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b1_wall1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36340 0 34.77140 137.36350 0 34.77140 137.36350 10 34.77140 137.36340 10 34.77140 137.36340 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall2_l06b1">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b1_wall2">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36350 0 34.77150 137.36350 0 34.77150 137.36350 10 34.77140 137.36350 10 34.77140 137.36350 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall3_l06b1">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b1_wall3">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77150 137.36350 0 34.77150 137.36340 0 34.77150 137.36340 10 34.77150 137.36350 10 34.77150 137.36350 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<!-- Defined but NOT referenced by the solid -> unmatched 参照先 gml:id -->
			<core:boundary>
				<con:WallSurface gml:id="wall4_l06b1">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b1_wall4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77150 137.36340 0 34.77140 137.36340 0 34.77140 137.36340 10 34.77150 137.36340 10 34.77150 137.36340 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
		</bldg:Building>
	</core:cityObjectMember>

	<!-- Building 2: has ONE unmatched "from" only.
	     core:lod2Solid references all six defined polygons PLUS poly_l06b2_missing
	     which is not defined -> unmatched 参照元 xlink:href (tag lod2Solid).
	     Every defined polygon is referenced, so no unmatched 参照先. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l06lod2-0002">
			<gml:name>16211-bldg-L06-2</gml:name>
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>422</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">10.0</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<core:lod2Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember xlink:href="#poly_l06b2_gnd"/>
							<gml:surfaceMember xlink:href="#poly_l06b2_roof"/>
							<gml:surfaceMember xlink:href="#poly_l06b2_wall1"/>
							<gml:surfaceMember xlink:href="#poly_l06b2_wall2"/>
							<gml:surfaceMember xlink:href="#poly_l06b2_wall3"/>
							<gml:surfaceMember xlink:href="#poly_l06b2_wall4"/>
							<gml:surfaceMember xlink:href="#poly_l06b2_missing"/>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<core:boundary>
				<con:GroundSurface gml:id="gnd_l06b2">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b2_gnd">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36360 0 34.77140 137.36370 0 34.77150 137.36370 0 34.77150 137.36360 0 34.77140 137.36360 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="roof_l06b2">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b2_roof">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36360 10 34.77150 137.36360 10 34.77150 137.36370 10 34.77140 137.36370 10 34.77140 137.36360 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall1_l06b2">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b2_wall1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36360 0 34.77140 137.36370 0 34.77140 137.36370 10 34.77140 137.36360 10 34.77140 137.36360 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall2_l06b2">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b2_wall2">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36370 0 34.77150 137.36370 0 34.77150 137.36370 10 34.77140 137.36370 10 34.77140 137.36370 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall3_l06b2">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b2_wall3">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77150 137.36370 0 34.77150 137.36360 0 34.77150 137.36360 10 34.77150 137.36370 10 34.77150 137.36370 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall4_l06b2">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b2_wall4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77150 137.36360 0 34.77140 137.36360 0 34.77140 137.36360 10 34.77150 137.36360 10 34.77150 137.36360 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
		</bldg:Building>
	</core:cityObjectMember>

	<!-- Building 3: control with NO error.
	     Every surfaceMember xlink:href resolves to a defined polygon and every
	     defined polygon is referenced -> contributes 0 to L-bldg-06. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l06lod2-0003">
			<gml:name>16211-bldg-L06-3</gml:name>
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>422</bldg:usage>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">10.0</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<core:lod2Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember xlink:href="#poly_l06b3_gnd"/>
							<gml:surfaceMember xlink:href="#poly_l06b3_roof"/>
							<gml:surfaceMember xlink:href="#poly_l06b3_wall1"/>
							<gml:surfaceMember xlink:href="#poly_l06b3_wall2"/>
							<gml:surfaceMember xlink:href="#poly_l06b3_wall3"/>
							<gml:surfaceMember xlink:href="#poly_l06b3_wall4"/>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<core:boundary>
				<con:GroundSurface gml:id="gnd_l06b3">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b3_gnd">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36380 0 34.77140 137.36390 0 34.77150 137.36390 0 34.77150 137.36380 0 34.77140 137.36380 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="roof_l06b3">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b3_roof">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36380 10 34.77150 137.36380 10 34.77150 137.36390 10 34.77140 137.36390 10 34.77140 137.36380 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall1_l06b3">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b3_wall1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36380 0 34.77140 137.36390 0 34.77140 137.36390 10 34.77140 137.36380 10 34.77140 137.36380 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall2_l06b3">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b3_wall2">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77140 137.36390 0 34.77150 137.36390 0 34.77150 137.36390 10 34.77140 137.36390 10 34.77140 137.36390 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall3_l06b3">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b3_wall3">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77150 137.36390 0 34.77150 137.36380 0 34.77150 137.36380 10 34.77150 137.36390 10 34.77150 137.36390 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="wall4_l06b3">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_l06b3_wall4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77150 137.36380 0 34.77140 137.36380 0 34.77140 137.36380 10 34.77150 137.36380 10 34.77150 137.36380 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
