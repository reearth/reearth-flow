<?xml version="1.0" encoding="UTF-8"?>
<!--
  City-code error test (CITY-CODE / 市区町村コードエラー) for plateau6.

  Each building carries a uro:city value under uro:BuildingIDAttribute. The check
  looks the value up in codelists/Common_localPublicAuthorities.xml and flags:
    - a value not present in the codelist (コードリストに該当なし)
    - a value that is a designated-city (政令指定都市) code, i.e. one that should
      instead use a ward code (要修正（区のコードとする）)

  Building 1: uro:city = 16211 (富山県射水市, in codelist, not designated) -> NOT flagged.
  Building 2: uro:city = 01100 (北海道札幌市, designated city)             -> flagged.
  Building 3: uro:city = 99999 (absent from codelist)                      -> flagged.

  uro:buildingID keeps the valid 16211-bldg-NN form on every building; the check
  reads uro:city, not the buildingID prefix. uro:city is written without codeSpace
  to match the sibling plateau6 Z-bldg-01_URO fixture and to keep the raw code
  readable for the (not-yet-wired) check.

  NOTE: the plateau6 city-code check is not yet wired into the workflow. The
  expected CSV / JSON here mirror the plateau4 CITY-CODE fixtures and are
  provisional; they will be reconciled with the actual output once the check is
  implemented. The i-UR objectlist is the provisional plateau4 version (see skill).
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
			<gml:upperCorner>36.648 137.058 110</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- Building 1: uro:city = 16211 (富山県射水市, valid, not designated) -> NOT flagged -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cc000001-0001-4a01-8001-aaaaaaaaaaaa">
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
			<core:lod1Solid>
				<gml:Solid gml:id="solid_lod1_cc000001-0001">
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
					<uro:buildingID>16211-bldg-77</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<!-- Building 2: uro:city = 01100 (北海道札幌市, designated city) -> flagged 要修正（区のコードとする） -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cc000001-0002-4a01-8001-bbbbbbbbbbbb">
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
			<core:lod1Solid>
				<gml:Solid gml:id="solid_lod1_cc000001-0002">
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
					<uro:buildingID>16211-bldg-78</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>01100</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<!-- Building 3: uro:city = 99999 (absent from codelist) -> flagged コードリストに該当なし -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cc000001-0003-4a01-8001-cccccccccccc">
			<gml:name>16211-bldg-79</gml:name>
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
			<core:lod1Solid>
				<gml:Solid gml:id="solid_lod1_cc000001-0003">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0577 0 36.6477 137.0576 0 36.6478 137.0576 0 36.6478 137.0577 0 36.6477 137.0577 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0577 0 36.6478 137.0577 0 36.6478 137.0577 9 36.6477 137.0577 9 36.6477 137.0577 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6478 137.0577 0 36.6478 137.0576 0 36.6478 137.0576 9 36.6478 137.0577 9 36.6478 137.0577 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6478 137.0576 0 36.6477 137.0576 0 36.6477 137.0576 9 36.6478 137.0576 9 36.6478 137.0576 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0576 0 36.6477 137.0577 0 36.6477 137.0577 9 36.6477 137.0576 9 36.6477 137.0576 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6477 137.0577 9 36.6478 137.0577 9 36.6478 137.0576 9 36.6477 137.0576 9 36.6477 137.0577 9</gml:posList>
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
					<uro:buildingID>16211-bldg-79</uro:buildingID>
					<uro:prefecture>16</uro:prefecture>
					<uro:city>99999</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
