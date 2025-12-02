<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:gml="http://www.opengis.net/gml" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.0 135.0 0</gml:lowerCorner>
			<gml:upperCorner>35.001 135.001 10</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="test-building-001">
			<bldg:measuredHeight uom="m">10.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>3</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.0 135.0 0 35.0 135.001 0 35.001 135.001 0 35.001 135.0 0 35.0 135.0 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0 135.0 0 35.0 135.001 0 35.001 135.001 0 35.001 135.0 0 35.0 135.0 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.0 135.0 10 35.001 135.0 10 35.001 135.001 10 35.0 135.001 10 35.0 135.0 10</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>TEST-BLDG-001</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13101</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">300.5</uro:totalFloorArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">601</uro:buildingStructureType>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
