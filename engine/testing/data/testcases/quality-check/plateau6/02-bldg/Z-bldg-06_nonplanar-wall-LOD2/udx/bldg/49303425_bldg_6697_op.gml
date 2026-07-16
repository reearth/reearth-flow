<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
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
			<gml:upperCorner>34.77150 137.36350 10</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- Demo building: a single LOD2 box with six boundary surfaces
	     (1 ground + 1 roof + 4 walls). Five surfaces are clean planar
	     rectangles; the west wall (wall_w) has one top vertex pushed ~4.5 m
	     out of the wall plane, so exactly that one WallSurface is reported
	     under "LOD2以上境界面のエラー" with issue NonPlanarSurface.
	     NonPlanarSurface is detected before the validator's Rotator3D step, so
	     the emitted error geometry keeps its original coordinates and lines up
	     with the real wall when overlaid in a GIS tool. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
			<gml:name>16211-bldg-nonplanar-wall-demo</gml:name>
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
			<core:boundary>
				<con:GroundSurface gml:id="gnd_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_gnd_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
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
				<con:RoofSurface gml:id="roof_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_roof_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
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
				<con:WallSurface gml:id="wall_s_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_wall_s_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
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
				<con:WallSurface gml:id="wall_e_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_wall_e_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
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
				<con:WallSurface gml:id="wall_n_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_wall_n_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
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
			<!-- Defective west wall: the third vertex is pushed ~4.5 m west
			     (137.36340 -> 137.36335) so the four corners are no longer
			     coplanar -> NonPlanarSurface on this WallSurface only. -->
			<core:boundary>
				<con:WallSurface gml:id="wall_w_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_wall_w_3cc56ca3-af5a-48a9-beef-5cc5bed76265">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.77150 137.36340 0 34.77140 137.36340 0 34.77140 137.36335 10 34.77150 137.36340 10 34.77150 137.36340 0</gml:posList>
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
