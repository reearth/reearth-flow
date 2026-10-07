<?xml version="1.0" encoding="UTF-8"?>
<!--
  A sediment disaster prone area whose exterior and hole both cross themselves.

  The face is given in a local grid of (east, north) steps of 1e-5 degrees
  from (36.182, 140.130). The exterior runs along two bow ties joined end to
  end, (0, 0) (8, 5) (16, 0) (16, 8) (8, 3) (0, 8), and crosses itself at
  (6.4, 4) and (9.6, 4). The hole is a bow tie in the left lobe, (1, 2) (1, 5)
  (3, 3) (3, 4), crossing itself at (2.5, 3.5).
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
			<gml:upperCorner>36.182080 140.130160 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<urf:SedimentDisasterProneArea gml:id="lsld_9c0ff17e-5ec4-4e2c-9481-d342285b2de7">
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
						<gml:Polygon gml:id="poly_aa7d5ff6-b9ad-4806-bc5e-ce6a56f89533">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.182000 140.130000 0 36.182050 140.130080 0 36.182000 140.130160 0 36.182080 140.130160 0 36.182030 140.130080 0 36.182080 140.130000 0 36.182000 140.130000 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
								<gml:interior>
									<gml:LinearRing>
										<gml:posList>36.182020 140.130010 0 36.182050 140.130010 0 36.182030 140.130030 0 36.182040 140.130030 0 36.182020 140.130010 0</gml:posList>
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
