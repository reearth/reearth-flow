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
			<gml:lowerCorner>36.651322509656545 137.0531049468111 0</gml:lowerCorner>
			<gml:upperCorner>36.65819162754668 137.06149279348406 119.52524</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<!-- TEST: duplicate gml:id -->
		<bldg:Building gml:id="bldg_b3eef114-02e4-11f0-a3af-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655625698897225 137.06041109343982 0 36.65568319325029 137.06040687584996 0 36.655680715034414 137.06035654510586 0 36.6556233108055 137.06036076260912 0 36.655625698897225 137.06041109343982 0</gml:posList>
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
											<gml:posList>36.655625698897225 137.06041109343982 60.6 36.6556233108055 137.06036076260912 60.6 36.655680715034414 137.06035654510586 60.6 36.65568319325029 137.06040687584996 60.6 36.655625698897225 137.06041109343982 60.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655625698897225 137.06041109343982 60.6 36.65568319325029 137.06040687584996 60.6 36.65568319325029 137.06040687584996 63.6 36.655625698897225 137.06041109343982 63.6 36.655625698897225 137.06041109343982 60.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65568319325029 137.06040687584996 60.6 36.655680715034414 137.06035654510586 60.6 36.655680715034414 137.06035654510586 63.6 36.65568319325029 137.06040687584996 63.6 36.65568319325029 137.06040687584996 60.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655680715034414 137.06035654510586 60.6 36.6556233108055 137.06036076260912 60.6 36.6556233108055 137.06036076260912 63.6 36.655680715034414 137.06035654510586 63.6 36.655680715034414 137.06035654510586 60.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6556233108055 137.06036076260912 60.6 36.655625698897225 137.06041109343982 60.6 36.655625698897225 137.06041109343982 63.6 36.6556233108055 137.06036076260912 63.6 36.6556233108055 137.06036076260912 60.6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655625698897225 137.06041109343982 63.6 36.65568319325029 137.06040687584996 63.6 36.655680715034414 137.06035654510586 63.6 36.6556233108055 137.06036076260912 63.6 36.655625698897225 137.06041109343982 63.6</gml:posList>
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
					<uro:buildingID>16211-bldg-91</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">28.8</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef117-02e4-11f0-acb5-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">6.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65638105652599 137.06137122273006 0 36.65641834184954 137.0613425371453 0 36.656384847007686 137.0612753581031 0 36.656347561700166 137.06130404370774 0 36.65638105652599 137.06137122273006 0</gml:posList>
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
											<gml:posList>36.65638105652599 137.06137122273006 63.1527 36.656347561700166 137.06130404370774 63.1527 36.656384847007686 137.0612753581031 63.1527 36.65641834184954 137.0613425371453 63.1527 36.65638105652599 137.06137122273006 63.1527</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65638105652599 137.06137122273006 63.1527 36.65641834184954 137.0613425371453 63.1527 36.65641834184954 137.0613425371453 68.83521 36.65638105652599 137.06137122273006 68.83521 36.65638105652599 137.06137122273006 63.1527</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65641834184954 137.0613425371453 63.1527 36.656384847007686 137.0612753581031 63.1527 36.656384847007686 137.0612753581031 68.83521 36.65641834184954 137.0613425371453 68.83521 36.65641834184954 137.0613425371453 63.1527</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656384847007686 137.0612753581031 63.1527 36.656347561700166 137.06130404370774 63.1527 36.656347561700166 137.06130404370774 68.83521 36.656384847007686 137.0612753581031 68.83521 36.656384847007686 137.0612753581031 63.1527</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656347561700166 137.06130404370774 63.1527 36.65638105652599 137.06137122273006 63.1527 36.65638105652599 137.06137122273006 68.83521 36.656347561700166 137.06130404370774 68.83521 36.656347561700166 137.06130404370774 63.1527</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65638105652599 137.06137122273006 68.83521 36.65641834184954 137.0613425371453 68.83521 36.656384847007686 137.0612753581031 68.83521 36.656347561700166 137.06130404370774 68.83521 36.65638105652599 137.06137122273006 68.83521</gml:posList>
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
					<uro:buildingID>16211-bldg-118</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">31.2</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">31.2</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">34.4</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef118-02e4-11f0-a96d-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>1998</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">14.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.656025385732605 137.060718906267 0 36.656081953546966 137.06067500177784 0 36.656016950545 137.06054601118728 0 36.656036493535105 137.06053077219806 0 36.65600588783318 137.060469965557 0 36.65594275565438 137.06051893247866 0 36.6558538275424 137.06034232536584 0 36.65567965129944 137.06047746037936 0 36.65572813290607 137.06057381172045 0 36.65572326958804 137.06057750956603 0 36.65573627021325 137.06060332994497 0 36.65569736425569 137.06063358380607 0 36.65570449656414 137.06064777944587 0 36.655649830035564 137.0606902469194 0 36.65565298554666 137.06069650975166 0 36.655694539398006 137.06077909405127 0 36.65569840169222 137.06078671008737 0 36.65566715071796 137.0608109132658 0 36.65575779343943 137.06099087376253 0 36.65581047887238 137.06095008695826 0 36.65578709588294 137.06090358773724 0 36.655823840463 137.06087501474187 0 36.655846501094615 137.06091994900945 0 36.65589207160687 137.06088454083437 0 36.65589468992224 137.06088990624335 0 36.65594431314751 137.06085136056936 0 36.65593086107021 137.06082453401098 0 36.655951394845516 137.06080862262843 0 36.65593965826455 137.06078537305524 0 36.655989271358024 137.0607469188951 0 36.656025385732605 137.060718906267 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
							<gml:interior>
								<gml:LinearRing>
									<gml:posList>36.65585375775204 137.06066849148135 0 36.65582640210047 137.06061394428863 0 36.6559288010468 137.06053505861425 0 36.65595606651216 137.06058949412082 0 36.65585375775204 137.06066849148135 0</gml:posList>
								</gml:LinearRing>
							</gml:interior>
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
											<gml:posList>36.656025385732605 137.060718906267 61.09517 36.655989271358024 137.0607469188951 61.09517 36.65593965826455 137.06078537305524 61.09517 36.655951394845516 137.06080862262843 61.09517 36.65593086107021 137.06082453401098 61.09517 36.65594431314751 137.06085136056936 61.09517 36.65589468992224 137.06088990624335 61.09517 36.65589207160687 137.06088454083437 61.09517 36.655846501094615 137.06091994900945 61.09517 36.655823840463 137.06087501474187 61.09517 36.65578709588294 137.06090358773724 61.09517 36.65581047887238 137.06095008695826 61.09517 36.65575779343943 137.06099087376253 61.09517 36.65566715071796 137.0608109132658 61.09517 36.65569840169222 137.06078671008737 61.09517 36.655694539398006 137.06077909405127 61.09517 36.65565298554666 137.06069650975166 61.09517 36.655649830035564 137.0606902469194 61.09517 36.65570449656414 137.06064777944587 61.09517 36.65569736425569 137.06063358380607 61.09517 36.65573627021325 137.06060332994497 61.09517 36.65572326958804 137.06057750956603 61.09517 36.65572813290607 137.06057381172045 61.09517 36.65567965129944 137.06047746037936 61.09517 36.6558538275424 137.06034232536584 61.09517 36.65594275565438 137.06051893247866 61.09517 36.65600588783318 137.060469965557 61.09517 36.656036493535105 137.06053077219806 61.09517 36.656016950545 137.06054601118728 61.09517 36.656081953546966 137.06067500177784 61.09517 36.656025385732605 137.060718906267 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
									<gml:interior>
										<gml:LinearRing>
											<gml:posList>36.65585375775204 137.06066849148135 61.09517 36.65595606651216 137.06058949412082 61.09517 36.6559288010468 137.06053505861425 61.09517 36.65582640210047 137.06061394428863 61.09517 36.65585375775204 137.06066849148135 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:interior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656025385732605 137.060718906267 61.09517 36.656081953546966 137.06067500177784 61.09517 36.656081953546966 137.06067500177784 72.19863000000001 36.656025385732605 137.060718906267 72.19863000000001 36.656025385732605 137.060718906267 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656081953546966 137.06067500177784 61.09517 36.656016950545 137.06054601118728 61.09517 36.656016950545 137.06054601118728 72.19863000000001 36.656081953546966 137.06067500177784 72.19863000000001 36.656081953546966 137.06067500177784 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656016950545 137.06054601118728 61.09517 36.656036493535105 137.06053077219806 61.09517 36.656036493535105 137.06053077219806 72.19863000000001 36.656016950545 137.06054601118728 72.19863000000001 36.656016950545 137.06054601118728 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656036493535105 137.06053077219806 61.09517 36.65600588783318 137.060469965557 61.09517 36.65600588783318 137.060469965557 72.19863000000001 36.656036493535105 137.06053077219806 72.19863000000001 36.656036493535105 137.06053077219806 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65600588783318 137.060469965557 61.09517 36.65594275565438 137.06051893247866 61.09517 36.65594275565438 137.06051893247866 72.19863000000001 36.65600588783318 137.060469965557 72.19863000000001 36.65600588783318 137.060469965557 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65594275565438 137.06051893247866 61.09517 36.6558538275424 137.06034232536584 61.09517 36.6558538275424 137.06034232536584 72.19863000000001 36.65594275565438 137.06051893247866 72.19863000000001 36.65594275565438 137.06051893247866 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6558538275424 137.06034232536584 61.09517 36.65567965129944 137.06047746037936 61.09517 36.65567965129944 137.06047746037936 72.19863000000001 36.6558538275424 137.06034232536584 72.19863000000001 36.6558538275424 137.06034232536584 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65567965129944 137.06047746037936 61.09517 36.65572813290607 137.06057381172045 61.09517 36.65572813290607 137.06057381172045 72.19863000000001 36.65567965129944 137.06047746037936 72.19863000000001 36.65567965129944 137.06047746037936 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65572813290607 137.06057381172045 61.09517 36.65572326958804 137.06057750956603 61.09517 36.65572326958804 137.06057750956603 72.19863000000001 36.65572813290607 137.06057381172045 72.19863000000001 36.65572813290607 137.06057381172045 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65572326958804 137.06057750956603 61.09517 36.65573627021325 137.06060332994497 61.09517 36.65573627021325 137.06060332994497 72.19863000000001 36.65572326958804 137.06057750956603 72.19863000000001 36.65572326958804 137.06057750956603 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65573627021325 137.06060332994497 61.09517 36.65569736425569 137.06063358380607 61.09517 36.65569736425569 137.06063358380607 72.19863000000001 36.65573627021325 137.06060332994497 72.19863000000001 36.65573627021325 137.06060332994497 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65569736425569 137.06063358380607 61.09517 36.65570449656414 137.06064777944587 61.09517 36.65570449656414 137.06064777944587 72.19863000000001 36.65569736425569 137.06063358380607 72.19863000000001 36.65569736425569 137.06063358380607 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65570449656414 137.06064777944587 61.09517 36.655649830035564 137.0606902469194 61.09517 36.655649830035564 137.0606902469194 72.19863000000001 36.65570449656414 137.06064777944587 72.19863000000001 36.65570449656414 137.06064777944587 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655649830035564 137.0606902469194 61.09517 36.65565298554666 137.06069650975166 61.09517 36.65565298554666 137.06069650975166 72.19863000000001 36.655649830035564 137.0606902469194 72.19863000000001 36.655649830035564 137.0606902469194 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65565298554666 137.06069650975166 61.09517 36.655694539398006 137.06077909405127 61.09517 36.655694539398006 137.06077909405127 72.19863000000001 36.65565298554666 137.06069650975166 72.19863000000001 36.65565298554666 137.06069650975166 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655694539398006 137.06077909405127 61.09517 36.65569840169222 137.06078671008737 61.09517 36.65569840169222 137.06078671008737 72.19863000000001 36.655694539398006 137.06077909405127 72.19863000000001 36.655694539398006 137.06077909405127 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65569840169222 137.06078671008737 61.09517 36.65566715071796 137.0608109132658 61.09517 36.65566715071796 137.0608109132658 72.19863000000001 36.65569840169222 137.06078671008737 72.19863000000001 36.65569840169222 137.06078671008737 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65566715071796 137.0608109132658 61.09517 36.65575779343943 137.06099087376253 61.09517 36.65575779343943 137.06099087376253 72.19863000000001 36.65566715071796 137.0608109132658 72.19863000000001 36.65566715071796 137.0608109132658 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65575779343943 137.06099087376253 61.09517 36.65581047887238 137.06095008695826 61.09517 36.65581047887238 137.06095008695826 72.19863000000001 36.65575779343943 137.06099087376253 72.19863000000001 36.65575779343943 137.06099087376253 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65581047887238 137.06095008695826 61.09517 36.65578709588294 137.06090358773724 61.09517 36.65578709588294 137.06090358773724 72.19863000000001 36.65581047887238 137.06095008695826 72.19863000000001 36.65581047887238 137.06095008695826 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65578709588294 137.06090358773724 61.09517 36.655823840463 137.06087501474187 61.09517 36.655823840463 137.06087501474187 72.19863000000001 36.65578709588294 137.06090358773724 72.19863000000001 36.65578709588294 137.06090358773724 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655823840463 137.06087501474187 61.09517 36.655846501094615 137.06091994900945 61.09517 36.655846501094615 137.06091994900945 72.19863000000001 36.655823840463 137.06087501474187 72.19863000000001 36.655823840463 137.06087501474187 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655846501094615 137.06091994900945 61.09517 36.65589207160687 137.06088454083437 61.09517 36.65589207160687 137.06088454083437 72.19863000000001 36.655846501094615 137.06091994900945 72.19863000000001 36.655846501094615 137.06091994900945 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65589207160687 137.06088454083437 61.09517 36.65589468992224 137.06088990624335 61.09517 36.65589468992224 137.06088990624335 72.19863000000001 36.65589207160687 137.06088454083437 72.19863000000001 36.65589207160687 137.06088454083437 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65589468992224 137.06088990624335 61.09517 36.65594431314751 137.06085136056936 61.09517 36.65594431314751 137.06085136056936 72.19863000000001 36.65589468992224 137.06088990624335 72.19863000000001 36.65589468992224 137.06088990624335 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65594431314751 137.06085136056936 61.09517 36.65593086107021 137.06082453401098 61.09517 36.65593086107021 137.06082453401098 72.19863000000001 36.65594431314751 137.06085136056936 72.19863000000001 36.65594431314751 137.06085136056936 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65593086107021 137.06082453401098 61.09517 36.655951394845516 137.06080862262843 61.09517 36.655951394845516 137.06080862262843 72.19863000000001 36.65593086107021 137.06082453401098 72.19863000000001 36.65593086107021 137.06082453401098 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655951394845516 137.06080862262843 61.09517 36.65593965826455 137.06078537305524 61.09517 36.65593965826455 137.06078537305524 72.19863000000001 36.655951394845516 137.06080862262843 72.19863000000001 36.655951394845516 137.06080862262843 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65593965826455 137.06078537305524 61.09517 36.655989271358024 137.0607469188951 61.09517 36.655989271358024 137.0607469188951 72.19863000000001 36.65593965826455 137.06078537305524 72.19863000000001 36.65593965826455 137.06078537305524 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655989271358024 137.0607469188951 61.09517 36.656025385732605 137.060718906267 61.09517 36.656025385732605 137.060718906267 72.19863000000001 36.655989271358024 137.0607469188951 72.19863000000001 36.655989271358024 137.0607469188951 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65585375775204 137.06066849148135 61.09517 36.65582640210047 137.06061394428863 61.09517 36.65582640210047 137.06061394428863 72.19863000000001 36.65585375775204 137.06066849148135 72.19863000000001 36.65585375775204 137.06066849148135 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65582640210047 137.06061394428863 61.09517 36.6559288010468 137.06053505861425 61.09517 36.6559288010468 137.06053505861425 72.19863000000001 36.65582640210047 137.06061394428863 72.19863000000001 36.65582640210047 137.06061394428863 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6559288010468 137.06053505861425 61.09517 36.65595606651216 137.06058949412082 61.09517 36.65595606651216 137.06058949412082 72.19863000000001 36.6559288010468 137.06053505861425 72.19863000000001 36.6559288010468 137.06053505861425 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65595606651216 137.06058949412082 61.09517 36.65585375775204 137.06066849148135 61.09517 36.65585375775204 137.06066849148135 72.19863000000001 36.65595606651216 137.06058949412082 72.19863000000001 36.65595606651216 137.06058949412082 61.09517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656025385732605 137.060718906267 72.19863000000001 36.656081953546966 137.06067500177784 72.19863000000001 36.656016950545 137.06054601118728 72.19863000000001 36.656036493535105 137.06053077219806 72.19863000000001 36.65600588783318 137.060469965557 72.19863000000001 36.65594275565438 137.06051893247866 72.19863000000001 36.6558538275424 137.06034232536584 72.19863000000001 36.65567965129944 137.06047746037936 72.19863000000001 36.65572813290607 137.06057381172045 72.19863000000001 36.65572326958804 137.06057750956603 72.19863000000001 36.65573627021325 137.06060332994497 72.19863000000001 36.65569736425569 137.06063358380607 72.19863000000001 36.65570449656414 137.06064777944587 72.19863000000001 36.655649830035564 137.0606902469194 72.19863000000001 36.65565298554666 137.06069650975166 72.19863000000001 36.655694539398006 137.06077909405127 72.19863000000001 36.65569840169222 137.06078671008737 72.19863000000001 36.65566715071796 137.0608109132658 72.19863000000001 36.65575779343943 137.06099087376253 72.19863000000001 36.65581047887238 137.06095008695826 72.19863000000001 36.65578709588294 137.06090358773724 72.19863000000001 36.655823840463 137.06087501474187 72.19863000000001 36.655846501094615 137.06091994900945 72.19863000000001 36.65589207160687 137.06088454083437 72.19863000000001 36.65589468992224 137.06088990624335 72.19863000000001 36.65594431314751 137.06085136056936 72.19863000000001 36.65593086107021 137.06082453401098 72.19863000000001 36.655951394845516 137.06080862262843 72.19863000000001 36.65593965826455 137.06078537305524 72.19863000000001 36.655989271358024 137.0607469188951 72.19863000000001 36.656025385732605 137.060718906267 72.19863000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
									<gml:interior>
										<gml:LinearRing>
											<gml:posList>36.65585375775204 137.06066849148135 72.19863000000001 36.65582640210047 137.06061394428863 72.19863000000001 36.6559288010468 137.06053505861425 72.19863000000001 36.65595606651216 137.06058949412082 72.19863000000001 36.65585375775204 137.06066849148135 72.19863000000001</gml:posList>
										</gml:LinearRing>
									</gml:interior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>16211-bldg-102</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">446.4</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">446.4</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">1417.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">8.0</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef119-02e4-11f0-ba31-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">11.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65558579187789 137.06114009772415 0 36.65563965886681 137.06111049486017 0 36.655548137283844 137.06085402792286 0 36.655494270358076 137.06088363093014 0 36.65558579187789 137.06114009772415 0</gml:posList>
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
											<gml:posList>36.65558579187789 137.06114009772415 61.55064 36.655494270358076 137.06088363093014 61.55064 36.655548137283844 137.06085402792286 61.55064 36.65563965886681 137.06111049486017 61.55064 36.65558579187789 137.06114009772415 61.55064</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65558579187789 137.06114009772415 61.55064 36.65563965886681 137.06111049486017 61.55064 36.65563965886681 137.06111049486017 69.95721 36.65558579187789 137.06114009772415 69.95721 36.65558579187789 137.06114009772415 61.55064</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65563965886681 137.06111049486017 61.55064 36.655548137283844 137.06085402792286 61.55064 36.655548137283844 137.06085402792286 69.95721 36.65563965886681 137.06111049486017 69.95721 36.65563965886681 137.06111049486017 61.55064</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655548137283844 137.06085402792286 61.55064 36.655494270358076 137.06088363093014 61.55064 36.655494270358076 137.06088363093014 69.95721 36.655548137283844 137.06085402792286 69.95721 36.655548137283844 137.06085402792286 61.55064</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655494270358076 137.06088363093014 61.55064 36.65558579187789 137.06114009772415 61.55064 36.65558579187789 137.06114009772415 69.95721 36.655494270358076 137.06088363093014 69.95721 36.655494270358076 137.06088363093014 61.55064</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65558579187789 137.06114009772415 69.95721 36.65563965886681 137.06111049486017 69.95721 36.655548137283844 137.06085402792286 69.95721 36.655494270358076 137.06088363093014 69.95721 36.65558579187789 137.06114009772415 69.95721</gml:posList>
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
					<uro:buildingID>16211-bldg-88</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">163.9</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef11a-02e4-11f0-a090-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">3.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65635218512582 137.06115271285287 0 36.65639102028012 137.06112255617694 0 36.65636474858666 137.06107057962157 0 36.65632593908695 137.0611006045103 0 36.65635218512582 137.06115271285287 0</gml:posList>
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
											<gml:posList>36.65635218512582 137.06115271285287 63.23084 36.65632593908695 137.0611006045103 63.23084 36.65636474858666 137.06107057962157 63.23084 36.65639102028012 137.06112255617694 63.23084 36.65635218512582 137.06115271285287 63.23084</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65635218512582 137.06115271285287 63.23084 36.65639102028012 137.06112255617694 63.23084 36.65639102028012 137.06112255617694 66.47721 36.65635218512582 137.06115271285287 66.47721 36.65635218512582 137.06115271285287 63.23084</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65639102028012 137.06112255617694 63.23084 36.65636474858666 137.06107057962157 63.23084 36.65636474858666 137.06107057962157 66.47721 36.65639102028012 137.06112255617694 66.47721 36.65639102028012 137.06112255617694 63.23084</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65636474858666 137.06107057962157 63.23084 36.65632593908695 137.0611006045103 63.23084 36.65632593908695 137.0611006045103 66.47721 36.65636474858666 137.06107057962157 66.47721 36.65636474858666 137.06107057962157 63.23084</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65632593908695 137.0611006045103 63.23084 36.65635218512582 137.06115271285287 63.23084 36.65635218512582 137.06115271285287 66.47721 36.65632593908695 137.0611006045103 66.47721 36.65632593908695 137.0611006045103 63.23084</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65635218512582 137.06115271285287 66.47721 36.65639102028012 137.06112255617694 66.47721 36.65636474858666 137.06107057962157 66.47721 36.65632593908695 137.0611006045103 66.47721 36.65635218512582 137.06115271285287 66.47721</gml:posList>
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
					<uro:buildingID>16211-bldg-116</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">27.9</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef11b-02e4-11f0-a8d0-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">12.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6559054944713 137.0614890932971 0 36.655981594981334 137.0614291478411 0 36.65587533720786 137.06122146781846 0 36.65579923690015 137.06128152524965 0 36.6559054944713 137.0614890932971 0</gml:posList>
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
											<gml:posList>36.6559054944713 137.0614890932971 61.9816 36.65579923690015 137.06128152524965 61.9816 36.65587533720786 137.06122146781846 61.9816 36.655981594981334 137.0614291478411 61.9816 36.6559054944713 137.0614890932971 61.9816</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6559054944713 137.0614890932971 61.9816 36.655981594981334 137.0614291478411 61.9816 36.655981594981334 137.0614291478411 71.99321 36.6559054944713 137.0614890932971 71.99321 36.6559054944713 137.0614890932971 61.9816</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655981594981334 137.0614291478411 61.9816 36.65587533720786 137.06122146781846 61.9816 36.65587533720786 137.06122146781846 71.99321 36.655981594981334 137.0614291478411 71.99321 36.655981594981334 137.0614291478411 61.9816</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65587533720786 137.06122146781846 61.9816 36.65579923690015 137.06128152524965 61.9816 36.65579923690015 137.06128152524965 71.99321 36.65587533720786 137.06122146781846 71.99321 36.65587533720786 137.06122146781846 61.9816</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65579923690015 137.06128152524965 61.9816 36.6559054944713 137.0614890932971 61.9816 36.6559054944713 137.0614890932971 71.99321 36.65579923690015 137.06128152524965 71.99321 36.65579923690015 137.06128152524965 61.9816</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6559054944713 137.0614890932971 71.99321 36.655981594981334 137.0614291478411 71.99321 36.65587533720786 137.06122146781846 71.99321 36.65579923690015 137.06128152524965 71.99321 36.6559054944713 137.0614890932971 71.99321</gml:posList>
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
					<uro:buildingID>16211-bldg-99</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">220.0</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef11c-02e4-11f0-adbb-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>2012</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">5.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.656202828571836 137.06051741935718 0 36.65627345422086 137.060461991978 0 36.656225424935606 137.0603676527868 0 36.65614067900834 137.0604340989059 0 36.65618870824155 137.06052843803525 0 36.656202828571836 137.06051741935718 0</gml:posList>
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
											<gml:posList>36.656202828571836 137.06051741935718 63.26222 36.65618870824155 137.06052843803525 63.26222 36.65614067900834 137.0604340989059 63.26222 36.656225424935606 137.0603676527868 63.26222 36.65627345422086 137.060461991978 63.26222 36.656202828571836 137.06051741935718 63.26222</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656202828571836 137.06051741935718 63.26222 36.65627345422086 137.060461991978 63.26222 36.65627345422086 137.060461991978 67.16963 36.656202828571836 137.06051741935718 67.16963 36.656202828571836 137.06051741935718 63.26222</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65627345422086 137.060461991978 63.26222 36.656225424935606 137.0603676527868 63.26222 36.656225424935606 137.0603676527868 67.16963 36.65627345422086 137.060461991978 67.16963 36.65627345422086 137.060461991978 63.26222</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656225424935606 137.0603676527868 63.26222 36.65614067900834 137.0604340989059 63.26222 36.65614067900834 137.0604340989059 67.16963 36.656225424935606 137.0603676527868 67.16963 36.656225424935606 137.0603676527868 63.26222</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65614067900834 137.0604340989059 63.26222 36.65618870824155 137.06052843803525 63.26222 36.65618870824155 137.06052843803525 67.16963 36.65614067900834 137.0604340989059 67.16963 36.65614067900834 137.0604340989059 63.26222</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65618870824155 137.06052843803525 63.26222 36.656202828571836 137.06051741935718 63.26222 36.656202828571836 137.06051741935718 67.16963 36.65618870824155 137.06052843803525 67.16963 36.65618870824155 137.06052843803525 63.26222</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656202828571836 137.06051741935718 67.16963 36.65627345422086 137.060461991978 67.16963 36.656225424935606 137.0603676527868 67.16963 36.65614067900834 137.0604340989059 67.16963 36.65618870824155 137.06052843803525 67.16963 36.656202828571836 137.06051741935718 67.16963</gml:posList>
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
					<uro:buildingID>16211-bldg-108</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">102.4</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">102.4</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">111.0</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">5.1</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef11d-02e4-11f0-bac2-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">9.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65594518531862 137.0611082774604 0 36.6559587708266 137.06109741973577 0 36.655925458470115 137.06103292544591 0 36.655911685347064 137.06104378932773 0 36.65594518531862 137.0611082774604 0</gml:posList>
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
											<gml:posList>36.65594518531862 137.0611082774604 62.22243 36.655911685347064 137.06104378932773 62.22243 36.655925458470115 137.06103292544591 62.22243 36.6559587708266 137.06109741973577 62.22243 36.65594518531862 137.0611082774604 62.22243</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65594518531862 137.0611082774604 62.22243 36.6559587708266 137.06109741973577 62.22243 36.6559587708266 137.06109741973577 65.22243 36.65594518531862 137.0611082774604 65.22243 36.65594518531862 137.0611082774604 62.22243</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6559587708266 137.06109741973577 62.22243 36.655925458470115 137.06103292544591 62.22243 36.655925458470115 137.06103292544591 65.22243 36.6559587708266 137.06109741973577 65.22243 36.6559587708266 137.06109741973577 62.22243</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655925458470115 137.06103292544591 62.22243 36.655911685347064 137.06104378932773 62.22243 36.655911685347064 137.06104378932773 65.22243 36.655925458470115 137.06103292544591 65.22243 36.655925458470115 137.06103292544591 62.22243</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655911685347064 137.06104378932773 62.22243 36.65594518531862 137.0611082774604 62.22243 36.65594518531862 137.0611082774604 65.22243 36.655911685347064 137.06104378932773 65.22243 36.655911685347064 137.06104378932773 62.22243</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65594518531862 137.0611082774604 65.22243 36.6559587708266 137.06109741973577 65.22243 36.655925458470115 137.06103292544591 65.22243 36.655911685347064 137.06104378932773 65.22243 36.65594518531862 137.0611082774604 65.22243</gml:posList>
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
					<uro:buildingID>16211-bldg-96</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">12.4</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef11e-02e4-11f0-932d-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65555865077831 137.06041442932283 0 36.65558793804075 137.06041159274176 0 36.65558510382969 137.06036629595766 0 36.65555581656835 137.0603691325558 0 36.65555865077831 137.06041442932283 0</gml:posList>
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
											<gml:posList>36.65555865077831 137.06041442932283 60.60617 36.65555581656835 137.0603691325558 60.60617 36.65558510382969 137.06036629595766 60.60617 36.65558793804075 137.06041159274176 60.60617 36.65555865077831 137.06041442932283 60.60617</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65555865077831 137.06041442932283 60.60617 36.65558793804075 137.06041159274176 60.60617 36.65558793804075 137.06041159274176 63.60617 36.65555865077831 137.06041442932283 63.60617 36.65555865077831 137.06041442932283 60.60617</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65558793804075 137.06041159274176 60.60617 36.65558510382969 137.06036629595766 60.60617 36.65558510382969 137.06036629595766 63.60617 36.65558793804075 137.06041159274176 63.60617 36.65558793804075 137.06041159274176 60.60617</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65558510382969 137.06036629595766 60.60617 36.65555581656835 137.0603691325558 60.60617 36.65555581656835 137.0603691325558 63.60617 36.65558510382969 137.06036629595766 63.60617 36.65558510382969 137.06036629595766 60.60617</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65555581656835 137.0603691325558 60.60617 36.65555865077831 137.06041442932283 60.60617 36.65555865077831 137.06041442932283 63.60617 36.65555581656835 137.0603691325558 63.60617 36.65555581656835 137.0603691325558 60.60617</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65555865077831 137.06041442932283 63.60617 36.65558793804075 137.06041159274176 63.60617 36.65558510382969 137.06036629595766 63.60617 36.65555581656835 137.0603691325558 63.60617 36.65555865077831 137.06041442932283 63.60617</gml:posList>
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
					<uro:buildingID>16211-bldg-87</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">13.2</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef11f-02e4-11f0-9e7a-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65627309701117 137.0614813227391 0 36.656390356427146 137.06139078463735 0 36.65632499262378 137.0612605630216 0 36.65620782352695 137.06135121297535 0 36.65627309701117 137.0614813227391 0</gml:posList>
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
											<gml:posList>36.65627309701117 137.0614813227391 62.27054 36.65620782352695 137.06135121297535 62.27054 36.65632499262378 137.0612605630216 62.27054 36.656390356427146 137.06139078463735 62.27054 36.65627309701117 137.0614813227391 62.27054</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65627309701117 137.0614813227391 62.27054 36.656390356427146 137.06139078463735 62.27054 36.656390356427146 137.06139078463735 69.96521 36.65627309701117 137.0614813227391 69.96521 36.65627309701117 137.0614813227391 62.27054</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656390356427146 137.06139078463735 62.27054 36.65632499262378 137.0612605630216 62.27054 36.65632499262378 137.0612605630216 69.96521 36.656390356427146 137.06139078463735 69.96521 36.656390356427146 137.06139078463735 62.27054</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65632499262378 137.0612605630216 62.27054 36.65620782352695 137.06135121297535 62.27054 36.65620782352695 137.06135121297535 69.96521 36.65632499262378 137.0612605630216 69.96521 36.65632499262378 137.0612605630216 62.27054</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65620782352695 137.06135121297535 62.27054 36.65627309701117 137.0614813227391 62.27054 36.65627309701117 137.0614813227391 69.96521 36.65620782352695 137.06135121297535 69.96521 36.65620782352695 137.06135121297535 62.27054</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65627309701117 137.0614813227391 69.96521 36.656390356427146 137.06139078463735 69.96521 36.65632499262378 137.0612605630216 69.96521 36.65620782352695 137.06135121297535 69.96521 36.65627309701117 137.0614813227391 69.96521</gml:posList>
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
					<uro:buildingID>16211-bldg-115</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">212.7</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">212.7</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">210.1</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef120-02e4-11f0-a1f3-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">2</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.656820251231444 137.06048920477528 0 36.65684088894131 137.0604888408702 0 36.65684069024874 137.06046814790443 0 36.656820052539004 137.06046851181503 0 36.656820251231444 137.06048920477528 0</gml:posList>
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
											<gml:posList>36.656820251231444 137.06048920477528 72.12009 36.656820052539004 137.06046851181503 72.12009 36.65684069024874 137.06046814790443 72.12009 36.65684088894131 137.0604888408702 72.12009 36.656820251231444 137.06048920477528 72.12009</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656820251231444 137.06048920477528 72.12009 36.65684088894131 137.0604888408702 72.12009 36.65684088894131 137.0604888408702 74.04613 36.656820251231444 137.06048920477528 74.04613 36.656820251231444 137.06048920477528 72.12009</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65684088894131 137.0604888408702 72.12009 36.65684069024874 137.06046814790443 72.12009 36.65684069024874 137.06046814790443 74.04613 36.65684088894131 137.0604888408702 74.04613 36.65684088894131 137.0604888408702 72.12009</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65684069024874 137.06046814790443 72.12009 36.656820052539004 137.06046851181503 72.12009 36.656820052539004 137.06046851181503 74.04613 36.65684069024874 137.06046814790443 74.04613 36.65684069024874 137.06046814790443 72.12009</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656820052539004 137.06046851181503 72.12009 36.656820251231444 137.06048920477528 72.12009 36.656820251231444 137.06048920477528 74.04613 36.656820052539004 137.06046851181503 74.04613 36.656820052539004 137.06046851181503 72.12009</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656820251231444 137.06048920477528 74.04613 36.65684088894131 137.0604888408702 74.04613 36.65684069024874 137.06046814790443 74.04613 36.656820052539004 137.06046851181503 74.04613 36.656820251231444 137.06048920477528 74.04613</gml:posList>
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
					<uro:buildingID>16211-bldg-126</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">4.2</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef121-02e4-11f0-b43f-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65637011856275 137.0612318667134 0 36.65638335783034 137.06122200544903 0 36.65637613487437 137.06120713862728 0 36.656362895607955 137.06121699989328 0 36.65637011856275 137.0612318667134 0</gml:posList>
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
											<gml:posList>36.65637011856275 137.0612318667134 63.41035 36.656362895607955 137.06121699989328 63.41035 36.65637613487437 137.06120713862728 63.41035 36.65638335783034 137.06122200544903 63.41035 36.65637011856275 137.0612318667134 63.41035</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65637011856275 137.0612318667134 63.41035 36.65638335783034 137.06122200544903 63.41035 36.65638335783034 137.06122200544903 66.41035 36.65637011856275 137.0612318667134 66.41035 36.65637011856275 137.0612318667134 63.41035</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65638335783034 137.06122200544903 63.41035 36.65637613487437 137.06120713862728 63.41035 36.65637613487437 137.06120713862728 66.41035 36.65638335783034 137.06122200544903 66.41035 36.65638335783034 137.06122200544903 63.41035</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65637613487437 137.06120713862728 63.41035 36.656362895607955 137.06121699989328 63.41035 36.656362895607955 137.06121699989328 66.41035 36.65637613487437 137.06120713862728 66.41035 36.65637613487437 137.06120713862728 63.41035</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656362895607955 137.06121699989328 63.41035 36.65637011856275 137.0612318667134 63.41035 36.65637011856275 137.0612318667134 66.41035 36.656362895607955 137.06121699989328 66.41035 36.656362895607955 137.06121699989328 63.41035</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65637011856275 137.0612318667134 66.41035 36.65638335783034 137.06122200544903 66.41035 36.65637613487437 137.06120713862728 66.41035 36.656362895607955 137.06121699989328 66.41035 36.65637011856275 137.0612318667134 66.41035</gml:posList>
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
					<uro:buildingID>16211-bldg-114</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">2.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a39-02e4-11f0-a200-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">1.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65601966562779 137.06025574592613 0 36.65600963326713 137.0602235457394 0 36.656000534810815 137.06022792057502 0 36.65599800398911 137.06021964685345 0 36.65598890553293 137.06022402168884 0 36.65599215943182 137.0602346433507 0 36.65596081025548 137.0602496749457 0 36.65597011953261 137.06027952717457 0 36.65601966562779 137.06025574592613 0</gml:posList>
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
											<gml:posList>36.65601966562779 137.06025574592613 63.43349 36.65597011953261 137.06027952717457 63.43349 36.65596081025548 137.0602496749457 63.43349 36.65599215943182 137.0602346433507 63.43349 36.65598890553293 137.06022402168884 63.43349 36.65599800398911 137.06021964685345 63.43349 36.656000534810815 137.06022792057502 63.43349 36.65600963326713 137.0602235457394 63.43349 36.65601966562779 137.06025574592613 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65601966562779 137.06025574592613 63.43349 36.65600963326713 137.0602235457394 63.43349 36.65600963326713 137.0602235457394 66.43349 36.65601966562779 137.06025574592613 66.43349 36.65601966562779 137.06025574592613 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65600963326713 137.0602235457394 63.43349 36.656000534810815 137.06022792057502 63.43349 36.656000534810815 137.06022792057502 66.43349 36.65600963326713 137.0602235457394 66.43349 36.65600963326713 137.0602235457394 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656000534810815 137.06022792057502 63.43349 36.65599800398911 137.06021964685345 63.43349 36.65599800398911 137.06021964685345 66.43349 36.656000534810815 137.06022792057502 66.43349 36.656000534810815 137.06022792057502 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65599800398911 137.06021964685345 63.43349 36.65598890553293 137.06022402168884 63.43349 36.65598890553293 137.06022402168884 66.43349 36.65599800398911 137.06021964685345 66.43349 36.65599800398911 137.06021964685345 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65598890553293 137.06022402168884 63.43349 36.65599215943182 137.0602346433507 63.43349 36.65599215943182 137.0602346433507 66.43349 36.65598890553293 137.06022402168884 66.43349 36.65598890553293 137.06022402168884 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65599215943182 137.0602346433507 63.43349 36.65596081025548 137.0602496749457 63.43349 36.65596081025548 137.0602496749457 66.43349 36.65599215943182 137.0602346433507 66.43349 36.65599215943182 137.0602346433507 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65596081025548 137.0602496749457 63.43349 36.65597011953261 137.06027952717457 63.43349 36.65597011953261 137.06027952717457 66.43349 36.65596081025548 137.0602496749457 66.43349 36.65596081025548 137.0602496749457 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65597011953261 137.06027952717457 63.43349 36.65601966562779 137.06025574592613 63.43349 36.65601966562779 137.06025574592613 66.43349 36.65597011953261 137.06027952717457 66.43349 36.65597011953261 137.06027952717457 63.43349</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65601966562779 137.06025574592613 66.43349 36.65600963326713 137.0602235457394 66.43349 36.656000534810815 137.06022792057502 66.43349 36.65599800398911 137.06021964685345 66.43349 36.65598890553293 137.06022402168884 66.43349 36.65599215943182 137.0602346433507 66.43349 36.65596081025548 137.0602496749457 66.43349 36.65597011953261 137.06027952717457 66.43349 36.65601966562779 137.06025574592613 66.43349</gml:posList>
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
					<uro:buildingID>16211-bldg-100</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">18.2</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a3a-02e4-11f0-a6c9-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">2.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65614558390091 137.06149279348406 0 36.65616864020221 137.06147576025927 0 36.65615753460787 137.06145250966088 0 36.65613447830986 137.06146954289014 0 36.65614558390091 137.06149279348406 0</gml:posList>
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
											<gml:posList>36.65614558390091 137.06149279348406 62.32904 36.65613447830986 137.06146954289014 62.32904 36.65615753460787 137.06145250966088 62.32904 36.65616864020221 137.06147576025927 62.32904 36.65614558390091 137.06149279348406 62.32904</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65614558390091 137.06149279348406 62.32904 36.65616864020221 137.06147576025927 62.32904 36.65616864020221 137.06147576025927 64.67121 36.65614558390091 137.06149279348406 64.67121 36.65614558390091 137.06149279348406 62.32904</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65616864020221 137.06147576025927 62.32904 36.65615753460787 137.06145250966088 62.32904 36.65615753460787 137.06145250966088 64.67121 36.65616864020221 137.06147576025927 64.67121 36.65616864020221 137.06147576025927 62.32904</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65615753460787 137.06145250966088 62.32904 36.65613447830986 137.06146954289014 62.32904 36.65613447830986 137.06146954289014 64.67121 36.65615753460787 137.06145250966088 64.67121 36.65615753460787 137.06145250966088 62.32904</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65613447830986 137.06146954289014 62.32904 36.65614558390091 137.06149279348406 62.32904 36.65614558390091 137.06149279348406 64.67121 36.65613447830986 137.06146954289014 64.67121 36.65613447830986 137.06146954289014 62.32904</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65614558390091 137.06149279348406 64.67121 36.65616864020221 137.06147576025927 64.67121 36.65615753460787 137.06145250966088 64.67121 36.65613447830986 137.06146954289014 64.67121 36.65614558390091 137.06149279348406 64.67121</gml:posList>
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
					<uro:buildingID>16211-bldg-105</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">7.2</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a3b-02e4-11f0-9dec-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>2003</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">3.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65634040284756 137.06125975903254 0 36.65636309931832 137.06124350916838 0 36.65634594340229 137.06120662053104 0 36.65632324683747 137.0612227585479 0 36.65634040284756 137.06125975903254 0</gml:posList>
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
											<gml:posList>36.65634040284756 137.06125975903254 62.80782 36.65632324683747 137.0612227585479 62.80782 36.65634594340229 137.06120662053104 62.80782 36.65636309931832 137.06124350916838 62.80782 36.65634040284756 137.06125975903254 62.80782</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65634040284756 137.06125975903254 62.80782 36.65636309931832 137.06124350916838 62.80782 36.65636309931832 137.06124350916838 66.07221 36.65634040284756 137.06125975903254 66.07221 36.65634040284756 137.06125975903254 62.80782</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65636309931832 137.06124350916838 62.80782 36.65634594340229 137.06120662053104 62.80782 36.65634594340229 137.06120662053104 66.07221 36.65636309931832 137.06124350916838 66.07221 36.65636309931832 137.06124350916838 62.80782</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65634594340229 137.06120662053104 62.80782 36.65632324683747 137.0612227585479 62.80782 36.65632324683747 137.0612227585479 66.07221 36.65634594340229 137.06120662053104 66.07221 36.65634594340229 137.06120662053104 62.80782</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65632324683747 137.0612227585479 62.80782 36.65634040284756 137.06125975903254 62.80782 36.65634040284756 137.06125975903254 66.07221 36.65632324683747 137.0612227585479 66.07221 36.65632324683747 137.0612227585479 62.80782</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65634040284756 137.06125975903254 66.07221 36.65636309931832 137.06124350916838 66.07221 36.65634594340229 137.06120662053104 66.07221 36.65632324683747 137.0612227585479 66.07221 36.65634040284756 137.06125975903254 66.07221</gml:posList>
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
					<uro:buildingID>16211-bldg-111</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">9.2</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">9.2</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">11.1</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">5.1</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a3c-02e4-11f0-86ae-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">3.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.656344673234464 137.06129890233072 0 36.65638214106181 137.0612727891336 0 36.65637139542253 137.06124897873622 0 36.65633392769933 137.06127520379573 0 36.656344673234464 137.06129890233072 0</gml:posList>
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
											<gml:posList>36.656344673234464 137.06129890233072 62.76305 36.65633392769933 137.06127520379573 62.76305 36.65637139542253 137.06124897873622 62.76305 36.65638214106181 137.0612727891336 62.76305 36.656344673234464 137.06129890233072 62.76305</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656344673234464 137.06129890233072 62.76305 36.65638214106181 137.0612727891336 62.76305 36.65638214106181 137.0612727891336 65.76304999999999 36.656344673234464 137.06129890233072 65.76304999999999 36.656344673234464 137.06129890233072 62.76305</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65638214106181 137.0612727891336 62.76305 36.65637139542253 137.06124897873622 62.76305 36.65637139542253 137.06124897873622 65.76304999999999 36.65638214106181 137.0612727891336 65.76304999999999 36.65638214106181 137.0612727891336 62.76305</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65637139542253 137.06124897873622 62.76305 36.65633392769933 137.06127520379573 62.76305 36.65633392769933 137.06127520379573 65.76304999999999 36.65637139542253 137.06124897873622 65.76304999999999 36.65637139542253 137.06124897873622 62.76305</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65633392769933 137.06127520379573 62.76305 36.656344673234464 137.06129890233072 62.76305 36.656344673234464 137.06129890233072 65.76304999999999 36.65633392769933 137.06127520379573 65.76304999999999 36.65633392769933 137.06127520379573 62.76305</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656344673234464 137.06129890233072 65.76304999999999 36.65638214106181 137.0612727891336 65.76304999999999 36.65637139542253 137.06124897873622 65.76304999999999 36.65633392769933 137.06127520379573 65.76304999999999 36.656344673234464 137.06129890233072 65.76304999999999</gml:posList>
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
					<uro:buildingID>16211-bldg-113</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">11.6</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a3d-02e4-11f0-9eaf-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65679398605184 137.06034382898267 0 36.656831204968256 137.06034198811963 0 36.656829903808294 137.06029780705424 0 36.65679259467037 137.0602995362074 0 36.65679398605184 137.06034382898267 0</gml:posList>
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
											<gml:posList>36.65679398605184 137.06034382898267 72.24491 36.65679259467037 137.0602995362074 72.24491 36.656829903808294 137.06029780705424 72.24491 36.656831204968256 137.06034198811963 72.24491 36.65679398605184 137.06034382898267 72.24491</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65679398605184 137.06034382898267 72.24491 36.656831204968256 137.06034198811963 72.24491 36.656831204968256 137.06034198811963 75.25214 36.65679398605184 137.06034382898267 75.25214 36.65679398605184 137.06034382898267 72.24491</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656831204968256 137.06034198811963 72.24491 36.656829903808294 137.06029780705424 72.24491 36.656829903808294 137.06029780705424 75.25214 36.656831204968256 137.06034198811963 75.25214 36.656831204968256 137.06034198811963 72.24491</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656829903808294 137.06029780705424 72.24491 36.65679259467037 137.0602995362074 72.24491 36.65679259467037 137.0602995362074 75.25214 36.656829903808294 137.06029780705424 75.25214 36.656829903808294 137.06029780705424 72.24491</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65679259467037 137.0602995362074 72.24491 36.65679398605184 137.06034382898267 72.24491 36.65679398605184 137.06034382898267 75.25214 36.65679259467037 137.0602995362074 75.25214 36.65679259467037 137.0602995362074 72.24491</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65679398605184 137.06034382898267 75.25214 36.656831204968256 137.06034198811963 75.25214 36.656829903808294 137.06029780705424 75.25214 36.65679259467037 137.0602995362074 75.25214 36.65679398605184 137.06034382898267 75.25214</gml:posList>
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
					<uro:buildingID>16211-bldg-125</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">22.1</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">22.1</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">16.4</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4111</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a3e-02e4-11f0-bcb6-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">454</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">3.2</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65661684307711 137.059781777378 0 36.656662771423306 137.05974379501683 0 36.656614204355805 137.05965337146787 0 36.65656836616043 137.05969135373402 0 36.65661684307711 137.059781777378 0</gml:posList>
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
											<gml:posList>36.65661684307711 137.059781777378 75.46582 36.65656836616043 137.05969135373402 75.46582 36.656614204355805 137.05965337146787 75.46582 36.656662771423306 137.05974379501683 75.46582 36.65661684307711 137.059781777378 75.46582</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65661684307711 137.059781777378 75.46582 36.656662771423306 137.05974379501683 75.46582 36.656662771423306 137.05974379501683 78.32964 36.65661684307711 137.059781777378 78.32964 36.65661684307711 137.059781777378 75.46582</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656662771423306 137.05974379501683 75.46582 36.656614204355805 137.05965337146787 75.46582 36.656614204355805 137.05965337146787 78.32964 36.656662771423306 137.05974379501683 78.32964 36.656662771423306 137.05974379501683 75.46582</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656614204355805 137.05965337146787 75.46582 36.65656836616043 137.05969135373402 75.46582 36.65656836616043 137.05969135373402 78.32964 36.656614204355805 137.05965337146787 78.32964 36.656614204355805 137.05965337146787 75.46582</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65656836616043 137.05969135373402 75.46582 36.65661684307711 137.059781777378 75.46582 36.65661684307711 137.059781777378 78.32964 36.65656836616043 137.05969135373402 78.32964 36.65656836616043 137.05969135373402 75.46582</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65661684307711 137.059781777378 78.32964 36.656662771423306 137.05974379501683 78.32964 36.656614204355805 137.05965337146787 78.32964 36.65656836616043 137.05969135373402 78.32964 36.65661684307711 137.059781777378 78.32964</gml:posList>
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
					<uro:buildingID>16211-bldg-121</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">59.0</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">59.0</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">59.4</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">223</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4541</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a3f-02e4-11f0-8067-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">14.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65619259087654 137.0613489966339 0 36.65622285080978 137.06132501856683 0 36.65620822586405 137.06129662746565 0 36.65617796593646 137.06132060553924 0 36.65619259087654 137.0613489966339 0</gml:posList>
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
											<gml:posList>36.65619259087654 137.0613489966339 62.10599 36.65617796593646 137.06132060553924 62.10599 36.65620822586405 137.06129662746565 62.10599 36.65622285080978 137.06132501856683 62.10599 36.65619259087654 137.0613489966339 62.10599</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65619259087654 137.0613489966339 62.10599 36.65622285080978 137.06132501856683 62.10599 36.65622285080978 137.06132501856683 70.07971 36.65619259087654 137.0613489966339 70.07971 36.65619259087654 137.0613489966339 62.10599</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65622285080978 137.06132501856683 62.10599 36.65620822586405 137.06129662746565 62.10599 36.65620822586405 137.06129662746565 70.07971 36.65622285080978 137.06132501856683 70.07971 36.65622285080978 137.06132501856683 62.10599</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65620822586405 137.06129662746565 62.10599 36.65617796593646 137.06132060553924 62.10599 36.65617796593646 137.06132060553924 70.07971 36.65620822586405 137.06129662746565 70.07971 36.65620822586405 137.06129662746565 62.10599</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65617796593646 137.06132060553924 62.10599 36.65619259087654 137.0613489966339 62.10599 36.65619259087654 137.0613489966339 70.07971 36.65617796593646 137.06132060553924 70.07971 36.65617796593646 137.06132060553924 62.10599</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65619259087654 137.0613489966339 70.07971 36.65622285080978 137.06132501856683 70.07971 36.65620822586405 137.06129662746565 70.07971 36.65617796593646 137.06132060553924 70.07971 36.65619259087654 137.0613489966339 70.07971</gml:posList>
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
					<uro:buildingID>16211-bldg-107</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">12.0</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a40-02e4-11f0-8ceb-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">9.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65552622080413 137.0607338154621 0 36.65565518312291 137.06062927935508 0 36.65562078824695 137.06056389217673 0 36.655491825984896 137.06066842834605 0 36.65552622080413 137.0607338154621 0</gml:posList>
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
											<gml:posList>36.65552622080413 137.0607338154621 60.95911 36.655491825984896 137.06066842834605 60.95911 36.65562078824695 137.06056389217673 60.95911 36.65565518312291 137.06062927935508 60.95911 36.65552622080413 137.0607338154621 60.95911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65552622080413 137.0607338154621 60.95911 36.65565518312291 137.06062927935508 60.95911 36.65565518312291 137.06062927935508 70.35288 36.65552622080413 137.0607338154621 70.35288 36.65552622080413 137.0607338154621 60.95911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65565518312291 137.06062927935508 60.95911 36.65562078824695 137.06056389217673 60.95911 36.65562078824695 137.06056389217673 70.35288 36.65565518312291 137.06062927935508 70.35288 36.65565518312291 137.06062927935508 60.95911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65562078824695 137.06056389217673 60.95911 36.655491825984896 137.06066842834605 60.95911 36.655491825984896 137.06066842834605 70.35288 36.65562078824695 137.06056389217673 70.35288 36.65562078824695 137.06056389217673 60.95911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655491825984896 137.06066842834605 60.95911 36.65552622080413 137.0607338154621 60.95911 36.65552622080413 137.0607338154621 70.35288 36.655491825984896 137.06066842834605 70.35288 36.655491825984896 137.06066842834605 60.95911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65552622080413 137.0607338154621 70.35288 36.65565518312291 137.06062927935508 70.35288 36.65562078824695 137.06056389217673 70.35288 36.655491825984896 137.06066842834605 70.35288 36.65552622080413 137.0607338154621 70.35288</gml:posList>
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
					<uro:buildingID>16211-bldg-89</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">119.3</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a41-02e4-11f0-97a3-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">1.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65612908726876 137.06148800618647 0 36.656134756085514 137.06147793158652 0 36.656124474336835 137.06146910908285 0 36.656118805619585 137.06147929553626 0 36.65612908726876 137.06148800618647 0</gml:posList>
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
											<gml:posList>36.65612908726876 137.06148800618647 62.38928 36.656118805619585 137.06147929553626 62.38928 36.656124474336835 137.06146910908285 62.38928 36.656134756085514 137.06147793158652 62.38928 36.65612908726876 137.06148800618647 62.38928</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65612908726876 137.06148800618647 62.38928 36.656134756085514 137.06147793158652 62.38928 36.656134756085514 137.06147793158652 65.38928 36.65612908726876 137.06148800618647 65.38928 36.65612908726876 137.06148800618647 62.38928</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656134756085514 137.06147793158652 62.38928 36.656124474336835 137.06146910908285 62.38928 36.656124474336835 137.06146910908285 65.38928 36.656134756085514 137.06147793158652 65.38928 36.656134756085514 137.06147793158652 62.38928</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656124474336835 137.06146910908285 62.38928 36.656118805619585 137.06147929553626 62.38928 36.656118805619585 137.06147929553626 65.38928 36.656124474336835 137.06146910908285 65.38928 36.656124474336835 137.06146910908285 62.38928</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656118805619585 137.06147929553626 62.38928 36.65612908726876 137.06148800618647 62.38928 36.65612908726876 137.06148800618647 65.38928 36.656118805619585 137.06147929553626 65.38928 36.656118805619585 137.06147929553626 62.38928</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65612908726876 137.06148800618647 65.38928 36.656134756085514 137.06147793158652 65.38928 36.656124474336835 137.06146910908285 65.38928 36.656118805619585 137.06147929553626 65.38928 36.65612908726876 137.06148800618647 65.38928</gml:posList>
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
					<uro:buildingID>16211-bldg-104</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">1.5</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a42-02e4-11f0-8ca3-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>2003</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">15.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>2</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65630224572776 137.06122367525384 0 36.65636385060461 137.0611759479326 0 36.65635218512582 137.06115271285287 0 36.65632593908695 137.0611006045103 0 36.65627538349356 137.06100011471065 0 36.656257679539976 137.06096491147946 0 36.656250204449584 137.0609706262768 0 36.656171835165175 137.06081492392752 0 36.656167325842894 137.0608059780771 0 36.655949379511796 137.06097484051298 0 36.655953343432856 137.0609826714303 0 36.65597384962385 137.06102323677962 0 36.656013930249344 137.06110248971962 0 36.6558762312025 137.06121330126106 0 36.65599829112862 137.06145574640018 0 36.656256405182596 137.06125573491957 0 36.65625766907281 137.06125819399566 0 36.656272885039286 137.06124641994762 0 36.65630224572776 137.06122367525384 0</gml:posList>
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
											<gml:posList>36.65630224572776 137.06122367525384 62.1 36.656272885039286 137.06124641994762 62.1 36.65625766907281 137.06125819399566 62.1 36.656256405182596 137.06125573491957 62.1 36.65599829112862 137.06145574640018 62.1 36.6558762312025 137.06121330126106 62.1 36.656013930249344 137.06110248971962 62.1 36.65597384962385 137.06102323677962 62.1 36.655953343432856 137.0609826714303 62.1 36.655949379511796 137.06097484051298 62.1 36.656167325842894 137.0608059780771 62.1 36.656171835165175 137.06081492392752 62.1 36.656250204449584 137.0609706262768 62.1 36.656257679539976 137.06096491147946 62.1 36.65627538349356 137.06100011471065 62.1 36.65632593908695 137.0611006045103 62.1 36.65635218512582 137.06115271285287 62.1 36.65636385060461 137.0611759479326 62.1 36.65630224572776 137.06122367525384 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65630224572776 137.06122367525384 62.1 36.65636385060461 137.0611759479326 62.1 36.65636385060461 137.0611759479326 76.57821 36.65630224572776 137.06122367525384 76.57821 36.65630224572776 137.06122367525384 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65636385060461 137.0611759479326 62.1 36.65635218512582 137.06115271285287 62.1 36.65635218512582 137.06115271285287 76.57821 36.65636385060461 137.0611759479326 76.57821 36.65636385060461 137.0611759479326 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65635218512582 137.06115271285287 62.1 36.65632593908695 137.0611006045103 62.1 36.65632593908695 137.0611006045103 76.57821 36.65635218512582 137.06115271285287 76.57821 36.65635218512582 137.06115271285287 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65632593908695 137.0611006045103 62.1 36.65627538349356 137.06100011471065 62.1 36.65627538349356 137.06100011471065 76.57821 36.65632593908695 137.0611006045103 76.57821 36.65632593908695 137.0611006045103 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65627538349356 137.06100011471065 62.1 36.656257679539976 137.06096491147946 62.1 36.656257679539976 137.06096491147946 76.57821 36.65627538349356 137.06100011471065 76.57821 36.65627538349356 137.06100011471065 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656257679539976 137.06096491147946 62.1 36.656250204449584 137.0609706262768 62.1 36.656250204449584 137.0609706262768 76.57821 36.656257679539976 137.06096491147946 76.57821 36.656257679539976 137.06096491147946 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656250204449584 137.0609706262768 62.1 36.656171835165175 137.06081492392752 62.1 36.656171835165175 137.06081492392752 76.57821 36.656250204449584 137.0609706262768 76.57821 36.656250204449584 137.0609706262768 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656171835165175 137.06081492392752 62.1 36.656167325842894 137.0608059780771 62.1 36.656167325842894 137.0608059780771 76.57821 36.656171835165175 137.06081492392752 76.57821 36.656171835165175 137.06081492392752 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656167325842894 137.0608059780771 62.1 36.655949379511796 137.06097484051298 62.1 36.655949379511796 137.06097484051298 76.57821 36.656167325842894 137.0608059780771 76.57821 36.656167325842894 137.0608059780771 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655949379511796 137.06097484051298 62.1 36.655953343432856 137.0609826714303 62.1 36.655953343432856 137.0609826714303 76.57821 36.655949379511796 137.06097484051298 76.57821 36.655949379511796 137.06097484051298 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655953343432856 137.0609826714303 62.1 36.65597384962385 137.06102323677962 62.1 36.65597384962385 137.06102323677962 76.57821 36.655953343432856 137.0609826714303 76.57821 36.655953343432856 137.0609826714303 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65597384962385 137.06102323677962 62.1 36.656013930249344 137.06110248971962 62.1 36.656013930249344 137.06110248971962 76.57821 36.65597384962385 137.06102323677962 76.57821 36.65597384962385 137.06102323677962 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656013930249344 137.06110248971962 62.1 36.6558762312025 137.06121330126106 62.1 36.6558762312025 137.06121330126106 76.57821 36.656013930249344 137.06110248971962 76.57821 36.656013930249344 137.06110248971962 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6558762312025 137.06121330126106 62.1 36.65599829112862 137.06145574640018 62.1 36.65599829112862 137.06145574640018 76.57821 36.6558762312025 137.06121330126106 76.57821 36.6558762312025 137.06121330126106 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65599829112862 137.06145574640018 62.1 36.656256405182596 137.06125573491957 62.1 36.656256405182596 137.06125573491957 76.57821 36.65599829112862 137.06145574640018 76.57821 36.65599829112862 137.06145574640018 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656256405182596 137.06125573491957 62.1 36.65625766907281 137.06125819399566 62.1 36.65625766907281 137.06125819399566 76.57821 36.656256405182596 137.06125573491957 76.57821 36.656256405182596 137.06125573491957 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65625766907281 137.06125819399566 62.1 36.656272885039286 137.06124641994762 62.1 36.656272885039286 137.06124641994762 76.57821 36.65625766907281 137.06125819399566 76.57821 36.65625766907281 137.06125819399566 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656272885039286 137.06124641994762 62.1 36.65630224572776 137.06122367525384 62.1 36.65630224572776 137.06122367525384 76.57821 36.656272885039286 137.06124641994762 76.57821 36.656272885039286 137.06124641994762 62.1</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65630224572776 137.06122367525384 76.57821 36.65636385060461 137.0611759479326 76.57821 36.65635218512582 137.06115271285287 76.57821 36.65632593908695 137.0611006045103 76.57821 36.65627538349356 137.06100011471065 76.57821 36.656257679539976 137.06096491147946 76.57821 36.656250204449584 137.0609706262768 76.57821 36.656171835165175 137.06081492392752 76.57821 36.656167325842894 137.0608059780771 76.57821 36.655949379511796 137.06097484051298 76.57821 36.655953343432856 137.0609826714303 76.57821 36.65597384962385 137.06102323677962 76.57821 36.656013930249344 137.06110248971962 76.57821 36.6558762312025 137.06121330126106 76.57821 36.65599829112862 137.06145574640018 76.57821 36.656256405182596 137.06125573491957 76.57821 36.65625766907281 137.06125819399566 76.57821 36.656272885039286 137.06124641994762 76.57821 36.65630224572776 137.06122367525384 76.57821</gml:posList>
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
					<uro:buildingID>16211-bldg-112</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">1735.2</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">1554.3</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">1613.9</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">8.0</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a43-02e4-11f0-bb64-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">5.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.654133535461526 137.05919888467815 0 36.65415830462603 137.05908297214103 0 36.65411350048385 137.05906827008886 0 36.65408873133336 137.05918418256343 0 36.654133535461526 137.05919888467815 0</gml:posList>
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
											<gml:posList>36.654133535461526 137.05919888467815 57.43993 36.65408873133336 137.05918418256343 57.43993 36.65411350048385 137.05906827008886 57.43993 36.65415830462603 137.05908297214103 57.43993 36.654133535461526 137.05919888467815 57.43993</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.654133535461526 137.05919888467815 57.43993 36.65415830462603 137.05908297214103 57.43993 36.65415830462603 137.05908297214103 62.52789 36.654133535461526 137.05919888467815 62.52789 36.654133535461526 137.05919888467815 57.43993</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65415830462603 137.05908297214103 57.43993 36.65411350048385 137.05906827008886 57.43993 36.65411350048385 137.05906827008886 62.52789 36.65415830462603 137.05908297214103 62.52789 36.65415830462603 137.05908297214103 57.43993</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65411350048385 137.05906827008886 57.43993 36.65408873133336 137.05918418256343 57.43993 36.65408873133336 137.05918418256343 62.52789 36.65411350048385 137.05906827008886 62.52789 36.65411350048385 137.05906827008886 57.43993</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65408873133336 137.05918418256343 57.43993 36.654133535461526 137.05919888467815 57.43993 36.654133535461526 137.05919888467815 62.52789 36.65408873133336 137.05918418256343 62.52789 36.65408873133336 137.05918418256343 57.43993</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.654133535461526 137.05919888467815 62.52789 36.65415830462603 137.05908297214103 62.52789 36.65411350048385 137.05906827008886 62.52789 36.65408873133336 137.05918418256343 62.52789 36.654133535461526 137.05919888467815 62.52789</gml:posList>
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
					<uro:buildingID>16211-bldg-80</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">55.1</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a44-02e4-11f0-b584-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">2.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65610471591117 137.06144463987602 0 36.65612146799131 137.06143253681532 0 36.65611054243542 137.06140906228364 0 36.65609379035764 137.0614211653477 0 36.65610471591117 137.06144463987602 0</gml:posList>
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
											<gml:posList>36.65610471591117 137.06144463987602 62.28918 36.65609379035764 137.0614211653477 62.28918 36.65611054243542 137.06140906228364 62.28918 36.65612146799131 137.06143253681532 62.28918 36.65610471591117 137.06144463987602 62.28918</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65610471591117 137.06144463987602 62.28918 36.65612146799131 137.06143253681532 62.28918 36.65612146799131 137.06143253681532 65.28918 36.65610471591117 137.06144463987602 65.28918 36.65610471591117 137.06144463987602 62.28918</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65612146799131 137.06143253681532 62.28918 36.65611054243542 137.06140906228364 62.28918 36.65611054243542 137.06140906228364 65.28918 36.65612146799131 137.06143253681532 65.28918 36.65612146799131 137.06143253681532 62.28918</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65611054243542 137.06140906228364 62.28918 36.65609379035764 137.0614211653477 62.28918 36.65609379035764 137.0614211653477 65.28918 36.65611054243542 137.06140906228364 65.28918 36.65611054243542 137.06140906228364 62.28918</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65609379035764 137.0614211653477 62.28918 36.65610471591117 137.06144463987602 62.28918 36.65610471591117 137.06144463987602 65.28918 36.65609379035764 137.0614211653477 65.28918 36.65609379035764 137.0614211653477 62.28918</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65610471591117 137.06144463987602 65.28918 36.65612146799131 137.06143253681532 65.28918 36.65611054243542 137.06140906228364 65.28918 36.65609379035764 137.0614211653477 65.28918 36.65610471591117 137.06144463987602 65.28918</gml:posList>
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
					<uro:buildingID>16211-bldg-103</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">5.2</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a45-02e4-11f0-822d-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:yearOfConstruction>2010</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">6.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655852467910094 137.05897933137334 0 36.6557299312582 137.0588628945037 0 36.65562212271298 137.0590377600951 0 36.65574822210729 137.0591576036663 0 36.655735441696656 137.05917641286428 0 36.65575600967094 137.0591986431738 0 36.6557704105709 137.05917793020745 0 36.655805592690044 137.05921602346294 0 36.65586058697331 137.05913787300437 0 36.655872404608765 137.05915071977412 0 36.65589517637306 137.05911836231292 0 36.65587975060909 137.05910194123683 0 36.6559171863276 137.0590408168739 0 36.655852467910094 137.05897933137334 0</gml:posList>
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
											<gml:posList>36.655852467910094 137.05897933137334 78.37469 36.6559171863276 137.0590408168739 78.37469 36.65587975060909 137.05910194123683 78.37469 36.65589517637306 137.05911836231292 78.37469 36.655872404608765 137.05915071977412 78.37469 36.65586058697331 137.05913787300437 78.37469 36.655805592690044 137.05921602346294 78.37469 36.6557704105709 137.05917793020745 78.37469 36.65575600967094 137.0591986431738 78.37469 36.655735441696656 137.05917641286428 78.37469 36.65574822210729 137.0591576036663 78.37469 36.65562212271298 137.0590377600951 78.37469 36.6557299312582 137.0588628945037 78.37469 36.655852467910094 137.05897933137334 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655852467910094 137.05897933137334 78.37469 36.6557299312582 137.0588628945037 78.37469 36.6557299312582 137.0588628945037 82.95564 36.655852467910094 137.05897933137334 82.95564 36.655852467910094 137.05897933137334 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6557299312582 137.0588628945037 78.37469 36.65562212271298 137.0590377600951 78.37469 36.65562212271298 137.0590377600951 82.95564 36.6557299312582 137.0588628945037 82.95564 36.6557299312582 137.0588628945037 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65562212271298 137.0590377600951 78.37469 36.65574822210729 137.0591576036663 78.37469 36.65574822210729 137.0591576036663 82.95564 36.65562212271298 137.0590377600951 82.95564 36.65562212271298 137.0590377600951 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65574822210729 137.0591576036663 78.37469 36.655735441696656 137.05917641286428 78.37469 36.655735441696656 137.05917641286428 82.95564 36.65574822210729 137.0591576036663 82.95564 36.65574822210729 137.0591576036663 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655735441696656 137.05917641286428 78.37469 36.65575600967094 137.0591986431738 78.37469 36.65575600967094 137.0591986431738 82.95564 36.655735441696656 137.05917641286428 82.95564 36.655735441696656 137.05917641286428 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65575600967094 137.0591986431738 78.37469 36.6557704105709 137.05917793020745 78.37469 36.6557704105709 137.05917793020745 82.95564 36.65575600967094 137.0591986431738 82.95564 36.65575600967094 137.0591986431738 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6557704105709 137.05917793020745 78.37469 36.655805592690044 137.05921602346294 78.37469 36.655805592690044 137.05921602346294 82.95564 36.6557704105709 137.05917793020745 82.95564 36.6557704105709 137.05917793020745 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655805592690044 137.05921602346294 78.37469 36.65586058697331 137.05913787300437 78.37469 36.65586058697331 137.05913787300437 82.95564 36.655805592690044 137.05921602346294 82.95564 36.655805592690044 137.05921602346294 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65586058697331 137.05913787300437 78.37469 36.655872404608765 137.05915071977412 78.37469 36.655872404608765 137.05915071977412 82.95564 36.65586058697331 137.05913787300437 82.95564 36.65586058697331 137.05913787300437 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655872404608765 137.05915071977412 78.37469 36.65589517637306 137.05911836231292 78.37469 36.65589517637306 137.05911836231292 82.95564 36.655872404608765 137.05915071977412 82.95564 36.655872404608765 137.05915071977412 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65589517637306 137.05911836231292 78.37469 36.65587975060909 137.05910194123683 78.37469 36.65587975060909 137.05910194123683 82.95564 36.65589517637306 137.05911836231292 82.95564 36.65589517637306 137.05911836231292 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65587975060909 137.05910194123683 78.37469 36.6559171863276 137.0590408168739 78.37469 36.6559171863276 137.0590408168739 82.95564 36.65587975060909 137.05910194123683 82.95564 36.65587975060909 137.05910194123683 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6559171863276 137.0590408168739 78.37469 36.655852467910094 137.05897933137334 78.37469 36.655852467910094 137.05897933137334 82.95564 36.6559171863276 137.0590408168739 82.95564 36.6559171863276 137.0590408168739 78.37469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655852467910094 137.05897933137334 82.95564 36.6557299312582 137.0588628945037 82.95564 36.65562212271298 137.0590377600951 82.95564 36.65574822210729 137.0591576036663 82.95564 36.655735441696656 137.05917641286428 82.95564 36.65575600967094 137.0591986431738 82.95564 36.6557704105709 137.05917793020745 82.95564 36.655805592690044 137.05921602346294 82.95564 36.65586058697331 137.05913787300437 82.95564 36.655872404608765 137.05915071977412 82.95564 36.65589517637306 137.05911836231292 82.95564 36.65587975060909 137.05910194123683 82.95564 36.6559171863276 137.0590408168739 82.95564 36.655852467910094 137.05897933137334 82.95564</gml:posList>
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
					<uro:buildingID>16211-bldg-94</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">493.8</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">582.2</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">531.0</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">603</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1001</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4521</uro:detailedUsage>
					<uro:buildingHeight uom="m">5.1</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a46-02e4-11f0-ae3d-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">5.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.656272885039286 137.06124641994762 0 36.65627893803948 137.0612582768564 0 36.656308297918784 137.06123564220914 0 36.65630224572776 137.06122367525384 0 36.656272885039286 137.06124641994762 0</gml:posList>
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
											<gml:posList>36.656272885039286 137.06124641994762 62.18719 36.65630224572776 137.06122367525384 62.18719 36.656308297918784 137.06123564220914 62.18719 36.65627893803948 137.0612582768564 62.18719 36.656272885039286 137.06124641994762 62.18719</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656272885039286 137.06124641994762 62.18719 36.65627893803948 137.0612582768564 62.18719 36.65627893803948 137.0612582768564 66.20522 36.656272885039286 137.06124641994762 66.20522 36.656272885039286 137.06124641994762 62.18719</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65627893803948 137.0612582768564 62.18719 36.656308297918784 137.06123564220914 62.18719 36.656308297918784 137.06123564220914 66.20522 36.65627893803948 137.0612582768564 66.20522 36.65627893803948 137.0612582768564 62.18719</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656308297918784 137.06123564220914 62.18719 36.65630224572776 137.06122367525384 62.18719 36.65630224572776 137.06122367525384 66.20522 36.656308297918784 137.06123564220914 66.20522 36.656308297918784 137.06123564220914 62.18719</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65630224572776 137.06122367525384 62.18719 36.656272885039286 137.06124641994762 62.18719 36.656272885039286 137.06124641994762 66.20522 36.65630224572776 137.06122367525384 66.20522 36.65630224572776 137.06122367525384 62.18719</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656272885039286 137.06124641994762 66.20522 36.65627893803948 137.0612582768564 66.20522 36.656308297918784 137.06123564220914 66.20522 36.65630224572776 137.06122367525384 66.20522 36.656272885039286 137.06124641994762 66.20522</gml:posList>
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
					<uro:buildingID>16211-bldg-110</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">4.8</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a47-02e4-11f0-adad-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65815038009025 137.06111131790405 0 36.65819162754668 137.0610790466914 0 36.65818034252587 137.06105680250312 0 36.6581390950754 137.06108907372288 0 36.65815038009025 137.06111131790405 0</gml:posList>
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
											<gml:posList>36.65815038009025 137.06111131790405 57.35224 36.6581390950754 137.06108907372288 57.35224 36.65818034252587 137.06105680250312 57.35224 36.65819162754668 137.0610790466914 57.35224 36.65815038009025 137.06111131790405 57.35224</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65815038009025 137.06111131790405 57.35224 36.65819162754668 137.0610790466914 57.35224 36.65819162754668 137.0610790466914 60.35224 36.65815038009025 137.06111131790405 60.35224 36.65815038009025 137.06111131790405 57.35224</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65819162754668 137.0610790466914 57.35224 36.65818034252587 137.06105680250312 57.35224 36.65818034252587 137.06105680250312 60.35224 36.65819162754668 137.0610790466914 60.35224 36.65819162754668 137.0610790466914 57.35224</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65818034252587 137.06105680250312 57.35224 36.6581390950754 137.06108907372288 57.35224 36.6581390950754 137.06108907372288 60.35224 36.65818034252587 137.06105680250312 60.35224 36.65818034252587 137.06105680250312 57.35224</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6581390950754 137.06108907372288 57.35224 36.65815038009025 137.06111131790405 57.35224 36.65815038009025 137.06111131790405 60.35224 36.6581390950754 137.06108907372288 60.35224 36.6581390950754 137.06108907372288 57.35224</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65815038009025 137.06111131790405 60.35224 36.65819162754668 137.0610790466914 60.35224 36.65818034252587 137.06105680250312 60.35224 36.6581390950754 137.06108907372288 60.35224 36.65815038009025 137.06111131790405 60.35224</gml:posList>
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
					<uro:buildingID>16211-bldg-127</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">12.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a48-02e4-11f0-84cc-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">7.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.651389556650315 137.05315376373488 0 36.65137828325337 137.05314527987628 0 36.65138466946974 137.0531321843513 0 36.651348414262394 137.0531049468111 0 36.651322509656545 137.05315811234198 0 36.65137003824608 137.05319383376036 0 36.651389556650315 137.05315376373488 0</gml:posList>
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
											<gml:posList>36.651389556650315 137.05315376373488 116.53659 36.65137003824608 137.05319383376036 116.53659 36.651322509656545 137.05315811234198 116.53659 36.651348414262394 137.0531049468111 116.53659 36.65138466946974 137.0531321843513 116.53659 36.65137828325337 137.05314527987628 116.53659 36.651389556650315 137.05315376373488 116.53659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.651389556650315 137.05315376373488 116.53659 36.65137828325337 137.05314527987628 116.53659 36.65137828325337 137.05314527987628 119.52524 36.651389556650315 137.05315376373488 119.52524 36.651389556650315 137.05315376373488 116.53659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65137828325337 137.05314527987628 116.53659 36.65138466946974 137.0531321843513 116.53659 36.65138466946974 137.0531321843513 119.52524 36.65137828325337 137.05314527987628 119.52524 36.65137828325337 137.05314527987628 116.53659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65138466946974 137.0531321843513 116.53659 36.651348414262394 137.0531049468111 116.53659 36.651348414262394 137.0531049468111 119.52524 36.65138466946974 137.0531321843513 119.52524 36.65138466946974 137.0531321843513 116.53659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.651348414262394 137.0531049468111 116.53659 36.651322509656545 137.05315811234198 116.53659 36.651322509656545 137.05315811234198 119.52524 36.651348414262394 137.0531049468111 119.52524 36.651348414262394 137.0531049468111 116.53659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.651322509656545 137.05315811234198 116.53659 36.65137003824608 137.05319383376036 116.53659 36.65137003824608 137.05319383376036 119.52524 36.651322509656545 137.05315811234198 119.52524 36.651322509656545 137.05315811234198 116.53659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65137003824608 137.05319383376036 116.53659 36.651389556650315 137.05315376373488 116.53659 36.651389556650315 137.05315376373488 119.52524 36.65137003824608 137.05319383376036 119.52524 36.65137003824608 137.05319383376036 116.53659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.651389556650315 137.05315376373488 119.52524 36.65137828325337 137.05314527987628 119.52524 36.65138466946974 137.0531321843513 119.52524 36.651348414262394 137.0531049468111 119.52524 36.651322509656545 137.05315811234198 119.52524 36.65137003824608 137.05319383376036 119.52524 36.651389556650315 137.05315376373488 119.52524</gml:posList>
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
					<uro:buildingID>16211-bldg-79</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">34.3</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">34.3</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">32.2</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4111</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a49-02e4-11f0-b89b-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">6.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65643872598757 137.0608526973727 0 36.65647274262639 137.06079683527153 0 36.65635773590734 137.06068368397916 0 36.65630514119098 137.06072503040295 0 36.65643872598757 137.0608526973727 0</gml:posList>
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
											<gml:posList>36.65643872598757 137.0608526973727 63.23266 36.65630514119098 137.06072503040295 63.23266 36.65635773590734 137.06068368397916 63.23266 36.65647274262639 137.06079683527153 63.23266 36.65643872598757 137.0608526973727 63.23266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65643872598757 137.0608526973727 63.23266 36.65647274262639 137.06079683527153 63.23266 36.65647274262639 137.06079683527153 69.01363 36.65643872598757 137.0608526973727 69.01363 36.65643872598757 137.0608526973727 63.23266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65647274262639 137.06079683527153 63.23266 36.65635773590734 137.06068368397916 63.23266 36.65635773590734 137.06068368397916 69.01363 36.65647274262639 137.06079683527153 69.01363 36.65647274262639 137.06079683527153 63.23266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65635773590734 137.06068368397916 63.23266 36.65630514119098 137.06072503040295 63.23266 36.65630514119098 137.06072503040295 69.01363 36.65635773590734 137.06068368397916 69.01363 36.65635773590734 137.06068368397916 63.23266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65630514119098 137.06072503040295 63.23266 36.65643872598757 137.0608526973727 63.23266 36.65643872598757 137.0608526973727 69.01363 36.65630514119098 137.06072503040295 69.01363 36.65630514119098 137.06072503040295 63.23266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65643872598757 137.0608526973727 69.01363 36.65647274262639 137.06079683527153 69.01363 36.65635773590734 137.06068368397916 69.01363 36.65630514119098 137.06072503040295 69.01363 36.65643872598757 137.0608526973727 69.01363</gml:posList>
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
					<uro:buildingID>16211-bldg-120</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">111.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a4a-02e4-11f0-a08a-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">13.2</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655953343432856 137.0609826714303 0 36.6559378615191 137.0609947662751 0 36.65595835520186 137.06103522944136 0 36.65597384962385 137.06102323677962 0 36.655953343432856 137.0609826714303 0</gml:posList>
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
											<gml:posList>36.655953343432856 137.0609826714303 62.33476 36.65597384962385 137.06102323677962 62.33476 36.65595835520186 137.06103522944136 62.33476 36.6559378615191 137.0609947662751 62.33476 36.655953343432856 137.0609826714303 62.33476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655953343432856 137.0609826714303 62.33476 36.6559378615191 137.0609947662751 62.33476 36.6559378615191 137.0609947662751 66.89621 36.655953343432856 137.0609826714303 66.89621 36.655953343432856 137.0609826714303 62.33476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6559378615191 137.0609947662751 62.33476 36.65595835520186 137.06103522944136 62.33476 36.65595835520186 137.06103522944136 66.89621 36.6559378615191 137.0609947662751 66.89621 36.6559378615191 137.0609947662751 62.33476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65595835520186 137.06103522944136 62.33476 36.65597384962385 137.06102323677962 62.33476 36.65597384962385 137.06102323677962 66.89621 36.65595835520186 137.06103522944136 66.89621 36.65595835520186 137.06103522944136 62.33476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65597384962385 137.06102323677962 62.33476 36.655953343432856 137.0609826714303 62.33476 36.655953343432856 137.0609826714303 66.89621 36.65597384962385 137.06102323677962 66.89621 36.65597384962385 137.06102323677962 62.33476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655953343432856 137.0609826714303 66.89621 36.6559378615191 137.0609947662751 66.89621 36.65595835520186 137.06103522944136 66.89621 36.65597384962385 137.06102323677962 66.89621 36.655953343432856 137.0609826714303 66.89621</gml:posList>
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
					<uro:buildingID>16211-bldg-98</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">8.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a4b-02e4-11f0-bfc5-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.6559171863276 137.0590408168739 0 36.65595282098374 137.05898126091688 0 36.65588751780323 137.05892095071934 0 36.655852467910094 137.05897933137334 0 36.6559171863276 137.0590408168739 0</gml:posList>
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
											<gml:posList>36.6559171863276 137.0590408168739 78.32317 36.655852467910094 137.05897933137334 78.32317 36.65588751780323 137.05892095071934 78.32317 36.65595282098374 137.05898126091688 78.32317 36.6559171863276 137.0590408168739 78.32317</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6559171863276 137.0590408168739 78.32317 36.65595282098374 137.05898126091688 78.32317 36.65595282098374 137.05898126091688 83.14464000000001 36.6559171863276 137.0590408168739 83.14464000000001 36.6559171863276 137.0590408168739 78.32317</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65595282098374 137.05898126091688 78.32317 36.65588751780323 137.05892095071934 78.32317 36.65588751780323 137.05892095071934 83.14464000000001 36.65595282098374 137.05898126091688 83.14464000000001 36.65595282098374 137.05898126091688 78.32317</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65588751780323 137.05892095071934 78.32317 36.655852467910094 137.05897933137334 78.32317 36.655852467910094 137.05897933137334 83.14464000000001 36.65588751780323 137.05892095071934 83.14464000000001 36.65588751780323 137.05892095071934 78.32317</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655852467910094 137.05897933137334 78.32317 36.6559171863276 137.0590408168739 78.32317 36.6559171863276 137.0590408168739 83.14464000000001 36.655852467910094 137.05897933137334 83.14464000000001 36.655852467910094 137.05897933137334 78.32317</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6559171863276 137.0590408168739 83.14464000000001 36.65595282098374 137.05898126091688 83.14464000000001 36.65588751780323 137.05892095071934 83.14464000000001 36.655852467910094 137.05897933137334 83.14464000000001 36.6559171863276 137.0590408168739 83.14464000000001</gml:posList>
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
					<uro:buildingID>16211-bldg-95</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">59.4</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a4c-02e4-11f0-8284-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">4.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65671037378634 137.06046877430637 0 36.65677886305286 137.0604645415969 0 36.65677683607088 137.0604148806493 0 36.656708436928476 137.0604191132788 0 36.65671037378634 137.06046877430637 0</gml:posList>
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
											<gml:posList>36.65671037378634 137.06046877430637 71.99739 36.656708436928476 137.0604191132788 71.99739 36.65677683607088 137.0604148806493 71.99739 36.65677886305286 137.0604645415969 71.99739 36.65671037378634 137.06046877430637 71.99739</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65671037378634 137.06046877430637 71.99739 36.65677886305286 137.0604645415969 71.99739 36.65677886305286 137.0604645415969 76.04912999999999 36.65671037378634 137.06046877430637 76.04912999999999 36.65671037378634 137.06046877430637 71.99739</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65677886305286 137.0604645415969 71.99739 36.65677683607088 137.0604148806493 71.99739 36.65677683607088 137.0604148806493 76.04912999999999 36.65677886305286 137.0604645415969 76.04912999999999 36.65677886305286 137.0604645415969 71.99739</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65677683607088 137.0604148806493 71.99739 36.656708436928476 137.0604191132788 71.99739 36.656708436928476 137.0604191132788 76.04912999999999 36.65677683607088 137.0604148806493 76.04912999999999 36.65677683607088 137.0604148806493 71.99739</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656708436928476 137.0604191132788 71.99739 36.65671037378634 137.06046877430637 71.99739 36.65671037378634 137.06046877430637 76.04912999999999 36.656708436928476 137.0604191132788 76.04912999999999 36.656708436928476 137.0604191132788 71.99739</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65671037378634 137.06046877430637 76.04912999999999 36.65677886305286 137.0604645415969 76.04912999999999 36.65677683607088 137.0604148806493 76.04912999999999 36.656708436928476 137.0604191132788 76.04912999999999 36.65671037378634 137.06046877430637 76.04912999999999</gml:posList>
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
					<uro:buildingID>16211-bldg-123</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">31.0</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">31.0</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">33.8</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4111</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a4d-02e4-11f0-a573-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">8.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65627538349356 137.06100011471065 0 36.656283480129204 137.06099373468123 0 36.65618606929739 137.06080371532866 0 36.656171835165175 137.06081492392752 0 36.656250204449584 137.0609706262768 0 36.656257679539976 137.06096491147946 0 36.65627538349356 137.06100011471065 0</gml:posList>
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
											<gml:posList>36.65627538349356 137.06100011471065 63.3615 36.656257679539976 137.06096491147946 63.3615 36.656250204449584 137.0609706262768 63.3615 36.656171835165175 137.06081492392752 63.3615 36.65618606929739 137.06080371532866 63.3615 36.656283480129204 137.06099373468123 63.3615 36.65627538349356 137.06100011471065 63.3615</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65627538349356 137.06100011471065 63.3615 36.656283480129204 137.06099373468123 63.3615 36.656283480129204 137.06099373468123 68.23821 36.65627538349356 137.06100011471065 68.23821 36.65627538349356 137.06100011471065 63.3615</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656283480129204 137.06099373468123 63.3615 36.65618606929739 137.06080371532866 63.3615 36.65618606929739 137.06080371532866 68.23821 36.656283480129204 137.06099373468123 68.23821 36.656283480129204 137.06099373468123 63.3615</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65618606929739 137.06080371532866 63.3615 36.656171835165175 137.06081492392752 63.3615 36.656171835165175 137.06081492392752 68.23821 36.65618606929739 137.06080371532866 68.23821 36.65618606929739 137.06080371532866 63.3615</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656171835165175 137.06081492392752 63.3615 36.656250204449584 137.0609706262768 63.3615 36.656250204449584 137.0609706262768 68.23821 36.656171835165175 137.06081492392752 68.23821 36.656171835165175 137.06081492392752 63.3615</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656250204449584 137.0609706262768 63.3615 36.656257679539976 137.06096491147946 63.3615 36.656257679539976 137.06096491147946 68.23821 36.656250204449584 137.0609706262768 68.23821 36.656250204449584 137.0609706262768 63.3615</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656257679539976 137.06096491147946 63.3615 36.65627538349356 137.06100011471065 63.3615 36.65627538349356 137.06100011471065 68.23821 36.656257679539976 137.06096491147946 68.23821 36.656257679539976 137.06096491147946 63.3615</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65627538349356 137.06100011471065 68.23821 36.656283480129204 137.06099373468123 68.23821 36.65618606929739 137.06080371532866 68.23821 36.656171835165175 137.06081492392752 68.23821 36.656250204449584 137.0609706262768 68.23821 36.656257679539976 137.06096491147946 68.23821 36.65627538349356 137.06100011471065 68.23821</gml:posList>
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
					<uro:buildingID>16211-bldg-109</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">35.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a4e-02e4-11f0-aa24-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>2012</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">16.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655679803681075 137.0609527263917 0 36.65568971137938 137.0609462253333 0 36.65570898475011 137.06093176985456 0 36.655675849268974 137.0608635844314 0 36.655687737567035 137.0608547317246 0 36.65566814504565 137.06081426751408 0 36.65563212006655 137.06084127354546 0 36.655679803681075 137.0609527263917 0</gml:posList>
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
											<gml:posList>36.655679803681075 137.0609527263917 61.45515 36.65563212006655 137.06084127354546 61.45515 36.65566814504565 137.06081426751408 61.45515 36.655687737567035 137.0608547317246 61.45515 36.655675849268974 137.0608635844314 61.45515 36.65570898475011 137.06093176985456 61.45515 36.65568971137938 137.0609462253333 61.45515 36.655679803681075 137.0609527263917 61.45515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655679803681075 137.0609527263917 61.45515 36.65568971137938 137.0609462253333 61.45515 36.65568971137938 137.0609462253333 73.44721 36.655679803681075 137.0609527263917 73.44721 36.655679803681075 137.0609527263917 61.45515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65568971137938 137.0609462253333 61.45515 36.65570898475011 137.06093176985456 61.45515 36.65570898475011 137.06093176985456 73.44721 36.65568971137938 137.0609462253333 73.44721 36.65568971137938 137.0609462253333 61.45515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65570898475011 137.06093176985456 61.45515 36.655675849268974 137.0608635844314 61.45515 36.655675849268974 137.0608635844314 73.44721 36.65570898475011 137.06093176985456 73.44721 36.65570898475011 137.06093176985456 61.45515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655675849268974 137.0608635844314 61.45515 36.655687737567035 137.0608547317246 61.45515 36.655687737567035 137.0608547317246 73.44721 36.655675849268974 137.0608635844314 73.44721 36.655675849268974 137.0608635844314 61.45515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655687737567035 137.0608547317246 61.45515 36.65566814504565 137.06081426751408 61.45515 36.65566814504565 137.06081426751408 73.44721 36.655687737567035 137.0608547317246 73.44721 36.655687737567035 137.0608547317246 61.45515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65566814504565 137.06081426751408 61.45515 36.65563212006655 137.06084127354546 61.45515 36.65563212006655 137.06084127354546 73.44721 36.65566814504565 137.06081426751408 73.44721 36.65566814504565 137.06081426751408 61.45515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65563212006655 137.06084127354546 61.45515 36.655679803681075 137.0609527263917 61.45515 36.655679803681075 137.0609527263917 73.44721 36.65563212006655 137.06084127354546 73.44721 36.65563212006655 137.06084127354546 61.45515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655679803681075 137.0609527263917 73.44721 36.65568971137938 137.0609462253333 73.44721 36.65570898475011 137.06093176985456 73.44721 36.655675849268974 137.0608635844314 73.44721 36.655687737567035 137.0608547317246 73.44721 36.65566814504565 137.06081426751408 73.44721 36.65563212006655 137.06084127354546 73.44721 36.655679803681075 137.0609527263917 73.44721</gml:posList>
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
					<uro:buildingID>16211-bldg-93</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">148.6</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">148.6</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">45.4</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">5.1</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a4f-02e4-11f0-aaac-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:yearOfConstruction>1982</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">5.2</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65529806399683 137.05896192731993 0 36.65538111017047 137.05881069793477 0 36.65531499974942 137.0587547517249 0 36.65523195374697 137.05890609289335 0 36.65529806399683 137.05896192731993 0</gml:posList>
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
											<gml:posList>36.65529806399683 137.05896192731993 78.33093 36.65523195374697 137.05890609289335 78.33093 36.65531499974942 137.0587547517249 78.33093 36.65538111017047 137.05881069793477 78.33093 36.65529806399683 137.05896192731993 78.33093</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65529806399683 137.05896192731993 78.33093 36.65538111017047 137.05881069793477 78.33093 36.65538111017047 137.05881069793477 82.14788999999999 36.65529806399683 137.05896192731993 82.14788999999999 36.65529806399683 137.05896192731993 78.33093</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65538111017047 137.05881069793477 78.33093 36.65531499974942 137.0587547517249 78.33093 36.65531499974942 137.0587547517249 82.14788999999999 36.65538111017047 137.05881069793477 82.14788999999999 36.65538111017047 137.05881069793477 78.33093</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65531499974942 137.0587547517249 78.33093 36.65523195374697 137.05890609289335 78.33093 36.65523195374697 137.05890609289335 82.14788999999999 36.65531499974942 137.0587547517249 82.14788999999999 36.65531499974942 137.0587547517249 78.33093</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65523195374697 137.05890609289335 78.33093 36.65529806399683 137.05896192731993 78.33093 36.65529806399683 137.05896192731993 82.14788999999999 36.65523195374697 137.05890609289335 82.14788999999999 36.65523195374697 137.05890609289335 78.33093</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65529806399683 137.05896192731993 82.14788999999999 36.65538111017047 137.05881069793477 82.14788999999999 36.65531499974942 137.0587547517249 82.14788999999999 36.65523195374697 137.05890609289335 82.14788999999999 36.65529806399683 137.05896192731993 82.14788999999999</gml:posList>
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
					<uro:buildingID>16211-bldg-84</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">115.0</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">129.4</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">145.3</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4521</uro:detailedUsage>
					<uro:buildingHeight uom="m">5.1</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f02a50-02e4-11f0-a301-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">3.2</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65641824994692 137.06134052388526 0 36.65644815057506 137.06131788849154 0 36.65642232918258 137.0612654637986 0 36.65639242856431 137.061288099205 0 36.65641824994692 137.06134052388526 0</gml:posList>
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
											<gml:posList>36.65641824994692 137.06134052388526 63.21119 36.65639242856431 137.061288099205 63.21119 36.65642232918258 137.0612654637986 63.21119 36.65644815057506 137.06131788849154 63.21119 36.65641824994692 137.06134052388526 63.21119</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65641824994692 137.06134052388526 63.21119 36.65644815057506 137.06131788849154 63.21119 36.65644815057506 137.06131788849154 66.19371 36.65641824994692 137.06134052388526 66.19371 36.65641824994692 137.06134052388526 63.21119</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65644815057506 137.06131788849154 63.21119 36.65642232918258 137.0612654637986 63.21119 36.65642232918258 137.0612654637986 66.19371 36.65644815057506 137.06131788849154 66.19371 36.65644815057506 137.06131788849154 63.21119</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65642232918258 137.0612654637986 63.21119 36.65639242856431 137.061288099205 63.21119 36.65639242856431 137.061288099205 66.19371 36.65642232918258 137.0612654637986 66.19371 36.65642232918258 137.0612654637986 63.21119</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65639242856431 137.061288099205 63.21119 36.65641824994692 137.06134052388526 63.21119 36.65641824994692 137.06134052388526 66.19371 36.65639242856431 137.061288099205 66.19371 36.65639242856431 137.061288099205 63.21119</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65641824994692 137.06134052388526 66.19371 36.65644815057506 137.06131788849154 66.19371 36.65642232918258 137.0612654637986 66.19371 36.65639242856431 137.061288099205 66.19371 36.65641824994692 137.06134052388526 66.19371</gml:posList>
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
					<uro:buildingID>16211-bldg-119</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">21.3</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078d2-02e4-11f0-b8d1-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">4.8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655651539131675 137.05897843673134 0 36.65566053548615 137.05896086318185 0 36.65563636540688 137.0589417698821 0 36.65562736905514 137.05895934342826 0 36.655651539131675 137.05897843673134 0</gml:posList>
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
											<gml:posList>36.655651539131675 137.05897843673134 78.53543 36.65562736905514 137.05895934342826 78.53543 36.65563636540688 137.0589417698821 78.53543 36.65566053548615 137.05896086318185 78.53543 36.655651539131675 137.05897843673134 78.53543</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655651539131675 137.05897843673134 78.53543 36.65566053548615 137.05896086318185 78.53543 36.65566053548615 137.05896086318185 81.45314 36.655651539131675 137.05897843673134 81.45314 36.655651539131675 137.05897843673134 78.53543</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65566053548615 137.05896086318185 78.53543 36.65563636540688 137.0589417698821 78.53543 36.65563636540688 137.0589417698821 81.45314 36.65566053548615 137.05896086318185 81.45314 36.65566053548615 137.05896086318185 78.53543</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65563636540688 137.0589417698821 78.53543 36.65562736905514 137.05895934342826 78.53543 36.65562736905514 137.05895934342826 81.45314 36.65563636540688 137.0589417698821 81.45314 36.65563636540688 137.0589417698821 78.53543</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65562736905514 137.05895934342826 78.53543 36.655651539131675 137.05897843673134 78.53543 36.655651539131675 137.05897843673134 81.45314 36.65562736905514 137.05895934342826 81.45314 36.65562736905514 137.05895934342826 78.53543</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655651539131675 137.05897843673134 81.45314 36.65566053548615 137.05896086318185 81.45314 36.65563636540688 137.0589417698821 81.45314 36.65562736905514 137.05895934342826 81.45314 36.655651539131675 137.05897843673134 81.45314</gml:posList>
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
					<uro:buildingID>16211-bldg-90</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">5.9</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078d3-02e4-11f0-a32f-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>2003</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">6.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.656202828571836 137.06051741935718 0 36.656306581058885 137.06072267948423 0 36.65639538026006 137.0606535434684 0 36.65629164676095 137.06044832073562 0 36.656202828571836 137.06051741935718 0</gml:posList>
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
											<gml:posList>36.656202828571836 137.06051741935718 63.25488 36.65629164676095 137.06044832073562 63.25488 36.65639538026006 137.0606535434684 63.25488 36.656306581058885 137.06072267948423 63.25488 36.656202828571836 137.06051741935718 63.25488</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656202828571836 137.06051741935718 63.25488 36.656306581058885 137.06072267948423 63.25488 36.656306581058885 137.06072267948423 68.48863 36.656202828571836 137.06051741935718 68.48863 36.656202828571836 137.06051741935718 63.25488</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656306581058885 137.06072267948423 63.25488 36.65639538026006 137.0606535434684 63.25488 36.65639538026006 137.0606535434684 68.48863 36.656306581058885 137.06072267948423 68.48863 36.656306581058885 137.06072267948423 63.25488</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65639538026006 137.0606535434684 63.25488 36.65629164676095 137.06044832073562 63.25488 36.65629164676095 137.06044832073562 68.48863 36.65639538026006 137.0606535434684 68.48863 36.65639538026006 137.0606535434684 63.25488</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65629164676095 137.06044832073562 63.25488 36.656202828571836 137.06051741935718 63.25488 36.656202828571836 137.06051741935718 68.48863 36.65629164676095 137.06044832073562 68.48863 36.65629164676095 137.06044832073562 63.25488</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656202828571836 137.06051741935718 68.48863 36.656306581058885 137.06072267948423 68.48863 36.65639538026006 137.0606535434684 68.48863 36.65629164676095 137.06044832073562 68.48863 36.656202828571836 137.06051741935718 68.48863</gml:posList>
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
					<uro:buildingID>16211-bldg-117</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">224.2</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">224.2</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">251.9</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">5.1</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078d4-02e4-11f0-9973-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">1.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65681427274303 137.06045509711652 0 36.65681269716693 137.06040633036852 0 36.65677755096271 137.06040805649653 0 36.656779495006084 137.0604657711367 0 36.65673957261745 137.06046783936236 0 36.656740682802564 137.06050005213493 0 36.65680700983238 137.06049660540742 0 36.656805621400224 137.06045555642123 0 36.65681427274303 137.06045509711652 0</gml:posList>
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
											<gml:posList>36.65681427274303 137.06045509711652 72.14777 36.656805621400224 137.06045555642123 72.14777 36.65680700983238 137.06049660540742 72.14777 36.656740682802564 137.06050005213493 72.14777 36.65673957261745 137.06046783936236 72.14777 36.656779495006084 137.0604657711367 72.14777 36.65677755096271 137.06040805649653 72.14777 36.65681269716693 137.06040633036852 72.14777 36.65681427274303 137.06045509711652 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65681427274303 137.06045509711652 72.14777 36.65681269716693 137.06040633036852 72.14777 36.65681269716693 137.06040633036852 75.14777 36.65681427274303 137.06045509711652 75.14777 36.65681427274303 137.06045509711652 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65681269716693 137.06040633036852 72.14777 36.65677755096271 137.06040805649653 72.14777 36.65677755096271 137.06040805649653 75.14777 36.65681269716693 137.06040633036852 75.14777 36.65681269716693 137.06040633036852 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65677755096271 137.06040805649653 72.14777 36.656779495006084 137.0604657711367 72.14777 36.656779495006084 137.0604657711367 75.14777 36.65677755096271 137.06040805649653 75.14777 36.65677755096271 137.06040805649653 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656779495006084 137.0604657711367 72.14777 36.65673957261745 137.06046783936236 72.14777 36.65673957261745 137.06046783936236 75.14777 36.656779495006084 137.0604657711367 75.14777 36.656779495006084 137.0604657711367 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65673957261745 137.06046783936236 72.14777 36.656740682802564 137.06050005213493 72.14777 36.656740682802564 137.06050005213493 75.14777 36.65673957261745 137.06046783936236 75.14777 36.65673957261745 137.06046783936236 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656740682802564 137.06050005213493 72.14777 36.65680700983238 137.06049660540742 72.14777 36.65680700983238 137.06049660540742 75.14777 36.656740682802564 137.06050005213493 75.14777 36.656740682802564 137.06050005213493 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65680700983238 137.06049660540742 72.14777 36.656805621400224 137.06045555642123 72.14777 36.656805621400224 137.06045555642123 75.14777 36.65680700983238 137.06049660540742 75.14777 36.65680700983238 137.06049660540742 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656805621400224 137.06045555642123 72.14777 36.65681427274303 137.06045509711652 72.14777 36.65681427274303 137.06045509711652 75.14777 36.656805621400224 137.06045555642123 75.14777 36.656805621400224 137.06045555642123 72.14777</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65681427274303 137.06045509711652 75.14777 36.65681269716693 137.06040633036852 75.14777 36.65677755096271 137.06040805649653 75.14777 36.656779495006084 137.0604657711367 75.14777 36.65673957261745 137.06046783936236 75.14777 36.656740682802564 137.06050005213493 75.14777 36.65680700983238 137.06049660540742 75.14777 36.656805621400224 137.06045555642123 75.14777 36.65681427274303 137.06045509711652 75.14777</gml:posList>
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
					<uro:buildingID>16211-bldg-124</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">40.6</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078d5-02e4-11f0-8212-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">3.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65540481111335 137.05910830641517 0 36.65542081722736 137.05906879994058 0 36.65539764275437 137.05905440311713 0 36.655381636543986 137.0590937977298 0 36.65540481111335 137.05910830641517 0</gml:posList>
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
											<gml:posList>36.65540481111335 137.05910830641517 77.89426 36.655381636543986 137.0590937977298 77.89426 36.65539764275437 137.05905440311713 77.89426 36.65542081722736 137.05906879994058 77.89426 36.65540481111335 137.05910830641517 77.89426</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65540481111335 137.05910830641517 77.89426 36.65542081722736 137.05906879994058 77.89426 36.65542081722736 137.05906879994058 81.43039 36.65540481111335 137.05910830641517 81.43039 36.65540481111335 137.05910830641517 77.89426</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65542081722736 137.05906879994058 77.89426 36.65539764275437 137.05905440311713 77.89426 36.65539764275437 137.05905440311713 81.43039 36.65542081722736 137.05906879994058 81.43039 36.65542081722736 137.05906879994058 77.89426</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65539764275437 137.05905440311713 77.89426 36.655381636543986 137.0590937977298 77.89426 36.655381636543986 137.0590937977298 81.43039 36.65539764275437 137.05905440311713 81.43039 36.65539764275437 137.05905440311713 77.89426</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655381636543986 137.0590937977298 77.89426 36.65540481111335 137.05910830641517 77.89426 36.65540481111335 137.05910830641517 81.43039 36.655381636543986 137.0590937977298 81.43039 36.655381636543986 137.0590937977298 77.89426</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65540481111335 137.05910830641517 81.43039 36.65542081722736 137.05906879994058 81.43039 36.65539764275437 137.05905440311713 81.43039 36.655381636543986 137.0590937977298 81.43039 36.65540481111335 137.05910830641517 81.43039</gml:posList>
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
					<uro:buildingID>16211-bldg-85</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">11.4</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078d6-02e4-11f0-869c-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">6.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655989271358024 137.0607469188951 0 36.656002463580926 137.0607741016431 0 36.656038939054376 137.0607469829794 0 36.656025385732605 137.060718906267 0 36.655989271358024 137.0607469188951 0</gml:posList>
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
											<gml:posList>36.655989271358024 137.0607469188951 63.0698 36.656025385732605 137.060718906267 63.0698 36.656038939054376 137.0607469829794 63.0698 36.656002463580926 137.0607741016431 63.0698 36.655989271358024 137.0607469188951 63.0698</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655989271358024 137.0607469188951 63.0698 36.656002463580926 137.0607741016431 63.0698 36.656002463580926 137.0607741016431 65.39113 36.655989271358024 137.0607469188951 65.39113 36.655989271358024 137.0607469188951 63.0698</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656002463580926 137.0607741016431 63.0698 36.656038939054376 137.0607469829794 63.0698 36.656038939054376 137.0607469829794 65.39113 36.656002463580926 137.0607741016431 65.39113 36.656002463580926 137.0607741016431 63.0698</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656038939054376 137.0607469829794 63.0698 36.656025385732605 137.060718906267 63.0698 36.656025385732605 137.060718906267 65.39113 36.656038939054376 137.0607469829794 65.39113 36.656038939054376 137.0607469829794 63.0698</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656025385732605 137.060718906267 63.0698 36.655989271358024 137.0607469188951 63.0698 36.655989271358024 137.0607469188951 65.39113 36.656025385732605 137.060718906267 65.39113 36.656025385732605 137.060718906267 63.0698</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655989271358024 137.0607469188951 65.39113 36.656002463580926 137.0607741016431 65.39113 36.656038939054376 137.0607469829794 65.39113 36.656025385732605 137.060718906267 65.39113 36.655989271358024 137.0607469188951 65.39113</gml:posList>
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
					<uro:buildingID>16211-bldg-101</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">13.6</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078d7-02e4-11f0-aaae-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">4.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655540617572136 137.06050572652765 0 36.655581418161226 137.0604778189859 0 36.65556453137611 137.06043981198644 0 36.65552373089557 137.06046783139544 0 36.655540617572136 137.06050572652765 0</gml:posList>
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
											<gml:posList>36.655540617572136 137.06050572652765 60.61588 36.65552373089557 137.06046783139544 60.61588 36.65556453137611 137.06043981198644 60.61588 36.655581418161226 137.0604778189859 60.61588 36.655540617572136 137.06050572652765 60.61588</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655540617572136 137.06050572652765 60.61588 36.655581418161226 137.0604778189859 60.61588 36.655581418161226 137.0604778189859 64.08288999999999 36.655540617572136 137.06050572652765 64.08288999999999 36.655540617572136 137.06050572652765 60.61588</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655581418161226 137.0604778189859 60.61588 36.65556453137611 137.06043981198644 60.61588 36.65556453137611 137.06043981198644 64.08288999999999 36.655581418161226 137.0604778189859 64.08288999999999 36.655581418161226 137.0604778189859 60.61588</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65556453137611 137.06043981198644 60.61588 36.65552373089557 137.06046783139544 60.61588 36.65552373089557 137.06046783139544 64.08288999999999 36.65556453137611 137.06043981198644 64.08288999999999 36.65556453137611 137.06043981198644 60.61588</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65552373089557 137.06046783139544 60.61588 36.655540617572136 137.06050572652765 60.61588 36.655540617572136 137.06050572652765 64.08288999999999 36.65552373089557 137.06046783139544 64.08288999999999 36.65552373089557 137.06046783139544 60.61588</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655540617572136 137.06050572652765 64.08288999999999 36.655581418161226 137.0604778189859 64.08288999999999 36.65556453137611 137.06043981198644 64.08288999999999 36.65552373089557 137.06046783139544 64.08288999999999 36.655540617572136 137.06050572652765 64.08288999999999</gml:posList>
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
					<uro:buildingID>16211-bldg-86</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">20.0</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078d8-02e4-11f0-b046-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">5.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65664988012265 137.0598400082006 0 36.65669283625733 137.05980404335781 0 36.656667469932046 137.05975732298594 0 36.65662451381134 137.05979328784284 0 36.65664988012265 137.0598400082006 0</gml:posList>
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
											<gml:posList>36.65664988012265 137.0598400082006 75.66905 36.65662451381134 137.05979328784284 75.66905 36.656667469932046 137.05975732298594 75.66905 36.65669283625733 137.05980404335781 75.66905 36.65664988012265 137.0598400082006 75.66905</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65664988012265 137.0598400082006 75.66905 36.65669283625733 137.05980404335781 75.66905 36.65669283625733 137.05980404335781 78.66905 36.65664988012265 137.0598400082006 78.66905 36.65664988012265 137.0598400082006 75.66905</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65669283625733 137.05980404335781 75.66905 36.656667469932046 137.05975732298594 75.66905 36.656667469932046 137.05975732298594 78.66905 36.65669283625733 137.05980404335781 78.66905 36.65669283625733 137.05980404335781 75.66905</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656667469932046 137.05975732298594 75.66905 36.65662451381134 137.05979328784284 75.66905 36.65662451381134 137.05979328784284 78.66905 36.656667469932046 137.05975732298594 78.66905 36.656667469932046 137.05975732298594 75.66905</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65662451381134 137.05979328784284 75.66905 36.65664988012265 137.0598400082006 75.66905 36.65664988012265 137.0598400082006 78.66905 36.65662451381134 137.05979328784284 78.66905 36.65662451381134 137.05979328784284 75.66905</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65664988012265 137.0598400082006 78.66905 36.65669283625733 137.05980404335781 78.66905 36.656667469932046 137.05975732298594 78.66905 36.65662451381134 137.05979328784284 78.66905 36.65664988012265 137.0598400082006 78.66905</gml:posList>
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
					<uro:buildingID>16211-bldg-122</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">29.0</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078d9-02e4-11f0-9d91-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">441</bldg:usage>
			<bldg:yearOfConstruction>1998</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">11.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655911685347064 137.06104378932773 0 36.65588306168736 137.06098868919588 0 36.6557167235683 137.06112179874032 0 36.65579562517842 137.06127381225747 0 36.655962053587785 137.0611407027804 0 36.65594518531862 137.0611082774604 0 36.655911685347064 137.06104378932773 0</gml:posList>
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
											<gml:posList>36.655911685347064 137.06104378932773 61.64267 36.65594518531862 137.0611082774604 61.64267 36.655962053587785 137.0611407027804 61.64267 36.65579562517842 137.06127381225747 61.64267 36.6557167235683 137.06112179874032 61.64267 36.65588306168736 137.06098868919588 61.64267 36.655911685347064 137.06104378932773 61.64267</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655911685347064 137.06104378932773 61.64267 36.65588306168736 137.06098868919588 61.64267 36.65588306168736 137.06098868919588 71.72621000000001 36.655911685347064 137.06104378932773 71.72621000000001 36.655911685347064 137.06104378932773 61.64267</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65588306168736 137.06098868919588 61.64267 36.6557167235683 137.06112179874032 61.64267 36.6557167235683 137.06112179874032 71.72621000000001 36.65588306168736 137.06098868919588 71.72621000000001 36.65588306168736 137.06098868919588 61.64267</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6557167235683 137.06112179874032 61.64267 36.65579562517842 137.06127381225747 61.64267 36.65579562517842 137.06127381225747 71.72621000000001 36.6557167235683 137.06112179874032 71.72621000000001 36.6557167235683 137.06112179874032 61.64267</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65579562517842 137.06127381225747 61.64267 36.655962053587785 137.0611407027804 61.64267 36.655962053587785 137.0611407027804 71.72621000000001 36.65579562517842 137.06127381225747 71.72621000000001 36.65579562517842 137.06127381225747 61.64267</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655962053587785 137.0611407027804 61.64267 36.65594518531862 137.0611082774604 61.64267 36.65594518531862 137.0611082774604 71.72621000000001 36.655962053587785 137.0611407027804 71.72621000000001 36.655962053587785 137.0611407027804 61.64267</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65594518531862 137.0611082774604 61.64267 36.655911685347064 137.06104378932773 61.64267 36.655911685347064 137.06104378932773 71.72621000000001 36.65594518531862 137.0611082774604 71.72621000000001 36.65594518531862 137.0611082774604 61.64267</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655911685347064 137.06104378932773 71.72621000000001 36.65588306168736 137.06098868919588 71.72621000000001 36.6557167235683 137.06112179874032 71.72621000000001 36.65579562517842 137.06127381225747 71.72621000000001 36.655962053587785 137.0611407027804 71.72621000000001 36.65594518531862 137.0611082774604 71.72621000000001 36.655911685347064 137.06104378932773 71.72621000000001</gml:posList>
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
					<uro:buildingID>16211-bldg-97</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">315.0</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">315.0</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">355.1</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4411</uro:detailedUsage>
					<uro:buildingHeight uom="m">5.1</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078da-02e4-11f0-8706-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">11.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.65565298554666 137.06069650975166 0 36.65562804311914 137.06071577939701 0 36.6556696630093 137.06079838221783 0 36.655694539398006 137.06077909405127 0 36.65565298554666 137.06069650975166 0</gml:posList>
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
											<gml:posList>36.65565298554666 137.06069650975166 61.4 36.655694539398006 137.06077909405127 61.4 36.6556696630093 137.06079838221783 61.4 36.65562804311914 137.06071577939701 61.4 36.65565298554666 137.06069650975166 61.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65565298554666 137.06069650975166 61.4 36.65562804311914 137.06071577939701 61.4 36.65562804311914 137.06071577939701 66.75864 36.65565298554666 137.06069650975166 66.75864 36.65565298554666 137.06069650975166 61.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65562804311914 137.06071577939701 61.4 36.6556696630093 137.06079838221783 61.4 36.6556696630093 137.06079838221783 66.75864 36.65562804311914 137.06071577939701 66.75864 36.65562804311914 137.06071577939701 61.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6556696630093 137.06079838221783 61.4 36.655694539398006 137.06077909405127 61.4 36.655694539398006 137.06077909405127 66.75864 36.6556696630093 137.06079838221783 66.75864 36.6556696630093 137.06079838221783 61.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655694539398006 137.06077909405127 61.4 36.65565298554666 137.06069650975166 61.4 36.65565298554666 137.06069650975166 66.75864 36.655694539398006 137.06077909405127 66.75864 36.655694539398006 137.06077909405127 61.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65565298554666 137.06069650975166 66.75864 36.65562804311914 137.06071577939701 66.75864 36.6556696630093 137.06079838221783 66.75864 36.655694539398006 137.06077909405127 66.75864 36.65565298554666 137.06069650975166 66.75864</gml:posList>
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
					<uro:buildingID>16211-bldg-92</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">28.4</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078db-02e4-11f0-9c9e-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:yearOfConstruction>1982</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">7.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.655239649805516 137.0591450002326 0 36.655316398973504 137.0590063074777 0 36.65522259933861 137.05892635132258 0 36.65514585026236 137.0590650439881 0 36.655239649805516 137.0591450002326 0</gml:posList>
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
											<gml:posList>36.655239649805516 137.0591450002326 78.10268 36.65514585026236 137.0590650439881 78.10268 36.65522259933861 137.05892635132258 78.10268 36.655316398973504 137.0590063074777 78.10268 36.655239649805516 137.0591450002326 78.10268</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655239649805516 137.0591450002326 78.10268 36.655316398973504 137.0590063074777 78.10268 36.655316398973504 137.0590063074777 84.07989 36.655239649805516 137.0591450002326 84.07989 36.655239649805516 137.0591450002326 78.10268</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655316398973504 137.0590063074777 78.10268 36.65522259933861 137.05892635132258 78.10268 36.65522259933861 137.05892635132258 84.07989 36.655316398973504 137.0590063074777 84.07989 36.655316398973504 137.0590063074777 78.10268</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65522259933861 137.05892635132258 78.10268 36.65514585026236 137.0590650439881 78.10268 36.65514585026236 137.0590650439881 84.07989 36.65522259933861 137.05892635132258 84.07989 36.65522259933861 137.05892635132258 78.10268</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65514585026236 137.0590650439881 78.10268 36.655239649805516 137.0591450002326 78.10268 36.655239649805516 137.0591450002326 84.07989 36.65514585026236 137.0590650439881 84.07989 36.65514585026236 137.0590650439881 78.10268</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.655239649805516 137.0591450002326 84.07989 36.655316398973504 137.0590063074777 84.07989 36.65522259933861 137.05892635132258 84.07989 36.65514585026236 137.0590650439881 84.07989 36.655239649805516 137.0591450002326 84.07989</gml:posList>
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
					<uro:buildingID>16211-bldg-83</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">164.0</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">190.0</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">189.9</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">604</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1003</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">4521</uro:detailedUsage>
					<uro:buildingHeight uom="m">5.1</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078dc-02e4-11f0-b224-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">12.2</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.654779240236984 137.05977469808863 0 36.65481677737842 137.0597257666953 0 36.654741986594615 137.05963750680354 0 36.65470444948897 137.05968643819222 0 36.654779240236984 137.05977469808863 0</gml:posList>
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
											<gml:posList>36.654779240236984 137.05977469808863 57.43541 36.65470444948897 137.05968643819222 57.43541 36.654741986594615 137.05963750680354 57.43541 36.65481677737842 137.0597257666953 57.43541 36.654779240236984 137.05977469808863 57.43541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.654779240236984 137.05977469808863 57.43541 36.65481677737842 137.0597257666953 57.43541 36.65481677737842 137.0597257666953 62.916889999999995 36.654779240236984 137.05977469808863 62.916889999999995 36.654779240236984 137.05977469808863 57.43541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65481677737842 137.0597257666953 57.43541 36.654741986594615 137.05963750680354 57.43541 36.654741986594615 137.05963750680354 62.916889999999995 36.65481677737842 137.0597257666953 62.916889999999995 36.65481677737842 137.0597257666953 57.43541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.654741986594615 137.05963750680354 57.43541 36.65470444948897 137.05968643819222 57.43541 36.65470444948897 137.05968643819222 62.916889999999995 36.654741986594615 137.05963750680354 62.916889999999995 36.654741986594615 137.05963750680354 57.43541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65470444948897 137.05968643819222 57.43541 36.654779240236984 137.05977469808863 57.43541 36.654779240236984 137.05977469808863 62.916889999999995 36.65470444948897 137.05968643819222 62.916889999999995 36.65470444948897 137.05968643819222 57.43541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.654779240236984 137.05977469808863 62.916889999999995 36.65481677737842 137.0597257666953 62.916889999999995 36.654741986594615 137.05963750680354 62.916889999999995 36.65470444948897 137.05968643819222 62.916889999999995 36.654779240236984 137.05977469808863 62.916889999999995</gml:posList>
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
					<uro:buildingID>16211-bldg-82</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">69.2</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3f078dd-02e4-11f0-9133-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">2.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.656148394997295 137.06141035300945 0 36.65616622666348 137.06139612323176 0 36.65616965724288 137.06140282982375 0 36.656176411672185 137.0613974516184 0 36.65618128658097 137.0614068407506 0 36.656209024781646 137.06138476766856 0 36.65617896252496 137.06132653246405 0 36.65612663824662 137.06136821354963 0 36.656148394997295 137.06141035300945 0</gml:posList>
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
											<gml:posList>36.656148394997295 137.06141035300945 62.10476 36.65612663824662 137.06136821354963 62.10476 36.65617896252496 137.06132653246405 62.10476 36.656209024781646 137.06138476766856 62.10476 36.65618128658097 137.0614068407506 62.10476 36.656176411672185 137.0613974516184 62.10476 36.65616965724288 137.06140282982375 62.10476 36.65616622666348 137.06139612323176 62.10476 36.656148394997295 137.06141035300945 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656148394997295 137.06141035300945 62.10476 36.65616622666348 137.06139612323176 62.10476 36.65616622666348 137.06139612323176 65.10476 36.656148394997295 137.06141035300945 65.10476 36.656148394997295 137.06141035300945 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65616622666348 137.06139612323176 62.10476 36.65616965724288 137.06140282982375 62.10476 36.65616965724288 137.06140282982375 65.10476 36.65616622666348 137.06139612323176 65.10476 36.65616622666348 137.06139612323176 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65616965724288 137.06140282982375 62.10476 36.656176411672185 137.0613974516184 62.10476 36.656176411672185 137.0613974516184 65.10476 36.65616965724288 137.06140282982375 65.10476 36.65616965724288 137.06140282982375 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656176411672185 137.0613974516184 62.10476 36.65618128658097 137.0614068407506 62.10476 36.65618128658097 137.0614068407506 65.10476 36.656176411672185 137.0613974516184 65.10476 36.656176411672185 137.0613974516184 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65618128658097 137.0614068407506 62.10476 36.656209024781646 137.06138476766856 62.10476 36.656209024781646 137.06138476766856 65.10476 36.65618128658097 137.0614068407506 65.10476 36.65618128658097 137.0614068407506 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656209024781646 137.06138476766856 62.10476 36.65617896252496 137.06132653246405 62.10476 36.65617896252496 137.06132653246405 65.10476 36.656209024781646 137.06138476766856 65.10476 36.656209024781646 137.06138476766856 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65617896252496 137.06132653246405 62.10476 36.65612663824662 137.06136821354963 62.10476 36.65612663824662 137.06136821354963 65.10476 36.65617896252496 137.06132653246405 65.10476 36.65617896252496 137.06132653246405 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.65612663824662 137.06136821354963 62.10476 36.656148394997295 137.06141035300945 62.10476 36.656148394997295 137.06141035300945 65.10476 36.65612663824662 137.06136821354963 65.10476 36.65612663824662 137.06136821354963 62.10476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.656148394997295 137.06141035300945 65.10476 36.65616622666348 137.06139612323176 65.10476 36.65616965724288 137.06140282982375 65.10476 36.656176411672185 137.0613974516184 65.10476 36.65618128658097 137.0614068407506 65.10476 36.656209024781646 137.06138476766856 65.10476 36.65617896252496 137.06132653246405 65.10476 36.65612663824662 137.06136821354963 65.10476 36.656148394997295 137.06141035300945 65.10476</gml:posList>
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
					<uro:buildingID>16211-bldg-106</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">16</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">16211</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:totalFloorArea uom="m2">-9999</uro:totalFloorArea>
					<uro:buildingFootprintArea uom="m2">-9999</uro:buildingFootprintArea>
					<uro:buildingRoofEdgeArea uom="m2">37.7</uro:buildingRoofEdgeArea>
					<uro:buildingStructureType codeSpace="../../codelists/BuildingDetailAttribute_buildingStructureType.xml">611</uro:buildingStructureType>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">461</uro:detailedUsage>
					<uro:buildingHeight uom="m">-9999</uro:buildingHeight>
					<uro:surveyYear>2020</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
