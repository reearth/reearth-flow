<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd  https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
			<gml:lowerCorner>156859.58319971853 22594.769200002396 0</gml:lowerCorner>
			<gml:upperCorner>156867.1873997172 22607.458399999356 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:Pipe gml:id="unf_49e8b6bc-eb4a-4edc-b435-cf3160336cc9">
			<gml:name>送風管路</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E2</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:MultiCurve>
							<gml:curveMembers>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>156867.1873997172 22607.35820000304 0 156866.51239971607 22607.458399999356 0 156865.41619971965 22600.070399998727 0 156861.4727997177 22596.66940000092 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>156859.58319971853 22594.769200002396 0 156861.47109971807 22596.667699999543 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
							</gml:curveMembers>
						</gml:MultiCurve>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2015</uro:year>
		</uro:Pipe>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_16bb8aaf-b2bd-4839-96d7-774afc2fdada">
			<gml:name>送風その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE754</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>156866.51239971607 22607.458399999356 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2015</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
</core:CityModel>
