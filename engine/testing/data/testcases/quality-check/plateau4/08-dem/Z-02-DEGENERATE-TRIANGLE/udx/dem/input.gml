<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.1249999998 139.9874999996 13.2</gml:lowerCorner>
			<gml:upperCorner>36.1416671635 140 21.7</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<dem:ReliefFeature gml:id="dem_01cabc87-2292-489c-8259-c766bcc64f44">
			<gml:name>54391759</gml:name>
			<core:creationDate>0001-01-01</core:creationDate>
			<dem:lod>1</dem:lod>
			<dem:reliefComponent>
				<dem:TINRelief gml:id="dem_53b46b12-a0ef-4e79-882f-889f84fcddf6">
					<gml:name>54391759</gml:name>
					<core:creationDate>0001-01-01</core:creationDate>
					<dem:lod>1</dem:lod>
					<dem:tin>
						<gml:TriangulatedSurface>
							<gml:trianglePatches>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>
                                                36.12918 139.99380 15.9 
                                                36.12914 139.99374 15.9
                                                36.12916 139.99377 15.9  
                                                36.12918 139.99380 15.9
                                            </gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>
                                                36.12918 139.99380 15.9 
                                                36.12914 139.99374 15.9
                                                36.12914 139.99374 15.9
                                                36.12918 139.99380 15.9
                                            </gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
                            </gml:trianglePatches>
						</gml:TriangulatedSurface>
					</dem:tin>
				</dem:TINRelief>
			</dem:reliefComponent>
			<uro:demDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">700</uro:thematicSrcDesc>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">022</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:demDataQualityAttribute>
		</dem:ReliefFeature>
	</core:cityObjectMember>
</core:CityModel>