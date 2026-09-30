<?xml version="1.0" encoding="UTF-8"?>
<!--
  Triangles with no or almost no area on the ground, to check that a triangle
  whose corners lie on one line is linear or point-like.
  Coordinates are in EPSG:6697. Orientation is read as (longitude, latitude).
  1. Three collinear corners. Projected, the area is not exactly zero.
  2. Three almost collinear corners, wound clockwise. The third corner is
     1e-6 degrees (about 11 cm) off the line through the other two.
  3. Three almost collinear corners, wound counter-clockwise, 1e-6 degrees
     off the line.
  4. Two corners at the same ground position with heights 15.9 and 16.9.
  5. A ring of five positions A B C B A, which goes out and back and
     encloses no area.
  Expected: 1 and 5 are linear or point-like, 2, 4 and 5 are incorrectly
  oriented, and 5 also has an incorrect vertex count. The three edges of 4
  have no shared edge. The original implementation does not report 1 as
  linear or point-like, and reports its three edges as unshared instead.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.645 137.052 15.9</gml:lowerCorner>
			<gml:upperCorner>36.64718 137.0528 16.9</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_97346922-9a79-4d64-8f1c-ceb9eefea0a6">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">3</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_d0103f05-0112-4a93-ad31-702952112281">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_4dd05056-a1bb-4917-8b3f-c50ea5c978f9">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64718 137.0528 15.9 36.64714 137.05274 15.9 36.64716 137.05277 15.9 36.64718 137.0528 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5d31bac7-3d96-4e78-8ab5-ff227326891e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6465 137.052 15.9 36.6465 137.0525 15.9 36.646499 137.05225 15.9 36.6465 137.052 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5c54ac06-ca30-4c5b-94f2-560882c5d9bf">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.646 137.052 15.9 36.646 137.0525 15.9 36.646001 137.05225 15.9 36.646 137.052 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_80073eef-c2d1-4162-b526-215f697fdcee">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6455 137.052 15.9 36.6455 137.0525 15.9 36.6455 137.0525 16.9 36.6455 137.052 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_01c83721-eaa2-4819-b6e5-0ecac0549bc1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.645 137.052 15.9 36.645 137.0525 15.9 36.6452 137.05225 15.9 36.645 137.0525 15.9 36.645 137.052 15.9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
</core:CityModel>
