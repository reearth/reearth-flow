<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd
https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd
http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd
http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd
http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd
http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd
http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd
http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd
http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd
http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd
http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd
">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.658 137.074 0</gml:lowerCorner>
			<gml:upperCorner>36.668 137.076 25.0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>

	<!-- Building with exactly equal areas in two adjacent 1km meshes -->
	<!-- Mesh boundary at 137.075 longitude (multiple of 0.0125 degree intervals) -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_equal_area_cross_mesh">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">422</bldg:usage>
			<bldg:yearOfConstruction>2021</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">12.0</bldg:measuredHeight>
			<bldg:storeysAboveGround>3</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>

            <bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>
                                        36.66625 137.0745 0
                                        36.66625 137.0755 0
                                        36.66750 137.0755 0
                                        36.66750 137.0745 0
                                        36.66625 137.0745 0
                                    </gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
		</bldg:Building>
	</core:cityObjectMember>

	<!-- Circular building precisely centered on mesh boundary for perfect equal division -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_circular_equal_area">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">431</bldg:usage>
			<bldg:yearOfConstruction>2019</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">8.0</bldg:measuredHeight>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>

            <bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<!-- Approximated circle centered exactly on longitude boundary 137.075 -->
									<gml:posList>
                                        36.660 137.0745 0
                                        36.66025 137.07465 0
                                        36.66041421 137.07480 0
                                        36.66050 137.075 0
                                        36.66041421 137.0752 0
                                        36.66025 137.07535 0
                                        36.660 137.0755 0
                                        36.65975 137.07535 0
                                        36.65958579 137.0752 0
                                        36.6595 137.075 0
                                        36.65958579 137.07480 0
                                        36.65975 137.07465 0
                                        36.660 137.0745 0
                                    </gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
		</bldg:Building>
	</core:cityObjectMember>

	<!-- Rectangular building crossing latitude mesh boundary -->
	<!-- Mesh boundary at 36.666666667 latitude (multiple of 0.008333333 degree intervals) -->
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_lat_boundary_equal">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3002</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:yearOfConstruction>2022</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">10.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>1</bldg:storeysBelowGround>

            <bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>
                                        36.66625 137.0751 0
                                        36.66708333333 137.0751 0
                                        36.66708333333 137.0754 0
                                        36.66625 137.0754 0
                                        36.66625 137.0751 0
                                    </gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
		</bldg:Building>
	</core:cityObjectMember>

</core:CityModel>