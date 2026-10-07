<?xml version="1.0" encoding="UTF-8"?>
<!--
  A sediment disaster prone area whose hole reaches outside the exterior.

  The face is given in a local grid of (east, north) steps of 1e-5 degrees
  from (36.182, 140.130). The exterior is the square (0, 0) to (10, 10); the
  hole is the rectangle (7, 3) to (13, 6), so its part beyond x = 10 lies
  outside the face.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:urf="https://www.geospatial.jp/iur/urf/4.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
https://www.geospatial.jp/iur/urf/4.0 ../../schemas/iur/urf/4.0/urbanFunction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.182000 140.130000 0</gml:lowerCorner>
			<gml:upperCorner>36.182100 140.130130 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<urf:SedimentDisasterProneArea gml:id="lsld_e8fbf938-1f24-4fcd-a8c8-812fe52a3389">
			<gml:description>220-Ⅰ-015</gml:description>
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:validFrom>2012-02-09T00:00:00</core:validFrom>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_359c4bcc-5648-4a7b-8e90-27cfc5e64bfc">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.182000 140.130000 0 36.182000 140.130100 0 36.182100 140.130100 0 36.182100 140.130000 0 36.182000 140.130000 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
								<gml:interior>
									<gml:LinearRing>
										<gml:posList>36.182030 140.130070 0 36.182060 140.130070 0 36.182060 140.130130 0 36.182030 140.130130 0 36.182030 140.130070 0</gml:posList>
									</gml:LinearRing>
								</gml:interior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<urf:validFromType codeSpace="../../codelists/Common_validType.xml">1</urf:validFromType>
			<urf:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">08</urf:prefecture>
			<urf:location>山口</urf:location>
			<urf:disasterType codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">2</urf:disasterType>
			<urf:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">2</urf:areaType>
			<urf:zoneNumber>220-Ⅰ-015</urf:zoneNumber>
			<urf:zoneName>八幡沢</urf:zoneName>
		</urf:SedimentDisasterProneArea>
	</core:cityObjectMember>
</core:CityModel>
