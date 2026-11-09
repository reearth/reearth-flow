<?xml version="1.0" encoding="UTF-8"?>
<!--
  Names of generic city objects that are not coded (EPSG:6697).
  - GenericThematicSurface: a gml:name 20 without a codeSpace.
  - GenericOccupiedSpace: a gml:name 30 without a codeSpace.
  - GenericUnoccupiedSpace: no gml:name at all.
  - GenericOccupiedSpace: a coded gml:name. Its roof, a nested
    GenericThematicSurface, has a gml:name 40 without a codeSpace, which is
    the nested object's name and not counted.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:gen="http://www.opengis.net/citygml/generics/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>33.717 130.481 0</gml:lowerCorner>
			<gml:upperCorner>33.7177 130.4811 5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<gen:GenericThematicSurface gml:id="gen_e5f6a7b8-c9d0-1e2f-3a4b-5c6d7e8f9a0b">
			<gml:name>20</gml:name>
			<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>33.717 130.481 0 33.717 130.4811 0 33.7171 130.4811 0 33.7171 130.481 0 33.717 130.481 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
		</gen:GenericThematicSurface>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<gen:GenericOccupiedSpace gml:id="gen_a1b2c3d4-e5f6-7890-abcd-ef1234567890">
			<gml:name>30</gml:name>
			<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7172 130.481 0 33.7173 130.481 0 33.7173 130.4811 0 33.7172 130.4811 0 33.7172 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7172 130.481 5 33.7172 130.4811 5 33.7173 130.4811 5 33.7173 130.481 5 33.7172 130.481 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7172 130.481 0 33.7172 130.4811 0 33.7172 130.4811 5 33.7172 130.481 5 33.7172 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7172 130.4811 0 33.7173 130.4811 0 33.7173 130.4811 5 33.7172 130.4811 5 33.7172 130.4811 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7173 130.4811 0 33.7173 130.481 0 33.7173 130.481 5 33.7173 130.4811 5 33.7173 130.4811 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7173 130.481 0 33.7172 130.481 0 33.7172 130.481 5 33.7173 130.481 5 33.7173 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
		</gen:GenericOccupiedSpace>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<gen:GenericUnoccupiedSpace gml:id="gen_c4d5e6f7-0819-42a3-bc4d-5e6f708192a3">
			<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod3MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>33.7174 130.481 2 33.7174 130.4811 2 33.7175 130.4811 2 33.7175 130.481 2 33.7174 130.481 2</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</gen:GenericUnoccupiedSpace>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<gen:GenericOccupiedSpace gml:id="gen_d5e6f708-192a-43b4-8d5e-6f708192a3b4">
			<gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
			<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<gen:GenericThematicSurface gml:id="gen_e6f70819-2a3b-44c5-9e6f-708192a3b4c5">
					<gml:name>40</gml:name>
					<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7176 130.481 5 33.7176 130.4811 5 33.7177 130.4811 5 33.7177 130.481 5 33.7176 130.481 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</gen:GenericThematicSurface>
			</core:boundary>
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7176 130.481 0 33.7177 130.481 0 33.7177 130.4811 0 33.7176 130.4811 0 33.7176 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7176 130.481 5 33.7176 130.4811 5 33.7177 130.4811 5 33.7177 130.481 5 33.7176 130.481 5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7176 130.481 0 33.7176 130.4811 0 33.7176 130.4811 5 33.7176 130.481 5 33.7176 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7176 130.4811 0 33.7177 130.4811 0 33.7177 130.4811 5 33.7176 130.4811 5 33.7176 130.4811 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7177 130.4811 0 33.7177 130.481 0 33.7177 130.481 5 33.7177 130.4811 5 33.7177 130.4811 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>33.7177 130.481 0 33.7176 130.481 0 33.7176 130.481 5 33.7177 130.481 5 33.7177 130.481 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
		</gen:GenericOccupiedSpace>
	</core:cityObjectMember>
</core:CityModel>
