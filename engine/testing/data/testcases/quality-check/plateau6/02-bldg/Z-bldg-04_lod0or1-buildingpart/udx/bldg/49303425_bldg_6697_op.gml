<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>34.73450000 135.50200000 0</gml:lowerCorner>
			<gml:upperCorner>34.73495000 135.50220000 12.5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- Z-bldg-04 error detection (CityGML 3.0): a BuildingPart that carries LOD0 or LOD1
	     geometry. LOD0/LOD1 BuildingParts are not allowed and are reported per LOD;
	     LOD2/LOD3 BuildingParts route to the separate L-bldg-02 connectivity check.
	     CityGML 2.0 -> 3.0 changes vs the plateau4 fixture (Z-bldg-04_lod0or1-buildingpart):
	     bldg:consistsOfBuildingPart -> bldg:buildingPart, bldg:lod0FootPrint ->
	     core:lod0MultiSurface, bldg:lodNSolid -> core:lodNSolid, gml 3.1.1 -> gml 3.2,
	     i-UR 3.1 -> i-UR 4.0, bldg:measuredHeight -> con:height. Coordinates are ported
	     verbatim from the plateau4 fixture so the reported LODs are identical. -->
	<!-- Error building: a BuildingPart with both LOD0 (footprint) and LOD1 (solid) geometry.
	     Reported twice: one row for lod=0 and one row for lod=1. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_z04-0001">
			<gml:name>テスト建物</gml:name>
			<bldg:buildingPart>
				<bldg:BuildingPart gml:id="bpart_z04-0001">
					<gml:name>テスト建物部分</gml:name>
					<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">12.5</con:value></con:Height></con:height>
					<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
					<core:lod0MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>34.73450000 135.50200000 0 34.73450000 135.50220000 0 34.73470000 135.50220000 0 34.73470000 135.50200000 0 34.73450000 135.50200000 0</gml:posList>
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
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73450000 135.50200000 0 34.73470000 135.50200000 0 34.73470000 135.50220000 0 34.73450000 135.50220000 0 34.73450000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73450000 135.50200000 0 34.73450000 135.50220000 0 34.73450000 135.50220000 12.5 34.73450000 135.50200000 12.5 34.73450000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73450000 135.50220000 0 34.73470000 135.50220000 0 34.73470000 135.50220000 12.5 34.73450000 135.50220000 12.5 34.73450000 135.50220000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73470000 135.50220000 0 34.73470000 135.50200000 0 34.73470000 135.50200000 12.5 34.73470000 135.50220000 12.5 34.73470000 135.50220000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73470000 135.50200000 0 34.73450000 135.50200000 0 34.73450000 135.50200000 12.5 34.73470000 135.50200000 12.5 34.73470000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73450000 135.50200000 12.5 34.73450000 135.50220000 12.5 34.73470000 135.50220000 12.5 34.73470000 135.50200000 12.5 34.73450000 135.50200000 12.5</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
								</gml:CompositeSurface>
							</gml:exterior>
						</gml:Solid>
					</core:lod1Solid>
				</bldg:BuildingPart>
			</bldg:buildingPart>
		</bldg:Building>
	</core:cityObjectMember>
	<!-- Control building: a BuildingPart with only LOD2 geometry. Not a Z-bldg-04 target
	     (LOD2 parts route to the L-bldg-02 connectivity check), so it produces no
	     LOD0-1 BuildingPart error. -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_z04-0002">
			<gml:name>テスト建物2</gml:name>
			<bldg:buildingPart>
				<bldg:BuildingPart gml:id="bpart_z04-0002">
					<gml:name>テスト建物部分2 LOD2</gml:name>
					<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">10.0</con:value></con:Height></con:height>
					<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
					<core:lod2Solid>
						<gml:Solid>
							<gml:exterior>
								<gml:CompositeSurface>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50200000 0 34.73495000 135.50200000 0 34.73495000 135.50215000 0 34.73480000 135.50215000 0 34.73480000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50200000 0 34.73480000 135.50215000 0 34.73480000 135.50215000 10.0 34.73480000 135.50200000 10.0 34.73480000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50215000 0 34.73495000 135.50215000 0 34.73495000 135.50215000 10.0 34.73480000 135.50215000 10.0 34.73480000 135.50215000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73495000 135.50215000 0 34.73495000 135.50200000 0 34.73495000 135.50200000 10.0 34.73495000 135.50215000 10.0 34.73495000 135.50215000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73495000 135.50200000 0 34.73480000 135.50200000 0 34.73480000 135.50200000 10.0 34.73495000 135.50200000 10.0 34.73495000 135.50200000 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
									<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.73480000 135.50200000 10.0 34.73480000 135.50215000 10.0 34.73495000 135.50215000 10.0 34.73495000 135.50200000 10.0 34.73480000 135.50200000 10.0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
								</gml:CompositeSurface>
							</gml:exterior>
						</gml:Solid>
					</core:lod2Solid>
				</bldg:BuildingPart>
			</bldg:buildingPart>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
