<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../../schemas/iur/uro/3.1/urbanObject.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.32927745 139.88327292 2.109</gml:lowerCorner>
			<gml:upperCorner>35.32936764 139.88338298 2.109</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="tnm_26c96c99-bde2-43e8-a724-7e9a9e161a35">
			<gml:name>千葉県津波浸水想定図</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">2</wtr:function>
			<wtr:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.32932255 139.88332795 2.109 35.3293676 139.88338298 2.109 35.32936764 139.88327298 2.109 35.32932255 139.88332795 2.109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.32932255 139.88332795 2.109 35.32936764 139.88327298 2.109 35.3292775 139.88327292 2.109 35.32932255 139.88332795 2.109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.32932255 139.88332795 2.109 35.3292775 139.88327292 2.109 35.32927745 139.88338292 2.109 35.32932255 139.88332795 2.109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.32932255 139.88332795 2.109 35.32927745 139.88338292 2.109 35.3293676 139.88338298 2.109 35.32932255 139.88332795 2.109</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</wtr:lod1MultiSurface>
			<uro:floodingRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../../codelists/TsunamiRiskAttribute_description.xml">1</uro:description>
					<uro:rank codeSpace="../../../codelists/TsunamiRiskAttribute_rank.xml">1</uro:rank>
				</uro:TsunamiRiskAttribute>
			</uro:floodingRiskAttribute>
			<uro:wtrDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:wtrDataQualityAttribute>
		</wtr:WaterBody>
	</core:cityObjectMember>
</core:CityModel>