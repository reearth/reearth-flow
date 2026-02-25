<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.2" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd https://www.geospatial.jp/iur/uro/3.2 ../../schemas/iur/uro/3.2/urbanObject.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>27.089321818566095 142.18910232137011 1.929</gml:lowerCorner>
			<gml:upperCorner>27.09164303599194 142.1911248523022 35.635</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_0d1cf86e-37d6-479d-9f1a-6668c28c7e95">
			<core:creationDate>2026-03-13</core:creationDate>
			<gen:stringAttribute name="延べ面積換算係数">
				<gen:value>1</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="大字・町コード">
				<gen:value>3</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="町・丁目コード">
				<gen:value>0</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="13+区市町村コード+大字・町コード+町・丁目コード">
				<gen:value>13421003000</gen:value>
			</gen:stringAttribute>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:measuredHeight uom="m">3.533</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="bldg_9300ae14-cd53-4bfa-a87a-ab53ade5733a">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>27.089409276306718 142.19057552051424 2.083 27.089409899180037 142.19051552157416 2.083 27.089322531695885 142.19051426452825 2.083 27.089321818566095 142.19057426326904 2.083 27.089409276306718 142.19057552051424 2.083</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid gml:id="bldg_926c4a89-76df-4744-abbb-4a496cba75f2" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089409276306718 142.19057552051424 2.083 27.089321818566095 142.19057426326904 2.083 27.089322531695885 142.19051426452825 2.083 27.089409899180037 142.19051552157416 2.083 27.089409276306718 142.19057552051424 2.083</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089409276306718 142.19057552051424 2.083 27.089409899180037 142.19051552157416 2.083 27.089409899180037 142.19051552157416 5.506 27.089409276306718 142.19057552051424 5.506 27.089409276306718 142.19057552051424 2.083</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089409899180037 142.19051552157416 2.083 27.089322531695885 142.19051426452825 2.083 27.089322531695885 142.19051426452825 5.506 27.089409899180037 142.19051552157416 5.506 27.089409899180037 142.19051552157416 2.083</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089322531695885 142.19051426452825 2.083 27.089321818566095 142.19057426326904 2.083 27.089321818566095 142.19057426326904 5.506 27.089322531695885 142.19051426452825 5.506 27.089322531695885 142.19051426452825 2.083</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089321818566095 142.19057426326904 2.083 27.089409276306718 142.19057552051424 2.083 27.089409276306718 142.19057552051424 5.506 27.089321818566095 142.19057426326904 5.506 27.089321818566095 142.19057426326904 2.083</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089409276306718 142.19057552051424 5.506 27.089409899180037 142.19051552157416 5.506 27.089322531695885 142.19051426452825 5.506 27.089321818566095 142.19057426326904 5.506 27.089409276306718 142.19057552051424 5.506</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<bldg:address>
				<core:Address gml:id="adrs_0c718301-0979-43ed-b916-89829d377fbf">
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都小笠原村父島</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">2</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">100</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">11</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">101</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">1</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">102</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">103</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">104</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">107</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key107.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">61.909</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1001</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1142</uro:detailedUsage>
					<uro:surveyYear>2022</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13421-bldg-1083</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13421</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_7711a1e1-4f5b-456d-a279-f57c6590ed00">
			<core:creationDate>2026-03-13</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.236</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="bldg_5c73f9be-fe46-403d-9721-90b6e28783c9">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>27.091487661069173 142.18915202008844 30.419 27.091500178999897 142.1891727137675 30.419 27.09152330832004 142.18915530691947 30.419 27.091510700129607 142.18913461308645 30.419 27.091487661069173 142.18915202008844 30.419</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid gml:id="bldg_c4525194-e34a-4988-919d-f816268cc7b3" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091487661069173 142.18915202008844 30.419 27.091510700129607 142.18913461308645 30.419 27.09152330832004 142.18915530691947 30.419 27.091500178999897 142.1891727137675 30.419 27.091487661069173 142.18915202008844 30.419</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091487661069173 142.18915202008844 30.419 27.091500178999897 142.1891727137675 30.419 27.091500178999897 142.1891727137675 33.248 27.091487661069173 142.18915202008844 33.248 27.091487661069173 142.18915202008844 30.419</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091500178999897 142.1891727137675 30.419 27.09152330832004 142.18915530691947 30.419 27.09152330832004 142.18915530691947 33.248 27.091500178999897 142.1891727137675 33.248 27.091500178999897 142.1891727137675 30.419</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.09152330832004 142.18915530691947 30.419 27.091510700129607 142.18913461308645 30.419 27.091510700129607 142.18913461308645 33.248 27.09152330832004 142.18915530691947 33.248 27.09152330832004 142.18915530691947 30.419</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091510700129607 142.18913461308645 30.419 27.091487661069173 142.18915202008844 30.419 27.091487661069173 142.18915202008844 33.248 27.091510700129607 142.18913461308645 33.248 27.091510700129607 142.18913461308645 30.419</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091487661069173 142.18915202008844 33.248 27.091500178999897 142.1891727137675 33.248 27.09152330832004 142.18915530691947 33.248 27.091510700129607 142.18913461308645 33.248 27.091487661069173 142.18915202008844 33.248</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<bldg:address>
				<core:Address gml:id="adrs_6bf9f362-ef2f-4dfd-accf-a01bf4137a85">
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都小笠原村父島</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">2</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">100</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">99</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">101</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">102</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">103</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">104</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">107</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key107.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">-9999.000</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">212</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:surveyYear>2022</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13421-bldg-1077</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13421</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_64c84be4-a0ab-4be0-980f-4ec423b83a6e">
			<core:creationDate>2026-03-13</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.526</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="bldg_9e690360-e3bf-4a10-abbe-e8dc27a4a290">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>27.091465763170813 142.18912626855072 30.423 27.091473508057906 142.18913908851547 30.423 27.09150522049103 142.18911524217881 30.423 27.09149747573747 142.18910232137011 30.423 27.091465763170813 142.18912626855072 30.423</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid gml:id="bldg_670e651a-5474-4f1a-8aab-6d268252d502" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091465763170813 142.18912626855072 30.423 27.09149747573747 142.18910232137011 30.423 27.09150522049103 142.18911524217881 30.423 27.091473508057906 142.18913908851547 30.423 27.091465763170813 142.18912626855072 30.423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091465763170813 142.18912626855072 30.423 27.091473508057906 142.18913908851547 30.423 27.091473508057906 142.18913908851547 33.653 27.091465763170813 142.18912626855072 33.653 27.091465763170813 142.18912626855072 30.423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091473508057906 142.18913908851547 30.423 27.09150522049103 142.18911524217881 30.423 27.09150522049103 142.18911524217881 33.653 27.091473508057906 142.18913908851547 33.653 27.091473508057906 142.18913908851547 30.423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.09150522049103 142.18911524217881 30.423 27.09149747573747 142.18910232137011 30.423 27.09149747573747 142.18910232137011 33.653 27.09150522049103 142.18911524217881 33.653 27.09150522049103 142.18911524217881 30.423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.09149747573747 142.18910232137011 30.423 27.091465763170813 142.18912626855072 30.423 27.091465763170813 142.18912626855072 33.653 27.09149747573747 142.18910232137011 33.653 27.09149747573747 142.18910232137011 30.423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091465763170813 142.18912626855072 33.653 27.091473508057906 142.18913908851547 33.653 27.09150522049103 142.18911524217881 33.653 27.09149747573747 142.18910232137011 33.653 27.091465763170813 142.18912626855072 33.653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<bldg:address>
				<core:Address gml:id="adrs_2d369b2c-6de3-486b-9b33-cb2b860a4364">
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都小笠原村父島</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">2</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">100</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">99</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">101</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">102</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">103</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">104</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">107</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key107.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">-9999.000</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">212</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:surveyYear>2022</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13421-bldg-1078</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13421</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_a5064b70-5bd7-4024-b64c-8c3f35b4a8d5">
			<gml:name>小笠原村情報センター</gml:name>
			<core:creationDate>2026-03-13</core:creationDate>
			<gen:stringAttribute name="延べ面積換算係数">
				<gen:value>1</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="大字・町コード">
				<gen:value>3</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="町・丁目コード">
				<gen:value>0</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="13+区市町村コード+大字・町コード+町・丁目コード">
				<gen:value>13421003000</gen:value>
			</gen:stringAttribute>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">401</bldg:usage>
			<bldg:measuredHeight uom="m">7.25</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="bldg_604991bd-46ce-4d31-873a-13d3649c267c">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>27.09141162201706 142.18925051595596 30.23 27.091512212063055 142.1894193940363 30.23 27.09164303599194 142.18932199872881 30.23 27.091542355574184 142.18915312038615 30.23 27.09141162201706 142.18925051595596 30.23</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid gml:id="bldg_94b13041-6609-4665-b16a-8614ef540282" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.09141162201706 142.18925051595596 30.23 27.091542355574184 142.18915312038615 30.23 27.09164303599194 142.18932199872881 30.23 27.091512212063055 142.1894193940363 30.23 27.09141162201706 142.18925051595596 30.23</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.09141162201706 142.18925051595596 30.23 27.091512212063055 142.1894193940363 30.23 27.091512212063055 142.1894193940363 35.635 27.09141162201706 142.18925051595596 35.635 27.09141162201706 142.18925051595596 30.23</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091512212063055 142.1894193940363 30.23 27.09164303599194 142.18932199872881 30.23 27.09164303599194 142.18932199872881 35.635 27.091512212063055 142.1894193940363 35.635 27.091512212063055 142.1894193940363 30.23</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.09164303599194 142.18932199872881 30.23 27.091542355574184 142.18915312038615 30.23 27.091542355574184 142.18915312038615 35.635 27.09164303599194 142.18932199872881 35.635 27.09164303599194 142.18932199872881 30.23</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091542355574184 142.18915312038615 30.23 27.09141162201706 142.18925051595596 30.23 27.09141162201706 142.18925051595596 35.635 27.091542355574184 142.18915312038615 35.635 27.091542355574184 142.18915312038615 30.23</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.09141162201706 142.18925051595596 35.635 27.091512212063055 142.1894193940363 35.635 27.09164303599194 142.18932199872881 35.635 27.091542355574184 142.18915312038615 35.635 27.09141162201706 142.18925051595596 35.635</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<bldg:address>
				<core:Address gml:id="adrs_77edfb3e-466a-4125-9be7-62791b3baea3">
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都小笠原村父島</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">2</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">100</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">11</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">101</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">1</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">102</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">103</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">104</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">107</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key107.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">350.341</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1001</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">212</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1210</uro:detailedUsage>
					<uro:surveyYear>2022</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13421-bldg-1076</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13421</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_a32476f4-2a04-4d94-8709-2354dda083aa">
			<core:creationDate>2026-03-13</core:creationDate>
			<gen:stringAttribute name="延べ面積換算係数">
				<gen:value>1</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="大字・町コード">
				<gen:value>3</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="町・丁目コード">
				<gen:value>0</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="13+区市町村コード+大字・町コード+町・丁目コード">
				<gen:value>13421003000</gen:value>
			</gen:stringAttribute>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:measuredHeight uom="m">6.007</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="bldg_d65cf808-8f94-47f8-93f0-af12496c9598">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>27.08943301492628 142.19097388005412 2.346 27.08948050090684 142.1909660950315 2.346 27.089479250456645 142.1909564122339 2.346 27.089565284621923 142.19094254129487 2.346 27.089549384797902 142.19082029571052 2.346 27.089518780620807 142.1908253867145 2.346 27.089516815631693 142.19081015648123 2.346 27.08946617010957 142.19081833957995 2.346 27.0894683142433 142.19083457851383 2.346 27.08935104410784 142.1908535394423 2.346 27.089366406882068 142.19097245617556 2.346 27.08943140659847 142.19096187732575 2.346 27.08943301492628 142.19097388005412 2.346</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid gml:id="bldg_793290c4-2f2f-464f-b3fb-04600b0d8002" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08943301492628 142.19097388005412 2.346 27.08943140659847 142.19096187732575 2.346 27.089366406882068 142.19097245617556 2.346 27.08935104410784 142.1908535394423 2.346 27.0894683142433 142.19083457851383 2.346 27.08946617010957 142.19081833957995 2.346 27.089516815631693 142.19081015648123 2.346 27.089518780620807 142.1908253867145 2.346 27.089549384797902 142.19082029571052 2.346 27.089565284621923 142.19094254129487 2.346 27.089479250456645 142.1909564122339 2.346 27.08948050090684 142.1909660950315 2.346 27.08943301492628 142.19097388005412 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08943301492628 142.19097388005412 2.346 27.08948050090684 142.1909660950315 2.346 27.08948050090684 142.1909660950315 7.176 27.08943301492628 142.19097388005412 7.176 27.08943301492628 142.19097388005412 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08948050090684 142.1909660950315 2.346 27.089479250456645 142.1909564122339 2.346 27.089479250456645 142.1909564122339 7.176 27.08948050090684 142.1909660950315 7.176 27.08948050090684 142.1909660950315 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089479250456645 142.1909564122339 2.346 27.089565284621923 142.19094254129487 2.346 27.089565284621923 142.19094254129487 7.176 27.089479250456645 142.1909564122339 7.176 27.089479250456645 142.1909564122339 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089565284621923 142.19094254129487 2.346 27.089549384797902 142.19082029571052 2.346 27.089549384797902 142.19082029571052 7.176 27.089565284621923 142.19094254129487 7.176 27.089565284621923 142.19094254129487 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089549384797902 142.19082029571052 2.346 27.089518780620807 142.1908253867145 2.346 27.089518780620807 142.1908253867145 7.176 27.089549384797902 142.19082029571052 7.176 27.089549384797902 142.19082029571052 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089518780620807 142.1908253867145 2.346 27.089516815631693 142.19081015648123 2.346 27.089516815631693 142.19081015648123 7.176 27.089518780620807 142.1908253867145 7.176 27.089518780620807 142.1908253867145 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089516815631693 142.19081015648123 2.346 27.08946617010957 142.19081833957995 2.346 27.08946617010957 142.19081833957995 7.176 27.089516815631693 142.19081015648123 7.176 27.089516815631693 142.19081015648123 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08946617010957 142.19081833957995 2.346 27.0894683142433 142.19083457851383 2.346 27.0894683142433 142.19083457851383 7.176 27.08946617010957 142.19081833957995 7.176 27.08946617010957 142.19081833957995 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.0894683142433 142.19083457851383 2.346 27.08935104410784 142.1908535394423 2.346 27.08935104410784 142.1908535394423 7.176 27.0894683142433 142.19083457851383 7.176 27.0894683142433 142.19083457851383 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08935104410784 142.1908535394423 2.346 27.089366406882068 142.19097245617556 2.346 27.089366406882068 142.19097245617556 7.176 27.08935104410784 142.1908535394423 7.176 27.08935104410784 142.1908535394423 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089366406882068 142.19097245617556 2.346 27.08943140659847 142.19096187732575 2.346 27.08943140659847 142.19096187732575 7.176 27.089366406882068 142.19097245617556 7.176 27.089366406882068 142.19097245617556 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08943140659847 142.19096187732575 2.346 27.08943301492628 142.19097388005412 2.346 27.08943301492628 142.19097388005412 7.176 27.08943140659847 142.19096187732575 7.176 27.08943140659847 142.19096187732575 2.346</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08943301492628 142.19097388005412 7.176 27.08948050090684 142.1909660950315 7.176 27.089479250456645 142.1909564122339 7.176 27.089565284621923 142.19094254129487 7.176 27.089549384797902 142.19082029571052 7.176 27.089518780620807 142.1908253867145 7.176 27.089516815631693 142.19081015648123 7.176 27.08946617010957 142.19081833957995 7.176 27.0894683142433 142.19083457851383 7.176 27.08935104410784 142.1908535394423 7.176 27.089366406882068 142.19097245617556 7.176 27.08943140659847 142.19096187732575 7.176 27.08943301492628 142.19097388005412 7.176</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<bldg:address>
				<core:Address gml:id="adrs_c594d397-a0eb-44a6-984e-6eb06dac6c08">
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都小笠原村父島</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">1</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">2</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">2</uro:rankOrg>
					<uro:depth uom="m">0.38</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">3</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">4</uro:rankOrg>
					<uro:depth uom="m">2.97</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">4</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">4</uro:rankOrg>
					<uro:depth uom="m">3.059</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">5</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">4</uro:rankOrg>
					<uro:depth uom="m">3.06</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">6</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">3</uro:rankOrg>
					<uro:depth uom="m">1.63</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">100</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">12</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">101</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">1</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">102</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">103</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">104</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">107</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key107.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">282.831</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1002</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1142</uro:detailedUsage>
					<uro:surveyYear>2022</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13421-bldg-1081</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13421</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_0477b70a-641f-4a4a-8579-735fd2ce1fdd">
			<core:creationDate>2026-03-13</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.922</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="bldg_6192a456-fb0b-43e3-8339-7630386916da">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>27.091415235440053 142.18918114264574 30.431 27.091424330651197 142.18919658677274 30.431 27.091465438256947 142.1891665040316 30.431 27.09145634304256 142.18915105990135 30.431 27.091415235440053 142.18918114264574 30.431</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid gml:id="bldg_5f48bfc6-46c5-41e3-8c7a-b977628e1b56" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091415235440053 142.18918114264574 30.431 27.09145634304256 142.18915105990135 30.431 27.091465438256947 142.1891665040316 30.431 27.091424330651197 142.18919658677274 30.431 27.091415235440053 142.18918114264574 30.431</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091415235440053 142.18918114264574 30.431 27.091424330651197 142.18919658677274 30.431 27.091424330651197 142.18919658677274 33.277 27.091415235440053 142.18918114264574 33.277 27.091415235440053 142.18918114264574 30.431</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091424330651197 142.18919658677274 30.431 27.091465438256947 142.1891665040316 30.431 27.091465438256947 142.1891665040316 33.277 27.091424330651197 142.18919658677274 33.277 27.091424330651197 142.18919658677274 30.431</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091465438256947 142.1891665040316 30.431 27.09145634304256 142.18915105990135 30.431 27.09145634304256 142.18915105990135 33.277 27.091465438256947 142.1891665040316 33.277 27.091465438256947 142.1891665040316 30.431</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.09145634304256 142.18915105990135 30.431 27.091415235440053 142.18918114264574 30.431 27.091415235440053 142.18918114264574 33.277 27.09145634304256 142.18915105990135 33.277 27.09145634304256 142.18915105990135 30.431</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.091415235440053 142.18918114264574 33.277 27.091424330651197 142.18919658677274 33.277 27.091465438256947 142.1891665040316 33.277 27.09145634304256 142.18915105990135 33.277 27.091415235440053 142.18918114264574 33.277</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<bldg:address>
				<core:Address gml:id="adrs_dfcc1da5-c17a-499f-ba41-9595a08921fc">
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都小笠原村父島</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">1</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">100</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">99</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">101</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">102</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">103</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">104</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">107</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key107.xml">9</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">-9999.000</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1011</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">212</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">0</uro:detailedUsage>
					<uro:surveyYear>2022</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13421-bldg-1079</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13421</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_4d484f8e-a9f0-4e92-8269-6b6b5c0c93f9">
			<gml:name>父島し尿処理場</gml:name>
			<core:creationDate>2026-03-13</core:creationDate>
			<gen:stringAttribute name="延べ面積換算係数">
				<gen:value>1</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="大字・町コード">
				<gen:value>3</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="町・丁目コード">
				<gen:value>0</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="13+区市町村コード+大字・町コード+町・丁目コード">
				<gen:value>13421003000</gen:value>
			</gen:stringAttribute>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:measuredHeight uom="m">6.041</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="bldg_17dbca8c-1839-48a0-aaef-2c36dbcb62a0">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>27.08943301492628 142.19097388005412 1.929 27.089365306533512 142.1909851601855 1.929 27.08938199481242 142.1911248523022 1.929 27.08962898940987 142.19108785965966 1.929 27.08961818998495 142.19099829496085 1.929 27.08957476735939 142.19100477594483 1.929 27.089568609619104 142.19095323602207 1.929 27.08948050090684 142.1909660950315 1.929 27.08943301492628 142.19097388005412 1.929</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid gml:id="bldg_9ce05666-8f7f-4d23-a0a5-ba2541aca2c4" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08943301492628 142.19097388005412 1.929 27.08948050090684 142.1909660950315 1.929 27.089568609619104 142.19095323602207 1.929 27.08957476735939 142.19100477594483 1.929 27.08961818998495 142.19099829496085 1.929 27.08962898940987 142.19108785965966 1.929 27.08938199481242 142.1911248523022 1.929 27.089365306533512 142.1909851601855 1.929 27.08943301492628 142.19097388005412 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08943301492628 142.19097388005412 1.929 27.089365306533512 142.1909851601855 1.929 27.089365306533512 142.1909851601855 6.833 27.08943301492628 142.19097388005412 6.833 27.08943301492628 142.19097388005412 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089365306533512 142.1909851601855 1.929 27.08938199481242 142.1911248523022 1.929 27.08938199481242 142.1911248523022 6.833 27.089365306533512 142.1909851601855 6.833 27.089365306533512 142.1909851601855 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08938199481242 142.1911248523022 1.929 27.08962898940987 142.19108785965966 1.929 27.08962898940987 142.19108785965966 6.833 27.08938199481242 142.1911248523022 6.833 27.08938199481242 142.1911248523022 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08962898940987 142.19108785965966 1.929 27.08961818998495 142.19099829496085 1.929 27.08961818998495 142.19099829496085 6.833 27.08962898940987 142.19108785965966 6.833 27.08962898940987 142.19108785965966 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08961818998495 142.19099829496085 1.929 27.08957476735939 142.19100477594483 1.929 27.08957476735939 142.19100477594483 6.833 27.08961818998495 142.19099829496085 6.833 27.08961818998495 142.19099829496085 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08957476735939 142.19100477594483 1.929 27.089568609619104 142.19095323602207 1.929 27.089568609619104 142.19095323602207 6.833 27.08957476735939 142.19100477594483 6.833 27.08957476735939 142.19100477594483 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089568609619104 142.19095323602207 1.929 27.08948050090684 142.1909660950315 1.929 27.08948050090684 142.1909660950315 6.833 27.089568609619104 142.19095323602207 6.833 27.089568609619104 142.19095323602207 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08948050090684 142.1909660950315 1.929 27.08943301492628 142.19097388005412 1.929 27.08943301492628 142.19097388005412 6.833 27.08948050090684 142.1909660950315 6.833 27.08948050090684 142.1909660950315 1.929</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08943301492628 142.19097388005412 6.833 27.089365306533512 142.1909851601855 6.833 27.08938199481242 142.1911248523022 6.833 27.08962898940987 142.19108785965966 6.833 27.08961818998495 142.19099829496085 6.833 27.08957476735939 142.19100477594483 6.833 27.089568609619104 142.19095323602207 6.833 27.08948050090684 142.1909660950315 6.833 27.08943301492628 142.19097388005412 6.833</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<bldg:address>
				<core:Address gml:id="adrs_96b582b5-afb3-4ea7-a44b-7517b0b9ad55">
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都小笠原村父島</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">1</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">1</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">2</uro:rankOrg>
					<uro:depth uom="m">0.827</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">2</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">4</uro:rankOrg>
					<uro:depth uom="m">2.976</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">3</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">4</uro:rankOrg>
					<uro:depth uom="m">3.269</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">4</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">5</uro:rankOrg>
					<uro:depth uom="m">5.727</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">5</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">4</uro:rankOrg>
					<uro:depth uom="m">3.433</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">6</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">4</uro:rankOrg>
					<uro:depth uom="m">3.601</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">100</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">11</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">101</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">1</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">102</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">103</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">104</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">107</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key107.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">364.227</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1001</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1142</uro:detailedUsage>
					<uro:surveyYear>2022</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13421-bldg-1080</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13421</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_20baf9d9-8c17-45a5-9a5a-153487ee7bee">
			<core:creationDate>2026-03-13</core:creationDate>
			<gen:stringAttribute name="延べ面積換算係数">
				<gen:value>1</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="大字・町コード">
				<gen:value>3</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="町・丁目コード">
				<gen:value>0</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="13+区市町村コード+大字・町コード+町・丁目コード">
				<gen:value>13421003000</gen:value>
			</gen:stringAttribute>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">452</bldg:usage>
			<bldg:measuredHeight uom="m">4.93</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>0</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="bldg_ef82d80c-0b2d-471e-88b8-b2d2bbe4188f">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>27.08946617010957 142.19081833957995 2.18 27.089516815631693 142.19081015648123 2.18 27.08954444159395 142.19080485874625 2.18 27.089516476450868 142.1905944582456 2.18 27.089359395667003 142.19061869658296 2.18 27.089384923694197 142.19082919350623 2.18 27.08946617010957 142.19081833957995 2.18</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid gml:id="bldg_a94a73c1-c5db-48fe-b9e8-631735e0be9b" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08946617010957 142.19081833957995 2.18 27.089384923694197 142.19082919350623 2.18 27.089359395667003 142.19061869658296 2.18 27.089516476450868 142.1905944582456 2.18 27.08954444159395 142.19080485874625 2.18 27.089516815631693 142.19081015648123 2.18 27.08946617010957 142.19081833957995 2.18</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08946617010957 142.19081833957995 2.18 27.089516815631693 142.19081015648123 2.18 27.089516815631693 142.19081015648123 6.505000000000001 27.08946617010957 142.19081833957995 6.505000000000001 27.08946617010957 142.19081833957995 2.18</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089516815631693 142.19081015648123 2.18 27.08954444159395 142.19080485874625 2.18 27.08954444159395 142.19080485874625 6.505000000000001 27.089516815631693 142.19081015648123 6.505000000000001 27.089516815631693 142.19081015648123 2.18</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08954444159395 142.19080485874625 2.18 27.089516476450868 142.1905944582456 2.18 27.089516476450868 142.1905944582456 6.505000000000001 27.08954444159395 142.19080485874625 6.505000000000001 27.08954444159395 142.19080485874625 2.18</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089516476450868 142.1905944582456 2.18 27.089359395667003 142.19061869658296 2.18 27.089359395667003 142.19061869658296 6.505000000000001 27.089516476450868 142.1905944582456 6.505000000000001 27.089516476450868 142.1905944582456 2.18</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089359395667003 142.19061869658296 2.18 27.089384923694197 142.19082919350623 2.18 27.089384923694197 142.19082919350623 6.505000000000001 27.089359395667003 142.19061869658296 6.505000000000001 27.089359395667003 142.19061869658296 2.18</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.089384923694197 142.19082919350623 2.18 27.08946617010957 142.19081833957995 2.18 27.08946617010957 142.19081833957995 6.505000000000001 27.089384923694197 142.19082919350623 6.505000000000001 27.089384923694197 142.19082919350623 2.18</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>27.08946617010957 142.19081833957995 6.505000000000001 27.089516815631693 142.19081015648123 6.505000000000001 27.08954444159395 142.19080485874625 6.505000000000001 27.089516476450868 142.1905944582456 6.505000000000001 27.089359395667003 142.19061869658296 6.505000000000001 27.089384923694197 142.19082919350623 6.505000000000001 27.08946617010957 142.19081833957995 6.505000000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<bldg:address>
				<core:Address gml:id="adrs_d9ed12cd-ef8c-456d-8574-fae4008e7587">
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都小笠原村父島</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">2</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">2</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">2</uro:rankOrg>
					<uro:depth uom="m">0.369</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">3</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">2</uro:rankOrg>
					<uro:depth uom="m">0.736</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">4</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">2</uro:rankOrg>
					<uro:depth uom="m">0.874</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:TsunamiRiskAttribute>
					<uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">5</uro:description>
					<uro:rankOrg codeSpace="../../codelists/TsunamiRiskAttribute_rankOrg.xml">2</uro:rankOrg>
					<uro:depth uom="m">0.806</uro:depth>
				</uro:TsunamiRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">100</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key100.xml">12</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">101</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key101.xml">1</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">102</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key102.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">103</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key103.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">104</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key104.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:bldgKeyValuePairAttribute>
				<uro:KeyValuePairAttribute>
					<uro:key codeSpace="../../codelists/KeyValuePairAttribute_key.xml">107</uro:key>
					<uro:codeValue codeSpace="../../codelists/KeyValuePairAttribute_key107.xml">0</uro:codeValue>
				</uro:KeyValuePairAttribute>
			</uro:bldgKeyValuePairAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">374.743</uro:buildingRoofEdgeArea>
					<uro:fireproofStructureType codeSpace="../../codelists/BuildingDetailAttribute_fireproofStructureType.xml">1002</uro:fireproofStructureType>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:detailedUsage codeSpace="../../codelists/BuildingDetailAttribute_detailedUsage.xml">1142</uro:detailedUsage>
					<uro:surveyYear>2022</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>13421-bldg-1082</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13421</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>