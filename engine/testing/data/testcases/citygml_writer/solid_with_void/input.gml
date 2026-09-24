<?xml version="1.0" encoding="UTF-8"?>
<!--
  A solid with one void shell, plus a specialized geometry property.

  Neither survives the legacy reader: it logs "interior of Solid is not
  supported, skipped" while parsing, and it records no source property name, so
  `bldg:lod0RoofEdge` would come back out as `bldg:lod0MultiSurface`. That is
  why this case has its own expectation and is skipped in the legacy build,
  rather than being folded into the old/new comparison in `roundtrip/`.
-->
<core:CityModel xmlns:gml="http://www.opengis.net/gml" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.0 135.0 0</gml:lowerCorner>
			<gml:upperCorner>35.002 135.002 20</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="test-building-002">
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.0 135.0 20 35.002 135.0 20 35.002 135.002 20 35.0 135.002 20 35.0 135.0 20</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0 135.0 0 35.002 135.0 0 35.0 135.002 0 35.0 135.0 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0 135.0 0 35.0 135.0 20 35.002 135.0 0 35.0 135.0 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.002 135.0 0 35.0 135.0 20 35.0 135.002 0 35.002 135.0 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0 135.002 0 35.0 135.0 20 35.0 135.0 0 35.0 135.002 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
					<gml:interior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0005 135.0005 1 35.001 135.0005 1 35.0005 135.001 1 35.0005 135.0005 1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0005 135.0005 1 35.0005 135.0005 5 35.001 135.0005 1 35.0005 135.0005 1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.001 135.0005 1 35.0005 135.0005 5 35.0005 135.001 1 35.001 135.0005 1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0005 135.001 1 35.0005 135.0005 5 35.0005 135.0005 1 35.0005 135.001 1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:interior>
				</gml:Solid>
			</bldg:lod1Solid>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
