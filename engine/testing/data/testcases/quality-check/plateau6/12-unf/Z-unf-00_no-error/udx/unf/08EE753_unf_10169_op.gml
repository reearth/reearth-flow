<?xml version="1.0" encoding="UTF-8"?>
<!--
  Valid utility network elements with geometry at LOD0 to LOD3.
  uro:ElectricityCable: the digital map line (urc:DmGeometricAttribute
  urc:lod0Geometry), a core:lod0MultiCurve, box solids at LOD1 and LOD2 and the
  same box as core:lod3MultiSurface.
  uro:TelecommunicationsCable: the same without core:lod0MultiCurve, so its only
  LOD0 geometry is the digital map line.
  uro:Manhole: the digital map point, a core:lod0Point and a box at LOD1 to LOD3.
  uro:WaterPipe: a box solid at LOD1 and, as LOD2, a core:lod2MultiSurface whose
  only member is a CompositeSurface of the six faces of the box.
  Two uro:SewerPipe and two more uro:Manhole, each with a triangulated
  core:lod2Solid.

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
			<gml:lowerCorner>156561.10929971817 21007.437699998685 0</gml:lowerCorner>
			<gml:upperCorner>157517.0072997175 22010.374800001056 225.3813</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:ElectricityCable gml:id="unf_2a0d9e73-3adc-4545-abeb-f4205ac3ba0b">
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
			<core:adeOfAbstractCityObject>
				<urc:DmGeometricAttribute>
					<urc:dmCode codeSpace="../../codelists/Common_dmCode.xml">4265</urc:dmCode>
					<urc:meshCode>08EE753</urc:meshCode>
					<urc:geometryType codeSpace="../../codelists/Common_geometryType.xml">E2</urc:geometryType>
					<urc:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</urc:mapLevel>
					<urc:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</urc:shapeType>
					<urc:lod0Geometry>
						<gml:MultiCurve srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
							<gml:curveMember>
								<gml:LineString>
									<gml:posList>157000.5 21600 0 157000.5 21604 0</gml:posList>
								</gml:LineString>
							</gml:curveMember>
						</gml:MultiCurve>
					</urc:lod0Geometry>
				</urc:DmGeometricAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod0MultiCurve>
				<gml:MultiCurve srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:curveMember>
						<gml:LineString>
							<gml:posList>157000.5 21600 110 157000.5 21604 110</gml:posList>
						</gml:LineString>
					</gml:curveMember>
				</gml:MultiCurve>
			</core:lod0MultiCurve>
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21600 109 157001 21604 109 157000 21604 109 157000 21600 109 157001 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21604 110 157001 21604 110 157001 21600 110 157000 21600 110 157000 21604 110</gml:posList>
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
											<gml:posList>157001 21604 109 157001 21604 110 157000 21604 110 157000 21604 109 157001 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21604 109 157000 21604 110 157000 21600 110 157000 21600 109 157000 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21600 110 157001 21604 110 157001 21604 109 157001 21600 109 157001 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<core:lod2Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21600 109 157001 21604 109 157000 21604 109 157000 21600 109 157001 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21604 110 157001 21604 110 157001 21600 110 157000 21600 110 157000 21604 110</gml:posList>
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
											<gml:posList>157001 21604 109 157001 21604 110 157000 21604 110 157000 21604 109 157001 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157000 21604 109 157000 21604 110 157000 21600 110 157000 21600 109 157000 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157001 21600 110 157001 21604 110 157001 21604 109 157001 21600 109 157001 21600 110</gml:posList>
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
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157001 21600 109 157001 21604 109 157000 21604 109 157000 21600 109 157001 21600 109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157000 21604 110 157001 21604 110 157001 21600 110 157000 21600 110 157000 21604 110</gml:posList>
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
									<gml:posList>157001 21604 109 157001 21604 110 157000 21604 110 157000 21604 109 157001 21604 109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157000 21604 109 157000 21604 110 157000 21600 110 157000 21600 109 157000 21604 109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157001 21600 110 157001 21604 110 157001 21604 109 157001 21600 109 157001 21600 110</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</uro:ElectricityCable>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:TelecommunicationsCable gml:id="unf_a374a2d9-a97d-440e-973e-3c15cd522671">
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
			<core:adeOfAbstractCityObject>
				<urc:DmGeometricAttribute>
					<urc:dmCode codeSpace="../../codelists/Common_dmCode.xml">4265</urc:dmCode>
					<urc:meshCode>08EE753</urc:meshCode>
					<urc:geometryType codeSpace="../../codelists/Common_geometryType.xml">E2</urc:geometryType>
					<urc:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</urc:mapLevel>
					<urc:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</urc:shapeType>
					<urc:lod0Geometry>
						<gml:MultiCurve srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
							<gml:curveMember>
								<gml:LineString>
									<gml:posList>157010.5 21600 0 157010.5 21604 0</gml:posList>
								</gml:LineString>
							</gml:curveMember>
						</gml:MultiCurve>
					</urc:lod0Geometry>
				</urc:DmGeometricAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157011 21600 109 157011 21604 109 157010 21604 109 157010 21600 109 157011 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157010 21604 110 157011 21604 110 157011 21600 110 157010 21600 110 157010 21604 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157010 21600 110 157011 21600 110 157011 21600 109 157010 21600 109 157010 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157011 21604 109 157011 21604 110 157010 21604 110 157010 21604 109 157011 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157010 21604 109 157010 21604 110 157010 21600 110 157010 21600 109 157010 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157011 21600 110 157011 21604 110 157011 21604 109 157011 21600 109 157011 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<core:lod2Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157011 21600 109 157011 21604 109 157010 21604 109 157010 21600 109 157011 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157010 21604 110 157011 21604 110 157011 21600 110 157010 21600 110 157010 21604 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157010 21600 110 157011 21600 110 157011 21600 109 157010 21600 109 157010 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157011 21604 109 157011 21604 110 157010 21604 110 157010 21604 109 157011 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157010 21604 109 157010 21604 110 157010 21600 110 157010 21600 109 157010 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157011 21600 110 157011 21604 110 157011 21604 109 157011 21600 109 157011 21600 110</gml:posList>
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
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157011 21600 109 157011 21604 109 157010 21604 109 157010 21600 109 157011 21600 109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157010 21604 110 157011 21604 110 157011 21600 110 157010 21600 110 157010 21604 110</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157010 21600 110 157011 21600 110 157011 21600 109 157010 21600 109 157010 21600 110</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157011 21604 109 157011 21604 110 157010 21604 110 157010 21604 109 157011 21604 109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157010 21604 109 157010 21604 110 157010 21600 110 157010 21600 109 157010 21604 109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157011 21600 110 157011 21604 110 157011 21604 109 157011 21600 109 157011 21600 110</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</uro:TelecommunicationsCable>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_1a42aad7-bb10-4405-8405-05065c8ef078">
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
			<core:adeOfAbstractCityObject>
				<urc:DmGeometricAttribute>
					<urc:dmCode codeSpace="../../codelists/Common_dmCode.xml">8140</urc:dmCode>
					<urc:meshCode>08EE753</urc:meshCode>
					<urc:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</urc:geometryType>
					<urc:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</urc:mapLevel>
					<urc:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</urc:shapeType>
					<urc:lod0Geometry>
						<gml:Point srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
							<gml:pos>157020.5 21600.5 0</gml:pos>
						</gml:Point>
					</urc:lod0Geometry>
				</urc:DmGeometricAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod0Point>
				<gml:Point srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:pos>157020.5 21600.5 110</gml:pos>
				</gml:Point>
			</core:lod0Point>
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157021 21600 108 157021 21601 108 157020 21601 108 157020 21600 108 157021 21600 108</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157020 21601 110 157021 21601 110 157021 21600 110 157020 21600 110 157020 21601 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157020 21600 110 157021 21600 110 157021 21600 108 157020 21600 108 157020 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157021 21601 108 157021 21601 110 157020 21601 110 157020 21601 108 157021 21601 108</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157020 21601 108 157020 21601 110 157020 21600 110 157020 21600 108 157020 21601 108</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157021 21600 110 157021 21601 110 157021 21601 108 157021 21600 108 157021 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<core:lod2Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157021 21600 108 157021 21601 108 157020 21601 108 157020 21600 108 157021 21600 108</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157020 21601 110 157021 21601 110 157021 21600 110 157020 21600 110 157020 21601 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157020 21600 110 157021 21600 110 157021 21600 108 157020 21600 108 157020 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157021 21601 108 157021 21601 110 157020 21601 110 157020 21601 108 157021 21601 108</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157020 21601 108 157020 21601 110 157020 21600 110 157020 21600 108 157020 21601 108</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157021 21600 110 157021 21601 110 157021 21601 108 157021 21600 108 157021 21600 110</gml:posList>
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
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157021 21600 108 157021 21601 108 157020 21601 108 157020 21600 108 157021 21600 108</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157020 21601 110 157021 21601 110 157021 21600 110 157020 21600 110 157020 21601 110</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157020 21600 110 157021 21600 110 157021 21600 108 157020 21600 108 157020 21600 110</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157021 21601 108 157021 21601 110 157020 21601 110 157020 21601 108 157021 21601 108</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157020 21601 108 157020 21601 110 157020 21600 110 157020 21600 108 157020 21601 108</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>157021 21600 110 157021 21601 110 157021 21601 108 157021 21600 108 157021 21600 110</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:WaterPipe gml:id="unf_c614a0b8-ec62-4853-aa91-f6cddb9022f1">
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
			<core:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157031 21600 109 157031 21604 109 157030 21604 109 157030 21600 109 157031 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157030 21604 110 157031 21604 110 157031 21600 110 157030 21600 110 157030 21604 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157030 21600 110 157031 21600 110 157031 21600 109 157030 21600 109 157030 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157031 21604 109 157031 21604 110 157030 21604 110 157030 21604 109 157031 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157030 21604 109 157030 21604 110 157030 21600 110 157030 21600 109 157030 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157031 21600 110 157031 21604 110 157031 21604 109 157031 21600 109 157031 21600 110</gml:posList>
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
											<gml:posList>157031 21600 109 157031 21604 109 157030 21604 109 157030 21600 109 157031 21600 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157030 21604 110 157031 21604 110 157031 21600 110 157030 21600 110 157030 21604 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157030 21600 110 157031 21600 110 157031 21600 109 157030 21600 109 157030 21600 110</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157031 21604 109 157031 21604 110 157030 21604 110 157030 21604 109 157031 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157030 21604 109 157030 21604 110 157030 21600 110 157030 21600 109 157030 21604 109</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157031 21600 110 157031 21604 110 157031 21604 109 157031 21600 109 157031 21600 110</gml:posList>
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
	<core:cityObjectMember>
		<uro:SewerPipe gml:id="unf_700d4ace-0a8b-416b-91cc-0dd4bff26c8d">
			<gml:name>雨水管路</gml:name>
			<core:creationDate>2025-03-21T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod1>
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
								<gml:Polygon gml:id="poly-4e74cf2c-6ebe-4389-894a-51f0e9684428">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.0843 21657.1904 111.1032 156931.3657 21656.9823 111.1032 156931.0843 21657.1904 110.7532 156931.0843 21657.1904 111.1032</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-dd0c5537-699d-4200-b45e-08ec1a3657a0">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.0843 21657.1904 110.7532 156931.3657 21656.9823 111.1032 156931.3657 21656.9823 110.7532 156931.0843 21657.1904 110.7532</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-7ad55ac5-04ed-4989-9542-8d553b4e1432">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156947.0362 21678.0094 109.9398 156931.0843 21657.1904 111.1032 156947.0362 21678.0094 109.5898 156947.0362 21678.0094 109.9398</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-2cae209f-e891-4965-a211-0a4a698728ab">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156947.0362 21678.0094 109.5898 156931.0843 21657.1904 111.1032 156931.0843 21657.1904 110.7532 156947.0362 21678.0094 109.5898</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-3e8b26d6-cb0c-4962-8427-4a97dc138789">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156947.305 21677.7852 109.9398 156947.0362 21678.0094 109.9398 156947.305 21677.7852 109.5898 156947.305 21677.7852 109.9398</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-8b19f7ea-3f26-4a3b-b7b0-8b1e368c937c">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156947.305 21677.7852 109.5898 156947.0362 21678.0094 109.9398 156947.0362 21678.0094 109.5898 156947.305 21677.7852 109.5898</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-c3ef3057-37b7-4cda-9caf-8297de753c88">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156947.305 21677.7852 109.5898 156931.3657 21656.9823 110.7532 156931.3657 21656.9823 111.1032 156947.305 21677.7852 109.5898</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-6066f7bf-487e-48c9-8f33-42f6d1481407">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156947.305 21677.7852 109.5898 156931.3657 21656.9823 111.1032 156947.305 21677.7852 109.9398 156947.305 21677.7852 109.5898</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-02de0eb7-4b2b-48e2-a8c1-5d03428137fd">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.3657 21656.9823 111.1032 156931.0843 21657.1904 111.1032 156947.305 21677.7852 109.9398 156931.3657 21656.9823 111.1032</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-44bd0fd8-2c69-41c3-baea-d9d7bf53af27">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156947.305 21677.7852 109.9398 156931.0843 21657.1904 111.1032 156947.0362 21678.0094 109.9398 156947.305 21677.7852 109.9398</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-ce7a89f5-1a7e-4c24-a6a7-d343ed0ba285">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.0843 21657.1904 110.7532 156931.3657 21656.9823 110.7532 156947.305 21677.7852 109.5898 156931.0843 21657.1904 110.7532</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-8322a1e0-aba5-4898-8735-10b9cb6d9b86">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.0843 21657.1904 110.7532 156947.305 21677.7852 109.5898 156947.0362 21678.0094 109.5898 156931.0843 21657.1904 110.7532</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
		</uro:SewerPipe>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:SewerPipe gml:id="unf_93ca659c-75bd-4053-a94d-b269183382ba">
			<gml:name>雨水管路</gml:name>
			<core:creationDate>2025-03-21T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod1>
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
								<gml:Polygon gml:id="poly-2bcb23f5-4a28-49c7-b5ff-d9ddd43e3fe5">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156955.3879 21643.529 110.916 156931.3657 21656.9823 111.1098 156955.3879 21643.529 110.566 156955.3879 21643.529 110.916</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-ea12c13f-0f4f-429d-aff6-4c0588436631">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156955.3879 21643.529 110.566 156931.3657 21656.9823 111.1098 156931.3657 21656.9823 110.7598 156955.3879 21643.529 110.566</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-4cbfbb16-c387-4947-9bb7-0e0901d46c73">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156955.1798 21643.2476 110.916 156955.3879 21643.529 110.916 156955.1798 21643.2476 110.566 156955.1798 21643.2476 110.916</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-83db8384-95e0-43a5-9d2d-9cb38d24f257">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156955.1798 21643.2476 110.566 156955.3879 21643.529 110.916 156955.3879 21643.529 110.566 156955.1798 21643.2476 110.566</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-eafb03d6-f4d4-49c2-89f2-ec2bf6b41546">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.1575 21656.7009 111.1098 156955.1798 21643.2476 110.916 156931.1575 21656.7009 110.7598 156931.1575 21656.7009 111.1098</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-19d92ff4-e5d4-4972-a62c-470fb5f6a8b6">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.1575 21656.7009 110.7598 156955.1798 21643.2476 110.916 156955.1798 21643.2476 110.566 156931.1575 21656.7009 110.7598</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-31aa4012-1411-4b1a-bbbb-c9f25eb2a751">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.3657 21656.9823 111.1098 156931.1575 21656.7009 111.1098 156931.3657 21656.9823 110.7598 156931.3657 21656.9823 111.1098</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-87d3dd80-d600-4b4b-99b8-9a05791e4612">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.3657 21656.9823 110.7598 156931.1575 21656.7009 111.1098 156931.1575 21656.7009 110.7598 156931.3657 21656.9823 110.7598</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-58406c22-8fb5-4330-9c97-278e5b1302e0">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.1575 21656.7009 111.1098 156931.3657 21656.9823 111.1098 156955.3879 21643.529 110.916 156931.1575 21656.7009 111.1098</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-20adf3ed-6a87-453d-9caa-2bc5898db9f0">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156931.1575 21656.7009 111.1098 156955.3879 21643.529 110.916 156955.1798 21643.2476 110.916 156931.1575 21656.7009 111.1098</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-a52741ea-ec78-4df5-b815-7a7f951ea747">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156955.3879 21643.529 110.566 156931.3657 21656.9823 110.7598 156931.1575 21656.7009 110.7598 156955.3879 21643.529 110.566</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-38b654e2-7e3f-49ae-b30f-cf07a68abf48">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>156955.3879 21643.529 110.566 156931.1575 21656.7009 110.7598 156955.1798 21643.2476 110.566 156955.3879 21643.529 110.566</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
		</uro:SewerPipe>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_97a8eb9d-0c74-44bd-9a9d-536d29d1d697">
			<gml:name>雨水マンホール</gml:name>
			<core:creationDate>2025-03-21T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod2>
					<urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod3>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod2Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-87cdb310-e1a7-43bf-9a9b-419ef54d0240">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157112.3805 21579.9834 135.146 157112.8815 21579.6533 135.146 157112.8815 21579.6533 133.796 157112.3805 21579.9834 133.796 157112.3805 21579.9834 135.146</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-6816fcdc-8074-4c28-ae06-76b4b8511001">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157113.2116 21580.1544 133.796 157112.8815 21579.6533 133.796 157112.8815 21579.6533 135.146 157113.2116 21580.1544 135.146 157113.2116 21580.1544 133.796</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-0858715a-3c54-409e-b03d-5801cc710af4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157112.7105 21580.4844 135.146 157113.2116 21580.1544 135.146 157112.8815 21579.6533 135.146 157112.3805 21579.9834 135.146 157112.7105 21580.4844 135.146</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-8568b3f7-7caa-4910-b2bf-76d50d27907a">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157112.3805 21579.9834 133.796 157112.8815 21579.6533 133.796 157113.2116 21580.1544 133.796 157112.7105 21580.4844 133.796 157112.3805 21579.9834 133.796</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-c3c5f313-7e04-4e7b-91e5-214f91e3fdf0">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157112.7105 21580.4844 133.796 157113.2116 21580.1544 133.796 157113.2116 21580.1544 135.146 157112.7105 21580.4844 135.146 157112.7105 21580.4844 133.796</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-2c0b28cd-da16-4694-9353-34efe9e63f97">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157112.3805 21579.9834 133.796 157112.7105 21580.4844 133.796 157112.7105 21580.4844 135.146 157112.3805 21579.9834 135.146 157112.3805 21579.9834 133.796</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_d288d7ab-a62c-4f68-ac63-9208255655ca">
			<gml:name>雨水マンホール</gml:name>
			<core:creationDate>2025-03-21T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</urc:geometrySrcDescLod2>
					<urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod3>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod2Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
					<gml:exterior>
						<gml:Shell>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-c5be910c-e460-44d6-8f69-0f5acf9571cb">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157107.9336 21548.1977 137.066 157108.0515 21547.6094 137.066 157108.0515 21547.6094 135.716 157107.9336 21548.1977 135.716 157107.9336 21548.1977 137.066</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-8c475eae-8cab-4133-a417-1c14c29c1f3e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157108.6398 21547.7274 135.716 157108.0515 21547.6094 135.716 157108.0515 21547.6094 137.066 157108.6398 21547.7274 137.066 157108.6398 21547.7274 135.716</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-6b567da1-7119-4de4-9d57-6916c2759c82">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157108.5219 21548.3157 137.066 157108.6398 21547.7274 137.066 157108.0515 21547.6094 137.066 157107.9336 21548.1977 137.066 157108.5219 21548.3157 137.066</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-d677d109-5d7e-44f2-a9b7-7f411b5005d6">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157107.9336 21548.1977 135.716 157108.0515 21547.6094 135.716 157108.6398 21547.7274 135.716 157108.5219 21548.3157 135.716 157107.9336 21548.1977 135.716</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-dba469d0-b858-47b8-97c7-7136beca6ad6">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157108.5219 21548.3157 135.716 157108.6398 21547.7274 135.716 157108.6398 21547.7274 137.066 157108.5219 21548.3157 137.066 157108.5219 21548.3157 135.716</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly-1bddd4b0-0b2d-4e65-a86f-89f0f0a94554">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>157107.9336 21548.1977 135.716 157108.5219 21548.3157 135.716 157108.5219 21548.3157 137.066 157107.9336 21548.1977 137.066 157107.9336 21548.1977 135.716</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:Shell>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
		</uro:Manhole>
	</core:cityObjectMember>
</core:CityModel>
