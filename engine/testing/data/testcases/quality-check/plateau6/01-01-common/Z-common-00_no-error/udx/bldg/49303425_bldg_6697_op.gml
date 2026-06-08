<?xml version="1.0" encoding="UTF-8"?>
<!--
  CityGML 3.0 + GML 3.2 minimal no-error fixture for plateau6 quality-check
  01-01-common Z-common-00. Uses only the OGC CityGML 3.0 core/building/
  construction modules so XSD validation resolves entirely from
  schemas.opengis.net and does not depend on the i-UR 4.0 ADE
  (substitutionGroup handling for ADEOfAbstractBuilding is not yet supported
  in fastxml; see README §7). DomainOfDefinitionValidator expectations:
    - bldg:Building gml:id matches `bldg_<uuid>` format (gml:id書式不正 = 0)
    - srsName is EPSG/0/6697 (L05 = OK)
    - posList coords stay within the boundedBy envelope (L06 = 0)
    - no codeSpaces (L04 / inCorrectCodeSpace = 0)
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647 137.052 0</gml:lowerCorner>
			<gml:upperCorner>36.648 137.054 110</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_f036798c-4381-4af2-97fc-e78d8fe34001">
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_f036798c-4381-4af2-97fc-e78d8fe34001">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_f036798c-4381-4af2-97fc-e78d8fe34001">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6477 137.0537 0 36.6478 137.0537 0 36.6478 137.0536 0 36.6477 137.0536 0 36.6477 137.0537 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
