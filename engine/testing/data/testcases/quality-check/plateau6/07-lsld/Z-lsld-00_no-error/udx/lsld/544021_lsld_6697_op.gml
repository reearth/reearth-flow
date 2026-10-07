<?xml version="1.0" encoding="UTF-8"?>
<!--
  A valid sediment disaster prone area that none of the lsld checks flags.

  One urf:SedimentDisasterProneArea within tsukuba-shi mesh 544021 (EPSG:6697)
  with a core:lod1MultiSurface holding one planar face, wound counter-clockwise
  in the inspection frame.
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
			<gml:lowerCorner>36.1806149045000 140.1274163783400 0</gml:lowerCorner>
			<gml:upperCorner>36.1809186591360 140.1284657766350 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<urf:SedimentDisasterProneArea gml:id="lsld_119ba6f8-904d-4807-8064-85d957385ee9">
			<gml:description>220-Ⅰ-012</gml:description>
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
						<gml:Polygon gml:id="poly_73afde43-6482-4026-af12-da93739a4148">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.18073068380549 140.1284557766348 0 36.18077932933202 140.12823371334815 0 36.180810081208236 140.12801383009094 0 36.18084530834087 140.1277995087887 0 36.18087982480669 140.1275786131128 0 36.18090865913571 140.1274615452315 0 36.18083693050238 140.12742637834043 0 36.18082235026854 140.1275533027844 0 36.18077040909538 140.1277665254469 0 36.18072545075063 140.12798671013707 0 36.18068679815076 140.1282040476881 0 36.18062490450015 140.12840783975736 0 36.18066931477245 140.12842796541878 0 36.18073068380549 140.1284557766348 0</gml:posList>
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
			<urf:zoneNumber>220-Ⅰ-012</urf:zoneNumber>
			<urf:zoneName>八幡沢</urf:zoneName>
		</urf:SedimentDisasterProneArea>
	</core:cityObjectMember>
</core:CityModel>
