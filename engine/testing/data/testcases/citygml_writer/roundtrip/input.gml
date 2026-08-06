<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:gml="http://www.opengis.net/gml" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.0 135.0 0</gml:lowerCorner>
			<gml:upperCorner>35.001 135.001 10</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="test-building-001">
			<bldg:measuredHeight uom="m">10.5</bldg:measuredHeight>
			<bldg:lod2MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.0 135.0 0 35.0 135.001 0 35.001 135.001 0 35.001 135.0 0 35.0 135.0 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>35.0002 135.0002 0 35.0008 135.0002 0 35.0008 135.0008 0 35.0002 135.0008 0 35.0002 135.0002 0</gml:posList>
								</gml:LinearRing>
							</gml:interior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.0 135.0 10 35.001 135.0 10 35.001 135.001 10 35.0 135.001 10 35.0 135.0 10</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod2MultiSurface>
			<!-- A void-free tetrahedron. Both geometry worlds can carry it, which
			     is what makes it usable for the strict old/new comparison; a solid
			     with a void goes to the new-world-only `solid_with_void` case,
			     because the legacy reader discards interior shells while parsing. -->
			<bldg:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0 135.0 0 35.001 135.0 0 35.0 135.001 0 35.0 135.0 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0 135.0 0 35.0 135.0 10 35.001 135.0 0 35.0 135.0 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.001 135.0 0 35.0 135.0 10 35.0 135.001 0 35.001 135.0 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0 135.001 0 35.0 135.0 10 35.0 135.0 0 35.0 135.001 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
