<?xml version="1.0" encoding="UTF-8"?>
<!--
  CityGML 3.0 + i-UR 4.0 no-error fixture.
  Intentionally exercises the major changes introduced in CityGML 3.0
  so that the plateau6 quality-check workflow can be verified to handle them:
    - core/3.0, bldg/3.0, con/3.0, gml/3.2, uro/4.0 namespaces
    - Space concept: geometric properties live on core: (core:lod1Solid, core:lod2Solid)
    - SpaceBoundary concept: core:boundary referencing con:RoofSurface / con:WallSurface
      / con:GroundSurface (thematic surfaces moved from bldg: to con: in 3.0)
    - construction module properties: con:dateOfConstruction (replaces
      bldg:yearOfConstruction), con:conditionOfConstruction
    - bldg:BuildingRoom (renamed from bldg:Room in 2.0) as an interior space at LOD3
    - LOD3 interior (LOD4 was abolished and folded into LOD3 in 3.0)
    - i-UR 4.0 ADE pattern: bldg:adeOfAbstractBuilding wrapping uro attributes
    - codeSpace URLs use https://www.geospatial.jp/iur/codelists/4.0/...
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:app="http://www.opengis.net/citygml/appearance/3.0"
	xmlns:gen="http://www.opengis.net/citygml/generics/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/3.0/construction.xsd
