<?xml version="1.0" encoding="UTF-8"?>
<!--
  Utility network elements whose CompositeSurface falls apart, at LOD3 and LOD2.
  One uro:SewerPipe has a closed core:lod2Solid box from (157000, 21600, 109) to
  (157001, 21601, 110). Its core:lod3MultiSurface holds one CompositeSurface of
  six faces in two parts that share no edge: the bottom, front and left faces of
  that box, and the same three faces of a box 5 m further north.
  One uro:WaterPipe is the same 10 m further east, with the box as its
  core:lod1Solid and the split CompositeSurface in its core:lod2MultiSurface.
  CityGML 2.0 has no LOD2 surface, so the original reports this LOD2 as an
  invalid geometry type instead.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
			<gml:lowerCorner>157000 21600 109</gml:lowerCorner>
			<gml:upperCorner>157006 21611 110</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- LOD3 CompositeSurface in two separate parts -->
	<core:cityObjectMember>
		<uro:SewerPipe gml:id="unf_4af983ec-fd12-4208-a013-f766225d112c">
			<core:creationDate>2025-03-21T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod2>
					<urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod3>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod2Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21600 109 157001 21601 109 157000 21601 109 157000 21600 109 157001 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21601 110 157001 21601 110 157001 21600 110 157000 21600 110 157000 21601 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21600 110 157001 21600 110 157001 21600 109 157000 21600 109 157000 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21601 109 157001 21601 110 157000 21601 110 157000 21601 109 157001 21601 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21601 109 157000 21601 110 157000 21600 110 157000 21600 109 157000 21601 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21600 110 157001 21601 110 157001 21601 109 157001 21600 109 157001 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<core:lod3MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:surfaceMember>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21600 109 157001 21601 109 157000 21601 109 157000 21600 109 157001 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21600 110 157001 21600 110 157001 21600 109 157000 21600 109 157000 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21601 109 157000 21601 110 157000 21600 110 157000 21600 109 157000 21601 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157006 21600 109 157006 21601 109 157005 21601 109 157005 21600 109 157006 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157005 21600 110 157006 21600 110 157006 21600 109 157005 21600 109 157005 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157005 21601 109 157005 21601 110 157005 21600 110 157005 21600 109 157005 21601 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</uro:SewerPipe>
	</core:cityObjectMember>
	<!-- LOD2 CompositeSurface in two separate parts -->
	<core:cityObjectMember>
		<uro:WaterPipe gml:id="unf_9c9dfde6-cb31-40b5-b0fe-f8c86a1d594f">
			<core:creationDate>2025-03-21T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod2>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21610 109 157001 21611 109 157000 21611 109 157000 21610 109 157001 21610 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21611 110 157001 21611 110 157001 21610 110 157000 21610 110 157000 21611 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21610 110 157001 21610 110 157001 21610 109 157000 21610 109 157000 21610 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21611 109 157001 21611 110 157000 21611 110 157000 21611 109 157001 21611 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21611 109 157000 21611 110 157000 21610 110 157000 21610 109 157000 21611 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21610 110 157001 21611 110 157001 21611 109 157001 21610 109 157001 21610 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<core:lod2MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:surfaceMember>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21610 109 157001 21611 109 157000 21611 109 157000 21610 109 157001 21610 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21610 110 157001 21610 110 157001 21610 109 157000 21610 109 157000 21610 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21611 109 157000 21611 110 157000 21610 110 157000 21610 109 157000 21611 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157006 21610 109 157006 21611 109 157005 21611 109 157005 21610 109 157006 21610 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157005 21610 110 157006 21610 110 157006 21610 109 157005 21610 109 157005 21610 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157005 21611 109 157005 21611 110 157005 21610 110 157005 21610 109 157005 21611 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod2MultiSurface>
		</uro:WaterPipe>
	</core:cityObjectMember>
</core:CityModel>
