<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>34.7715 137.3634 0</gml:lowerCorner>
			<gml:upperCorner>34.7716 137.3635 5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_shared_0001">
			<bldg:measuredHeight uom="m">5.0</bldg:measuredHeight>
			<!-- The shell is nothing but references: every face of the solid is a
			     polygon declared once under the boundary surface that names it. -->
			<bldg:lod2Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember xlink:href="#UUID_ground_0001"/>
							<gml:surfaceMember xlink:href="#UUID_roof_0001"/>
							<gml:surfaceMember xlink:href="#UUID_wall_0001"/>
							<gml:surfaceMember xlink:href="#UUID_wall_0002"/>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod2Solid>
			<bldg:boundedBy>
				<bldg:GroundSurface gml:id="bldg_ground_0001">
					<bldg:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="UUID_ground_0001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.7715 137.3634 0 34.7716 137.3634 0 34.7716 137.3635 0 34.7715 137.3635 0 34.7715 137.3634 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</bldg:lod2MultiSurface>
				</bldg:GroundSurface>
			</bldg:boundedBy>
			<bldg:boundedBy>
				<bldg:RoofSurface gml:id="bldg_roof_0001">
					<bldg:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="UUID_roof_0001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.7715 137.3634 5 34.7715 137.3635 5 34.7716 137.3635 5 34.7716 137.3634 5 34.7715 137.3634 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</bldg:lod2MultiSurface>
				</bldg:RoofSurface>
			</bldg:boundedBy>
			<bldg:boundedBy>
				<bldg:WallSurface gml:id="bldg_wall_0001">
					<bldg:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="UUID_wall_0001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.7715 137.3634 0 34.7715 137.3634 5 34.7716 137.3634 5 34.7716 137.3634 0 34.7715 137.3634 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</bldg:lod2MultiSurface>
				</bldg:WallSurface>
			</bldg:boundedBy>
			<bldg:boundedBy>
				<bldg:WallSurface gml:id="bldg_wall_0002">
					<bldg:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="UUID_wall_0002">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.7716 137.3635 0 34.7716 137.3635 5 34.7715 137.3635 5 34.7715 137.3635 0 34.7716 137.3635 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</bldg:lod2MultiSurface>
				</bldg:WallSurface>
			</bldg:boundedBy>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