http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/3.0/appearance.xsd
http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/3.0/generics.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>34.73450000 135.50200000 0</gml:lowerCorner>
			<gml:upperCorner>34.73470000 135.50220000 12.5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_49303425_b001">
			<core:creationDate>2026-04-01</core:creationDate>
			<!-- con: namespace (CityGML 3.0): construction-related properties -->
			<con:conditionOfConstruction>functional</con:conditionOfConstruction>
			<con:dateOfConstruction>2020-04-01</con:dateOfConstruction>
			<bldg:class codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Building_usage.xml">411</bldg:usage>
			<bldg:storeysAboveGround>3</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<!-- Space concept: LOD0 footprint on core: -->
			<core:lod0MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_lod0_b001">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>34.73455 135.50205 0 34.73455 135.50215 0 34.73465 135.50215 0 34.73465 135.50205 0 34.73455 135.50205 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<!-- Space concept: LOD1 solid on core: (was bldg:lod1Solid in 2.0) -->
			<core:lod1Solid>
				<gml:Solid gml:id="solid_lod1_b001">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50205 0 34.73465 135.50205 0 34.73465 135.50215 0 34.73455 135.50215 0 34.73455 135.50205 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50205 10 34.73455 135.50215 10 34.73465 135.50215 10 34.73465 135.50205 10 34.73455 135.50205 10</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50205 0 34.73455 135.50215 0 34.73455 135.50215 10 34.73455 135.50205 10 34.73455 135.50205 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73465 135.50205 0 34.73465 135.50205 10 34.73465 135.50215 10 34.73465 135.50215 0 34.73465 135.50205 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50205 0 34.73455 135.50205 10 34.73465 135.50205 10 34.73465 135.50205 0 34.73455 135.50205 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50215 0 34.73465 135.50215 0 34.73465 135.50215 10 34.73455 135.50215 10 34.73455 135.50215 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod1Solid>
			<!-- Space concept: LOD2 solid on core: -->
			<core:lod2Solid>
				<gml:Solid gml:id="solid_lod2_b001">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember xlink:href="#surf_ground_b001"/>
							<gml:surfaceMember xlink:href="#surf_roof_b001"/>
							<gml:surfaceMember xlink:href="#surf_wall_n_b001"/>
							<gml:surfaceMember xlink:href="#surf_wall_e_b001"/>
							<gml:surfaceMember xlink:href="#surf_wall_s_b001"/>
							<gml:surfaceMember xlink:href="#surf_wall_w_b001"/>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
			<!-- SpaceBoundary concept: core:boundary with con: thematic surfaces -->
			<core:boundary>
				<con:GroundSurface gml:id="gs_b001">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="surf_ground_b001">
									<gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50205 0 34.73465 135.50205 0 34.73465 135.50215 0 34.73455 135.50215 0 34.73455 135.50205 0</gml:posList></gml:LinearRing></gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:GroundSurface>
			</core:boundary>
			<core:boundary>
				<con:RoofSurface gml:id="rs_b001">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="surf_roof_b001">
									<gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50205 10 34.73455 135.50215 10 34.73465 135.50215 10 34.73465 135.50205 10 34.73455 135.50205 10</gml:posList></gml:LinearRing></gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:RoofSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="ws_n_b001">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="surf_wall_n_b001">
									<gml:exterior><gml:LinearRing><gml:posList>34.73465 135.50205 0 34.73465 135.50205 10 34.73465 135.50215 10 34.73465 135.50215 0 34.73465 135.50205 0</gml:posList></gml:LinearRing></gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="ws_e_b001">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="surf_wall_e_b001">
									<gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50215 0 34.73465 135.50215 0 34.73465 135.50215 10 34.73455 135.50215 10 34.73455 135.50215 0</gml:posList></gml:LinearRing></gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="ws_s_b001">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="surf_wall_s_b001">
									<gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50205 0 34.73455 135.50215 0 34.73455 135.50215 10 34.73455 135.50205 10 34.73455 135.50205 0</gml:posList></gml:LinearRing></gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<core:boundary>
				<con:WallSurface gml:id="ws_w_b001">
					<core:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="surf_wall_w_b001">
									<gml:exterior><gml:LinearRing><gml:posList>34.73455 135.50205 0 34.73455 135.50205 10 34.73465 135.50205 10 34.73465 135.50205 0 34.73455 135.50205 0</gml:posList></gml:LinearRing></gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod2MultiSurface>
				</con:WallSurface>
			</core:boundary>
			<!-- LOD3 interior space: BuildingRoom (renamed from Room in 3.0). LOD4 was abolished. -->
			<bldg:buildingRoom>
				<bldg:BuildingRoom gml:id="room_b001_1">
					<bldg:class codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Room_class.xml">SL_42</bldg:class>
					<core:lod3Solid>
						<gml:Solid gml:id="solid_lod3_room_b001_1">
							<gml:exterior>
								<gml:CompositeSurface>
									<gml:surfaceMember>
										<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73456 135.50206 0.3 34.73464 135.50206 0.3 34.73464 135.50214 0.3 34.73456 135.50214 0.3 34.73456 135.50206 0.3</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73456 135.50206 9.7 34.73456 135.50214 9.7 34.73464 135.50214 9.7 34.73464 135.50206 9.7 34.73456 135.50206 9.7</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73456 135.50206 0.3 34.73456 135.50214 0.3 34.73456 135.50214 9.7 34.73456 135.50206 9.7 34.73456 135.50206 0.3</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73464 135.50206 0.3 34.73464 135.50206 9.7 34.73464 135.50214 9.7 34.73464 135.50214 0.3 34.73464 135.50206 0.3</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73456 135.50206 0.3 34.73456 135.50206 9.7 34.73464 135.50206 9.7 34.73464 135.50206 0.3 34.73456 135.50206 0.3</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73456 135.50214 0.3 34.73464 135.50214 0.3 34.73464 135.50214 9.7 34.73456 135.50214 9.7 34.73456 135.50214 0.3</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>
									</gml:surfaceMember>
								</gml:CompositeSurface>
							</gml:exterior>
						</gml:Solid>
					</core:lod3Solid>
				</bldg:BuildingRoom>
			</bldg:buildingRoom>
			<!-- i-UR 4.0 ADE pattern: extension attributes wrapped via bldg:adeOfAbstractBuilding -->
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingIDAttribute>
					<uro:buildingID>27000-bldg-1</uro:buildingID>
					<uro:prefecture codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Common_localPublicAuthorities.xml">27</uro:prefecture>
					<uro:city codeSpace="https://www.geospatial.jp/iur/codelists/4.0/Common_localPublicAuthorities.xml">27000</uro:city>
				</uro:BuildingIDAttribute>
			</bldg:adeOfAbstractBuilding>
			<bldg:adeOfAbstractBuilding>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">300.0</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">100.0</uro:buildingFootprintArea>
					<uro:buildingStructureType codeSpace="https://www.geospatial.jp/iur/codelists/4.0/BuildingDetailAttribute_buildingStructureType.xml">603</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="https://www.geospatial.jp/iur/codelists/4.0/BuildingDetailAttribute_fireproofStructureType.xml">1001</uro:fireproofStructureType>
				</uro:BuildingDetailAttribute>
			</bldg:adeOfAbstractBuilding>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
