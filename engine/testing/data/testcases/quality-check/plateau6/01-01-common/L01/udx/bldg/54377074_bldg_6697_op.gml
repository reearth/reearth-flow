<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gen="http://www.opengis.net/citygml/generics/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.6470041354812 137.05268308385453 0</gml:lowerCorner>
			<gml:upperCorner>36.647798243275254 137.0537094956814 105.03314</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef114-02e4-11f0-a3af-18ece7a5508c">
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<bldg:class codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Building_class.xml">3003</bldg:class>
			<!-- TEST: intentionally malformed XML for L01 - the start tag is bldg:usage but the end tag is bldg:class, so the XML is not well-formed (tag mismatch). The XMLValidator emits a "SyntaxError" / "Invalid document structure". -->
			<bldg:usage codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Building_usage.xml">411</bldg:class>
			<bldg:storeysAboveGround>9</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<core:lod0MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.64773967207627 137.0537094956814 0 36.647798243275254 137.05370057460766 0 36.64778832538864 137.053600937653 0 36.64772975430324 137.053609970643 0 36.64773967207627 137.0537094956814 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-78</uro:buildingID>
					<uro:prefecture codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
