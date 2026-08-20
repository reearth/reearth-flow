<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:app="http://www.opengis.net/citygml/appearance/3.0"
	xmlns:gen="http://www.opengis.net/citygml/generics/3.0"
	xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd
http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd
http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd
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
			<con:dateOfConstruction>2020-01-01</con:dateOfConstruction>
			<!-- TEST (L04エラー): codeSpace resolves to Building_class.xml, but the value
			     100 is not a defined code in that codelist, so the code value is invalid
			     (L04エラー=1, 不正なcodeSpace数=0). -->
			<bldg:class codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Building_class.xml">100</bldg:class>
			<bldg:usage codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Building_usage.xml">411</bldg:usage>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
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
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64773967207627 137.0537094956814 101.19183 36.64772975430324 137.053609970643 101.19183 36.64778832538864 137.053600937653 101.19183 36.647798243275254 137.05370057460766 101.19183 36.64773967207627 137.0537094956814 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64773967207627 137.0537094956814 101.19183 36.647798243275254 137.05370057460766 101.19183 36.647798243275254 137.05370057460766 105.03314 36.64773967207627 137.0537094956814 105.03314 36.64773967207627 137.0537094956814 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647798243275254 137.05370057460766 101.19183 36.64778832538864 137.053600937653 101.19183 36.64778832538864 137.053600937653 105.03314 36.647798243275254 137.05370057460766 105.03314 36.647798243275254 137.05370057460766 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64778832538864 137.053600937653 101.19183 36.64772975430324 137.053609970643 101.19183 36.64772975430324 137.053609970643 105.03314 36.64778832538864 137.053600937653 105.03314 36.64778832538864 137.053600937653 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64772975430324 137.053609970643 101.19183 36.64773967207627 137.0537094956814 101.19183 36.64773967207627 137.0537094956814 105.03314 36.64772975430324 137.053609970643 105.03314 36.64772975430324 137.053609970643 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64773967207627 137.0537094956814 105.03314 36.647798243275254 137.05370057460766 105.03314 36.64778832538864 137.053600937653 105.03314 36.64772975430324 137.053609970643 105.03314 36.64773967207627 137.0537094956814 105.03314</gml:posList>
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
					<uro:prefecture codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">76.2</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">76.2</uro:buildingFootprintArea>
					<uro:buildingStructureType codeSpace="https://www.geospatial.jp/iur/codelists/4.0/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="https://www.geospatial.jp/iur/codelists/4.0/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020-04-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef115-02e4-11f0-810e-18ece7a5508c">
			<core:creationDate>2025-03-21T00:00:00</core:creationDate>
			<con:dateOfConstruction>0001-01-01</con:dateOfConstruction>
			<bldg:class codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Building_usage.xml">461</bldg:usage>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<core:lod0MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.64705388584696 137.0527173474674 0 36.64705502737237 137.05268591841337 0 36.64700527700596 137.05268308385453 0 36.6470041354812 137.05271451288834 0 36.64705388584696 137.0527173474674 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<core:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64705388584696 137.0527173474674 100.85896 36.6470041354812 137.05271451288834 100.85896 36.64700527700596 137.05268308385453 100.85896 36.64705502737237 137.05268591841337 100.85896 36.64705388584696 137.0527173474674 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64705388584696 137.0527173474674 100.85896 36.64705502737237 137.05268591841337 100.85896 36.64705502737237 137.05268591841337 104.29314 36.64705388584696 137.0527173474674 104.29314 36.64705388584696 137.0527173474674 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64705502737237 137.05268591841337 100.85896 36.64700527700596 137.05268308385453 100.85896 36.64700527700596 137.05268308385453 104.29314 36.64705502737237 137.05268591841337 104.29314 36.64705502737237 137.05268591841337 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64700527700596 137.05268308385453 100.85896 36.6470041354812 137.05271451288834 100.85896 36.6470041354812 137.05271451288834 104.29314 36.64700527700596 137.05268308385453 104.29314 36.64700527700596 137.05268308385453 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6470041354812 137.05271451288834 100.85896 36.64705388584696 137.0527173474674 100.85896 36.64705388584696 137.0527173474674 104.29314 36.6470041354812 137.05271451288834 104.29314 36.6470041354812 137.05271451288834 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64705388584696 137.0527173474674 104.29314 36.64705502737237 137.05268591841337 104.29314 36.64700527700596 137.05268308385453 104.29314 36.6470041354812 137.05271451288834 104.29314 36.64705388584696 137.0527173474674 104.29314</gml:posList>
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
					<uro:prefecture codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingStructureType codeSpace="https://www.geospatial.jp/iur/codelists/4.0/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="https://www.geospatial.jp/iur/codelists/4.0/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020-04-01</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
