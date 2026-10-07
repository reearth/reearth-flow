<?xml version="1.0" encoding="UTF-8"?>
<!--
  A sediment disaster prone area whose only face crosses itself at two points.

  The exterior ring runs along two bow ties joined end to end. In a local grid
  of (east, north) steps of 1e-5 degrees it visits
      (0, 0) (4, 2.5) (8, 0) (8, 4) (4, 1.5) (0, 4)
  and crosses itself at (3.2, 2) and (4.8, 2). The two outer lobes are wound
  counter-clockwise and are larger than the clockwise lobe between them, so
  the ring as a whole is counter-clockwise and the orientation check stays
  clean.
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
			<gml:lowerCorner>36.18100 140.12900 0</gml:lowerCorner>
			<gml:upperCorner>36.18104 140.12908 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<urf:SedimentDisasterProneArea gml:id="lsld_f6077c5b-3ae9-4f1b-98d4-e4d510073337">
			<gml:description>220-Ⅰ-014</gml:description>
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
						<gml:Polygon gml:id="poly_5511f8b9-747b-4a53-9e28-a32823a5d374">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.18100 140.12900 0 36.181025 140.12904 0 36.18100 140.12908 0 36.18104 140.12908 0 36.181015 140.12904 0 36.18104 140.12900 0 36.18100 140.12900 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<urf:validFromType codeSpace="../../codelists/Common_validType.xml">1</urf:validFromType>
			<urf:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">08</urf:prefecture>
			<urf:location>山口</urf:location>
			<urf:disasterType codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">2</urf:disasterType>
			<urf:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">2</urf:areaType>
			<urf:zoneNumber>220-Ⅰ-014</urf:zoneNumber>
			<urf:zoneName>八幡沢</urf:zoneName>
		</urf:SedimentDisasterProneArea>
	</core:cityObjectMember>
</core:CityModel>
