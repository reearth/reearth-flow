<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>34.742845093252896 137.4500523022449 0</gml:lowerCorner>
			<gml:upperCorner>34.74971667452049 137.46232632011126 308.777</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!-- L14 LOD2/LOD3 test data. Four buildings cover two defect types per LOD:
	     wrong top-face orientation (立体のエラー / InvalidSolidBoundaries) and a
	     missing top face (非水密立体 / Surface Not Closed). Solid geometries are
	     copied verbatim from L14_LOD1 (orientation-error building 0002 and
	     missing-face building 0004); only the lodNSolid property and gml:id differ. -->
	<!-- LOD2 立体のエラー: lod2Solid with wrong top-face orientation (water-tight) -> InvalidSolidBoundaries -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l14lod2-0001">
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>451</bldg:usage>
			<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">6.9</con:value></con:Height></con:height>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<core:lod2Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 86.791 34.74866465534792 137.45958769753437 86.791 34.74878883485885 137.45957022371005 86.791 34.74880269623192 137.45969172632664 86.791 34.74867644246834 137.4597095203081 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 86.791 34.74880269623192 137.45969172632664 86.791 34.74880269623192 137.45969172632664 93.017 34.74867644246834 137.4597095203081 93.017 34.74867644246834 137.4597095203081 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74880269623192 137.45969172632664 86.791 34.74878883485885 137.45957022371005 86.791 34.74878883485885 137.45957022371005 93.017 34.74880269623192 137.45969172632664 93.017 34.74880269623192 137.45969172632664 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74878883485885 137.45957022371005 86.791 34.74866465534792 137.45958769753437 86.791 34.74866465534792 137.45958769753437 93.017 34.74878883485885 137.45957022371005 93.017 34.74878883485885 137.45957022371005 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74866465534792 137.45958769753437 86.791 34.74867644246834 137.4597095203081 86.791 34.74867644246834 137.4597095203081 93.017 34.74866465534792 137.45958769753437 93.017 34.74866465534792 137.45958769753437 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 93.017 34.74866465534792 137.45958769753437 93.017 34.74878883485885 137.45957022371005 93.017 34.74880269623192 137.45969172632664 93.017 34.74867644246834 137.4597095203081 93.017</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
		</bldg:Building>
	</core:cityObjectMember>
	<!-- LOD2 非水密立体: lod2Solid missing the top face -> Surface Not Closed -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l14lod2-0002">
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>451</bldg:usage>
			<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">6.9</con:value></con:Height></con:height>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<core:lod2Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 86.791 34.74866465534792 137.45958769753437 86.791 34.74878883485885 137.45957022371005 86.791 34.74880269623192 137.45969172632664 86.791 34.74867644246834 137.4597095203081 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 86.791 34.74880269623192 137.45969172632664 86.791 34.74880269623192 137.45969172632664 93.017 34.74867644246834 137.4597095203081 93.017 34.74867644246834 137.4597095203081 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74880269623192 137.45969172632664 86.791 34.74878883485885 137.45957022371005 86.791 34.74878883485885 137.45957022371005 93.017 34.74880269623192 137.45969172632664 93.017 34.74880269623192 137.45969172632664 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74878883485885 137.45957022371005 86.791 34.74866465534792 137.45958769753437 86.791 34.74866465534792 137.45958769753437 93.017 34.74878883485885 137.45957022371005 93.017 34.74878883485885 137.45957022371005 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74866465534792 137.45958769753437 86.791 34.74867644246834 137.4597095203081 86.791 34.74867644246834 137.4597095203081 93.017 34.74866465534792 137.45958769753437 93.017 34.74866465534792 137.45958769753437 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod2Solid>
		</bldg:Building>
	</core:cityObjectMember>
	<!-- LOD3 立体のエラー: lod3Solid with wrong top-face orientation (water-tight) -> InvalidSolidBoundaries -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l14lod3-0001">
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>451</bldg:usage>
			<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">6.9</con:value></con:Height></con:height>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<core:lod3Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 86.791 34.74866465534792 137.45958769753437 86.791 34.74878883485885 137.45957022371005 86.791 34.74880269623192 137.45969172632664 86.791 34.74867644246834 137.4597095203081 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 86.791 34.74880269623192 137.45969172632664 86.791 34.74880269623192 137.45969172632664 93.017 34.74867644246834 137.4597095203081 93.017 34.74867644246834 137.4597095203081 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74880269623192 137.45969172632664 86.791 34.74878883485885 137.45957022371005 86.791 34.74878883485885 137.45957022371005 93.017 34.74880269623192 137.45969172632664 93.017 34.74880269623192 137.45969172632664 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74878883485885 137.45957022371005 86.791 34.74866465534792 137.45958769753437 86.791 34.74866465534792 137.45958769753437 93.017 34.74878883485885 137.45957022371005 93.017 34.74878883485885 137.45957022371005 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74866465534792 137.45958769753437 86.791 34.74867644246834 137.4597095203081 86.791 34.74867644246834 137.4597095203081 93.017 34.74866465534792 137.45958769753437 93.017 34.74866465534792 137.45958769753437 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 93.017 34.74866465534792 137.45958769753437 93.017 34.74878883485885 137.45957022371005 93.017 34.74880269623192 137.45969172632664 93.017 34.74867644246834 137.4597095203081 93.017</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod3Solid>
		</bldg:Building>
	</core:cityObjectMember>
	<!-- LOD3 非水密立体: lod3Solid missing the top face -> Surface Not Closed -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_l14lod3-0002">
			<core:creationDate>2024-03-15</core:creationDate>
			<bldg:class>3001</bldg:class>
			<bldg:usage>451</bldg:usage>
			<con:height><con:Height><con:highReference>highestRoofEdge</con:highReference><con:lowReference>lowestGroundPoint</con:lowReference><con:status>measured</con:status><con:value uom="m">6.9</con:value></con:Height></con:height>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<core:lod3Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 86.791 34.74866465534792 137.45958769753437 86.791 34.74878883485885 137.45957022371005 86.791 34.74880269623192 137.45969172632664 86.791 34.74867644246834 137.4597095203081 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74867644246834 137.4597095203081 86.791 34.74880269623192 137.45969172632664 86.791 34.74880269623192 137.45969172632664 93.017 34.74867644246834 137.4597095203081 93.017 34.74867644246834 137.4597095203081 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74880269623192 137.45969172632664 86.791 34.74878883485885 137.45957022371005 86.791 34.74878883485885 137.45957022371005 93.017 34.74880269623192 137.45969172632664 93.017 34.74880269623192 137.45969172632664 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74878883485885 137.45957022371005 86.791 34.74866465534792 137.45958769753437 86.791 34.74866465534792 137.45958769753437 93.017 34.74878883485885 137.45957022371005 93.017 34.74878883485885 137.45957022371005 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
							<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>34.74866465534792 137.45958769753437 86.791 34.74867644246834 137.4597095203081 86.791 34.74867644246834 137.4597095203081 93.017 34.74866465534792 137.45958769753437 93.017 34.74866465534792 137.45958769753437 86.791</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</core:lod3Solid>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
