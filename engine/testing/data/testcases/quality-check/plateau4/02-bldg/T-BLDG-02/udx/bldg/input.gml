<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:urf="https://www.geospatial.jp/iur/urf/3.1" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd  https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd  ">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.608079983099714 139.72493198512066 0</gml:lowerCorner>
			<gml:upperCorner>35.61709830940059 139.73768999958068 140.196</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>

	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b39af966-aa43-4c24-b656-0165e8e041a1">
			<!-- Should be ignored -->
			<bldg:outerBuildingInstallation>
				<bldg:BuildingInstallation gml:id="bldg_b39af966-aa43-4c24-b656-0165e8e041a1_BuildingInstallation_1061">
					<bldg:boundedBy>
					</bldg:boundedBy>
				</bldg:BuildingInstallation>
			</bldg:outerBuildingInstallation>

			<!-- Should pass -->
			<bldg:outerBuildingInstallation>
				<bldg:BuildingInstallation gml:id="bldg_b39af966-aa43-4c24-b656-0165e8e041a1_BuildingInstallation_1070">
					<bldg:lod2Geometry>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="fme-gen-9f310459-4bee-4dea-a1e0-dd2fb14a4c5b">
									<gml:exterior>
										<gml:LinearRing gml:id="fme-gen-9f310459-4bee-4dea-a1e0-dd2fb14a4c5b_0">
											<gml:posList>35.61702758653593 139.7276126681926 22.339 35.6170378475003 139.72764719546564 22.339 35.61706026109892 139.72763264979724 22.339 35.61702758653593 139.7276126681926 22.339</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</bldg:lod2Geometry>
				</bldg:BuildingInstallation>
			</bldg:outerBuildingInstallation>

			<!-- Should pass -->
			<bldg:outerBuildingInstallation>
				<bldg:BuildingInstallation gml:id="bldg_b39af966-aa43-4c24-b656-0165e8e041a1_BuildingInstallation_1031">
					<bldg:lod3Geometry>
						<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:exterior>
								<gml:CompositeSurface>
									<gml:surfaceMember>
										<gml:Polygon gml:id="fme-gen-9f310459-4bee-4dea-a1e0-dd2fb14a4c5b">
											<gml:exterior>
												<gml:LinearRing gml:id="fme-gen-9f310459-4bee-4dea-a1e0-dd2fb14a4c5b_0">
													<gml:posList>35.61663291490302 139.72781608187077 135.77 35.6166244539514 139.72781952606968 135.77 35.616611260350794 139.72777106072488 135.77 35.616619712287225 139.72776761653327 135.77 35.61663291490302 139.72781608187077 135.77</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:CompositeSurface>
							</gml:exterior>
						</gml:Solid>
					</bldg:lod3Geometry>
				</bldg:BuildingInstallation>
			</bldg:outerBuildingInstallation>

			<!-- Should fail -->
			<bldg:outerBuildingInstallation>
				<bldg:BuildingInstallation gml:id="bldg_b39af966-aa43-4c24-b656-0165e8e041a1_BuildingInstallation_1061">
					<bldg:lod2Geometry>
						<gml:CompositeSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.61663291490302 139.72781608187077 135.77 35.6166244539514 139.72781952606968 135.77 35.616611260350794 139.72777106072488 135.77 35.616619712287225 139.72776761653327 135.77 35.61663291490302 139.72781608187077 135.77</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</bldg:lod2Geometry>
				</bldg:BuildingInstallation>
			</bldg:outerBuildingInstallation>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
