<?xml version="1.0" encoding="UTF-8"?>
<!--
  L-bldg-04: building usage attribute dependency check (CityGML 3.0 / i-UR 4.0).

  Building usage attributes (uro:majorUsage / uro:detailedUsage / *FloorUsage ...)
  live in uro:BuildingDetailAttribute. In CityGML 3.0 / i-UR 4.0 this complexType
  is unchanged in name and element names; only the host property changes from the
  i-UR 3.x wrapper <uro:buildingDetailAttribute> to the CityGML 3.0 ADE hook
  <bldg:adeOfAbstractBuilding> (verified against urbanObject.xsd 4.0,
  substitutionGroup="bldg:ADEOfAbstractBuilding"). uro:surveyYear also changed from
  gYear to date in 4.0, so it is written as 2020-04-01.

  Building 1 (16211-bldg-78): normal - has uro:detailedUsage only (no derived attr).
  Building 2 (16211-bldg-77): error - carries TWO derived usage attributes without
  their required parents in a single building:
    - uro:majorUsage2      without uro:majorUsage
    - uro:secondFloorUsage without uro:groundFloorUsage
  -> 2 L-bldg-04,05 violation messages for the same building. This fixture pins
  down how a multi-violation building is rendered (one CSV row whose error cell
  carries both messages) so the usage-error attribute shape stays locked.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd
urn:oasis:names:tc:ciq:xal:3 ../../schemas/citygml/xAL/3.0/xAL.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647 137.052 0</gml:lowerCorner>
			<gml:upperCorner>36.648 137.056 110</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- Building 1: normal (uro:majorUsage absent, no derived usage attr either) -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_lbu04-0001-4a01-8001-aaaaaaaaaaaa">
			<gml:name>16211-bldg-78</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3003</bldg:class>
			<bldg:usage>411</bldg:usage>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.6</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:Address xmlns:xAL="urn:oasis:names:tc:ciq:xal:3">
							<xAL:FreeTextAddress>
								<xAL:AddressLine>富山県射水市</xAL:AddressLine>
							</xAL:FreeTextAddress>
						</xAL:Address>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lbu04-0001">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_lbu04-0001">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6477 137.0537 0 36.6478 137.0537 0 36.6478 137.0536 0 36.6477 137.0536 0 36.6477 137.0537 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<core:lod1Solid>
				<gml:Solid gml:id="solid_lod1_lbu04-0001">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0537 0 36.6477 137.0536 0 36.6478 137.0536 0 36.6478 137.0537 0 36.6477 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0537 0 36.6478 137.0537 0 36.6478 137.0537 9 36.6477 137.0537 9 36.6477 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6478 137.0537 0 36.6478 137.0536 0 36.6478 137.0536 9 36.6478 137.0537 9 36.6478 137.0537 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6478 137.0536 0 36.6477 137.0536 0 36.6477 137.0536 9 36.6478 137.0536 9 36.6478 137.0536 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0536 0 36.6477 137.0537 0 36.6477 137.0537 9 36.6477 137.0536 9 36.6477 137.0536 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0537 9 36.6478 137.0537 9 36.6478 137.0536 9 36.6477 137.0536 9 36.6477 137.0537 9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-78</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">76.2</uro:totalFloorArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4111</uro:detailedUsage>
					<uro:surveyYear>2020-04-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<!-- Building 2: error - uro:majorUsage2 present without uro:majorUsage -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_lbu04-0002-4a01-8001-bbbbbbbbbbbb">
			<gml:name>16211-bldg-77</gml:name>
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<bldg:class>3003</bldg:class>
			<bldg:usage>411</bldg:usage>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<con:height>
				<con:Height>
					<con:highReference>highestRoofEdge</con:highReference>
					<con:lowReference>lowestGroundPoint</con:lowReference>
					<con:status>measured</con:status>
					<con:value uom="m">8.6</con:value>
				</con:Height>
			</con:height>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:Address xmlns:xAL="urn:oasis:names:tc:ciq:xal:3">
							<xAL:FreeTextAddress>
								<xAL:AddressLine>富山県射水市</xAL:AddressLine>
							</xAL:FreeTextAddress>
						</xAL:Address>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<core:lod0MultiSurface>
				<gml:MultiSurface gml:id="ms_lbu04-0002">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_lbu04-0002">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6477 137.0557 0 36.6478 137.0557 0 36.6478 137.0556 0 36.6477 137.0556 0 36.6477 137.0557 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<core:lod1Solid>
				<gml:Solid gml:id="solid_lod1_lbu04-0002">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0557 0 36.6477 137.0556 0 36.6478 137.0556 0 36.6478 137.0557 0 36.6477 137.0557 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0557 0 36.6478 137.0557 0 36.6478 137.0557 9 36.6477 137.0557 9 36.6477 137.0557 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6478 137.0557 0 36.6478 137.0556 0 36.6478 137.0556 9 36.6478 137.0557 9 36.6478 137.0557 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6478 137.0556 0 36.6477 137.0556 0 36.6477 137.0556 9 36.6478 137.0556 9 36.6478 137.0556 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0556 0 36.6477 137.0557 0 36.6477 137.0557 9 36.6477 137.0556 9 36.6477 137.0556 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0557 9 36.6478 137.0557 9 36.6478 137.0556 9 36.6477 137.0556 9 36.6477 137.0557 9</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-77</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">15.5</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<!-- TEST: two simultaneous L-bldg-04,05 violations in ONE building.
					     (1) majorUsage2 present, majorUsage absent.
					     (2) secondFloorUsage present, groundFloorUsage absent.
					     No codeSpace: only key presence matters for this check. -->
					<uro:majorUsage2>01002</uro:majorUsage2>
					<uro:secondFloorUsage>412</uro:secondFloorUsage>
					<uro:detailedUsage>461</uro:detailedUsage>
					<uro:surveyYear>2020-04-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
