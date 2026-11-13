<?xml version="1.0" encoding="UTF-8"?>
<!-- L10: misoriented triangles, one error per triangle.
     One dem:TINRelief with three triangles. The first is counter-clockwise on
     the ground and has no error. The second is the same kind of triangle with its
     corners in reverse order, so it winds clockwise: one error. The third repeats
     its second corner, so it encloses no area and has no orientation; it is a
     linear or point-like triangle instead. -->
<core:CityModel xmlns:urf="https://www.geospatial.jp/iur/urf/4.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:xAL="urn:oasis:names:tc:ciq:xal:3" xmlns:ct="urn:oasis:names:tc:ciq:ct:3" xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0" xmlns:luse="http://www.opengis.net/citygml/landuse/3.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:pcl="http://www.opengis.net/citygml/pointcloud/3.0" xmlns:vers="http://www.opengis.net/citygml/versioning/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:dyn="http://www.opengis.net/citygml/dynamizer/3.0" xmlns:brid="http://www.opengis.net/citygml/bridge/3.0" xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:dem="http://www.opengis.net/citygml/relief/3.0" xmlns:app="http://www.opengis.net/citygml/appearance/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:tran="http://www.opengis.net/citygml/transportation/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xsi:schemaLocation="http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd http://www.opengis.net/citygml/bridge/3.0 http://schemas.opengis.net/citygml/bridge/3.0/bridge.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd http://www.opengis.net/citygml/cityobjectgroup/3.0 http://schemas.opengis.net/citygml/cityobjectgroup/3.0/cityObjectGroup.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd http://www.opengis.net/citygml/dynamizer/3.0 http://schemas.opengis.net/citygml/dynamizer/3.0/dynamizer.xsd http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd http://www.opengis.net/citygml/pointcloud/3.0 http://schemas.opengis.net/citygml/pointcloud/3.0/pointCloud.xsd http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd http://www.opengis.net/citygml/versioning/3.0 http://schemas.opengis.net/citygml/versioning/3.0/versioning.xsd http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd https://www.geospatial.jp/iur/urf/4.0 ../../schemas/iur/urf/4.0/urbanFunction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd  http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.64847365777379 139.7875000000721 7.389</gml:lowerCorner>
			<gml:upperCorner>35.648491690972044 139.78753698691426 7.45</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
		<core:cityObjectMember>
		<dem:ReliefFeature gml:id="dem_300dc37a-ec51-48d5-9a2d-ab84e5e57dc8">
			<gml:name>53393673</gml:name>
			<core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod0>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod2>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
					<urc:lodType codeSpace="../../codelists/Relief_lodType.xml">2.0</urc:lodType>
					<urc:publicSurveyDataQualityAttribute>
						<urc:PublicSurveyDataQualityAttribute>
							<urc:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">3</urc:srcScaleLod2>
							<urc:publicSurveySrcDescLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">012</urc:publicSurveySrcDescLod2>
						</urc:PublicSurveyDataQualityAttribute>
					</urc:publicSurveyDataQualityAttribute>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<dem:lod>2</dem:lod>
			<dem:reliefComponent>
				<dem:TINRelief gml:id="dem_983d22f9-63f9-4678-8dd7-8c193c42a2f1">
					<gml:name>53393673</gml:name>
					<core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
					<dem:lod>2</dem:lod>
					<dem:tin>
						<gml:TriangulatedSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:trianglePatches>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.64848581414039 139.7875000000721 7.389 35.64847365777379 139.78750000007275 7.403 35.64847366344299 139.78751490042606 7.45 35.64848581414039 139.7875000000721 7.389</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.64847366344299 139.78751490042606 7.45 35.648491690972044 139.78751489013234 7.42 35.64847367184294 139.78753698691426 7.42 35.64847366344299 139.78751490042606 7.45</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
								<gml:Triangle>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.64848581414039 139.7875000000721 7.389 35.64847366344299 139.78751490042606 7.45 35.64847366344299 139.78751490042606 7.45 35.64848581414039 139.7875000000721 7.389</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Triangle>
							</gml:trianglePatches>
						</gml:TriangulatedSurface>
					</dem:tin>
				</dem:TINRelief>
			</dem:reliefComponent>
		</dem:ReliefFeature>
	</core:cityObjectMember>
</core:CityModel>
