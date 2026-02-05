<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:urf="https://www.geospatial.jp/iur/urf/3.2" xmlns:uro="https://www.geospatial.jp/iur/uro/3.2" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.2 ../../schemas/iur/urf/3.2/urbanFunction.xsd
https://www.geospatial.jp/iur/uro/3.2 ../../schemas/iur/uro/3.2/urbanObject.xsd
http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd
http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd 
http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd 
http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd 
http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd 
http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd 
http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd 
http://www.opengis.net/gml http://schemas.opengis.net/gml/3.2.1/base/gml.xsd
http://www.opengis.net/citygml/appearance/2.0
http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.69334011909071 140.4551797582095 0</gml:lowerCorner>
			<gml:upperCorner>35.700071958482155 140.46097724750587 18.34435</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_e897d5e2-d93f-47a8-8112-0d70c0eb1a85">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69987670458783 140.45537407452608 0 35.69985184188534 140.45538868857415 0 35.69986545327417 140.45542338117164 0 35.69989031540966 140.45540887762877 0 35.69987670458783 140.45537407452608 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69987670458783 140.45537407452608 6.08236 35.69989031540966 140.45540887762877 6.08236 35.69986545327417 140.45542338117164 6.08236 35.69985184188534 140.45538868857415 6.08236 35.69987670458783 140.45537407452608 6.08236</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69987670458783 140.45537407452608 6.08236 35.69985184188534 140.45538868857415 6.08236 35.69985184188534 140.45538868857415 8.582360000000001 35.69987670458783 140.45537407452608 8.582360000000001 35.69987670458783 140.45537407452608 6.08236</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69985184188534 140.45538868857415 6.08236 35.69986545327417 140.45542338117164 6.08236 35.69986545327417 140.45542338117164 8.582360000000001 35.69985184188534 140.45538868857415 8.582360000000001 35.69985184188534 140.45538868857415 6.08236</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69986545327417 140.45542338117164 6.08236 35.69989031540966 140.45540887762877 6.08236 35.69989031540966 140.45540887762877 8.582360000000001 35.69986545327417 140.45542338117164 8.582360000000001 35.69986545327417 140.45542338117164 6.08236</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69989031540966 140.45540887762877 6.08236 35.69987670458783 140.45537407452608 6.08236 35.69987670458783 140.45537407452608 8.582360000000001 35.69989031540966 140.45540887762877 8.582360000000001 35.69989031540966 140.45540887762877 6.08236</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69987670458783 140.45537407452608 8.582360000000001 35.69985184188534 140.45538868857415 8.582360000000001 35.69986545327417 140.45542338117164 8.582360000000001 35.69989031540966 140.45540887762877 8.582360000000001 35.69987670458783 140.45537407452608 8.582360000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">10.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8308</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_8866fe16-65db-4d1f-a5c4-d7bbc3144f67">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.693448084588695 140.45961456816218 0 35.69341461357442 140.459620494048 0 35.69342005329753 140.45966639087135 0 35.69334011909071 140.45968068232628 0 35.693350483558774 140.45976749972615 0 35.69336320476349 140.45976516837806 0 35.693365536544874 140.45978474373194 0 35.69338123439253 140.45978199369532 0 35.69338279063694 140.4597947124712 0 35.69337106202931 140.4597968305932 0 35.693375983322426 140.45983841415205 0 35.69339944053897 140.45983417791942 0 35.69340358801429 140.45986857344656 0 35.69344923888142 140.4598604225882 0 35.693447165431465 140.45984316957066 0 35.69347630611378 140.459837983355 0 35.69347267814423 140.4598076800777 0 35.693479444339026 140.45980651757728 0 35.69347547078473 140.45977333879654 0 35.69346717062813 140.45977482078092 0 35.693448084588695 140.45961456816218 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693448084588695 140.45961456816218 5.0801 35.69346717062813 140.45977482078092 5.0801 35.69347547078473 140.45977333879654 5.0801 35.693479444339026 140.45980651757728 5.0801 35.69347267814423 140.4598076800777 5.0801 35.69347630611378 140.459837983355 5.0801 35.693447165431465 140.45984316957066 5.0801 35.69344923888142 140.4598604225882 5.0801 35.69340358801429 140.45986857344656 5.0801 35.69339944053897 140.45983417791942 5.0801 35.693375983322426 140.45983841415205 5.0801 35.69337106202931 140.4597968305932 5.0801 35.69338279063694 140.4597947124712 5.0801 35.69338123439253 140.45978199369532 5.0801 35.693365536544874 140.45978474373194 5.0801 35.69336320476349 140.45976516837806 5.0801 35.693350483558774 140.45976749972615 5.0801 35.69334011909071 140.45968068232628 5.0801 35.69342005329753 140.45966639087135 5.0801 35.69341461357442 140.459620494048 5.0801 35.693448084588695 140.45961456816218 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693448084588695 140.45961456816218 5.0801 35.69341461357442 140.459620494048 5.0801 35.69341461357442 140.459620494048 11.0801 35.693448084588695 140.45961456816218 11.0801 35.693448084588695 140.45961456816218 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69341461357442 140.459620494048 5.0801 35.69342005329753 140.45966639087135 5.0801 35.69342005329753 140.45966639087135 11.0801 35.69341461357442 140.459620494048 11.0801 35.69341461357442 140.459620494048 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69342005329753 140.45966639087135 5.0801 35.69334011909071 140.45968068232628 5.0801 35.69334011909071 140.45968068232628 11.0801 35.69342005329753 140.45966639087135 11.0801 35.69342005329753 140.45966639087135 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69334011909071 140.45968068232628 5.0801 35.693350483558774 140.45976749972615 5.0801 35.693350483558774 140.45976749972615 11.0801 35.69334011909071 140.45968068232628 11.0801 35.69334011909071 140.45968068232628 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693350483558774 140.45976749972615 5.0801 35.69336320476349 140.45976516837806 5.0801 35.69336320476349 140.45976516837806 11.0801 35.693350483558774 140.45976749972615 11.0801 35.693350483558774 140.45976749972615 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69336320476349 140.45976516837806 5.0801 35.693365536544874 140.45978474373194 5.0801 35.693365536544874 140.45978474373194 11.0801 35.69336320476349 140.45976516837806 11.0801 35.69336320476349 140.45976516837806 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693365536544874 140.45978474373194 5.0801 35.69338123439253 140.45978199369532 5.0801 35.69338123439253 140.45978199369532 11.0801 35.693365536544874 140.45978474373194 11.0801 35.693365536544874 140.45978474373194 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69338123439253 140.45978199369532 5.0801 35.69338279063694 140.4597947124712 5.0801 35.69338279063694 140.4597947124712 11.0801 35.69338123439253 140.45978199369532 11.0801 35.69338123439253 140.45978199369532 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69338279063694 140.4597947124712 5.0801 35.69337106202931 140.4597968305932 5.0801 35.69337106202931 140.4597968305932 11.0801 35.69338279063694 140.4597947124712 11.0801 35.69338279063694 140.4597947124712 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69337106202931 140.4597968305932 5.0801 35.693375983322426 140.45983841415205 5.0801 35.693375983322426 140.45983841415205 11.0801 35.69337106202931 140.4597968305932 11.0801 35.69337106202931 140.4597968305932 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693375983322426 140.45983841415205 5.0801 35.69339944053897 140.45983417791942 5.0801 35.69339944053897 140.45983417791942 11.0801 35.693375983322426 140.45983841415205 11.0801 35.693375983322426 140.45983841415205 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69339944053897 140.45983417791942 5.0801 35.69340358801429 140.45986857344656 5.0801 35.69340358801429 140.45986857344656 11.0801 35.69339944053897 140.45983417791942 11.0801 35.69339944053897 140.45983417791942 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69340358801429 140.45986857344656 5.0801 35.69344923888142 140.4598604225882 5.0801 35.69344923888142 140.4598604225882 11.0801 35.69340358801429 140.45986857344656 11.0801 35.69340358801429 140.45986857344656 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69344923888142 140.4598604225882 5.0801 35.693447165431465 140.45984316957066 5.0801 35.693447165431465 140.45984316957066 11.0801 35.69344923888142 140.4598604225882 11.0801 35.69344923888142 140.4598604225882 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693447165431465 140.45984316957066 5.0801 35.69347630611378 140.459837983355 5.0801 35.69347630611378 140.459837983355 11.0801 35.693447165431465 140.45984316957066 11.0801 35.693447165431465 140.45984316957066 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69347630611378 140.459837983355 5.0801 35.69347267814423 140.4598076800777 5.0801 35.69347267814423 140.4598076800777 11.0801 35.69347630611378 140.459837983355 11.0801 35.69347630611378 140.459837983355 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69347267814423 140.4598076800777 5.0801 35.693479444339026 140.45980651757728 5.0801 35.693479444339026 140.45980651757728 11.0801 35.69347267814423 140.4598076800777 11.0801 35.69347267814423 140.4598076800777 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693479444339026 140.45980651757728 5.0801 35.69347547078473 140.45977333879654 5.0801 35.69347547078473 140.45977333879654 11.0801 35.693479444339026 140.45980651757728 11.0801 35.693479444339026 140.45980651757728 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69347547078473 140.45977333879654 5.0801 35.69346717062813 140.45977482078092 5.0801 35.69346717062813 140.45977482078092 11.0801 35.69347547078473 140.45977333879654 11.0801 35.69347547078473 140.45977333879654 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69346717062813 140.45977482078092 5.0801 35.693448084588695 140.45961456816218 5.0801 35.693448084588695 140.45961456816218 11.0801 35.69346717062813 140.45977482078092 11.0801 35.69346717062813 140.45977482078092 5.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693448084588695 140.45961456816218 11.0801 35.69341461357442 140.459620494048 11.0801 35.69342005329753 140.45966639087135 11.0801 35.69334011909071 140.45968068232628 11.0801 35.693350483558774 140.45976749972615 11.0801 35.69336320476349 140.45976516837806 11.0801 35.693365536544874 140.45978474373194 11.0801 35.69338123439253 140.45978199369532 11.0801 35.69338279063694 140.4597947124712 11.0801 35.69337106202931 140.4597968305932 11.0801 35.693375983322426 140.45983841415205 11.0801 35.69339944053897 140.45983417791942 11.0801 35.69340358801429 140.45986857344656 11.0801 35.69344923888142 140.4598604225882 11.0801 35.693447165431465 140.45984316957066 11.0801 35.69347630611378 140.459837983355 11.0801 35.69347267814423 140.4598076800777 11.0801 35.693479444339026 140.45980651757728 11.0801 35.69347547078473 140.45977333879654 11.0801 35.69346717062813 140.45977482078092 11.0801 35.693448084588695 140.45961456816218 11.0801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">213.5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8271</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_ab6d7558-0965-44b5-a8b6-e87a196a3a5d">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">8.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69908315643535 140.4557223954904 0 35.699131771059655 140.4557859793879 0 35.69918998464076 140.45571891616302 0 35.69917583869201 140.4557004631922 0 35.69918608526281 140.45568860878973 0 35.699151525845124 140.45564358765108 0 35.69908315643535 140.4557223954904 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69908315643535 140.4557223954904 7.33653 35.699151525845124 140.45564358765108 7.33653 35.69918608526281 140.45568860878973 7.33653 35.69917583869201 140.4557004631922 7.33653 35.69918998464076 140.45571891616302 7.33653 35.699131771059655 140.4557859793879 7.33653 35.69908315643535 140.4557223954904 7.33653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69908315643535 140.4557223954904 7.33653 35.699131771059655 140.4557859793879 7.33653 35.699131771059655 140.4557859793879 14.13653 35.69908315643535 140.4557223954904 14.13653 35.69908315643535 140.4557223954904 7.33653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699131771059655 140.4557859793879 7.33653 35.69918998464076 140.45571891616302 7.33653 35.69918998464076 140.45571891616302 14.13653 35.699131771059655 140.4557859793879 14.13653 35.699131771059655 140.4557859793879 7.33653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69918998464076 140.45571891616302 7.33653 35.69917583869201 140.4557004631922 7.33653 35.69917583869201 140.4557004631922 14.13653 35.69918998464076 140.45571891616302 14.13653 35.69918998464076 140.45571891616302 7.33653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69917583869201 140.4557004631922 7.33653 35.69918608526281 140.45568860878973 7.33653 35.69918608526281 140.45568860878973 14.13653 35.69917583869201 140.4557004631922 14.13653 35.69917583869201 140.4557004631922 7.33653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69918608526281 140.45568860878973 7.33653 35.699151525845124 140.45564358765108 7.33653 35.699151525845124 140.45564358765108 14.13653 35.69918608526281 140.45568860878973 14.13653 35.69918608526281 140.45568860878973 7.33653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699151525845124 140.45564358765108 7.33653 35.69908315643535 140.4557223954904 7.33653 35.69908315643535 140.4557223954904 14.13653 35.699151525845124 140.45564358765108 14.13653 35.699151525845124 140.45564358765108 7.33653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69908315643535 140.4557223954904 14.13653 35.699131771059655 140.4557859793879 14.13653 35.69918998464076 140.45571891616302 14.13653 35.69917583869201 140.4557004631922 14.13653 35.69918608526281 140.45568860878973 14.13653 35.699151525845124 140.45564358765108 14.13653 35.69908315643535 140.4557223954904 14.13653</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">78.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8245</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_c648f3a7-9753-43e0-a9d4-995538e414be">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69966266567931 140.4553329632556 0 35.699650527199864 140.45529231515167 0 35.69966561457274 140.4552855812626 0 35.699634601740215 140.45518213259894 0 35.69961662274106 140.45519028056364 0 35.699613432235324 140.4551797582095 0 35.69952697136881 140.4552187566437 0 35.699530074023286 140.45522883630278 0 35.6994484916647 140.45526566251672 0 35.699491822004106 140.45540998139083 0 35.69966266567931 140.4553329632556 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69966266567931 140.4553329632556 6.61515 35.699491822004106 140.45540998139083 6.61515 35.6994484916647 140.45526566251672 6.61515 35.699530074023286 140.45522883630278 6.61515 35.69952697136881 140.4552187566437 6.61515 35.699613432235324 140.4551797582095 6.61515 35.69961662274106 140.45519028056364 6.61515 35.699634601740215 140.45518213259894 6.61515 35.69966561457274 140.4552855812626 6.61515 35.699650527199864 140.45529231515167 6.61515 35.69966266567931 140.4553329632556 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69966266567931 140.4553329632556 6.61515 35.699650527199864 140.45529231515167 6.61515 35.699650527199864 140.45529231515167 13.11515 35.69966266567931 140.4553329632556 13.11515 35.69966266567931 140.4553329632556 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699650527199864 140.45529231515167 6.61515 35.69966561457274 140.4552855812626 6.61515 35.69966561457274 140.4552855812626 13.11515 35.699650527199864 140.45529231515167 13.11515 35.699650527199864 140.45529231515167 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69966561457274 140.4552855812626 6.61515 35.699634601740215 140.45518213259894 6.61515 35.699634601740215 140.45518213259894 13.11515 35.69966561457274 140.4552855812626 13.11515 35.69966561457274 140.4552855812626 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699634601740215 140.45518213259894 6.61515 35.69961662274106 140.45519028056364 6.61515 35.69961662274106 140.45519028056364 13.11515 35.699634601740215 140.45518213259894 13.11515 35.699634601740215 140.45518213259894 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69961662274106 140.45519028056364 6.61515 35.699613432235324 140.4551797582095 6.61515 35.699613432235324 140.4551797582095 13.11515 35.69961662274106 140.45519028056364 13.11515 35.69961662274106 140.45519028056364 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699613432235324 140.4551797582095 6.61515 35.69952697136881 140.4552187566437 6.61515 35.69952697136881 140.4552187566437 13.11515 35.699613432235324 140.4551797582095 13.11515 35.699613432235324 140.4551797582095 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69952697136881 140.4552187566437 6.61515 35.699530074023286 140.45522883630278 6.61515 35.699530074023286 140.45522883630278 13.11515 35.69952697136881 140.4552187566437 13.11515 35.69952697136881 140.4552187566437 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699530074023286 140.45522883630278 6.61515 35.6994484916647 140.45526566251672 6.61515 35.6994484916647 140.45526566251672 13.11515 35.699530074023286 140.45522883630278 13.11515 35.699530074023286 140.45522883630278 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6994484916647 140.45526566251672 6.61515 35.699491822004106 140.45540998139083 6.61515 35.699491822004106 140.45540998139083 13.11515 35.6994484916647 140.45526566251672 13.11515 35.6994484916647 140.45526566251672 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699491822004106 140.45540998139083 6.61515 35.69966266567931 140.4553329632556 6.61515 35.69966266567931 140.4553329632556 13.11515 35.699491822004106 140.45540998139083 13.11515 35.699491822004106 140.45540998139083 6.61515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69966266567931 140.4553329632556 13.11515 35.699650527199864 140.45529231515167 13.11515 35.69966561457274 140.4552855812626 13.11515 35.699634601740215 140.45518213259894 13.11515 35.69961662274106 140.45519028056364 13.11515 35.699613432235324 140.4551797582095 13.11515 35.69952697136881 140.4552187566437 13.11515 35.699530074023286 140.45522883630278 13.11515 35.6994484916647 140.45526566251672 13.11515 35.699491822004106 140.45540998139083 13.11515 35.69966266567931 140.4553329632556 13.11515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">309</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8243</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_6bece29c-0d8d-41a9-a185-407c2590ff6a">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69880176498746 140.45613856072532 0 35.69879889621332 140.45613555492443 0 35.69880843320796 140.4561214851206 0 35.69875499490878 140.4560669247472 0 35.69873737446935 140.45609286565568 0 35.69873396776288 140.45608930317795 0 35.698704085795775 140.45613327066215 0 35.69876389024482 140.45619428953424 0 35.69880176498746 140.45613856072532 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69880176498746 140.45613856072532 6.33736 35.69876389024482 140.45619428953424 6.33736 35.698704085795775 140.45613327066215 6.33736 35.69873396776288 140.45608930317795 6.33736 35.69873737446935 140.45609286565568 6.33736 35.69875499490878 140.4560669247472 6.33736 35.69880843320796 140.4561214851206 6.33736 35.69879889621332 140.45613555492443 6.33736 35.69880176498746 140.45613856072532 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69880176498746 140.45613856072532 6.33736 35.69879889621332 140.45613555492443 6.33736 35.69879889621332 140.45613555492443 8.43736 35.69880176498746 140.45613856072532 8.43736 35.69880176498746 140.45613856072532 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69879889621332 140.45613555492443 6.33736 35.69880843320796 140.4561214851206 6.33736 35.69880843320796 140.4561214851206 8.43736 35.69879889621332 140.45613555492443 8.43736 35.69879889621332 140.45613555492443 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69880843320796 140.4561214851206 6.33736 35.69875499490878 140.4560669247472 6.33736 35.69875499490878 140.4560669247472 8.43736 35.69880843320796 140.4561214851206 8.43736 35.69880843320796 140.4561214851206 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69875499490878 140.4560669247472 6.33736 35.69873737446935 140.45609286565568 6.33736 35.69873737446935 140.45609286565568 8.43736 35.69875499490878 140.4560669247472 8.43736 35.69875499490878 140.4560669247472 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69873737446935 140.45609286565568 6.33736 35.69873396776288 140.45608930317795 6.33736 35.69873396776288 140.45608930317795 8.43736 35.69873737446935 140.45609286565568 8.43736 35.69873737446935 140.45609286565568 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69873396776288 140.45608930317795 6.33736 35.698704085795775 140.45613327066215 6.33736 35.698704085795775 140.45613327066215 8.43736 35.69873396776288 140.45608930317795 8.43736 35.69873396776288 140.45608930317795 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698704085795775 140.45613327066215 6.33736 35.69876389024482 140.45619428953424 6.33736 35.69876389024482 140.45619428953424 8.43736 35.698704085795775 140.45613327066215 8.43736 35.698704085795775 140.45613327066215 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69876389024482 140.45619428953424 6.33736 35.69880176498746 140.45613856072532 6.33736 35.69880176498746 140.45613856072532 8.43736 35.69876389024482 140.45619428953424 8.43736 35.69876389024482 140.45619428953424 6.33736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69880176498746 140.45613856072532 8.43736 35.69879889621332 140.45613555492443 8.43736 35.69880843320796 140.4561214851206 8.43736 35.69875499490878 140.4560669247472 8.43736 35.69873737446935 140.45609286565568 8.43736 35.69873396776288 140.45608930317795 8.43736 35.698704085795775 140.45613327066215 8.43736 35.69876389024482 140.45619428953424 8.43736 35.69880176498746 140.45613856072532 8.43736</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">68.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8239</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_4c63eb46-7ad4-4a4a-8cee-e70767a7b9a7">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69886448447304 140.4571335467194 0 35.69879435340601 140.45708250212178 0 35.698735771770416 140.45720304297532 0 35.69880590221358 140.4572541981349 0 35.69886448447304 140.4571335467194 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69886448447304 140.4571335467194 5.4 35.69880590221358 140.4572541981349 5.4 35.698735771770416 140.45720304297532 5.4 35.69879435340601 140.45708250212178 5.4 35.69886448447304 140.4571335467194 5.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69886448447304 140.4571335467194 5.4 35.69879435340601 140.45708250212178 5.4 35.69879435340601 140.45708250212178 9.100000000000001 35.69886448447304 140.4571335467194 9.100000000000001 35.69886448447304 140.4571335467194 5.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69879435340601 140.45708250212178 5.4 35.698735771770416 140.45720304297532 5.4 35.698735771770416 140.45720304297532 9.100000000000001 35.69879435340601 140.45708250212178 9.100000000000001 35.69879435340601 140.45708250212178 5.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698735771770416 140.45720304297532 5.4 35.69880590221358 140.4572541981349 5.4 35.69880590221358 140.4572541981349 9.100000000000001 35.698735771770416 140.45720304297532 9.100000000000001 35.698735771770416 140.45720304297532 5.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69880590221358 140.4572541981349 5.4 35.69886448447304 140.4571335467194 5.4 35.69886448447304 140.4571335467194 9.100000000000001 35.69880590221358 140.4572541981349 9.100000000000001 35.69880590221358 140.4572541981349 5.4</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69886448447304 140.4571335467194 9.100000000000001 35.69879435340601 140.45708250212178 9.100000000000001 35.698735771770416 140.45720304297532 9.100000000000001 35.69880590221358 140.4572541981349 9.100000000000001 35.69886448447304 140.4571335467194 9.100000000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">115</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8256</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_568d7670-2c41-41a0-bf8f-38a21571eb93">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69541782258613 140.46094474894815 0 35.695436490843335 140.46087362591356 0 35.6953493290536 140.4608392420976 0 35.69533066081713 140.4609103650634 0 35.69541782258613 140.46094474894815 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69541782258613 140.46094474894815 6.45601 35.69533066081713 140.4609103650634 6.45601 35.6953493290536 140.4608392420976 6.45601 35.695436490843335 140.46087362591356 6.45601 35.69541782258613 140.46094474894815 6.45601</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69541782258613 140.46094474894815 6.45601 35.695436490843335 140.46087362591356 6.45601 35.695436490843335 140.46087362591356 12.45601 35.69541782258613 140.46094474894815 12.45601 35.69541782258613 140.46094474894815 6.45601</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695436490843335 140.46087362591356 6.45601 35.6953493290536 140.4608392420976 6.45601 35.6953493290536 140.4608392420976 12.45601 35.695436490843335 140.46087362591356 12.45601 35.695436490843335 140.46087362591356 6.45601</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6953493290536 140.4608392420976 6.45601 35.69533066081713 140.4609103650634 6.45601 35.69533066081713 140.4609103650634 12.45601 35.6953493290536 140.4608392420976 12.45601 35.6953493290536 140.4608392420976 6.45601</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69533066081713 140.4609103650634 6.45601 35.69541782258613 140.46094474894815 6.45601 35.69541782258613 140.46094474894815 12.45601 35.69533066081713 140.4609103650634 12.45601 35.69533066081713 140.4609103650634 6.45601</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69541782258613 140.46094474894815 12.45601 35.695436490843335 140.46087362591356 12.45601 35.6953493290536 140.4608392420976 12.45601 35.69533066081713 140.4609103650634 12.45601 35.69541782258613 140.46094474894815 12.45601</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">3</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">68.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8196</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_6f79038e-e6a1-473c-83d3-97af537d2742">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69984728364218 140.4555381612545 0 35.69987746126932 140.45552414105103 0 35.69986948193641 140.45549844278133 0 35.699876438630774 140.45549529229 0 35.69985666882252 140.4554313795235 0 35.69981962464419 140.45544855095247 0 35.69984728364218 140.4555381612545 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69984728364218 140.4555381612545 5.95299 35.69981962464419 140.45544855095247 5.95299 35.69985666882252 140.4554313795235 5.95299 35.699876438630774 140.45549529229 5.95299 35.69986948193641 140.45549844278133 5.95299 35.69987746126932 140.45552414105103 5.95299 35.69984728364218 140.4555381612545 5.95299</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69984728364218 140.4555381612545 5.95299 35.69987746126932 140.45552414105103 5.95299 35.69987746126932 140.45552414105103 8.35299 35.69984728364218 140.4555381612545 8.35299 35.69984728364218 140.4555381612545 5.95299</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69987746126932 140.45552414105103 5.95299 35.69986948193641 140.45549844278133 5.95299 35.69986948193641 140.45549844278133 8.35299 35.69987746126932 140.45552414105103 8.35299 35.69987746126932 140.45552414105103 5.95299</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69986948193641 140.45549844278133 5.95299 35.699876438630774 140.45549529229 5.95299 35.699876438630774 140.45549529229 8.35299 35.69986948193641 140.45549844278133 8.35299 35.69986948193641 140.45549844278133 5.95299</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699876438630774 140.45549529229 5.95299 35.69985666882252 140.4554313795235 5.95299 35.69985666882252 140.4554313795235 8.35299 35.699876438630774 140.45549529229 8.35299 35.699876438630774 140.45549529229 5.95299</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69985666882252 140.4554313795235 5.95299 35.69981962464419 140.45544855095247 5.95299 35.69981962464419 140.45544855095247 8.35299 35.69985666882252 140.4554313795235 8.35299 35.69985666882252 140.4554313795235 5.95299</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69981962464419 140.45544855095247 5.95299 35.69984728364218 140.4555381612545 5.95299 35.69984728364218 140.4555381612545 8.35299 35.69981962464419 140.45544855095247 8.35299 35.69981962464419 140.45544855095247 5.95299</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69984728364218 140.4555381612545 8.35299 35.69987746126932 140.45552414105103 8.35299 35.69986948193641 140.45549844278133 8.35299 35.699876438630774 140.45549529229 8.35299 35.69985666882252 140.4554313795235 8.35299 35.69981962464419 140.45544855095247 8.35299 35.69984728364218 140.4555381612545 8.35299</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">36.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8318</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_7128aa28-9893-49a5-b9d8-5aec0698a893">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69839774686672 140.45811014948762 0 35.698432130197084 140.4580326262674 0 35.698361258389816 140.45798544285805 0 35.69832687508991 140.45806296603087 0 35.69839774686672 140.45811014948762 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69839774686672 140.45811014948762 6.1153 35.69832687508991 140.45806296603087 6.1153 35.698361258389816 140.45798544285805 6.1153 35.698432130197084 140.4580326262674 6.1153 35.69839774686672 140.45811014948762 6.1153</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69839774686672 140.45811014948762 6.1153 35.698432130197084 140.4580326262674 6.1153 35.698432130197084 140.4580326262674 9.9153 35.69839774686672 140.45811014948762 9.9153 35.69839774686672 140.45811014948762 6.1153</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698432130197084 140.4580326262674 6.1153 35.698361258389816 140.45798544285805 6.1153 35.698361258389816 140.45798544285805 9.9153 35.698432130197084 140.4580326262674 9.9153 35.698432130197084 140.4580326262674 6.1153</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698361258389816 140.45798544285805 6.1153 35.69832687508991 140.45806296603087 6.1153 35.69832687508991 140.45806296603087 9.9153 35.698361258389816 140.45798544285805 9.9153 35.698361258389816 140.45798544285805 6.1153</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69832687508991 140.45806296603087 6.1153 35.69839774686672 140.45811014948762 6.1153 35.69839774686672 140.45811014948762 9.9153 35.69832687508991 140.45806296603087 9.9153 35.69832687508991 140.45806296603087 6.1153</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69839774686672 140.45811014948762 9.9153 35.698432130197084 140.4580326262674 9.9153 35.698361258389816 140.45798544285805 9.9153 35.69832687508991 140.45806296603087 9.9153 35.69839774686672 140.45811014948762 9.9153</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">71.5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8211</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_70a6ca9f-10fb-4966-bb6f-ee09ea7c743f">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>1</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69733633181968 140.4602218783509 0 35.69734619277098 140.46023223181618 0 35.697322946457824 140.4602651991645 0 35.697420117240455 140.4603680597179 0 35.69744463520726 140.46033322385193 0 35.69745288241443 140.4603419072445 0 35.69751381417828 140.4602553118824 0 35.69751793778473 140.46025965357828 0 35.69757914161089 140.46017272871882 0 35.69745982898808 140.4600464903865 0 35.69733633181968 140.4602218783509 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69733633181968 140.4602218783509 6.52645 35.69745982898808 140.4600464903865 6.52645 35.69757914161089 140.46017272871882 6.52645 35.69751793778473 140.46025965357828 6.52645 35.69751381417828 140.4602553118824 6.52645 35.69745288241443 140.4603419072445 6.52645 35.69744463520726 140.46033322385193 6.52645 35.697420117240455 140.4603680597179 6.52645 35.697322946457824 140.4602651991645 6.52645 35.69734619277098 140.46023223181618 6.52645 35.69733633181968 140.4602218783509 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69733633181968 140.4602218783509 6.52645 35.69734619277098 140.46023223181618 6.52645 35.69734619277098 140.46023223181618 9.52645 35.69733633181968 140.4602218783509 9.52645 35.69733633181968 140.4602218783509 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69734619277098 140.46023223181618 6.52645 35.697322946457824 140.4602651991645 6.52645 35.697322946457824 140.4602651991645 9.52645 35.69734619277098 140.46023223181618 9.52645 35.69734619277098 140.46023223181618 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697322946457824 140.4602651991645 6.52645 35.697420117240455 140.4603680597179 6.52645 35.697420117240455 140.4603680597179 9.52645 35.697322946457824 140.4602651991645 9.52645 35.697322946457824 140.4602651991645 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697420117240455 140.4603680597179 6.52645 35.69744463520726 140.46033322385193 6.52645 35.69744463520726 140.46033322385193 9.52645 35.697420117240455 140.4603680597179 9.52645 35.697420117240455 140.4603680597179 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69744463520726 140.46033322385193 6.52645 35.69745288241443 140.4603419072445 6.52645 35.69745288241443 140.4603419072445 9.52645 35.69744463520726 140.46033322385193 9.52645 35.69744463520726 140.46033322385193 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69745288241443 140.4603419072445 6.52645 35.69751381417828 140.4602553118824 6.52645 35.69751381417828 140.4602553118824 9.52645 35.69745288241443 140.4603419072445 9.52645 35.69745288241443 140.4603419072445 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69751381417828 140.4602553118824 6.52645 35.69751793778473 140.46025965357828 6.52645 35.69751793778473 140.46025965357828 9.52645 35.69751381417828 140.4602553118824 9.52645 35.69751381417828 140.4602553118824 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69751793778473 140.46025965357828 6.52645 35.69757914161089 140.46017272871882 6.52645 35.69757914161089 140.46017272871882 9.52645 35.69751793778473 140.46025965357828 9.52645 35.69751793778473 140.46025965357828 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69757914161089 140.46017272871882 6.52645 35.69745982898808 140.4600464903865 6.52645 35.69745982898808 140.4600464903865 9.52645 35.69757914161089 140.46017272871882 9.52645 35.69757914161089 140.46017272871882 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69745982898808 140.4600464903865 6.52645 35.69733633181968 140.4602218783509 6.52645 35.69733633181968 140.4602218783509 9.52645 35.69745982898808 140.4600464903865 9.52645 35.69745982898808 140.4600464903865 6.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69733633181968 140.4602218783509 9.52645 35.69734619277098 140.46023223181618 9.52645 35.697322946457824 140.4602651991645 9.52645 35.697420117240455 140.4603680597179 9.52645 35.69744463520726 140.46033322385193 9.52645 35.69745288241443 140.4603419072445 9.52645 35.69751381417828 140.4602553118824 9.52645 35.69751793778473 140.46025965357828 9.52645 35.69757914161089 140.46017272871882 9.52645 35.69745982898808 140.4600464903865 9.52645 35.69733633181968 140.4602218783509 9.52645</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">9</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">416.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-9153</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_8121f4c2-780a-403b-8c4c-ca9a7d3dc50e">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69827419070759 140.45723756965225 0 35.69830452860349 140.4572622262663 0 35.69830862404308 140.457254633737 0 35.6983010839191 140.4572486080514 0 35.69830517935783 140.4572410155217 0 35.698334080875156 140.4572645559663 0 35.6983778545275 140.45718379060992 0 35.698326064503114 140.45714172906048 0 35.69827419070759 140.45723756965225 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69827419070759 140.45723756965225 6.41392 35.698326064503114 140.45714172906048 6.41392 35.6983778545275 140.45718379060992 6.41392 35.698334080875156 140.4572645559663 6.41392 35.69830517935783 140.4572410155217 6.41392 35.6983010839191 140.4572486080514 6.41392 35.69830862404308 140.457254633737 6.41392 35.69830452860349 140.4572622262663 6.41392 35.69827419070759 140.45723756965225 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69827419070759 140.45723756965225 6.41392 35.69830452860349 140.4572622262663 6.41392 35.69830452860349 140.4572622262663 12.413920000000001 35.69827419070759 140.45723756965225 12.413920000000001 35.69827419070759 140.45723756965225 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69830452860349 140.4572622262663 6.41392 35.69830862404308 140.457254633737 6.41392 35.69830862404308 140.457254633737 12.413920000000001 35.69830452860349 140.4572622262663 12.413920000000001 35.69830452860349 140.4572622262663 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69830862404308 140.457254633737 6.41392 35.6983010839191 140.4572486080514 6.41392 35.6983010839191 140.4572486080514 12.413920000000001 35.69830862404308 140.457254633737 12.413920000000001 35.69830862404308 140.457254633737 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6983010839191 140.4572486080514 6.41392 35.69830517935783 140.4572410155217 6.41392 35.69830517935783 140.4572410155217 12.413920000000001 35.6983010839191 140.4572486080514 12.413920000000001 35.6983010839191 140.4572486080514 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69830517935783 140.4572410155217 6.41392 35.698334080875156 140.4572645559663 6.41392 35.698334080875156 140.4572645559663 12.413920000000001 35.69830517935783 140.4572410155217 12.413920000000001 35.69830517935783 140.4572410155217 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698334080875156 140.4572645559663 6.41392 35.6983778545275 140.45718379060992 6.41392 35.6983778545275 140.45718379060992 12.413920000000001 35.698334080875156 140.4572645559663 12.413920000000001 35.698334080875156 140.4572645559663 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6983778545275 140.45718379060992 6.41392 35.698326064503114 140.45714172906048 6.41392 35.698326064503114 140.45714172906048 12.413920000000001 35.6983778545275 140.45718379060992 12.413920000000001 35.6983778545275 140.45718379060992 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698326064503114 140.45714172906048 6.41392 35.69827419070759 140.45723756965225 6.41392 35.69827419070759 140.45723756965225 12.413920000000001 35.698326064503114 140.45714172906048 12.413920000000001 35.698326064503114 140.45714172906048 6.41392</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69827419070759 140.45723756965225 12.413920000000001 35.69830452860349 140.4572622262663 12.413920000000001 35.69830862404308 140.457254633737 12.413920000000001 35.6983010839191 140.4572486080514 12.413920000000001 35.69830517935783 140.4572410155217 12.413920000000001 35.698334080875156 140.4572645559663 12.413920000000001 35.6983778545275 140.45718379060992 12.413920000000001 35.698326064503114 140.45714172906048 12.413920000000001 35.69827419070759 140.45723756965225 12.413920000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">66.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8254</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_06db9d80-7404-484f-a36f-780870c4cb9e">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69804740785696 140.45904400329087 0 35.6980381868223 140.45906658347639 0 35.69792893091282 140.4589995413006 0 35.69789998989692 140.4590703657305 0 35.697910861349385 140.4590770805738 0 35.69790310136937 140.45909602568312 0 35.69795746034358 140.4591292684544 0 35.69796595114658 140.45910845057026 0 35.69800368748316 140.45913161862043 0 35.698001404322845 140.45913734670827 0 35.698052797560685 140.45916890889526 0 35.69807360491299 140.45911956806492 0 35.6981051422766 140.4591388202599 0 35.69812404881262 140.4590910115214 0 35.69804740785696 140.45904400329087 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69804740785696 140.45904400329087 6.17535 35.69812404881262 140.4590910115214 6.17535 35.6981051422766 140.4591388202599 6.17535 35.69807360491299 140.45911956806492 6.17535 35.698052797560685 140.45916890889526 6.17535 35.698001404322845 140.45913734670827 6.17535 35.69800368748316 140.45913161862043 6.17535 35.69796595114658 140.45910845057026 6.17535 35.69795746034358 140.4591292684544 6.17535 35.69790310136937 140.45909602568312 6.17535 35.697910861349385 140.4590770805738 6.17535 35.69789998989692 140.4590703657305 6.17535 35.69792893091282 140.4589995413006 6.17535 35.6980381868223 140.45906658347639 6.17535 35.69804740785696 140.45904400329087 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69804740785696 140.45904400329087 6.17535 35.6980381868223 140.45906658347639 6.17535 35.6980381868223 140.45906658347639 9.975349999999999 35.69804740785696 140.45904400329087 9.975349999999999 35.69804740785696 140.45904400329087 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6980381868223 140.45906658347639 6.17535 35.69792893091282 140.4589995413006 6.17535 35.69792893091282 140.4589995413006 9.975349999999999 35.6980381868223 140.45906658347639 9.975349999999999 35.6980381868223 140.45906658347639 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69792893091282 140.4589995413006 6.17535 35.69789998989692 140.4590703657305 6.17535 35.69789998989692 140.4590703657305 9.975349999999999 35.69792893091282 140.4589995413006 9.975349999999999 35.69792893091282 140.4589995413006 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69789998989692 140.4590703657305 6.17535 35.697910861349385 140.4590770805738 6.17535 35.697910861349385 140.4590770805738 9.975349999999999 35.69789998989692 140.4590703657305 9.975349999999999 35.69789998989692 140.4590703657305 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697910861349385 140.4590770805738 6.17535 35.69790310136937 140.45909602568312 6.17535 35.69790310136937 140.45909602568312 9.975349999999999 35.697910861349385 140.4590770805738 9.975349999999999 35.697910861349385 140.4590770805738 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69790310136937 140.45909602568312 6.17535 35.69795746034358 140.4591292684544 6.17535 35.69795746034358 140.4591292684544 9.975349999999999 35.69790310136937 140.45909602568312 9.975349999999999 35.69790310136937 140.45909602568312 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69795746034358 140.4591292684544 6.17535 35.69796595114658 140.45910845057026 6.17535 35.69796595114658 140.45910845057026 9.975349999999999 35.69795746034358 140.4591292684544 9.975349999999999 35.69795746034358 140.4591292684544 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69796595114658 140.45910845057026 6.17535 35.69800368748316 140.45913161862043 6.17535 35.69800368748316 140.45913161862043 9.975349999999999 35.69796595114658 140.45910845057026 9.975349999999999 35.69796595114658 140.45910845057026 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69800368748316 140.45913161862043 6.17535 35.698001404322845 140.45913734670827 6.17535 35.698001404322845 140.45913734670827 9.975349999999999 35.69800368748316 140.45913161862043 9.975349999999999 35.69800368748316 140.45913161862043 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698001404322845 140.45913734670827 6.17535 35.698052797560685 140.45916890889526 6.17535 35.698052797560685 140.45916890889526 9.975349999999999 35.698001404322845 140.45913734670827 9.975349999999999 35.698001404322845 140.45913734670827 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698052797560685 140.45916890889526 6.17535 35.69807360491299 140.45911956806492 6.17535 35.69807360491299 140.45911956806492 9.975349999999999 35.698052797560685 140.45916890889526 9.975349999999999 35.698052797560685 140.45916890889526 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69807360491299 140.45911956806492 6.17535 35.6981051422766 140.4591388202599 6.17535 35.6981051422766 140.4591388202599 9.975349999999999 35.69807360491299 140.45911956806492 9.975349999999999 35.69807360491299 140.45911956806492 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6981051422766 140.4591388202599 6.17535 35.69812404881262 140.4590910115214 6.17535 35.69812404881262 140.4590910115214 9.975349999999999 35.6981051422766 140.4591388202599 9.975349999999999 35.6981051422766 140.4591388202599 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69812404881262 140.4590910115214 6.17535 35.69804740785696 140.45904400329087 6.17535 35.69804740785696 140.45904400329087 9.975349999999999 35.69812404881262 140.4590910115214 9.975349999999999 35.69812404881262 140.4590910115214 6.17535</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69804740785696 140.45904400329087 9.975349999999999 35.6980381868223 140.45906658347639 9.975349999999999 35.69792893091282 140.4589995413006 9.975349999999999 35.69789998989692 140.4590703657305 9.975349999999999 35.697910861349385 140.4590770805738 9.975349999999999 35.69790310136937 140.45909602568312 9.975349999999999 35.69795746034358 140.4591292684544 9.975349999999999 35.69796595114658 140.45910845057026 9.975349999999999 35.69800368748316 140.45913161862043 9.975349999999999 35.698001404322845 140.45913734670827 9.975349999999999 35.698052797560685 140.45916890889526 9.975349999999999 35.69807360491299 140.45911956806492 9.975349999999999 35.6981051422766 140.4591388202599 9.975349999999999 35.69812404881262 140.4590910115214 9.975349999999999 35.69804740785696 140.45904400329087 9.975349999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">183.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8222</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_f33e32a5-72df-43b9-a6f9-0a8e7ec83552">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69907454100969 140.45651649276385 0 35.69906571180251 140.45649830204889 0 35.69904897339178 140.45651054781678 0 35.69905789272873 140.45652873923058 0 35.69907454100969 140.45651649276385 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69907454100969 140.45651649276385 5.31233 35.69905789272873 140.45652873923058 5.31233 35.69904897339178 140.45651054781678 5.31233 35.69906571180251 140.45649830204889 5.31233 35.69907454100969 140.45651649276385 5.31233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69907454100969 140.45651649276385 5.31233 35.69906571180251 140.45649830204889 5.31233 35.69906571180251 140.45649830204889 7.21233 35.69907454100969 140.45651649276385 7.21233 35.69907454100969 140.45651649276385 5.31233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906571180251 140.45649830204889 5.31233 35.69904897339178 140.45651054781678 5.31233 35.69904897339178 140.45651054781678 7.21233 35.69906571180251 140.45649830204889 7.21233 35.69906571180251 140.45649830204889 5.31233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69904897339178 140.45651054781678 5.31233 35.69905789272873 140.45652873923058 5.31233 35.69905789272873 140.45652873923058 7.21233 35.69904897339178 140.45651054781678 7.21233 35.69904897339178 140.45651054781678 5.31233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69905789272873 140.45652873923058 5.31233 35.69907454100969 140.45651649276385 5.31233 35.69907454100969 140.45651649276385 7.21233 35.69905789272873 140.45652873923058 7.21233 35.69905789272873 140.45652873923058 5.31233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69907454100969 140.45651649276385 7.21233 35.69906571180251 140.45649830204889 7.21233 35.69904897339178 140.45651054781678 7.21233 35.69905789272873 140.45652873923058 7.21233 35.69907454100969 140.45651649276385 7.21233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">4.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8334</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_f06dac3a-c511-48e0-850a-b5911c90132b">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69548084700412 140.4609409334565 0 35.69547405744268 140.46096386323916 0 35.69550409282181 140.46097724750587 0 35.695510882385854 140.46095431771582 0 35.69548084700412 140.4609409334565 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69548084700412 140.4609409334565 6.88668 35.695510882385854 140.46095431771582 6.88668 35.69550409282181 140.46097724750587 6.88668 35.69547405744268 140.46096386323916 6.88668 35.69548084700412 140.4609409334565 6.88668</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69548084700412 140.4609409334565 6.88668 35.69547405744268 140.46096386323916 6.88668 35.69547405744268 140.46096386323916 12.88668 35.69548084700412 140.4609409334565 12.88668 35.69548084700412 140.4609409334565 6.88668</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69547405744268 140.46096386323916 6.88668 35.69550409282181 140.46097724750587 6.88668 35.69550409282181 140.46097724750587 12.88668 35.69547405744268 140.46096386323916 12.88668 35.69547405744268 140.46096386323916 6.88668</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69550409282181 140.46097724750587 6.88668 35.695510882385854 140.46095431771582 6.88668 35.695510882385854 140.46095431771582 12.88668 35.69550409282181 140.46097724750587 12.88668 35.69550409282181 140.46097724750587 6.88668</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695510882385854 140.46095431771582 6.88668 35.69548084700412 140.4609409334565 6.88668 35.69548084700412 140.4609409334565 12.88668 35.695510882385854 140.46095431771582 12.88668 35.695510882385854 140.46095431771582 6.88668</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69548084700412 140.4609409334565 12.88668 35.69547405744268 140.46096386323916 12.88668 35.69550409282181 140.46097724750587 12.88668 35.695510882385854 140.46095431771582 12.88668 35.69548084700412 140.4609409334565 12.88668</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">3</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">7.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8187</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_ca8676ec-d931-46b0-9b21-0f72f2254963">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">8.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69813829004357 140.45857067348336 0 35.698096394455035 140.45865410446774 0 35.698187669601474 140.45872962482804 0 35.698158242486905 140.4588072967147 0 35.69821189484188 140.45883777140824 0 35.69824342542986 140.45875425946335 0 35.69823139545419 140.45874499411343 0 35.698269282839505 140.45866970866686 0 35.69813829004357 140.45857067348336 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69813829004357 140.45857067348336 6.05911 35.698269282839505 140.45866970866686 6.05911 35.69823139545419 140.45874499411343 6.05911 35.69824342542986 140.45875425946335 6.05911 35.69821189484188 140.45883777140824 6.05911 35.698158242486905 140.4588072967147 6.05911 35.698187669601474 140.45872962482804 6.05911 35.698096394455035 140.45865410446774 6.05911 35.69813829004357 140.45857067348336 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69813829004357 140.45857067348336 6.05911 35.698096394455035 140.45865410446774 6.05911 35.698096394455035 140.45865410446774 10.75911 35.69813829004357 140.45857067348336 10.75911 35.69813829004357 140.45857067348336 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698096394455035 140.45865410446774 6.05911 35.698187669601474 140.45872962482804 6.05911 35.698187669601474 140.45872962482804 10.75911 35.698096394455035 140.45865410446774 10.75911 35.698096394455035 140.45865410446774 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698187669601474 140.45872962482804 6.05911 35.698158242486905 140.4588072967147 6.05911 35.698158242486905 140.4588072967147 10.75911 35.698187669601474 140.45872962482804 10.75911 35.698187669601474 140.45872962482804 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698158242486905 140.4588072967147 6.05911 35.69821189484188 140.45883777140824 6.05911 35.69821189484188 140.45883777140824 10.75911 35.698158242486905 140.4588072967147 10.75911 35.698158242486905 140.4588072967147 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69821189484188 140.45883777140824 6.05911 35.69824342542986 140.45875425946335 6.05911 35.69824342542986 140.45875425946335 10.75911 35.69821189484188 140.45883777140824 10.75911 35.69821189484188 140.45883777140824 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69824342542986 140.45875425946335 6.05911 35.69823139545419 140.45874499411343 6.05911 35.69823139545419 140.45874499411343 10.75911 35.69824342542986 140.45875425946335 10.75911 35.69824342542986 140.45875425946335 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69823139545419 140.45874499411343 6.05911 35.698269282839505 140.45866970866686 6.05911 35.698269282839505 140.45866970866686 10.75911 35.69823139545419 140.45874499411343 10.75911 35.69823139545419 140.45874499411343 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698269282839505 140.45866970866686 6.05911 35.69813829004357 140.45857067348336 6.05911 35.69813829004357 140.45857067348336 10.75911 35.698269282839505 140.45866970866686 10.75911 35.698269282839505 140.45866970866686 6.05911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69813829004357 140.45857067348336 10.75911 35.698096394455035 140.45865410446774 10.75911 35.698187669601474 140.45872962482804 10.75911 35.698158242486905 140.4588072967147 10.75911 35.69821189484188 140.45883777140824 10.75911 35.69824342542986 140.45875425946335 10.75911 35.69823139545419 140.45874499411343 10.75911 35.698269282839505 140.45866970866686 10.75911 35.69813829004357 140.45857067348336 10.75911</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">207.4</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8212</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b28b3bf2-ca5a-40ff-ae8d-f84c6b380b23">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69851530419624 140.45744586063603 0 35.698464969272415 140.4575403872562 0 35.69852879036509 140.4575914933562 0 35.698523601864544 140.4576012873556 0 35.69855008179101 140.45762248863267 0 35.698554542364015 140.45761401494994 0 35.69859242257498 140.45764425555734 0 35.69864348550876 140.45754840853388 0 35.69851530419624 140.45744586063603 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851530419624 140.45744586063603 5.8423 35.69864348550876 140.45754840853388 5.8423 35.69859242257498 140.45764425555734 5.8423 35.698554542364015 140.45761401494994 5.8423 35.69855008179101 140.45762248863267 5.8423 35.698523601864544 140.4576012873556 5.8423 35.69852879036509 140.4575914933562 5.8423 35.698464969272415 140.4575403872562 5.8423 35.69851530419624 140.45744586063603 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851530419624 140.45744586063603 5.8423 35.698464969272415 140.4575403872562 5.8423 35.698464969272415 140.4575403872562 11.8423 35.69851530419624 140.45744586063603 11.8423 35.69851530419624 140.45744586063603 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698464969272415 140.4575403872562 5.8423 35.69852879036509 140.4575914933562 5.8423 35.69852879036509 140.4575914933562 11.8423 35.698464969272415 140.4575403872562 11.8423 35.698464969272415 140.4575403872562 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69852879036509 140.4575914933562 5.8423 35.698523601864544 140.4576012873556 5.8423 35.698523601864544 140.4576012873556 11.8423 35.69852879036509 140.4575914933562 11.8423 35.69852879036509 140.4575914933562 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698523601864544 140.4576012873556 5.8423 35.69855008179101 140.45762248863267 5.8423 35.69855008179101 140.45762248863267 11.8423 35.698523601864544 140.4576012873556 11.8423 35.698523601864544 140.4576012873556 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69855008179101 140.45762248863267 5.8423 35.698554542364015 140.45761401494994 5.8423 35.698554542364015 140.45761401494994 11.8423 35.69855008179101 140.45762248863267 11.8423 35.69855008179101 140.45762248863267 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698554542364015 140.45761401494994 5.8423 35.69859242257498 140.45764425555734 5.8423 35.69859242257498 140.45764425555734 11.8423 35.698554542364015 140.45761401494994 11.8423 35.698554542364015 140.45761401494994 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69859242257498 140.45764425555734 5.8423 35.69864348550876 140.45754840853388 5.8423 35.69864348550876 140.45754840853388 11.8423 35.69859242257498 140.45764425555734 11.8423 35.69859242257498 140.45764425555734 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69864348550876 140.45754840853388 5.8423 35.69851530419624 140.45744586063603 5.8423 35.69851530419624 140.45744586063603 11.8423 35.69864348550876 140.45754840853388 11.8423 35.69864348550876 140.45754840853388 5.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851530419624 140.45744586063603 11.8423 35.698464969272415 140.4575403872562 11.8423 35.69852879036509 140.4575914933562 11.8423 35.698523601864544 140.4576012873556 11.8423 35.69855008179101 140.45762248863267 11.8423 35.698554542364015 140.45761401494994 11.8423 35.69859242257498 140.45764425555734 11.8423 35.69864348550876 140.45754840853388 11.8423 35.69851530419624 140.45744586063603 11.8423</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">178</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8253</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_13e10aef-35f4-4e1f-b43e-e146af486e12">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69813990622365 140.45820736605896 0 35.69809600749211 140.45819055925256 0 35.69808579352149 140.45823081159986 0 35.6981000969041 140.45823622713752 0 35.69809568052763 140.45825354098125 0 35.69812518631124 140.45826482107785 0 35.69813990622365 140.45820736605896 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69813990622365 140.45820736605896 12.23622 35.69812518631124 140.45826482107785 12.23622 35.69809568052763 140.45825354098125 12.23622 35.6981000969041 140.45823622713752 12.23622 35.69808579352149 140.45823081159986 12.23622 35.69809600749211 140.45819055925256 12.23622 35.69813990622365 140.45820736605896 12.23622</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69813990622365 140.45820736605896 12.23622 35.69809600749211 140.45819055925256 12.23622 35.69809600749211 140.45819055925256 15.23622 35.69813990622365 140.45820736605896 15.23622 35.69813990622365 140.45820736605896 12.23622</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69809600749211 140.45819055925256 12.23622 35.69808579352149 140.45823081159986 12.23622 35.69808579352149 140.45823081159986 15.23622 35.69809600749211 140.45819055925256 15.23622 35.69809600749211 140.45819055925256 12.23622</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69808579352149 140.45823081159986 12.23622 35.6981000969041 140.45823622713752 12.23622 35.6981000969041 140.45823622713752 15.23622 35.69808579352149 140.45823081159986 15.23622 35.69808579352149 140.45823081159986 12.23622</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6981000969041 140.45823622713752 12.23622 35.69809568052763 140.45825354098125 12.23622 35.69809568052763 140.45825354098125 15.23622 35.6981000969041 140.45823622713752 15.23622 35.6981000969041 140.45823622713752 12.23622</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69809568052763 140.45825354098125 12.23622 35.69812518631124 140.45826482107785 12.23622 35.69812518631124 140.45826482107785 15.23622 35.69809568052763 140.45825354098125 15.23622 35.69809568052763 140.45825354098125 12.23622</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69812518631124 140.45826482107785 12.23622 35.69813990622365 140.45820736605896 12.23622 35.69813990622365 140.45820736605896 15.23622 35.69812518631124 140.45826482107785 15.23622 35.69812518631124 140.45826482107785 12.23622</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69813990622365 140.45820736605896 15.23622 35.69809600749211 140.45819055925256 15.23622 35.69808579352149 140.45823081159986 15.23622 35.6981000969041 140.45823622713752 15.23622 35.69809568052763 140.45825354098125 15.23622 35.69812518631124 140.45826482107785 15.23622 35.69813990622365 140.45820736605896 15.23622</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">25.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8220</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cef6e146-64d2-4c5f-a687-34e9da99acd2">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698259990277116 140.45823747467566 0 35.69822111515531 140.45831208961903 0 35.698268514137084 140.45834914514555 0 35.69827552483571 140.4583356084803 0 35.69829132487747 140.4583478866677 0 35.69832309919826 140.45828680765385 0 35.698259990277116 140.45823747467566 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698259990277116 140.45823747467566 5.85343 35.69832309919826 140.45828680765385 5.85343 35.69829132487747 140.4583478866677 5.85343 35.69827552483571 140.4583356084803 5.85343 35.698268514137084 140.45834914514555 5.85343 35.69822111515531 140.45831208961903 5.85343 35.698259990277116 140.45823747467566 5.85343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698259990277116 140.45823747467566 5.85343 35.69822111515531 140.45831208961903 5.85343 35.69822111515531 140.45831208961903 10.15343 35.698259990277116 140.45823747467566 10.15343 35.698259990277116 140.45823747467566 5.85343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69822111515531 140.45831208961903 5.85343 35.698268514137084 140.45834914514555 5.85343 35.698268514137084 140.45834914514555 10.15343 35.69822111515531 140.45831208961903 10.15343 35.69822111515531 140.45831208961903 5.85343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698268514137084 140.45834914514555 5.85343 35.69827552483571 140.4583356084803 5.85343 35.69827552483571 140.4583356084803 10.15343 35.698268514137084 140.45834914514555 10.15343 35.698268514137084 140.45834914514555 5.85343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69827552483571 140.4583356084803 5.85343 35.69829132487747 140.4583478866677 5.85343 35.69829132487747 140.4583478866677 10.15343 35.69827552483571 140.4583356084803 10.15343 35.69827552483571 140.4583356084803 5.85343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69829132487747 140.4583478866677 5.85343 35.69832309919826 140.45828680765385 5.85343 35.69832309919826 140.45828680765385 10.15343 35.69829132487747 140.4583478866677 10.15343 35.69829132487747 140.4583478866677 5.85343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69832309919826 140.45828680765385 5.85343 35.698259990277116 140.45823747467566 5.85343 35.698259990277116 140.45823747467566 10.15343 35.69832309919826 140.45828680765385 10.15343 35.69832309919826 140.45828680765385 5.85343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698259990277116 140.45823747467566 10.15343 35.69822111515531 140.45831208961903 10.15343 35.698268514137084 140.45834914514555 10.15343 35.69827552483571 140.4583356084803 10.15343 35.69829132487747 140.4583478866677 10.15343 35.69832309919826 140.45828680765385 10.15343 35.698259990277116 140.45823747467566 10.15343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">63.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8208</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_8dbcb229-5f73-4441-9837-d82ac6953308">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.699837227323364 140.45567013200846 0 35.699879255357466 140.45575245049855 0 35.699943597156526 140.45570300396076 0 35.6999015685182 140.45562079592625 0 35.699837227323364 140.45567013200846 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699837227323364 140.45567013200846 5.6597 35.6999015685182 140.45562079592625 5.6597 35.699943597156526 140.45570300396076 5.6597 35.699879255357466 140.45575245049855 5.6597 35.699837227323364 140.45567013200846 5.6597</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699837227323364 140.45567013200846 5.6597 35.699879255357466 140.45575245049855 5.6597 35.699879255357466 140.45575245049855 8.1597 35.699837227323364 140.45567013200846 8.1597 35.699837227323364 140.45567013200846 5.6597</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699879255357466 140.45575245049855 5.6597 35.699943597156526 140.45570300396076 5.6597 35.699943597156526 140.45570300396076 8.1597 35.699879255357466 140.45575245049855 8.1597 35.699879255357466 140.45575245049855 5.6597</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699943597156526 140.45570300396076 5.6597 35.6999015685182 140.45562079592625 5.6597 35.6999015685182 140.45562079592625 8.1597 35.699943597156526 140.45570300396076 8.1597 35.699943597156526 140.45570300396076 5.6597</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6999015685182 140.45562079592625 5.6597 35.699837227323364 140.45567013200846 5.6597 35.699837227323364 140.45567013200846 8.1597 35.6999015685182 140.45562079592625 8.1597 35.6999015685182 140.45562079592625 5.6597</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699837227323364 140.45567013200846 8.1597 35.699879255357466 140.45575245049855 8.1597 35.699943597156526 140.45570300396076 8.1597 35.6999015685182 140.45562079592625 8.1597 35.699837227323364 140.45567013200846 8.1597</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">74</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8295</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_01a78213-de1d-4928-b37f-400f91e170dc">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69999023105615 140.45555695204928 0 35.69997323697341 140.4555140559955 0 35.69996292998902 140.45552016399688 0 35.69995563344135 140.45550187459276 0 35.699901656970184 140.45553394263425 0 35.69990913034909 140.4555528964102 0 35.69990479089606 140.45555540422447 0 35.699921517426034 140.4555977456648 0 35.69999023105615 140.45555695204928 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69999023105615 140.45555695204928 5.77897 35.699921517426034 140.4555977456648 5.77897 35.69990479089606 140.45555540422447 5.77897 35.69990913034909 140.4555528964102 5.77897 35.699901656970184 140.45553394263425 5.77897 35.69995563344135 140.45550187459276 5.77897 35.69996292998902 140.45552016399688 5.77897 35.69997323697341 140.4555140559955 5.77897 35.69999023105615 140.45555695204928 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69999023105615 140.45555695204928 5.77897 35.69997323697341 140.4555140559955 5.77897 35.69997323697341 140.4555140559955 8.17897 35.69999023105615 140.45555695204928 8.17897 35.69999023105615 140.45555695204928 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69997323697341 140.4555140559955 5.77897 35.69996292998902 140.45552016399688 5.77897 35.69996292998902 140.45552016399688 8.17897 35.69997323697341 140.4555140559955 8.17897 35.69997323697341 140.4555140559955 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69996292998902 140.45552016399688 5.77897 35.69995563344135 140.45550187459276 5.77897 35.69995563344135 140.45550187459276 8.17897 35.69996292998902 140.45552016399688 8.17897 35.69996292998902 140.45552016399688 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69995563344135 140.45550187459276 5.77897 35.699901656970184 140.45553394263425 5.77897 35.699901656970184 140.45553394263425 8.17897 35.69995563344135 140.45550187459276 8.17897 35.69995563344135 140.45550187459276 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699901656970184 140.45553394263425 5.77897 35.69990913034909 140.4555528964102 5.77897 35.69990913034909 140.4555528964102 8.17897 35.699901656970184 140.45553394263425 8.17897 35.699901656970184 140.45553394263425 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69990913034909 140.4555528964102 5.77897 35.69990479089606 140.45555540422447 5.77897 35.69990479089606 140.45555540422447 8.17897 35.69990913034909 140.4555528964102 8.17897 35.69990913034909 140.4555528964102 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69990479089606 140.45555540422447 5.77897 35.699921517426034 140.4555977456648 5.77897 35.699921517426034 140.4555977456648 8.17897 35.69990479089606 140.45555540422447 8.17897 35.69990479089606 140.45555540422447 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699921517426034 140.4555977456648 5.77897 35.69999023105615 140.45555695204928 5.77897 35.69999023105615 140.45555695204928 8.17897 35.699921517426034 140.4555977456648 8.17897 35.699921517426034 140.4555977456648 5.77897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69999023105615 140.45555695204928 8.17897 35.69997323697341 140.4555140559955 8.17897 35.69996292998902 140.45552016399688 8.17897 35.69995563344135 140.45550187459276 8.17897 35.699901656970184 140.45553394263425 8.17897 35.69990913034909 140.4555528964102 8.17897 35.69990479089606 140.45555540422447 8.17897 35.699921517426034 140.4555977456648 8.17897 35.69999023105615 140.45555695204928 8.17897</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">48.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8302</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_2533c79b-ea3f-4332-bb0c-ac5abd74cfc1">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69923731103731 140.45731954961994 0 35.69920275416151 140.45729154481415 0 35.69914769126791 140.4573939915108 0 35.699182338251475 140.45742199704313 0 35.69923731103731 140.45731954961994 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69923731103731 140.45731954961994 4.50461 35.699182338251475 140.45742199704313 4.50461 35.69914769126791 140.4573939915108 4.50461 35.69920275416151 140.45729154481415 4.50461 35.69923731103731 140.45731954961994 4.50461</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69923731103731 140.45731954961994 4.50461 35.69920275416151 140.45729154481415 4.50461 35.69920275416151 140.45729154481415 8.204609999999999 35.69923731103731 140.45731954961994 8.204609999999999 35.69923731103731 140.45731954961994 4.50461</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69920275416151 140.45729154481415 4.50461 35.69914769126791 140.4573939915108 4.50461 35.69914769126791 140.4573939915108 8.204609999999999 35.69920275416151 140.45729154481415 8.204609999999999 35.69920275416151 140.45729154481415 4.50461</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69914769126791 140.4573939915108 4.50461 35.699182338251475 140.45742199704313 4.50461 35.699182338251475 140.45742199704313 8.204609999999999 35.69914769126791 140.4573939915108 8.204609999999999 35.69914769126791 140.4573939915108 4.50461</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699182338251475 140.45742199704313 4.50461 35.69923731103731 140.45731954961994 4.50461 35.69923731103731 140.45731954961994 8.204609999999999 35.699182338251475 140.45742199704313 8.204609999999999 35.699182338251475 140.45742199704313 4.50461</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69923731103731 140.45731954961994 8.204609999999999 35.69920275416151 140.45729154481415 8.204609999999999 35.69914769126791 140.4573939915108 8.204609999999999 35.699182338251475 140.45742199704313 8.204609999999999 35.69923731103731 140.45731954961994 8.204609999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">51.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">201</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-9093</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_8579a052-feec-4d82-a570-c39beffe467d">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69921385844833 140.45624397808737 0 35.699200001845306 140.45622188076018 0 35.69917862558143 140.4562420465524 0 35.69919248275266 140.45626403338125 0 35.69921385844833 140.45624397808737 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69921385844833 140.45624397808737 5.50195 35.69919248275266 140.45626403338125 5.50195 35.69917862558143 140.4562420465524 5.50195 35.699200001845306 140.45622188076018 5.50195 35.69921385844833 140.45624397808737 5.50195</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69921385844833 140.45624397808737 5.50195 35.699200001845306 140.45622188076018 5.50195 35.699200001845306 140.45622188076018 8.00195 35.69921385844833 140.45624397808737 8.00195 35.69921385844833 140.45624397808737 5.50195</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699200001845306 140.45622188076018 5.50195 35.69917862558143 140.4562420465524 5.50195 35.69917862558143 140.4562420465524 8.00195 35.699200001845306 140.45622188076018 8.00195 35.699200001845306 140.45622188076018 5.50195</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69917862558143 140.4562420465524 5.50195 35.69919248275266 140.45626403338125 5.50195 35.69919248275266 140.45626403338125 8.00195 35.69917862558143 140.4562420465524 8.00195 35.69917862558143 140.4562420465524 5.50195</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69919248275266 140.45626403338125 5.50195 35.69921385844833 140.45624397808737 5.50195 35.69921385844833 140.45624397808737 8.00195 35.69919248275266 140.45626403338125 8.00195 35.69919248275266 140.45626403338125 5.50195</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69921385844833 140.45624397808737 8.00195 35.699200001845306 140.45622188076018 8.00195 35.69917862558143 140.4562420465524 8.00195 35.69919248275266 140.45626403338125 8.00195 35.69921385844833 140.45624397808737 8.00195</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">7.5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8333</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_481673bc-14d7-4c3d-b706-aac792c03321">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69995260073909 140.45568749324374 0 35.700000373025205 140.45565250404002 0 35.69997022788945 140.45559049954622 0 35.69992245619216 140.4556253782775 0 35.69995260073909 140.45568749324374 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69995260073909 140.45568749324374 5.62233 35.69992245619216 140.4556253782775 5.62233 35.69997022788945 140.45559049954622 5.62233 35.700000373025205 140.45565250404002 5.62233 35.69995260073909 140.45568749324374 5.62233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69995260073909 140.45568749324374 5.62233 35.700000373025205 140.45565250404002 5.62233 35.700000373025205 140.45565250404002 9.62233 35.69995260073909 140.45568749324374 9.62233 35.69995260073909 140.45568749324374 5.62233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.700000373025205 140.45565250404002 5.62233 35.69997022788945 140.45559049954622 5.62233 35.69997022788945 140.45559049954622 9.62233 35.700000373025205 140.45565250404002 9.62233 35.700000373025205 140.45565250404002 5.62233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69997022788945 140.45559049954622 5.62233 35.69992245619216 140.4556253782775 5.62233 35.69992245619216 140.4556253782775 9.62233 35.69997022788945 140.45559049954622 9.62233 35.69997022788945 140.45559049954622 5.62233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69992245619216 140.4556253782775 5.62233 35.69995260073909 140.45568749324374 5.62233 35.69995260073909 140.45568749324374 9.62233 35.69992245619216 140.4556253782775 9.62233 35.69992245619216 140.4556253782775 5.62233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69995260073909 140.45568749324374 9.62233 35.700000373025205 140.45565250404002 9.62233 35.69997022788945 140.45559049954622 9.62233 35.69992245619216 140.4556253782775 9.62233 35.69995260073909 140.45568749324374 9.62233</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">40.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8294</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_a0403eb3-e561-4501-8d30-a67a868fbc7e">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">7.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69883572313901 140.45680458590238 0 35.69876560238973 140.45690813072892 0 35.69880155681399 140.45694476507387 0 35.69879292848186 140.45695740535686 0 35.698826192683775 140.45699136679372 0 35.698837182759725 140.45697520890707 0 35.698863005539984 140.45700148793622 0 35.698898247373506 140.45694949593764 0 35.698908916927444 140.45696040800252 0 35.698934803065725 140.45692226610103 0 35.698885757543145 140.4568723803013 0 35.698892479366364 140.45686237714798 0 35.69883572313901 140.45680458590238 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69883572313901 140.45680458590238 5.88765 35.698892479366364 140.45686237714798 5.88765 35.698885757543145 140.4568723803013 5.88765 35.698934803065725 140.45692226610103 5.88765 35.698908916927444 140.45696040800252 5.88765 35.698898247373506 140.45694949593764 5.88765 35.698863005539984 140.45700148793622 5.88765 35.698837182759725 140.45697520890707 5.88765 35.698826192683775 140.45699136679372 5.88765 35.69879292848186 140.45695740535686 5.88765 35.69880155681399 140.45694476507387 5.88765 35.69876560238973 140.45690813072892 5.88765 35.69883572313901 140.45680458590238 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69883572313901 140.45680458590238 5.88765 35.69876560238973 140.45690813072892 5.88765 35.69876560238973 140.45690813072892 10.68765 35.69883572313901 140.45680458590238 10.68765 35.69883572313901 140.45680458590238 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69876560238973 140.45690813072892 5.88765 35.69880155681399 140.45694476507387 5.88765 35.69880155681399 140.45694476507387 10.68765 35.69876560238973 140.45690813072892 10.68765 35.69876560238973 140.45690813072892 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69880155681399 140.45694476507387 5.88765 35.69879292848186 140.45695740535686 5.88765 35.69879292848186 140.45695740535686 10.68765 35.69880155681399 140.45694476507387 10.68765 35.69880155681399 140.45694476507387 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69879292848186 140.45695740535686 5.88765 35.698826192683775 140.45699136679372 5.88765 35.698826192683775 140.45699136679372 10.68765 35.69879292848186 140.45695740535686 10.68765 35.69879292848186 140.45695740535686 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698826192683775 140.45699136679372 5.88765 35.698837182759725 140.45697520890707 5.88765 35.698837182759725 140.45697520890707 10.68765 35.698826192683775 140.45699136679372 10.68765 35.698826192683775 140.45699136679372 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698837182759725 140.45697520890707 5.88765 35.698863005539984 140.45700148793622 5.88765 35.698863005539984 140.45700148793622 10.68765 35.698837182759725 140.45697520890707 10.68765 35.698837182759725 140.45697520890707 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698863005539984 140.45700148793622 5.88765 35.698898247373506 140.45694949593764 5.88765 35.698898247373506 140.45694949593764 10.68765 35.698863005539984 140.45700148793622 10.68765 35.698863005539984 140.45700148793622 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698898247373506 140.45694949593764 5.88765 35.698908916927444 140.45696040800252 5.88765 35.698908916927444 140.45696040800252 10.68765 35.698898247373506 140.45694949593764 10.68765 35.698898247373506 140.45694949593764 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698908916927444 140.45696040800252 5.88765 35.698934803065725 140.45692226610103 5.88765 35.698934803065725 140.45692226610103 10.68765 35.698908916927444 140.45696040800252 10.68765 35.698908916927444 140.45696040800252 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698934803065725 140.45692226610103 5.88765 35.698885757543145 140.4568723803013 5.88765 35.698885757543145 140.4568723803013 10.68765 35.698934803065725 140.45692226610103 10.68765 35.698934803065725 140.45692226610103 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698885757543145 140.4568723803013 5.88765 35.698892479366364 140.45686237714798 5.88765 35.698892479366364 140.45686237714798 10.68765 35.698885757543145 140.4568723803013 10.68765 35.698885757543145 140.4568723803013 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698892479366364 140.45686237714798 5.88765 35.69883572313901 140.45680458590238 5.88765 35.69883572313901 140.45680458590238 10.68765 35.698892479366364 140.45686237714798 10.68765 35.698892479366364 140.45686237714798 5.88765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69883572313901 140.45680458590238 10.68765 35.69876560238973 140.45690813072892 10.68765 35.69880155681399 140.45694476507387 10.68765 35.69879292848186 140.45695740535686 10.68765 35.698826192683775 140.45699136679372 10.68765 35.698837182759725 140.45697520890707 10.68765 35.698863005539984 140.45700148793622 10.68765 35.698898247373506 140.45694949593764 10.68765 35.698908916927444 140.45696040800252 10.68765 35.698934803065725 140.45692226610103 10.68765 35.698885757543145 140.4568723803013 10.68765 35.698892479366364 140.45686237714798 10.68765 35.69883572313901 140.45680458590238 10.68765</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">173.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8234</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_308ac88d-5fb5-4395-9b84-4e95b55b6a24">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69854501003224 140.45795726215727 0 35.69848520614605 140.45793104942064 0 35.69845795025604 140.4580246507162 0 35.69851775412149 140.45805086351317 0 35.69854501003224 140.45795726215727 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69854501003224 140.45795726215727 5.87308 35.69851775412149 140.45805086351317 5.87308 35.69845795025604 140.4580246507162 5.87308 35.69848520614605 140.45793104942064 5.87308 35.69854501003224 140.45795726215727 5.87308</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69854501003224 140.45795726215727 5.87308 35.69848520614605 140.45793104942064 5.87308 35.69848520614605 140.45793104942064 8.87308 35.69854501003224 140.45795726215727 8.87308 35.69854501003224 140.45795726215727 5.87308</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69848520614605 140.45793104942064 5.87308 35.69845795025604 140.4580246507162 5.87308 35.69845795025604 140.4580246507162 8.87308 35.69848520614605 140.45793104942064 8.87308 35.69848520614605 140.45793104942064 5.87308</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69845795025604 140.4580246507162 5.87308 35.69851775412149 140.45805086351317 5.87308 35.69851775412149 140.45805086351317 8.87308 35.69845795025604 140.4580246507162 8.87308 35.69845795025604 140.4580246507162 5.87308</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851775412149 140.45805086351317 5.87308 35.69854501003224 140.45795726215727 5.87308 35.69854501003224 140.45795726215727 8.87308 35.69851775412149 140.45805086351317 8.87308 35.69851775412149 140.45805086351317 5.87308</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69854501003224 140.45795726215727 8.87308 35.69848520614605 140.45793104942064 8.87308 35.69845795025604 140.4580246507162 8.87308 35.69851775412149 140.45805086351317 8.87308 35.69854501003224 140.45795726215727 8.87308</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">63.4</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8192</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_eba9c944-6069-4b1e-ab49-27a79324e588">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.699510952427765 140.456015894854 0 35.69942368240091 140.45605444404129 0 35.699447066663964 140.45613396530553 0 35.69944011052484 140.45613700522864 0 35.69945472494636 140.45618684415842 0 35.699471619001635 140.45617935100017 0 35.69948419640316 140.4562222125722 0 35.6995032587643 140.45621374177642 0 35.69950875085796 140.45623234860832 0 35.69954163537041 140.45621779727344 0 35.6995369418794 140.45620162766204 0 35.69955473953819 140.45619369952396 0 35.69954145193751 140.4561487328813 0 35.699549040713 140.456145366373 0 35.699510952427765 140.456015894854 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699510952427765 140.456015894854 5.59359 35.699549040713 140.456145366373 5.59359 35.69954145193751 140.4561487328813 5.59359 35.69955473953819 140.45619369952396 5.59359 35.6995369418794 140.45620162766204 5.59359 35.69954163537041 140.45621779727344 5.59359 35.69950875085796 140.45623234860832 5.59359 35.6995032587643 140.45621374177642 5.59359 35.69948419640316 140.4562222125722 5.59359 35.699471619001635 140.45617935100017 5.59359 35.69945472494636 140.45618684415842 5.59359 35.69944011052484 140.45613700522864 5.59359 35.699447066663964 140.45613396530553 5.59359 35.69942368240091 140.45605444404129 5.59359 35.699510952427765 140.456015894854 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699510952427765 140.456015894854 5.59359 35.69942368240091 140.45605444404129 5.59359 35.69942368240091 140.45605444404129 9.593589999999999 35.699510952427765 140.456015894854 9.593589999999999 35.699510952427765 140.456015894854 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69942368240091 140.45605444404129 5.59359 35.699447066663964 140.45613396530553 5.59359 35.699447066663964 140.45613396530553 9.593589999999999 35.69942368240091 140.45605444404129 9.593589999999999 35.69942368240091 140.45605444404129 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699447066663964 140.45613396530553 5.59359 35.69944011052484 140.45613700522864 5.59359 35.69944011052484 140.45613700522864 9.593589999999999 35.699447066663964 140.45613396530553 9.593589999999999 35.699447066663964 140.45613396530553 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69944011052484 140.45613700522864 5.59359 35.69945472494636 140.45618684415842 5.59359 35.69945472494636 140.45618684415842 9.593589999999999 35.69944011052484 140.45613700522864 9.593589999999999 35.69944011052484 140.45613700522864 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69945472494636 140.45618684415842 5.59359 35.699471619001635 140.45617935100017 5.59359 35.699471619001635 140.45617935100017 9.593589999999999 35.69945472494636 140.45618684415842 9.593589999999999 35.69945472494636 140.45618684415842 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699471619001635 140.45617935100017 5.59359 35.69948419640316 140.4562222125722 5.59359 35.69948419640316 140.4562222125722 9.593589999999999 35.699471619001635 140.45617935100017 9.593589999999999 35.699471619001635 140.45617935100017 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69948419640316 140.4562222125722 5.59359 35.6995032587643 140.45621374177642 5.59359 35.6995032587643 140.45621374177642 9.593589999999999 35.69948419640316 140.4562222125722 9.593589999999999 35.69948419640316 140.4562222125722 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6995032587643 140.45621374177642 5.59359 35.69950875085796 140.45623234860832 5.59359 35.69950875085796 140.45623234860832 9.593589999999999 35.6995032587643 140.45621374177642 9.593589999999999 35.6995032587643 140.45621374177642 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69950875085796 140.45623234860832 5.59359 35.69954163537041 140.45621779727344 5.59359 35.69954163537041 140.45621779727344 9.593589999999999 35.69950875085796 140.45623234860832 9.593589999999999 35.69950875085796 140.45623234860832 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69954163537041 140.45621779727344 5.59359 35.6995369418794 140.45620162766204 5.59359 35.6995369418794 140.45620162766204 9.593589999999999 35.69954163537041 140.45621779727344 9.593589999999999 35.69954163537041 140.45621779727344 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6995369418794 140.45620162766204 5.59359 35.69955473953819 140.45619369952396 5.59359 35.69955473953819 140.45619369952396 9.593589999999999 35.6995369418794 140.45620162766204 9.593589999999999 35.6995369418794 140.45620162766204 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69955473953819 140.45619369952396 5.59359 35.69954145193751 140.4561487328813 5.59359 35.69954145193751 140.4561487328813 9.593589999999999 35.69955473953819 140.45619369952396 9.593589999999999 35.69955473953819 140.45619369952396 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69954145193751 140.4561487328813 5.59359 35.699549040713 140.456145366373 5.59359 35.699549040713 140.456145366373 9.593589999999999 35.69954145193751 140.4561487328813 9.593589999999999 35.69954145193751 140.4561487328813 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699549040713 140.456145366373 5.59359 35.699510952427765 140.456015894854 5.59359 35.699510952427765 140.456015894854 9.593589999999999 35.699549040713 140.456145366373 9.593589999999999 35.699549040713 140.456145366373 5.59359</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699510952427765 140.456015894854 9.593589999999999 35.69942368240091 140.45605444404129 9.593589999999999 35.699447066663964 140.45613396530553 9.593589999999999 35.69944011052484 140.45613700522864 9.593589999999999 35.69945472494636 140.45618684415842 9.593589999999999 35.699471619001635 140.45617935100017 9.593589999999999 35.69948419640316 140.4562222125722 9.593589999999999 35.6995032587643 140.45621374177642 9.593589999999999 35.69950875085796 140.45623234860832 9.593589999999999 35.69954163537041 140.45621779727344 9.593589999999999 35.6995369418794 140.45620162766204 9.593589999999999 35.69955473953819 140.45619369952396 9.593589999999999 35.69954145193751 140.4561487328813 9.593589999999999 35.699549040713 140.456145366373 9.593589999999999 35.699510952427765 140.456015894854 9.593589999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">173.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8266</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_4e78f997-4f84-400c-be08-5e067d1e7ffe">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698362135154156 140.458111750173 0 35.69831753027287 140.45821361383386 0 35.69843637390131 140.45829166950776 0 35.69847094564171 140.45821260080962 0 35.69845298013506 140.45820074772283 0 35.698462923208396 140.4581779519548 0 35.698362135154156 140.458111750173 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698362135154156 140.458111750173 5.79352 35.698462923208396 140.4581779519548 5.79352 35.69845298013506 140.45820074772283 5.79352 35.69847094564171 140.45821260080962 5.79352 35.69843637390131 140.45829166950776 5.79352 35.69831753027287 140.45821361383386 5.79352 35.698362135154156 140.458111750173 5.79352</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698362135154156 140.458111750173 5.79352 35.69831753027287 140.45821361383386 5.79352 35.69831753027287 140.45821361383386 11.293520000000001 35.698362135154156 140.458111750173 11.293520000000001 35.698362135154156 140.458111750173 5.79352</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69831753027287 140.45821361383386 5.79352 35.69843637390131 140.45829166950776 5.79352 35.69843637390131 140.45829166950776 11.293520000000001 35.69831753027287 140.45821361383386 11.293520000000001 35.69831753027287 140.45821361383386 5.79352</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69843637390131 140.45829166950776 5.79352 35.69847094564171 140.45821260080962 5.79352 35.69847094564171 140.45821260080962 11.293520000000001 35.69843637390131 140.45829166950776 11.293520000000001 35.69843637390131 140.45829166950776 5.79352</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69847094564171 140.45821260080962 5.79352 35.69845298013506 140.45820074772283 5.79352 35.69845298013506 140.45820074772283 11.293520000000001 35.69847094564171 140.45821260080962 11.293520000000001 35.69847094564171 140.45821260080962 5.79352</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69845298013506 140.45820074772283 5.79352 35.698462923208396 140.4581779519548 5.79352 35.698462923208396 140.4581779519548 11.293520000000001 35.69845298013506 140.45820074772283 11.293520000000001 35.69845298013506 140.45820074772283 5.79352</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698462923208396 140.4581779519548 5.79352 35.698362135154156 140.458111750173 5.79352 35.698362135154156 140.458111750173 11.293520000000001 35.698462923208396 140.4581779519548 11.293520000000001 35.698462923208396 140.4581779519548 5.79352</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698362135154156 140.458111750173 11.293520000000001 35.69831753027287 140.45821361383386 11.293520000000001 35.69843637390131 140.45829166950776 11.293520000000001 35.69847094564171 140.45821260080962 11.293520000000001 35.69845298013506 140.45820074772283 11.293520000000001 35.698462923208396 140.4581779519548 11.293520000000001 35.698362135154156 140.458111750173 11.293520000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">151.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8221</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_5ae48ed9-d56d-4e36-aa58-2a03b0cc6498">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">10.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.70000788109928 140.45568217675503 0 35.699930207264444 140.45573008322148 0 35.6999521053657 140.45578329403386 0 35.6999397173421 140.45579093284272 0 35.699960724958366 140.45584203723098 0 35.69997808622196 140.4558313430465 0 35.69998876828699 140.45585728339526 0 35.700071958482155 140.4558058838472 0 35.70004988355733 140.45575200856567 0 35.700039303880985 140.45575855642838 0 35.70000788109928 140.45568217675503 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70000788109928 140.45568217675503 5.63864 35.700039303880985 140.45575855642838 5.63864 35.70004988355733 140.45575200856567 5.63864 35.700071958482155 140.4558058838472 5.63864 35.69998876828699 140.45585728339526 5.63864 35.69997808622196 140.4558313430465 5.63864 35.699960724958366 140.45584203723098 5.63864 35.6999397173421 140.45579093284272 5.63864 35.6999521053657 140.45578329403386 5.63864 35.699930207264444 140.45573008322148 5.63864 35.70000788109928 140.45568217675503 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70000788109928 140.45568217675503 5.63864 35.699930207264444 140.45573008322148 5.63864 35.699930207264444 140.45573008322148 9.93864 35.70000788109928 140.45568217675503 9.93864 35.70000788109928 140.45568217675503 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699930207264444 140.45573008322148 5.63864 35.6999521053657 140.45578329403386 5.63864 35.6999521053657 140.45578329403386 9.93864 35.699930207264444 140.45573008322148 9.93864 35.699930207264444 140.45573008322148 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6999521053657 140.45578329403386 5.63864 35.6999397173421 140.45579093284272 5.63864 35.6999397173421 140.45579093284272 9.93864 35.6999521053657 140.45578329403386 9.93864 35.6999521053657 140.45578329403386 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6999397173421 140.45579093284272 5.63864 35.699960724958366 140.45584203723098 5.63864 35.699960724958366 140.45584203723098 9.93864 35.6999397173421 140.45579093284272 9.93864 35.6999397173421 140.45579093284272 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699960724958366 140.45584203723098 5.63864 35.69997808622196 140.4558313430465 5.63864 35.69997808622196 140.4558313430465 9.93864 35.699960724958366 140.45584203723098 9.93864 35.699960724958366 140.45584203723098 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69997808622196 140.4558313430465 5.63864 35.69998876828699 140.45585728339526 5.63864 35.69998876828699 140.45585728339526 9.93864 35.69997808622196 140.4558313430465 9.93864 35.69997808622196 140.4558313430465 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69998876828699 140.45585728339526 5.63864 35.700071958482155 140.4558058838472 5.63864 35.700071958482155 140.4558058838472 9.93864 35.69998876828699 140.45585728339526 9.93864 35.69998876828699 140.45585728339526 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.700071958482155 140.4558058838472 5.63864 35.70004988355733 140.45575200856567 5.63864 35.70004988355733 140.45575200856567 9.93864 35.700071958482155 140.4558058838472 9.93864 35.700071958482155 140.4558058838472 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70004988355733 140.45575200856567 5.63864 35.700039303880985 140.45575855642838 5.63864 35.700039303880985 140.45575855642838 9.93864 35.70004988355733 140.45575200856567 9.93864 35.70004988355733 140.45575200856567 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.700039303880985 140.45575855642838 5.63864 35.70000788109928 140.45568217675503 5.63864 35.70000788109928 140.45568217675503 9.93864 35.700039303880985 140.45575855642838 9.93864 35.700039303880985 140.45575855642838 5.63864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70000788109928 140.45568217675503 9.93864 35.699930207264444 140.45573008322148 9.93864 35.6999521053657 140.45578329403386 9.93864 35.6999397173421 140.45579093284272 9.93864 35.699960724958366 140.45584203723098 9.93864 35.69997808622196 140.4558313430465 9.93864 35.69998876828699 140.45585728339526 9.93864 35.700071958482155 140.4558058838472 9.93864 35.70004988355733 140.45575200856567 9.93864 35.700039303880985 140.45575855642838 9.93864 35.70000788109928 140.45568217675503 9.93864</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">140.9</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8292</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_fcb106e7-c1a9-4192-9afe-56ccb5c368fe">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69966567770665 140.4553256935876 0 35.699686370368546 140.4553156882075 0 35.69967900592442 140.45529308882138 0 35.6996584033959 140.45530309490644 0 35.69966567770665 140.4553256935876 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69966567770665 140.4553256935876 6.67708 35.6996584033959 140.45530309490644 6.67708 35.69967900592442 140.45529308882138 6.67708 35.699686370368546 140.4553156882075 6.67708 35.69966567770665 140.4553256935876 6.67708</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69966567770665 140.4553256935876 6.67708 35.699686370368546 140.4553156882075 6.67708 35.699686370368546 140.4553156882075 8.87708 35.69966567770665 140.4553256935876 8.87708 35.69966567770665 140.4553256935876 6.67708</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699686370368546 140.4553156882075 6.67708 35.69967900592442 140.45529308882138 6.67708 35.69967900592442 140.45529308882138 8.87708 35.699686370368546 140.4553156882075 8.87708 35.699686370368546 140.4553156882075 6.67708</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69967900592442 140.45529308882138 6.67708 35.6996584033959 140.45530309490644 6.67708 35.6996584033959 140.45530309490644 8.87708 35.69967900592442 140.45529308882138 8.87708 35.69967900592442 140.45529308882138 6.67708</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6996584033959 140.45530309490644 6.67708 35.69966567770665 140.4553256935876 6.67708 35.69966567770665 140.4553256935876 8.87708 35.6996584033959 140.45530309490644 8.87708 35.6996584033959 140.45530309490644 6.67708</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69966567770665 140.4553256935876 8.87708 35.699686370368546 140.4553156882075 8.87708 35.69967900592442 140.45529308882138 8.87708 35.6996584033959 140.45530309490644 8.87708 35.69966567770665 140.4553256935876 8.87708</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">5.4</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8242</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_5cf32c1c-3440-4958-a422-f87e9d39db3d">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">7.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.70003508831974 140.45542326232695 0 35.69999077325345 140.45531252723165 0 35.69992079100985 140.45535463722 0 35.69996510603865 140.45546537224067 0 35.70003508831974 140.45542326232695 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70003508831974 140.45542326232695 5.97476 35.69996510603865 140.45546537224067 5.97476 35.69992079100985 140.45535463722 5.97476 35.69999077325345 140.45531252723165 5.97476 35.70003508831974 140.45542326232695 5.97476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70003508831974 140.45542326232695 5.97476 35.69999077325345 140.45531252723165 5.97476 35.69999077325345 140.45531252723165 12.77476 35.70003508831974 140.45542326232695 12.77476 35.70003508831974 140.45542326232695 5.97476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69999077325345 140.45531252723165 5.97476 35.69992079100985 140.45535463722 5.97476 35.69992079100985 140.45535463722 12.77476 35.69999077325345 140.45531252723165 12.77476 35.69999077325345 140.45531252723165 5.97476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69992079100985 140.45535463722 5.97476 35.69996510603865 140.45546537224067 5.97476 35.69996510603865 140.45546537224067 12.77476 35.69992079100985 140.45535463722 12.77476 35.69992079100985 140.45535463722 5.97476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69996510603865 140.45546537224067 5.97476 35.70003508831974 140.45542326232695 5.97476 35.70003508831974 140.45542326232695 12.77476 35.69996510603865 140.45546537224067 12.77476 35.69996510603865 140.45546537224067 5.97476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70003508831974 140.45542326232695 12.77476 35.69999077325345 140.45531252723165 12.77476 35.69992079100985 140.45535463722 12.77476 35.69996510603865 140.45546537224067 12.77476 35.70003508831974 140.45542326232695 12.77476</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">96.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8306</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_850d7a91-e01f-463c-8584-13fc45054cb4">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">8.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69977971291108 140.45534039186674 0 35.6996674886402 140.45537642757628 0 35.69968799303297 140.45547261216186 0 35.69966740819495 140.4554791928105 0 35.699682192183815 140.45554859174865 0 35.699711534638965 140.45553920612997 0 35.69971620080129 140.45556067957378 0 35.69975087015918 140.45554956732505 0 35.69976433469422 140.45561265753605 0 35.69977309230928 140.45560985255494 0 35.69977819597745 140.45563387094364 0 35.69981656701982 140.4556215719686 0 35.69981102183122 140.45559578212075 0 35.69983268996752 140.4555888784043 0 35.69977971291108 140.45534039186674 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69977971291108 140.45534039186674 6.00619 35.69983268996752 140.4555888784043 6.00619 35.69981102183122 140.45559578212075 6.00619 35.69981656701982 140.4556215719686 6.00619 35.69977819597745 140.45563387094364 6.00619 35.69977309230928 140.45560985255494 6.00619 35.69976433469422 140.45561265753605 6.00619 35.69975087015918 140.45554956732505 6.00619 35.69971620080129 140.45556067957378 6.00619 35.699711534638965 140.45553920612997 6.00619 35.699682192183815 140.45554859174865 6.00619 35.69966740819495 140.4554791928105 6.00619 35.69968799303297 140.45547261216186 6.00619 35.6996674886402 140.45537642757628 6.00619 35.69977971291108 140.45534039186674 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69977971291108 140.45534039186674 6.00619 35.6996674886402 140.45537642757628 6.00619 35.6996674886402 140.45537642757628 11.70619 35.69977971291108 140.45534039186674 11.70619 35.69977971291108 140.45534039186674 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6996674886402 140.45537642757628 6.00619 35.69968799303297 140.45547261216186 6.00619 35.69968799303297 140.45547261216186 11.70619 35.6996674886402 140.45537642757628 11.70619 35.6996674886402 140.45537642757628 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69968799303297 140.45547261216186 6.00619 35.69966740819495 140.4554791928105 6.00619 35.69966740819495 140.4554791928105 11.70619 35.69968799303297 140.45547261216186 11.70619 35.69968799303297 140.45547261216186 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69966740819495 140.4554791928105 6.00619 35.699682192183815 140.45554859174865 6.00619 35.699682192183815 140.45554859174865 11.70619 35.69966740819495 140.4554791928105 11.70619 35.69966740819495 140.4554791928105 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699682192183815 140.45554859174865 6.00619 35.699711534638965 140.45553920612997 6.00619 35.699711534638965 140.45553920612997 11.70619 35.699682192183815 140.45554859174865 11.70619 35.699682192183815 140.45554859174865 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699711534638965 140.45553920612997 6.00619 35.69971620080129 140.45556067957378 6.00619 35.69971620080129 140.45556067957378 11.70619 35.699711534638965 140.45553920612997 11.70619 35.699711534638965 140.45553920612997 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69971620080129 140.45556067957378 6.00619 35.69975087015918 140.45554956732505 6.00619 35.69975087015918 140.45554956732505 11.70619 35.69971620080129 140.45556067957378 11.70619 35.69971620080129 140.45556067957378 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69975087015918 140.45554956732505 6.00619 35.69976433469422 140.45561265753605 6.00619 35.69976433469422 140.45561265753605 11.70619 35.69975087015918 140.45554956732505 11.70619 35.69975087015918 140.45554956732505 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69976433469422 140.45561265753605 6.00619 35.69977309230928 140.45560985255494 6.00619 35.69977309230928 140.45560985255494 11.70619 35.69976433469422 140.45561265753605 11.70619 35.69976433469422 140.45561265753605 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69977309230928 140.45560985255494 6.00619 35.69977819597745 140.45563387094364 6.00619 35.69977819597745 140.45563387094364 11.70619 35.69977309230928 140.45560985255494 11.70619 35.69977309230928 140.45560985255494 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69977819597745 140.45563387094364 6.00619 35.69981656701982 140.4556215719686 6.00619 35.69981656701982 140.4556215719686 11.70619 35.69977819597745 140.45563387094364 11.70619 35.69977819597745 140.45563387094364 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69981656701982 140.4556215719686 6.00619 35.69981102183122 140.45559578212075 6.00619 35.69981102183122 140.45559578212075 11.70619 35.69981656701982 140.4556215719686 11.70619 35.69981656701982 140.4556215719686 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69981102183122 140.45559578212075 6.00619 35.69983268996752 140.4555888784043 6.00619 35.69983268996752 140.4555888784043 11.70619 35.69981102183122 140.45559578212075 11.70619 35.69981102183122 140.45559578212075 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69983268996752 140.4555888784043 6.00619 35.69977971291108 140.45534039186674 6.00619 35.69977971291108 140.45534039186674 11.70619 35.69983268996752 140.4555888784043 11.70619 35.69983268996752 140.4555888784043 6.00619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69977971291108 140.45534039186674 11.70619 35.6996674886402 140.45537642757628 11.70619 35.69968799303297 140.45547261216186 11.70619 35.69966740819495 140.4554791928105 11.70619 35.699682192183815 140.45554859174865 11.70619 35.699711534638965 140.45553920612997 11.70619 35.69971620080129 140.45556067957378 11.70619 35.69975087015918 140.45554956732505 11.70619 35.69976433469422 140.45561265753605 11.70619 35.69977309230928 140.45560985255494 11.70619 35.69977819597745 140.45563387094364 11.70619 35.69981656701982 140.4556215719686 11.70619 35.69981102183122 140.45559578212075 11.70619 35.69983268996752 140.4555888784043 11.70619 35.69977971291108 140.45534039186674 11.70619</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">294.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8330</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_410c2ef5-b9af-40dc-8762-4ce3ecd317a0">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">6.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.6985995296282 140.45716010242487 0 35.69856136397819 140.45721969582033 0 35.69860602842342 140.4572626965308 0 35.69860157532076 140.4572697338041 0 35.698622831685064 140.45729012080187 0 35.698627284216116 140.4572831940229 0 35.69869490947165 140.45734814210934 0 35.69873307575759 140.45728843818247 0 35.6985995296282 140.45716010242487 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985995296282 140.45716010242487 5.52168 35.69873307575759 140.45728843818247 5.52168 35.69869490947165 140.45734814210934 5.52168 35.698627284216116 140.4572831940229 5.52168 35.698622831685064 140.45729012080187 5.52168 35.69860157532076 140.4572697338041 5.52168 35.69860602842342 140.4572626965308 5.52168 35.69856136397819 140.45721969582033 5.52168 35.6985995296282 140.45716010242487 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985995296282 140.45716010242487 5.52168 35.69856136397819 140.45721969582033 5.52168 35.69856136397819 140.45721969582033 8.82168 35.6985995296282 140.45716010242487 8.82168 35.6985995296282 140.45716010242487 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69856136397819 140.45721969582033 5.52168 35.69860602842342 140.4572626965308 5.52168 35.69860602842342 140.4572626965308 8.82168 35.69856136397819 140.45721969582033 8.82168 35.69856136397819 140.45721969582033 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69860602842342 140.4572626965308 5.52168 35.69860157532076 140.4572697338041 5.52168 35.69860157532076 140.4572697338041 8.82168 35.69860602842342 140.4572626965308 8.82168 35.69860602842342 140.4572626965308 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69860157532076 140.4572697338041 5.52168 35.698622831685064 140.45729012080187 5.52168 35.698622831685064 140.45729012080187 8.82168 35.69860157532076 140.4572697338041 8.82168 35.69860157532076 140.4572697338041 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698622831685064 140.45729012080187 5.52168 35.698627284216116 140.4572831940229 5.52168 35.698627284216116 140.4572831940229 8.82168 35.698622831685064 140.45729012080187 8.82168 35.698622831685064 140.45729012080187 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698627284216116 140.4572831940229 5.52168 35.69869490947165 140.45734814210934 5.52168 35.69869490947165 140.45734814210934 8.82168 35.698627284216116 140.4572831940229 8.82168 35.698627284216116 140.4572831940229 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69869490947165 140.45734814210934 5.52168 35.69873307575759 140.45728843818247 5.52168 35.69873307575759 140.45728843818247 8.82168 35.69869490947165 140.45734814210934 8.82168 35.69869490947165 140.45734814210934 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69873307575759 140.45728843818247 5.52168 35.6985995296282 140.45716010242487 5.52168 35.6985995296282 140.45716010242487 8.82168 35.69873307575759 140.45728843818247 8.82168 35.69873307575759 140.45728843818247 5.52168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985995296282 140.45716010242487 8.82168 35.69856136397819 140.45721969582033 8.82168 35.69860602842342 140.4572626965308 8.82168 35.69860157532076 140.4572697338041 8.82168 35.698622831685064 140.45729012080187 8.82168 35.698627284216116 140.4572831940229 8.82168 35.69869490947165 140.45734814210934 8.82168 35.69873307575759 140.45728843818247 8.82168 35.6985995296282 140.45716010242487 8.82168</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">131.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8191</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_c076cd50-78a3-46c7-a0e2-167eb29aff98">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.699427228764804 140.4556301493522 0 35.69947919907049 140.45558513739593 0 35.69945436411748 140.45554207019504 0 35.69946432314297 140.4555335285169 0 35.69942680256909 140.45546848367616 0 35.69936487271082 140.45552214784718 0 35.699427228764804 140.4556301493522 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699427228764804 140.4556301493522 6.54569 35.69936487271082 140.45552214784718 6.54569 35.69942680256909 140.45546848367616 6.54569 35.69946432314297 140.4555335285169 6.54569 35.69945436411748 140.45554207019504 6.54569 35.69947919907049 140.45558513739593 6.54569 35.699427228764804 140.4556301493522 6.54569</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699427228764804 140.4556301493522 6.54569 35.69947919907049 140.45558513739593 6.54569 35.69947919907049 140.45558513739593 10.945689999999999 35.699427228764804 140.4556301493522 10.945689999999999 35.699427228764804 140.4556301493522 6.54569</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69947919907049 140.45558513739593 6.54569 35.69945436411748 140.45554207019504 6.54569 35.69945436411748 140.45554207019504 10.945689999999999 35.69947919907049 140.45558513739593 10.945689999999999 35.69947919907049 140.45558513739593 6.54569</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69945436411748 140.45554207019504 6.54569 35.69946432314297 140.4555335285169 6.54569 35.69946432314297 140.4555335285169 10.945689999999999 35.69945436411748 140.45554207019504 10.945689999999999 35.69945436411748 140.45554207019504 6.54569</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69946432314297 140.4555335285169 6.54569 35.69942680256909 140.45546848367616 6.54569 35.69942680256909 140.45546848367616 10.945689999999999 35.69946432314297 140.4555335285169 10.945689999999999 35.69946432314297 140.4555335285169 6.54569</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69942680256909 140.45546848367616 6.54569 35.69936487271082 140.45552214784718 6.54569 35.69936487271082 140.45552214784718 10.945689999999999 35.69942680256909 140.45546848367616 10.945689999999999 35.69942680256909 140.45546848367616 6.54569</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69936487271082 140.45552214784718 6.54569 35.699427228764804 140.4556301493522 6.54569 35.699427228764804 140.4556301493522 10.945689999999999 35.69936487271082 140.45552214784718 10.945689999999999 35.69936487271082 140.45552214784718 6.54569</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699427228764804 140.4556301493522 10.945689999999999 35.69947919907049 140.45558513739593 10.945689999999999 35.69945436411748 140.45554207019504 10.945689999999999 35.69946432314297 140.4555335285169 10.945689999999999 35.69942680256909 140.45546848367616 10.945689999999999 35.69936487271082 140.45552214784718 10.945689999999999 35.699427228764804 140.4556301493522 10.945689999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">94.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8235</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_55915abd-46b9-4e74-8c24-03dc51b5a413">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69988843813819 140.45585915554835 0 35.699876106846425 140.4558383959698 0 35.69985383104519 140.45585822353664 0 35.69986616290512 140.4558788726156 0 35.69988843813819 140.45585915554835 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69988843813819 140.45585915554835 5.58148 35.69986616290512 140.4558788726156 5.58148 35.69985383104519 140.45585822353664 5.58148 35.699876106846425 140.4558383959698 5.58148 35.69988843813819 140.45585915554835 5.58148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69988843813819 140.45585915554835 5.58148 35.699876106846425 140.4558383959698 5.58148 35.699876106846425 140.4558383959698 7.9814799999999995 35.69988843813819 140.45585915554835 7.9814799999999995 35.69988843813819 140.45585915554835 5.58148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699876106846425 140.4558383959698 5.58148 35.69985383104519 140.45585822353664 5.58148 35.69985383104519 140.45585822353664 7.9814799999999995 35.699876106846425 140.4558383959698 7.9814799999999995 35.699876106846425 140.4558383959698 5.58148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69985383104519 140.45585822353664 5.58148 35.69986616290512 140.4558788726156 5.58148 35.69986616290512 140.4558788726156 7.9814799999999995 35.69985383104519 140.45585822353664 7.9814799999999995 35.69985383104519 140.45585822353664 5.58148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69986616290512 140.4558788726156 5.58148 35.69988843813819 140.45585915554835 5.58148 35.69988843813819 140.45585915554835 7.9814799999999995 35.69986616290512 140.4558788726156 7.9814799999999995 35.69986616290512 140.4558788726156 5.58148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69988843813819 140.45585915554835 7.9814799999999995 35.699876106846425 140.4558383959698 7.9814799999999995 35.69985383104519 140.45585822353664 7.9814799999999995 35.69986616290512 140.4558788726156 7.9814799999999995 35.69988843813819 140.45585915554835 7.9814799999999995</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">7.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8293</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b9337070-0234-480d-9209-fce667da5256">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698560523832555 140.45652928910852 0 35.69854808078148 140.45651272788155 0 35.69850111688114 140.4565656231269 0 35.69851346979347 140.4565821836508 0 35.698560523832555 140.45652928910852 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698560523832555 140.45652928910852 6.76252 35.69851346979347 140.4565821836508 6.76252 35.69850111688114 140.4565656231269 6.76252 35.69854808078148 140.45651272788155 6.76252 35.698560523832555 140.45652928910852 6.76252</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698560523832555 140.45652928910852 6.76252 35.69854808078148 140.45651272788155 6.76252 35.69854808078148 140.45651272788155 8.46252 35.698560523832555 140.45652928910852 8.46252 35.698560523832555 140.45652928910852 6.76252</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69854808078148 140.45651272788155 6.76252 35.69850111688114 140.4565656231269 6.76252 35.69850111688114 140.4565656231269 8.46252 35.69854808078148 140.45651272788155 8.46252 35.69854808078148 140.45651272788155 6.76252</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69850111688114 140.4565656231269 6.76252 35.69851346979347 140.4565821836508 6.76252 35.69851346979347 140.4565821836508 8.46252 35.69850111688114 140.4565656231269 8.46252 35.69850111688114 140.4565656231269 6.76252</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851346979347 140.4565821836508 6.76252 35.698560523832555 140.45652928910852 6.76252 35.698560523832555 140.45652928910852 8.46252 35.69851346979347 140.4565821836508 8.46252 35.69851346979347 140.4565821836508 6.76252</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698560523832555 140.45652928910852 8.46252 35.69854808078148 140.45651272788155 8.46252 35.69850111688114 140.4565656231269 8.46252 35.69851346979347 140.4565821836508 8.46252 35.698560523832555 140.45652928910852 8.46252</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">14.4</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-9092</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_d1cccddb-b7f3-43b2-beb1-6a0ab8765e8b">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.699848050648704 140.4553897641385 0 35.699825086417995 140.45540328791142 0 35.6998375409406 140.45543509848324 0 35.69986050517477 140.45542157471746 0 35.699848050648704 140.4553897641385 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699848050648704 140.4553897641385 6.09336 35.69986050517477 140.45542157471746 6.09336 35.6998375409406 140.45543509848324 6.09336 35.699825086417995 140.45540328791142 6.09336 35.699848050648704 140.4553897641385 6.09336</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699848050648704 140.4553897641385 6.09336 35.699825086417995 140.45540328791142 6.09336 35.699825086417995 140.45540328791142 9.09336 35.699848050648704 140.4553897641385 9.09336 35.699848050648704 140.4553897641385 6.09336</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699825086417995 140.45540328791142 6.09336 35.6998375409406 140.45543509848324 6.09336 35.6998375409406 140.45543509848324 9.09336 35.699825086417995 140.45540328791142 9.09336 35.699825086417995 140.45540328791142 6.09336</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6998375409406 140.45543509848324 6.09336 35.69986050517477 140.45542157471746 6.09336 35.69986050517477 140.45542157471746 9.09336 35.6998375409406 140.45543509848324 9.09336 35.6998375409406 140.45543509848324 6.09336</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69986050517477 140.45542157471746 6.09336 35.699848050648704 140.4553897641385 6.09336 35.699848050648704 140.4553897641385 9.09336 35.69986050517477 140.45542157471746 9.09336 35.69986050517477 140.45542157471746 6.09336</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699848050648704 140.4553897641385 9.09336 35.699825086417995 140.45540328791142 9.09336 35.6998375409406 140.45543509848324 9.09336 35.69986050517477 140.45542157471746 9.09336 35.699848050648704 140.4553897641385 9.09336</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">9</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8307</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_434f6f5d-b282-48c0-a233-30eadaae5f18">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">15.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.6986898394481 140.45639548585686 0 35.69867068855664 140.45636881698317 0 35.698640685709556 140.45640129143948 0 35.69865983659393 140.45642796031078 0 35.6986898394481 140.45639548585686 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6986898394481 140.45639548585686 6.85348 35.69865983659393 140.45642796031078 6.85348 35.698640685709556 140.45640129143948 6.85348 35.69867068855664 140.45636881698317 6.85348 35.6986898394481 140.45639548585686 6.85348</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6986898394481 140.45639548585686 6.85348 35.69867068855664 140.45636881698317 6.85348 35.69867068855664 140.45636881698317 9.853480000000001 35.6986898394481 140.45639548585686 9.853480000000001 35.6986898394481 140.45639548585686 6.85348</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69867068855664 140.45636881698317 6.85348 35.698640685709556 140.45640129143948 6.85348 35.698640685709556 140.45640129143948 9.853480000000001 35.69867068855664 140.45636881698317 9.853480000000001 35.69867068855664 140.45636881698317 6.85348</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698640685709556 140.45640129143948 6.85348 35.69865983659393 140.45642796031078 6.85348 35.69865983659393 140.45642796031078 9.853480000000001 35.698640685709556 140.45640129143948 9.853480000000001 35.698640685709556 140.45640129143948 6.85348</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69865983659393 140.45642796031078 6.85348 35.6986898394481 140.45639548585686 6.85348 35.6986898394481 140.45639548585686 9.853480000000001 35.69865983659393 140.45642796031078 9.853480000000001 35.69865983659393 140.45642796031078 6.85348</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6986898394481 140.45639548585686 9.853480000000001 35.69867068855664 140.45636881698317 9.853480000000001 35.698640685709556 140.45640129143948 9.853480000000001 35.69865983659393 140.45642796031078 9.853480000000001 35.6986898394481 140.45639548585686 9.853480000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">14.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8249</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_0a3abe27-6d28-4191-8f5c-d22d5696e119">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69975558977126 140.4555955724272 0 35.69969274851721 140.45561630034877 0 35.699706749808 140.45568016820263 0 35.69976968120403 140.45565944102825 0 35.69975558977126 140.4555955724272 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69975558977126 140.4555955724272 6.05321 35.69976968120403 140.45565944102825 6.05321 35.699706749808 140.45568016820263 6.05321 35.69969274851721 140.45561630034877 6.05321 35.69975558977126 140.4555955724272 6.05321</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69975558977126 140.4555955724272 6.05321 35.69969274851721 140.45561630034877 6.05321 35.69969274851721 140.45561630034877 10.55321 35.69975558977126 140.4555955724272 10.55321 35.69975558977126 140.4555955724272 6.05321</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69969274851721 140.45561630034877 6.05321 35.699706749808 140.45568016820263 6.05321 35.699706749808 140.45568016820263 10.55321 35.69969274851721 140.45561630034877 10.55321 35.69969274851721 140.45561630034877 6.05321</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699706749808 140.45568016820263 6.05321 35.69976968120403 140.45565944102825 6.05321 35.69976968120403 140.45565944102825 10.55321 35.699706749808 140.45568016820263 10.55321 35.699706749808 140.45568016820263 6.05321</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69976968120403 140.45565944102825 6.05321 35.69975558977126 140.4555955724272 6.05321 35.69975558977126 140.4555955724272 10.55321 35.69976968120403 140.45565944102825 10.55321 35.69976968120403 140.45565944102825 6.05321</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69975558977126 140.4555955724272 10.55321 35.69969274851721 140.45561630034877 10.55321 35.699706749808 140.45568016820263 10.55321 35.69976968120403 140.45565944102825 10.55321 35.69975558977126 140.4555955724272 10.55321</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">43.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8296</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_7112f75d-211b-443b-9120-804988fe39f4">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">1.8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69787221919315 140.45960938105847 0 35.69786445395574 140.45962932055104 0 35.697880000142746 140.45963839248893 0 35.69788776595658 140.45961834249945 0 35.69787221919315 140.45960938105847 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69787221919315 140.45960938105847 6.80458 35.69788776595658 140.45961834249945 6.80458 35.697880000142746 140.45963839248893 6.80458 35.69786445395574 140.45962932055104 6.80458 35.69787221919315 140.45960938105847 6.80458</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69787221919315 140.45960938105847 6.80458 35.69786445395574 140.45962932055104 6.80458 35.69786445395574 140.45962932055104 8.30458 35.69787221919315 140.45960938105847 8.30458 35.69787221919315 140.45960938105847 6.80458</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69786445395574 140.45962932055104 6.80458 35.697880000142746 140.45963839248893 6.80458 35.697880000142746 140.45963839248893 8.30458 35.69786445395574 140.45962932055104 8.30458 35.69786445395574 140.45962932055104 6.80458</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697880000142746 140.45963839248893 6.80458 35.69788776595658 140.45961834249945 6.80458 35.69788776595658 140.45961834249945 8.30458 35.697880000142746 140.45963839248893 8.30458 35.697880000142746 140.45963839248893 6.80458</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69788776595658 140.45961834249945 6.80458 35.69787221919315 140.45960938105847 6.80458 35.69787221919315 140.45960938105847 8.30458 35.69788776595658 140.45961834249945 8.30458 35.69788776595658 140.45961834249945 6.80458</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69787221919315 140.45960938105847 8.30458 35.69786445395574 140.45962932055104 8.30458 35.697880000142746 140.45963839248893 8.30458 35.69788776595658 140.45961834249945 8.30458 35.69787221919315 140.45960938105847 8.30458</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">3.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8214</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_bca14df8-465d-4e02-8e70-a953d30107cf">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69953567406108 140.4556451360824 0 35.69959674912247 140.45563478157632 0 35.69957408177208 140.4554339360037 0 35.69951300615512 140.45544440115683 0 35.69953567406108 140.4556451360824 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69953567406108 140.4556451360824 6.12845 35.69951300615512 140.45544440115683 6.12845 35.69957408177208 140.4554339360037 6.12845 35.69959674912247 140.45563478157632 6.12845 35.69953567406108 140.4556451360824 6.12845</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69953567406108 140.4556451360824 6.12845 35.69959674912247 140.45563478157632 6.12845 35.69959674912247 140.45563478157632 10.92845 35.69953567406108 140.4556451360824 10.92845 35.69953567406108 140.4556451360824 6.12845</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69959674912247 140.45563478157632 6.12845 35.69957408177208 140.4554339360037 6.12845 35.69957408177208 140.4554339360037 10.92845 35.69959674912247 140.45563478157632 10.92845 35.69959674912247 140.45563478157632 6.12845</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69957408177208 140.4554339360037 6.12845 35.69951300615512 140.45544440115683 6.12845 35.69951300615512 140.45544440115683 10.92845 35.69957408177208 140.4554339360037 10.92845 35.69957408177208 140.4554339360037 6.12845</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69951300615512 140.45544440115683 6.12845 35.69953567406108 140.4556451360824 6.12845 35.69953567406108 140.4556451360824 10.92845 35.69951300615512 140.45544440115683 10.92845 35.69951300615512 140.45544440115683 6.12845</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69953567406108 140.4556451360824 10.92845 35.69959674912247 140.45563478157632 10.92845 35.69957408177208 140.4554339360037 10.92845 35.69951300615512 140.45544440115683 10.92845 35.69953567406108 140.4556451360824 10.92845</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">125.5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8244</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_3440b67e-0a41-438b-99d9-7d61c31d6e2b">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">10.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.6987354780546 140.45638965292196 0 35.69871158770276 140.45637311312873 0 35.698693720587194 140.45641186998694 0 35.69871752080214 140.45642840908667 0 35.6987354780546 140.45638965292196 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6987354780546 140.45638965292196 6.13509 35.69871752080214 140.45642840908667 6.13509 35.698693720587194 140.45641186998694 6.13509 35.69871158770276 140.45637311312873 6.13509 35.6987354780546 140.45638965292196 6.13509</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6987354780546 140.45638965292196 6.13509 35.69871158770276 140.45637311312873 6.13509 35.69871158770276 140.45637311312873 9.13509 35.6987354780546 140.45638965292196 9.13509 35.6987354780546 140.45638965292196 6.13509</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69871158770276 140.45637311312873 6.13509 35.698693720587194 140.45641186998694 6.13509 35.698693720587194 140.45641186998694 9.13509 35.69871158770276 140.45637311312873 9.13509 35.69871158770276 140.45637311312873 6.13509</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698693720587194 140.45641186998694 6.13509 35.69871752080214 140.45642840908667 6.13509 35.69871752080214 140.45642840908667 9.13509 35.698693720587194 140.45641186998694 9.13509 35.698693720587194 140.45641186998694 6.13509</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69871752080214 140.45642840908667 6.13509 35.6987354780546 140.45638965292196 6.13509 35.6987354780546 140.45638965292196 9.13509 35.69871752080214 140.45642840908667 9.13509 35.69871752080214 140.45642840908667 6.13509</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6987354780546 140.45638965292196 9.13509 35.69871158770276 140.45637311312873 9.13509 35.698693720587194 140.45641186998694 9.13509 35.69871752080214 140.45642840908667 9.13509 35.6987354780546 140.45638965292196 9.13509</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">12.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8248</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_8bb9dd64-f039-4d48-b0e9-547cf9caa37f">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">6.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698627605585386 140.45661224369223 0 35.69867415186524 140.45667481716683 0 35.698668439815705 140.4566812921829 0 35.698691086312216 140.456711745323 0 35.69872018792614 140.45667926394913 0 35.69873083976648 140.45669360130398 0 35.698725581234974 140.45669952735503 0 35.69875467256268 140.45673864965968 0 35.69879555995577 140.45669300003794 0 35.69871043417209 140.45657852290438 0 35.698700189803205 140.45658993514164 0 35.69869419275321 140.45658182200438 0 35.69870072055858 140.45657446933043 0 35.698682907950605 140.45655046281362 0 35.698627605585386 140.45661224369223 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698627605585386 140.45661224369223 5.88876 35.698682907950605 140.45655046281362 5.88876 35.69870072055858 140.45657446933043 5.88876 35.69869419275321 140.45658182200438 5.88876 35.698700189803205 140.45658993514164 5.88876 35.69871043417209 140.45657852290438 5.88876 35.69879555995577 140.45669300003794 5.88876 35.69875467256268 140.45673864965968 5.88876 35.698725581234974 140.45669952735503 5.88876 35.69873083976648 140.45669360130398 5.88876 35.69872018792614 140.45667926394913 5.88876 35.698691086312216 140.456711745323 5.88876 35.698668439815705 140.4566812921829 5.88876 35.69867415186524 140.45667481716683 5.88876 35.698627605585386 140.45661224369223 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698627605585386 140.45661224369223 5.88876 35.69867415186524 140.45667481716683 5.88876 35.69867415186524 140.45667481716683 9.888760000000001 35.698627605585386 140.45661224369223 9.888760000000001 35.698627605585386 140.45661224369223 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69867415186524 140.45667481716683 5.88876 35.698668439815705 140.4566812921829 5.88876 35.698668439815705 140.4566812921829 9.888760000000001 35.69867415186524 140.45667481716683 9.888760000000001 35.69867415186524 140.45667481716683 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698668439815705 140.4566812921829 5.88876 35.698691086312216 140.456711745323 5.88876 35.698691086312216 140.456711745323 9.888760000000001 35.698668439815705 140.4566812921829 9.888760000000001 35.698668439815705 140.4566812921829 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698691086312216 140.456711745323 5.88876 35.69872018792614 140.45667926394913 5.88876 35.69872018792614 140.45667926394913 9.888760000000001 35.698691086312216 140.456711745323 9.888760000000001 35.698691086312216 140.456711745323 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69872018792614 140.45667926394913 5.88876 35.69873083976648 140.45669360130398 5.88876 35.69873083976648 140.45669360130398 9.888760000000001 35.69872018792614 140.45667926394913 9.888760000000001 35.69872018792614 140.45667926394913 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69873083976648 140.45669360130398 5.88876 35.698725581234974 140.45669952735503 5.88876 35.698725581234974 140.45669952735503 9.888760000000001 35.69873083976648 140.45669360130398 9.888760000000001 35.69873083976648 140.45669360130398 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698725581234974 140.45669952735503 5.88876 35.69875467256268 140.45673864965968 5.88876 35.69875467256268 140.45673864965968 9.888760000000001 35.698725581234974 140.45669952735503 9.888760000000001 35.698725581234974 140.45669952735503 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69875467256268 140.45673864965968 5.88876 35.69879555995577 140.45669300003794 5.88876 35.69879555995577 140.45669300003794 9.888760000000001 35.69875467256268 140.45673864965968 9.888760000000001 35.69875467256268 140.45673864965968 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69879555995577 140.45669300003794 5.88876 35.69871043417209 140.45657852290438 5.88876 35.69871043417209 140.45657852290438 9.888760000000001 35.69879555995577 140.45669300003794 9.888760000000001 35.69879555995577 140.45669300003794 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69871043417209 140.45657852290438 5.88876 35.698700189803205 140.45658993514164 5.88876 35.698700189803205 140.45658993514164 9.888760000000001 35.69871043417209 140.45657852290438 9.888760000000001 35.69871043417209 140.45657852290438 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698700189803205 140.45658993514164 5.88876 35.69869419275321 140.45658182200438 5.88876 35.69869419275321 140.45658182200438 9.888760000000001 35.698700189803205 140.45658993514164 9.888760000000001 35.698700189803205 140.45658993514164 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69869419275321 140.45658182200438 5.88876 35.69870072055858 140.45657446933043 5.88876 35.69870072055858 140.45657446933043 9.888760000000001 35.69869419275321 140.45658182200438 9.888760000000001 35.69869419275321 140.45658182200438 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69870072055858 140.45657446933043 5.88876 35.698682907950605 140.45655046281362 5.88876 35.698682907950605 140.45655046281362 9.888760000000001 35.69870072055858 140.45657446933043 9.888760000000001 35.69870072055858 140.45657446933043 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698682907950605 140.45655046281362 5.88876 35.698627605585386 140.45661224369223 5.88876 35.698627605585386 140.45661224369223 9.888760000000001 35.698682907950605 140.45655046281362 9.888760000000001 35.698682907950605 140.45655046281362 5.88876</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698627605585386 140.45661224369223 9.888760000000001 35.69867415186524 140.45667481716683 9.888760000000001 35.698668439815705 140.4566812921829 9.888760000000001 35.698691086312216 140.456711745323 9.888760000000001 35.69872018792614 140.45667926394913 9.888760000000001 35.69873083976648 140.45669360130398 9.888760000000001 35.698725581234974 140.45669952735503 9.888760000000001 35.69875467256268 140.45673864965968 9.888760000000001 35.69879555995577 140.45669300003794 9.888760000000001 35.69871043417209 140.45657852290438 9.888760000000001 35.698700189803205 140.45658993514164 9.888760000000001 35.69869419275321 140.45658182200438 9.888760000000001 35.69870072055858 140.45657446933043 9.888760000000001 35.698682907950605 140.45655046281362 9.888760000000001 35.698627605585386 140.45661224369223 9.888760000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">139.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8250</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_625c99ce-2267-4ab7-a89e-e07a98169d29">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69833931327489 140.45773300231573 0 35.69826990679742 140.4576639518515 0 35.69823438623998 140.4577174879418 0 35.69830379211377 140.45778664891546 0 35.69833931327489 140.45773300231573 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69833931327489 140.45773300231573 6.55288 35.69830379211377 140.45778664891546 6.55288 35.69823438623998 140.4577174879418 6.55288 35.69826990679742 140.4576639518515 6.55288 35.69833931327489 140.45773300231573 6.55288</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69833931327489 140.45773300231573 6.55288 35.69826990679742 140.4576639518515 6.55288 35.69826990679742 140.4576639518515 9.55288 35.69833931327489 140.45773300231573 9.55288 35.69833931327489 140.45773300231573 6.55288</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69826990679742 140.4576639518515 6.55288 35.69823438623998 140.4577174879418 6.55288 35.69823438623998 140.4577174879418 9.55288 35.69826990679742 140.4576639518515 9.55288 35.69826990679742 140.4576639518515 6.55288</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69823438623998 140.4577174879418 6.55288 35.69830379211377 140.45778664891546 6.55288 35.69830379211377 140.45778664891546 9.55288 35.69823438623998 140.4577174879418 9.55288 35.69823438623998 140.4577174879418 6.55288</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69830379211377 140.45778664891546 6.55288 35.69833931327489 140.45773300231573 6.55288 35.69833931327489 140.45773300231573 9.55288 35.69830379211377 140.45778664891546 9.55288 35.69830379211377 140.45778664891546 6.55288</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69833931327489 140.45773300231573 9.55288 35.69826990679742 140.4576639518515 9.55288 35.69823438623998 140.4577174879418 9.55288 35.69830379211377 140.45778664891546 9.55288 35.69833931327489 140.45773300231573 9.55288</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">62</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">203</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-9087</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_5721f2b9-f94d-4466-ae41-17d076ebcb18">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">7.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69926210907617 140.4560468892254 0 35.69919734734862 140.45609036479348 0 35.6992360185092 140.45617685574555 0 35.699245425354576 140.4561705198821 0 35.699257454177264 140.45619746501384 0 35.699312809669614 140.45616021487643 0 35.69926210907617 140.4560468892254 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69926210907617 140.4560468892254 5.64989 35.699312809669614 140.45616021487643 5.64989 35.699257454177264 140.45619746501384 5.64989 35.699245425354576 140.4561705198821 5.64989 35.6992360185092 140.45617685574555 5.64989 35.69919734734862 140.45609036479348 5.64989 35.69926210907617 140.4560468892254 5.64989</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69926210907617 140.4560468892254 5.64989 35.69919734734862 140.45609036479348 5.64989 35.69919734734862 140.45609036479348 11.14989 35.69926210907617 140.4560468892254 11.14989 35.69926210907617 140.4560468892254 5.64989</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69919734734862 140.45609036479348 5.64989 35.6992360185092 140.45617685574555 5.64989 35.6992360185092 140.45617685574555 11.14989 35.69919734734862 140.45609036479348 11.14989 35.69919734734862 140.45609036479348 5.64989</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6992360185092 140.45617685574555 5.64989 35.699245425354576 140.4561705198821 5.64989 35.699245425354576 140.4561705198821 11.14989 35.6992360185092 140.45617685574555 11.14989 35.6992360185092 140.45617685574555 5.64989</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699245425354576 140.4561705198821 5.64989 35.699257454177264 140.45619746501384 5.64989 35.699257454177264 140.45619746501384 11.14989 35.699245425354576 140.4561705198821 11.14989 35.699245425354576 140.4561705198821 5.64989</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699257454177264 140.45619746501384 5.64989 35.699312809669614 140.45616021487643 5.64989 35.699312809669614 140.45616021487643 11.14989 35.699257454177264 140.45619746501384 11.14989 35.699257454177264 140.45619746501384 5.64989</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699312809669614 140.45616021487643 5.64989 35.69926210907617 140.4560468892254 5.64989 35.69926210907617 140.4560468892254 11.14989 35.699312809669614 140.45616021487643 11.14989 35.699312809669614 140.45616021487643 5.64989</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69926210907617 140.4560468892254 11.14989 35.69919734734862 140.45609036479348 11.14989 35.6992360185092 140.45617685574555 11.14989 35.699245425354576 140.4561705198821 11.14989 35.699257454177264 140.45619746501384 11.14989 35.699312809669614 140.45616021487643 11.14989 35.69926210907617 140.4560468892254 11.14989</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">92.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8261</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_36df5ebc-f211-4bf9-a84f-85f97bb44f8d">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69810574547123 140.45813781682287 0 35.69815874189222 140.4580689474888 0 35.69806242561937 140.45795736601573 0 35.6980440946191 140.45798120126685 0 35.69803002825309 140.457964848261 0 35.697995272769944 140.45800988163325 0 35.69810574547123 140.45813781682287 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69810574547123 140.45813781682287 12.84435 35.697995272769944 140.45800988163325 12.84435 35.69803002825309 140.457964848261 12.84435 35.6980440946191 140.45798120126685 12.84435 35.69806242561937 140.45795736601573 12.84435 35.69815874189222 140.4580689474888 12.84435 35.69810574547123 140.45813781682287 12.84435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69810574547123 140.45813781682287 12.84435 35.69815874189222 140.4580689474888 12.84435 35.69815874189222 140.4580689474888 18.34435 35.69810574547123 140.45813781682287 18.34435 35.69810574547123 140.45813781682287 12.84435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69815874189222 140.4580689474888 12.84435 35.69806242561937 140.45795736601573 12.84435 35.69806242561937 140.45795736601573 18.34435 35.69815874189222 140.4580689474888 18.34435 35.69815874189222 140.4580689474888 12.84435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69806242561937 140.45795736601573 12.84435 35.6980440946191 140.45798120126685 12.84435 35.6980440946191 140.45798120126685 18.34435 35.69806242561937 140.45795736601573 18.34435 35.69806242561937 140.45795736601573 12.84435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6980440946191 140.45798120126685 12.84435 35.69803002825309 140.457964848261 12.84435 35.69803002825309 140.457964848261 18.34435 35.6980440946191 140.45798120126685 18.34435 35.6980440946191 140.45798120126685 12.84435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69803002825309 140.457964848261 12.84435 35.697995272769944 140.45800988163325 12.84435 35.697995272769944 140.45800988163325 18.34435 35.69803002825309 140.457964848261 18.34435 35.69803002825309 140.457964848261 12.84435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697995272769944 140.45800988163325 12.84435 35.69810574547123 140.45813781682287 12.84435 35.69810574547123 140.45813781682287 18.34435 35.697995272769944 140.45800988163325 18.34435 35.697995272769944 140.45800988163325 12.84435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69810574547123 140.45813781682287 18.34435 35.69815874189222 140.4580689474888 18.34435 35.69806242561937 140.45795736601573 18.34435 35.6980440946191 140.45798120126685 18.34435 35.69803002825309 140.457964848261 18.34435 35.697995272769944 140.45800988163325 18.34435 35.69810574547123 140.45813781682287 18.34435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">138.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8305</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_91f70513-8bd9-4c1f-a606-e635c86bcb0e">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69893945610926 140.45630217682844 0 35.698914242937256 140.45627999123528 0 35.69886058949427 140.45637150975173 0 35.69888580264947 140.4563936953584 0 35.69893945610926 140.45630217682844 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69893945610926 140.45630217682844 5.83541 35.69888580264947 140.4563936953584 5.83541 35.69886058949427 140.45637150975173 5.83541 35.698914242937256 140.45627999123528 5.83541 35.69893945610926 140.45630217682844 5.83541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69893945610926 140.45630217682844 5.83541 35.698914242937256 140.45627999123528 5.83541 35.698914242937256 140.45627999123528 8.13541 35.69893945610926 140.45630217682844 8.13541 35.69893945610926 140.45630217682844 5.83541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698914242937256 140.45627999123528 5.83541 35.69886058949427 140.45637150975173 5.83541 35.69886058949427 140.45637150975173 8.13541 35.698914242937256 140.45627999123528 8.13541 35.698914242937256 140.45627999123528 5.83541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69886058949427 140.45637150975173 5.83541 35.69888580264947 140.4563936953584 5.83541 35.69888580264947 140.4563936953584 8.13541 35.69886058949427 140.45637150975173 8.13541 35.69886058949427 140.45637150975173 5.83541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69888580264947 140.4563936953584 5.83541 35.69893945610926 140.45630217682844 5.83541 35.69893945610926 140.45630217682844 8.13541 35.69888580264947 140.4563936953584 8.13541 35.69888580264947 140.4563936953584 5.83541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69893945610926 140.45630217682844 8.13541 35.698914242937256 140.45627999123528 8.13541 35.69886058949427 140.45637150975173 8.13541 35.69888580264947 140.4563936953584 8.13541 35.69893945610926 140.45630217682844 8.13541</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">35.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8241</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_ee265a5c-8fc9-4f12-a25d-d36f80719960">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69885237532942 140.45654791401995 0 35.69884348636963 140.45655867382962 0 35.698877163009676 140.45660004182133 0 35.69889040632289 140.4565839014048 0 35.6989051847734 140.4566020278868 0 35.698927680435936 140.4565745779768 0 35.698934577085254 140.4565830296377 0 35.69896668790442 140.45654383103857 0 35.69897412191263 140.45655294988225 0 35.69899498489846 140.4565274762307 0 35.698952262149355 140.45647498782986 0 35.698938383353216 140.45649200733158 0 35.69889709411287 140.45644118763775 0 35.69888149252533 140.45646018272691 0 35.698855249282786 140.45642804410485 0 35.698804905360355 140.45648964278024 0 35.69885237532942 140.45654791401995 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69885237532942 140.45654791401995 5.43378 35.698804905360355 140.45648964278024 5.43378 35.698855249282786 140.45642804410485 5.43378 35.69888149252533 140.45646018272691 5.43378 35.69889709411287 140.45644118763775 5.43378 35.698938383353216 140.45649200733158 5.43378 35.698952262149355 140.45647498782986 5.43378 35.69899498489846 140.4565274762307 5.43378 35.69897412191263 140.45655294988225 5.43378 35.69896668790442 140.45654383103857 5.43378 35.698934577085254 140.4565830296377 5.43378 35.698927680435936 140.4565745779768 5.43378 35.6989051847734 140.4566020278868 5.43378 35.69889040632289 140.4565839014048 5.43378 35.698877163009676 140.45660004182133 5.43378 35.69884348636963 140.45655867382962 5.43378 35.69885237532942 140.45654791401995 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69885237532942 140.45654791401995 5.43378 35.69884348636963 140.45655867382962 5.43378 35.69884348636963 140.45655867382962 10.33378 35.69885237532942 140.45654791401995 10.33378 35.69885237532942 140.45654791401995 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69884348636963 140.45655867382962 5.43378 35.698877163009676 140.45660004182133 5.43378 35.698877163009676 140.45660004182133 10.33378 35.69884348636963 140.45655867382962 10.33378 35.69884348636963 140.45655867382962 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698877163009676 140.45660004182133 5.43378 35.69889040632289 140.4565839014048 5.43378 35.69889040632289 140.4565839014048 10.33378 35.698877163009676 140.45660004182133 10.33378 35.698877163009676 140.45660004182133 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69889040632289 140.4565839014048 5.43378 35.6989051847734 140.4566020278868 5.43378 35.6989051847734 140.4566020278868 10.33378 35.69889040632289 140.4565839014048 10.33378 35.69889040632289 140.4565839014048 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6989051847734 140.4566020278868 5.43378 35.698927680435936 140.4565745779768 5.43378 35.698927680435936 140.4565745779768 10.33378 35.6989051847734 140.4566020278868 10.33378 35.6989051847734 140.4566020278868 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698927680435936 140.4565745779768 5.43378 35.698934577085254 140.4565830296377 5.43378 35.698934577085254 140.4565830296377 10.33378 35.698927680435936 140.4565745779768 10.33378 35.698927680435936 140.4565745779768 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698934577085254 140.4565830296377 5.43378 35.69896668790442 140.45654383103857 5.43378 35.69896668790442 140.45654383103857 10.33378 35.698934577085254 140.4565830296377 10.33378 35.698934577085254 140.4565830296377 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69896668790442 140.45654383103857 5.43378 35.69897412191263 140.45655294988225 5.43378 35.69897412191263 140.45655294988225 10.33378 35.69896668790442 140.45654383103857 10.33378 35.69896668790442 140.45654383103857 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69897412191263 140.45655294988225 5.43378 35.69899498489846 140.4565274762307 5.43378 35.69899498489846 140.4565274762307 10.33378 35.69897412191263 140.45655294988225 10.33378 35.69897412191263 140.45655294988225 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69899498489846 140.4565274762307 5.43378 35.698952262149355 140.45647498782986 5.43378 35.698952262149355 140.45647498782986 10.33378 35.69899498489846 140.4565274762307 10.33378 35.69899498489846 140.4565274762307 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698952262149355 140.45647498782986 5.43378 35.698938383353216 140.45649200733158 5.43378 35.698938383353216 140.45649200733158 10.33378 35.698952262149355 140.45647498782986 10.33378 35.698952262149355 140.45647498782986 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698938383353216 140.45649200733158 5.43378 35.69889709411287 140.45644118763775 5.43378 35.69889709411287 140.45644118763775 10.33378 35.698938383353216 140.45649200733158 10.33378 35.698938383353216 140.45649200733158 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69889709411287 140.45644118763775 5.43378 35.69888149252533 140.45646018272691 5.43378 35.69888149252533 140.45646018272691 10.33378 35.69889709411287 140.45644118763775 10.33378 35.69889709411287 140.45644118763775 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69888149252533 140.45646018272691 5.43378 35.698855249282786 140.45642804410485 5.43378 35.698855249282786 140.45642804410485 10.33378 35.69888149252533 140.45646018272691 10.33378 35.69888149252533 140.45646018272691 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698855249282786 140.45642804410485 5.43378 35.698804905360355 140.45648964278024 5.43378 35.698804905360355 140.45648964278024 10.33378 35.698855249282786 140.45642804410485 10.33378 35.698855249282786 140.45642804410485 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698804905360355 140.45648964278024 5.43378 35.69885237532942 140.45654791401995 5.43378 35.69885237532942 140.45654791401995 10.33378 35.698804905360355 140.45648964278024 10.33378 35.698804905360355 140.45648964278024 5.43378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69885237532942 140.45654791401995 10.33378 35.69884348636963 140.45655867382962 10.33378 35.698877163009676 140.45660004182133 10.33378 35.69889040632289 140.4565839014048 10.33378 35.6989051847734 140.4566020278868 10.33378 35.698927680435936 140.4565745779768 10.33378 35.698934577085254 140.4565830296377 10.33378 35.69896668790442 140.45654383103857 10.33378 35.69897412191263 140.45655294988225 10.33378 35.69899498489846 140.4565274762307 10.33378 35.698952262149355 140.45647498782986 10.33378 35.698938383353216 140.45649200733158 10.33378 35.69889709411287 140.45644118763775 10.33378 35.69888149252533 140.45646018272691 10.33378 35.698855249282786 140.45642804410485 10.33378 35.698804905360355 140.45648964278024 10.33378 35.69885237532942 140.45654791401995 10.33378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">173.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8225</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_3e64f3d8-baa0-43f9-a981-d72943823895">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698725257422915 140.45695345294814 0 35.69869670180307 140.4569153292476 0 35.698668777561245 140.45694671469792 0 35.69869742330287 140.45698483909786 0 35.698725257422915 140.45695345294814 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698725257422915 140.45695345294814 5.86041 35.69869742330287 140.45698483909786 5.86041 35.698668777561245 140.45694671469792 5.86041 35.69869670180307 140.4569153292476 5.86041 35.698725257422915 140.45695345294814 5.86041</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698725257422915 140.45695345294814 5.86041 35.69869670180307 140.4569153292476 5.86041 35.69869670180307 140.4569153292476 8.36041 35.698725257422915 140.45695345294814 8.36041 35.698725257422915 140.45695345294814 5.86041</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69869670180307 140.4569153292476 5.86041 35.698668777561245 140.45694671469792 5.86041 35.698668777561245 140.45694671469792 8.36041 35.69869670180307 140.4569153292476 8.36041 35.69869670180307 140.4569153292476 5.86041</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698668777561245 140.45694671469792 5.86041 35.69869742330287 140.45698483909786 5.86041 35.69869742330287 140.45698483909786 8.36041 35.698668777561245 140.45694671469792 8.36041 35.698668777561245 140.45694671469792 5.86041</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69869742330287 140.45698483909786 5.86041 35.698725257422915 140.45695345294814 5.86041 35.698725257422915 140.45695345294814 8.36041 35.69869742330287 140.45698483909786 8.36041 35.69869742330287 140.45698483909786 5.86041</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698725257422915 140.45695345294814 8.36041 35.69869670180307 140.4569153292476 8.36041 35.698668777561245 140.45694671469792 8.36041 35.69869742330287 140.45698483909786 8.36041 35.698725257422915 140.45695345294814 8.36041</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">19.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8232</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_13da07fc-779b-4e4c-888a-6fb53bef9d5a">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69892886794085 140.45634518943018 0 35.69889187668833 140.4564043505967 0 35.69898561115186 140.45649259579938 0 35.69900951424893 140.4564544382808 0 35.69899076712731 140.45643683342539 0 35.69900394602638 140.4564157199579 0 35.69892886794085 140.45634518943018 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69892886794085 140.45634518943018 5.49099 35.69900394602638 140.4564157199579 5.49099 35.69899076712731 140.45643683342539 5.49099 35.69900951424893 140.4564544382808 5.49099 35.69898561115186 140.45649259579938 5.49099 35.69889187668833 140.4564043505967 5.49099 35.69892886794085 140.45634518943018 5.49099</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69892886794085 140.45634518943018 5.49099 35.69889187668833 140.4564043505967 5.49099 35.69889187668833 140.4564043505967 8.19099 35.69892886794085 140.45634518943018 8.19099 35.69892886794085 140.45634518943018 5.49099</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69889187668833 140.4564043505967 5.49099 35.69898561115186 140.45649259579938 5.49099 35.69898561115186 140.45649259579938 8.19099 35.69889187668833 140.4564043505967 8.19099 35.69889187668833 140.4564043505967 5.49099</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69898561115186 140.45649259579938 5.49099 35.69900951424893 140.4564544382808 5.49099 35.69900951424893 140.4564544382808 8.19099 35.69898561115186 140.45649259579938 8.19099 35.69898561115186 140.45649259579938 5.49099</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69900951424893 140.4564544382808 5.49099 35.69899076712731 140.45643683342539 5.49099 35.69899076712731 140.45643683342539 8.19099 35.69900951424893 140.4564544382808 8.19099 35.69900951424893 140.4564544382808 5.49099</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69899076712731 140.45643683342539 5.49099 35.69900394602638 140.4564157199579 5.49099 35.69900394602638 140.4564157199579 8.19099 35.69899076712731 140.45643683342539 8.19099 35.69899076712731 140.45643683342539 5.49099</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69900394602638 140.4564157199579 5.49099 35.69892886794085 140.45634518943018 5.49099 35.69892886794085 140.45634518943018 8.19099 35.69900394602638 140.4564157199579 8.19099 35.69900394602638 140.4564157199579 5.49099</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69892886794085 140.45634518943018 8.19099 35.69889187668833 140.4564043505967 8.19099 35.69898561115186 140.45649259579938 8.19099 35.69900951424893 140.4564544382808 8.19099 35.69899076712731 140.45643683342539 8.19099 35.69900394602638 140.4564157199579 8.19099 35.69892886794085 140.45634518943018 8.19099</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">82.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8224</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_daff76f4-739b-437a-841d-caa6d598364b">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698708061606794 140.4573322221423 0 35.698744835104534 140.457367316037 0 35.698772735196314 140.45732322305383 0 35.69876672616766 140.4573174302543 0 35.69877436022609 140.45730533476663 0 35.69874798984995 140.45728037739914 0 35.698739992976655 140.45729291205723 0 35.69873559829571 140.4572886788339 0 35.698708061606794 140.4573322221423 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698708061606794 140.4573322221423 5.51378 35.69873559829571 140.4572886788339 5.51378 35.698739992976655 140.45729291205723 5.51378 35.69874798984995 140.45728037739914 5.51378 35.69877436022609 140.45730533476663 5.51378 35.69876672616766 140.4573174302543 5.51378 35.698772735196314 140.45732322305383 5.51378 35.698744835104534 140.457367316037 5.51378 35.698708061606794 140.4573322221423 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698708061606794 140.4573322221423 5.51378 35.698744835104534 140.457367316037 5.51378 35.698744835104534 140.457367316037 8.91378 35.698708061606794 140.4573322221423 8.91378 35.698708061606794 140.4573322221423 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698744835104534 140.457367316037 5.51378 35.698772735196314 140.45732322305383 5.51378 35.698772735196314 140.45732322305383 8.91378 35.698744835104534 140.457367316037 8.91378 35.698744835104534 140.457367316037 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698772735196314 140.45732322305383 5.51378 35.69876672616766 140.4573174302543 5.51378 35.69876672616766 140.4573174302543 8.91378 35.698772735196314 140.45732322305383 8.91378 35.698772735196314 140.45732322305383 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69876672616766 140.4573174302543 5.51378 35.69877436022609 140.45730533476663 5.51378 35.69877436022609 140.45730533476663 8.91378 35.69876672616766 140.4573174302543 8.91378 35.69876672616766 140.4573174302543 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69877436022609 140.45730533476663 5.51378 35.69874798984995 140.45728037739914 5.51378 35.69874798984995 140.45728037739914 8.91378 35.69877436022609 140.45730533476663 8.91378 35.69877436022609 140.45730533476663 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69874798984995 140.45728037739914 5.51378 35.698739992976655 140.45729291205723 5.51378 35.698739992976655 140.45729291205723 8.91378 35.69874798984995 140.45728037739914 8.91378 35.69874798984995 140.45728037739914 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698739992976655 140.45729291205723 5.51378 35.69873559829571 140.4572886788339 5.51378 35.69873559829571 140.4572886788339 8.91378 35.698739992976655 140.45729291205723 8.91378 35.698739992976655 140.45729291205723 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69873559829571 140.4572886788339 5.51378 35.698708061606794 140.4573322221423 5.51378 35.698708061606794 140.4573322221423 8.91378 35.69873559829571 140.4572886788339 8.91378 35.69873559829571 140.4572886788339 5.51378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698708061606794 140.4573322221423 8.91378 35.698744835104534 140.457367316037 8.91378 35.698772735196314 140.45732322305383 8.91378 35.69876672616766 140.4573174302543 8.91378 35.69877436022609 140.45730533476663 8.91378 35.69874798984995 140.45728037739914 8.91378 35.698739992976655 140.45729291205723 8.91378 35.69873559829571 140.4572886788339 8.91378 35.698708061606794 140.4573322221423 8.91378</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">31.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8195</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_e597c95c-60dc-47f4-9094-a2589cfc0774">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69906586169044 140.45620812998567 0 35.69899092889474 140.4562836817062 0 35.699035204894955 140.45634955261917 0 35.69902116015145 140.4563638083686 0 35.69904924584018 140.45640568537624 0 35.69906483026063 140.4563900051061 0 35.69907958872086 140.45641199894064 0 35.69915298192417 140.45633787178483 0 35.69906586169044 140.45620812998567 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906586169044 140.45620812998567 5.48249 35.69915298192417 140.45633787178483 5.48249 35.69907958872086 140.45641199894064 5.48249 35.69906483026063 140.4563900051061 5.48249 35.69904924584018 140.45640568537624 5.48249 35.69902116015145 140.4563638083686 5.48249 35.699035204894955 140.45634955261917 5.48249 35.69899092889474 140.4562836817062 5.48249 35.69906586169044 140.45620812998567 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906586169044 140.45620812998567 5.48249 35.69899092889474 140.4562836817062 5.48249 35.69899092889474 140.4562836817062 11.28249 35.69906586169044 140.45620812998567 11.28249 35.69906586169044 140.45620812998567 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69899092889474 140.4562836817062 5.48249 35.699035204894955 140.45634955261917 5.48249 35.699035204894955 140.45634955261917 11.28249 35.69899092889474 140.4562836817062 11.28249 35.69899092889474 140.4562836817062 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699035204894955 140.45634955261917 5.48249 35.69902116015145 140.4563638083686 5.48249 35.69902116015145 140.4563638083686 11.28249 35.699035204894955 140.45634955261917 11.28249 35.699035204894955 140.45634955261917 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69902116015145 140.4563638083686 5.48249 35.69904924584018 140.45640568537624 5.48249 35.69904924584018 140.45640568537624 11.28249 35.69902116015145 140.4563638083686 11.28249 35.69902116015145 140.4563638083686 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69904924584018 140.45640568537624 5.48249 35.69906483026063 140.4563900051061 5.48249 35.69906483026063 140.4563900051061 11.28249 35.69904924584018 140.45640568537624 11.28249 35.69904924584018 140.45640568537624 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906483026063 140.4563900051061 5.48249 35.69907958872086 140.45641199894064 5.48249 35.69907958872086 140.45641199894064 11.28249 35.69906483026063 140.4563900051061 11.28249 35.69906483026063 140.4563900051061 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69907958872086 140.45641199894064 5.48249 35.69915298192417 140.45633787178483 5.48249 35.69915298192417 140.45633787178483 11.28249 35.69907958872086 140.45641199894064 11.28249 35.69907958872086 140.45641199894064 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69915298192417 140.45633787178483 5.48249 35.69906586169044 140.45620812998567 5.48249 35.69906586169044 140.45620812998567 11.28249 35.69915298192417 140.45633787178483 11.28249 35.69915298192417 140.45633787178483 5.48249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906586169044 140.45620812998567 11.28249 35.69899092889474 140.4562836817062 11.28249 35.699035204894955 140.45634955261917 11.28249 35.69902116015145 140.4563638083686 11.28249 35.69904924584018 140.45640568537624 11.28249 35.69906483026063 140.4563900051061 11.28249 35.69907958872086 140.45641199894064 11.28249 35.69915298192417 140.45633787178483 11.28249 35.69906586169044 140.45620812998567 11.28249</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">173.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8332</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_2d45e950-420e-499e-ad13-b588e2d3e133">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">6.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698530831745465 140.45794510715402 0 35.698560491762805 140.45783981155833 0 35.698517876607475 140.45781881589542 0 35.69853826384352 140.45778085258303 0 35.698454337619914 140.45771334619826 0 35.69845825167835 140.45770597326262 0 35.69841103791828 140.4576679249771 0 35.698406941306544 140.4576757384839 0 35.69839428491075 140.45766558439982 0 35.69839009816762 140.4576733972025 0 35.69838381474894 140.45766837575997 0 35.69833302784805 140.45776311955936 0 35.69839029503921 140.4578092021874 0 35.698383832640985 140.4578213067049 0 35.69842099346802 140.45785121023718 0 35.69842672736425 140.45784053652568 0 35.69844934649731 140.45785883477015 0 35.698468551143634 140.4578229617652 0 35.69847689783583 140.45782987780072 0 35.69845366572067 140.45791235007846 0 35.698530831745465 140.45794510715402 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698530831745465 140.45794510715402 6.05386 35.69845366572067 140.45791235007846 6.05386 35.69847689783583 140.45782987780072 6.05386 35.698468551143634 140.4578229617652 6.05386 35.69844934649731 140.45785883477015 6.05386 35.69842672736425 140.45784053652568 6.05386 35.69842099346802 140.45785121023718 6.05386 35.698383832640985 140.4578213067049 6.05386 35.69839029503921 140.4578092021874 6.05386 35.69833302784805 140.45776311955936 6.05386 35.69838381474894 140.45766837575997 6.05386 35.69839009816762 140.4576733972025 6.05386 35.69839428491075 140.45766558439982 6.05386 35.698406941306544 140.4576757384839 6.05386 35.69841103791828 140.4576679249771 6.05386 35.69845825167835 140.45770597326262 6.05386 35.698454337619914 140.45771334619826 6.05386 35.69853826384352 140.45778085258303 6.05386 35.698517876607475 140.45781881589542 6.05386 35.698560491762805 140.45783981155833 6.05386 35.698530831745465 140.45794510715402 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698530831745465 140.45794510715402 6.05386 35.698560491762805 140.45783981155833 6.05386 35.698560491762805 140.45783981155833 10.353860000000001 35.698530831745465 140.45794510715402 10.353860000000001 35.698530831745465 140.45794510715402 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698560491762805 140.45783981155833 6.05386 35.698517876607475 140.45781881589542 6.05386 35.698517876607475 140.45781881589542 10.353860000000001 35.698560491762805 140.45783981155833 10.353860000000001 35.698560491762805 140.45783981155833 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698517876607475 140.45781881589542 6.05386 35.69853826384352 140.45778085258303 6.05386 35.69853826384352 140.45778085258303 10.353860000000001 35.698517876607475 140.45781881589542 10.353860000000001 35.698517876607475 140.45781881589542 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69853826384352 140.45778085258303 6.05386 35.698454337619914 140.45771334619826 6.05386 35.698454337619914 140.45771334619826 10.353860000000001 35.69853826384352 140.45778085258303 10.353860000000001 35.69853826384352 140.45778085258303 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698454337619914 140.45771334619826 6.05386 35.69845825167835 140.45770597326262 6.05386 35.69845825167835 140.45770597326262 10.353860000000001 35.698454337619914 140.45771334619826 10.353860000000001 35.698454337619914 140.45771334619826 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69845825167835 140.45770597326262 6.05386 35.69841103791828 140.4576679249771 6.05386 35.69841103791828 140.4576679249771 10.353860000000001 35.69845825167835 140.45770597326262 10.353860000000001 35.69845825167835 140.45770597326262 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69841103791828 140.4576679249771 6.05386 35.698406941306544 140.4576757384839 6.05386 35.698406941306544 140.4576757384839 10.353860000000001 35.69841103791828 140.4576679249771 10.353860000000001 35.69841103791828 140.4576679249771 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698406941306544 140.4576757384839 6.05386 35.69839428491075 140.45766558439982 6.05386 35.69839428491075 140.45766558439982 10.353860000000001 35.698406941306544 140.4576757384839 10.353860000000001 35.698406941306544 140.4576757384839 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69839428491075 140.45766558439982 6.05386 35.69839009816762 140.4576733972025 6.05386 35.69839009816762 140.4576733972025 10.353860000000001 35.69839428491075 140.45766558439982 10.353860000000001 35.69839428491075 140.45766558439982 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69839009816762 140.4576733972025 6.05386 35.69838381474894 140.45766837575997 6.05386 35.69838381474894 140.45766837575997 10.353860000000001 35.69839009816762 140.4576733972025 10.353860000000001 35.69839009816762 140.4576733972025 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69838381474894 140.45766837575997 6.05386 35.69833302784805 140.45776311955936 6.05386 35.69833302784805 140.45776311955936 10.353860000000001 35.69838381474894 140.45766837575997 10.353860000000001 35.69838381474894 140.45766837575997 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69833302784805 140.45776311955936 6.05386 35.69839029503921 140.4578092021874 6.05386 35.69839029503921 140.4578092021874 10.353860000000001 35.69833302784805 140.45776311955936 10.353860000000001 35.69833302784805 140.45776311955936 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69839029503921 140.4578092021874 6.05386 35.698383832640985 140.4578213067049 6.05386 35.698383832640985 140.4578213067049 10.353860000000001 35.69839029503921 140.4578092021874 10.353860000000001 35.69839029503921 140.4578092021874 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698383832640985 140.4578213067049 6.05386 35.69842099346802 140.45785121023718 6.05386 35.69842099346802 140.45785121023718 10.353860000000001 35.698383832640985 140.4578213067049 10.353860000000001 35.698383832640985 140.4578213067049 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69842099346802 140.45785121023718 6.05386 35.69842672736425 140.45784053652568 6.05386 35.69842672736425 140.45784053652568 10.353860000000001 35.69842099346802 140.45785121023718 10.353860000000001 35.69842099346802 140.45785121023718 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69842672736425 140.45784053652568 6.05386 35.69844934649731 140.45785883477015 6.05386 35.69844934649731 140.45785883477015 10.353860000000001 35.69842672736425 140.45784053652568 10.353860000000001 35.69842672736425 140.45784053652568 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69844934649731 140.45785883477015 6.05386 35.698468551143634 140.4578229617652 6.05386 35.698468551143634 140.4578229617652 10.353860000000001 35.69844934649731 140.45785883477015 10.353860000000001 35.69844934649731 140.45785883477015 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698468551143634 140.4578229617652 6.05386 35.69847689783583 140.45782987780072 6.05386 35.69847689783583 140.45782987780072 10.353860000000001 35.698468551143634 140.4578229617652 10.353860000000001 35.698468551143634 140.4578229617652 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69847689783583 140.45782987780072 6.05386 35.69845366572067 140.45791235007846 6.05386 35.69845366572067 140.45791235007846 10.353860000000001 35.69847689783583 140.45782987780072 10.353860000000001 35.69847689783583 140.45782987780072 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69845366572067 140.45791235007846 6.05386 35.698530831745465 140.45794510715402 6.05386 35.698530831745465 140.45794510715402 10.353860000000001 35.69845366572067 140.45791235007846 10.353860000000001 35.69845366572067 140.45791235007846 6.05386</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698530831745465 140.45794510715402 10.353860000000001 35.698560491762805 140.45783981155833 10.353860000000001 35.698517876607475 140.45781881589542 10.353860000000001 35.69853826384352 140.45778085258303 10.353860000000001 35.698454337619914 140.45771334619826 10.353860000000001 35.69845825167835 140.45770597326262 10.353860000000001 35.69841103791828 140.4576679249771 10.353860000000001 35.698406941306544 140.4576757384839 10.353860000000001 35.69839428491075 140.45766558439982 10.353860000000001 35.69839009816762 140.4576733972025 10.353860000000001 35.69838381474894 140.45766837575997 10.353860000000001 35.69833302784805 140.45776311955936 10.353860000000001 35.69839029503921 140.4578092021874 10.353860000000001 35.698383832640985 140.4578213067049 10.353860000000001 35.69842099346802 140.45785121023718 10.353860000000001 35.69842672736425 140.45784053652568 10.353860000000001 35.69844934649731 140.45785883477015 10.353860000000001 35.698468551143634 140.4578229617652 10.353860000000001 35.69847689783583 140.45782987780072 10.353860000000001 35.69845366572067 140.45791235007846 10.353860000000001 35.698530831745465 140.45794510715402 10.353860000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">299.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8255</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_f1f22be7-018a-4d29-ae2a-53a12d6a0f28">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698853557137895 140.45712020163464 0 35.69887625069498 140.45707197946345 0 35.69884796174261 140.4570519796892 0 35.69882526819353 140.45710020184924 0 35.698853557137895 140.45712020163464 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698853557137895 140.45712020163464 5.54843 35.69882526819353 140.45710020184924 5.54843 35.69884796174261 140.4570519796892 5.54843 35.69887625069498 140.45707197946345 5.54843 35.698853557137895 140.45712020163464 5.54843</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698853557137895 140.45712020163464 5.54843 35.69887625069498 140.45707197946345 5.54843 35.69887625069498 140.45707197946345 7.74843 35.698853557137895 140.45712020163464 7.74843 35.698853557137895 140.45712020163464 5.54843</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69887625069498 140.45707197946345 5.54843 35.69884796174261 140.4570519796892 5.54843 35.69884796174261 140.4570519796892 7.74843 35.69887625069498 140.45707197946345 7.74843 35.69887625069498 140.45707197946345 5.54843</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69884796174261 140.4570519796892 5.54843 35.69882526819353 140.45710020184924 5.54843 35.69882526819353 140.45710020184924 7.74843 35.69884796174261 140.4570519796892 7.74843 35.69884796174261 140.4570519796892 5.54843</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69882526819353 140.45710020184924 5.54843 35.698853557137895 140.45712020163464 5.54843 35.698853557137895 140.45712020163464 7.74843 35.69882526819353 140.45710020184924 7.74843 35.69882526819353 140.45710020184924 5.54843</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698853557137895 140.45712020163464 7.74843 35.69887625069498 140.45707197946345 7.74843 35.69884796174261 140.4570519796892 7.74843 35.69882526819353 140.45710020184924 7.74843 35.698853557137895 140.45712020163464 7.74843</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">18.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8335</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_879e6bef-222e-4c82-9f07-56d1dc4264bd">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">11.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69832719315607 140.45859192748708 0 35.6983125573885 140.4585810948226 0 35.69830088222497 140.45860763394174 0 35.69831579584661 140.4586170322912 0 35.69832719315607 140.45859192748708 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69832719315607 140.45859192748708 6.34008 35.69831579584661 140.4586170322912 6.34008 35.69830088222497 140.45860763394174 6.34008 35.6983125573885 140.4585810948226 6.34008 35.69832719315607 140.45859192748708 6.34008</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69832719315607 140.45859192748708 6.34008 35.6983125573885 140.4585810948226 6.34008 35.6983125573885 140.4585810948226 16.34008 35.69832719315607 140.45859192748708 16.34008 35.69832719315607 140.45859192748708 6.34008</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6983125573885 140.4585810948226 6.34008 35.69830088222497 140.45860763394174 6.34008 35.69830088222497 140.45860763394174 16.34008 35.6983125573885 140.4585810948226 16.34008 35.6983125573885 140.4585810948226 6.34008</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69830088222497 140.45860763394174 6.34008 35.69831579584661 140.4586170322912 6.34008 35.69831579584661 140.4586170322912 16.34008 35.69830088222497 140.45860763394174 16.34008 35.69830088222497 140.45860763394174 6.34008</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69831579584661 140.4586170322912 6.34008 35.69832719315607 140.45859192748708 6.34008 35.69832719315607 140.45859192748708 16.34008 35.69831579584661 140.4586170322912 16.34008 35.69831579584661 140.4586170322912 6.34008</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69832719315607 140.45859192748708 16.34008 35.6983125573885 140.4585810948226 16.34008 35.69830088222497 140.45860763394174 16.34008 35.69831579584661 140.4586170322912 16.34008 35.69832719315607 140.45859192748708 16.34008</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8311</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_9189e465-80e9-4889-ad21-e311cd4c0756">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.2</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698348517834056 140.4581130804351 0 35.69832372525936 140.4580967541714 0 35.698294080350934 140.45816436931577 0 35.69831887291644 140.4581806955941 0 35.698348517834056 140.4581130804351 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698348517834056 140.4581130804351 6.27556 35.69831887291644 140.4581806955941 6.27556 35.698294080350934 140.45816436931577 6.27556 35.69832372525936 140.4580967541714 6.27556 35.698348517834056 140.4581130804351 6.27556</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698348517834056 140.4581130804351 6.27556 35.69832372525936 140.4580967541714 6.27556 35.69832372525936 140.4580967541714 9.47556 35.698348517834056 140.4581130804351 9.47556 35.698348517834056 140.4581130804351 6.27556</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69832372525936 140.4580967541714 6.27556 35.698294080350934 140.45816436931577 6.27556 35.698294080350934 140.45816436931577 9.47556 35.69832372525936 140.4580967541714 9.47556 35.69832372525936 140.4580967541714 6.27556</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698294080350934 140.45816436931577 6.27556 35.69831887291644 140.4581806955941 6.27556 35.69831887291644 140.4581806955941 9.47556 35.698294080350934 140.45816436931577 9.47556 35.698294080350934 140.45816436931577 6.27556</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69831887291644 140.4581806955941 6.27556 35.698348517834056 140.4581130804351 6.27556 35.698348517834056 140.4581130804351 9.47556 35.69831887291644 140.4581806955941 9.47556 35.69831887291644 140.4581806955941 6.27556</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698348517834056 140.4581130804351 9.47556 35.69832372525936 140.4580967541714 9.47556 35.698294080350934 140.45816436931577 9.47556 35.69831887291644 140.4581806955941 9.47556 35.698348517834056 140.4581130804351 9.47556</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">21.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8213</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_24c16f92-1955-4aea-b101-e4304239a1ef">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69841125294142 140.45680912704077 0 35.69836296326807 140.45685692874156 0 35.69845105422691 140.456990545123 0 35.69849934395232 140.4569427434518 0 35.69841125294142 140.45680912704077 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69841125294142 140.45680912704077 5.93923 35.69849934395232 140.4569427434518 5.93923 35.69845105422691 140.456990545123 5.93923 35.69836296326807 140.45685692874156 5.93923 35.69841125294142 140.45680912704077 5.93923</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69841125294142 140.45680912704077 5.93923 35.69836296326807 140.45685692874156 5.93923 35.69836296326807 140.45685692874156 11.93923 35.69841125294142 140.45680912704077 11.93923 35.69841125294142 140.45680912704077 5.93923</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69836296326807 140.45685692874156 5.93923 35.69845105422691 140.456990545123 5.93923 35.69845105422691 140.456990545123 11.93923 35.69836296326807 140.45685692874156 11.93923 35.69836296326807 140.45685692874156 5.93923</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69845105422691 140.456990545123 5.93923 35.69849934395232 140.4569427434518 5.93923 35.69849934395232 140.4569427434518 11.93923 35.69845105422691 140.456990545123 11.93923 35.69845105422691 140.456990545123 5.93923</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69849934395232 140.4569427434518 5.93923 35.69841125294142 140.45680912704077 5.93923 35.69841125294142 140.45680912704077 11.93923 35.69849934395232 140.4569427434518 11.93923 35.69849934395232 140.4569427434518 5.93923</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69841125294142 140.45680912704077 11.93923 35.69836296326807 140.45685692874156 11.93923 35.69845105422691 140.456990545123 11.93923 35.69849934395232 140.4569427434518 11.93923 35.69841125294142 140.45680912704077 11.93923</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">107.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8197</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_187ea345-bd8a-43a5-9ab4-949b9a80f7ed">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69862053965342 140.45747143256918 0 35.69869644527769 140.45752130690207 0 35.69873722108628 140.45742770000737 0 35.6986614049818 140.45737793693397 0 35.69862053965342 140.45747143256918 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69862053965342 140.45747143256918 5.36204 35.6986614049818 140.45737793693397 5.36204 35.69873722108628 140.45742770000737 5.36204 35.69869644527769 140.45752130690207 5.36204 35.69862053965342 140.45747143256918 5.36204</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69862053965342 140.45747143256918 5.36204 35.69869644527769 140.45752130690207 5.36204 35.69869644527769 140.45752130690207 8.06204 35.69862053965342 140.45747143256918 8.06204 35.69862053965342 140.45747143256918 5.36204</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69869644527769 140.45752130690207 5.36204 35.69873722108628 140.45742770000737 5.36204 35.69873722108628 140.45742770000737 8.06204 35.69869644527769 140.45752130690207 8.06204 35.69869644527769 140.45752130690207 5.36204</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69873722108628 140.45742770000737 5.36204 35.6986614049818 140.45737793693397 5.36204 35.6986614049818 140.45737793693397 8.06204 35.69873722108628 140.45742770000737 8.06204 35.69873722108628 140.45742770000737 5.36204</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6986614049818 140.45737793693397 5.36204 35.69862053965342 140.45747143256918 5.36204 35.69862053965342 140.45747143256918 8.06204 35.6986614049818 140.45737793693397 8.06204 35.6986614049818 140.45737793693397 5.36204</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69862053965342 140.45747143256918 8.06204 35.69869644527769 140.45752130690207 8.06204 35.69873722108628 140.45742770000737 8.06204 35.6986614049818 140.45737793693397 8.06204 35.69862053965342 140.45747143256918 8.06204</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">91.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8217</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_7f281376-dfcb-4aed-98a2-b067044ce415">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69862567580501 140.45762816084576 0 35.69865766062804 140.4576480790763 0 35.698688238655365 140.4575742827315 0 35.69865625382006 140.45755436452245 0 35.69862567580501 140.45762816084576 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69862567580501 140.45762816084576 5.66776 35.69865625382006 140.45755436452245 5.66776 35.698688238655365 140.4575742827315 5.66776 35.69865766062804 140.4576480790763 5.66776 35.69862567580501 140.45762816084576 5.66776</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69862567580501 140.45762816084576 5.66776 35.69865766062804 140.4576480790763 5.66776 35.69865766062804 140.4576480790763 8.667760000000001 35.69862567580501 140.45762816084576 8.667760000000001 35.69862567580501 140.45762816084576 5.66776</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69865766062804 140.4576480790763 5.66776 35.698688238655365 140.4575742827315 5.66776 35.698688238655365 140.4575742827315 8.667760000000001 35.69865766062804 140.4576480790763 8.667760000000001 35.69865766062804 140.4576480790763 5.66776</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698688238655365 140.4575742827315 5.66776 35.69865625382006 140.45755436452245 5.66776 35.69865625382006 140.45755436452245 8.667760000000001 35.698688238655365 140.4575742827315 8.667760000000001 35.698688238655365 140.4575742827315 5.66776</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69865625382006 140.45755436452245 5.66776 35.69862567580501 140.45762816084576 5.66776 35.69862567580501 140.45762816084576 8.667760000000001 35.69865625382006 140.45755436452245 8.667760000000001 35.69865625382006 140.45755436452245 5.66776</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69862567580501 140.45762816084576 8.667760000000001 35.69865766062804 140.4576480790763 8.667760000000001 35.698688238655365 140.4575742827315 8.667760000000001 35.69865625382006 140.45755436452245 8.667760000000001 35.69862567580501 140.45762816084576 8.667760000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">29.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8193</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_dc286fdc-b4ce-4b9e-9c64-3daf20c2afdb">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69851101789048 140.45705554365176 0 35.69846353556993 140.45699970325177 0 35.69840020119922 140.45708075845434 0 35.69844759392368 140.4571364876603 0 35.69851101789048 140.45705554365176 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851101789048 140.45705554365176 5.88788 35.69844759392368 140.4571364876603 5.88788 35.69840020119922 140.45708075845434 5.88788 35.69846353556993 140.45699970325177 5.88788 35.69851101789048 140.45705554365176 5.88788</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851101789048 140.45705554365176 5.88788 35.69846353556993 140.45699970325177 5.88788 35.69846353556993 140.45699970325177 11.88788 35.69851101789048 140.45705554365176 11.88788 35.69851101789048 140.45705554365176 5.88788</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69846353556993 140.45699970325177 5.88788 35.69840020119922 140.45708075845434 5.88788 35.69840020119922 140.45708075845434 11.88788 35.69846353556993 140.45699970325177 11.88788 35.69846353556993 140.45699970325177 5.88788</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69840020119922 140.45708075845434 5.88788 35.69844759392368 140.4571364876603 5.88788 35.69844759392368 140.4571364876603 11.88788 35.69840020119922 140.45708075845434 11.88788 35.69840020119922 140.45708075845434 5.88788</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69844759392368 140.4571364876603 5.88788 35.69851101789048 140.45705554365176 5.88788 35.69851101789048 140.45705554365176 11.88788 35.69844759392368 140.4571364876603 11.88788 35.69844759392368 140.4571364876603 5.88788</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851101789048 140.45705554365176 11.88788 35.69846353556993 140.45699970325177 11.88788 35.69840020119922 140.45708075845434 11.88788 35.69844759392368 140.4571364876603 11.88788 35.69851101789048 140.45705554365176 11.88788</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">74.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8252</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_42f19ca7-fa78-4d98-a97e-ca5c3970a4e1">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">1.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69905781220212 140.45675316425493 0 35.69903868628621 140.45673909237217 0 35.69901955606649 140.4567780604727 0 35.69903868197784 140.45679213236124 0 35.69905781220212 140.45675316425493 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69905781220212 140.45675316425493 5.63279 35.69903868197784 140.45679213236124 5.63279 35.69901955606649 140.4567780604727 5.63279 35.69903868628621 140.45673909237217 5.63279 35.69905781220212 140.45675316425493 5.63279</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69905781220212 140.45675316425493 5.63279 35.69903868628621 140.45673909237217 5.63279 35.69903868628621 140.45673909237217 7.13279 35.69905781220212 140.45675316425493 7.13279 35.69905781220212 140.45675316425493 5.63279</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69903868628621 140.45673909237217 5.63279 35.69901955606649 140.4567780604727 5.63279 35.69901955606649 140.4567780604727 7.13279 35.69903868628621 140.45673909237217 7.13279 35.69903868628621 140.45673909237217 5.63279</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69901955606649 140.4567780604727 5.63279 35.69903868197784 140.45679213236124 5.63279 35.69903868197784 140.45679213236124 7.13279 35.69901955606649 140.4567780604727 7.13279 35.69901955606649 140.4567780604727 5.63279</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69903868197784 140.45679213236124 5.63279 35.69905781220212 140.45675316425493 5.63279 35.69905781220212 140.45675316425493 7.13279 35.69903868197784 140.45679213236124 7.13279 35.69903868197784 140.45679213236124 5.63279</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69905781220212 140.45675316425493 7.13279 35.69903868628621 140.45673909237217 7.13279 35.69901955606649 140.4567780604727 7.13279 35.69903868197784 140.45679213236124 7.13279 35.69905781220212 140.45675316425493 7.13279</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">10.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8329</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b481bd6a-efba-4f81-a711-ef1c2e286699">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.694889359841234 140.46040857829072 0 35.69496924175446 140.4605254437623 0 35.69493915630275 140.4605562571112 0 35.69499506476905 140.46063801880325 0 35.69503874309687 140.46059327910808 0 35.695018436956204 140.46056361806959 0 35.69507570867792 140.46050484145292 0 35.694960134758844 140.4603357640731 0 35.694889359841234 140.46040857829072 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.694889359841234 140.46040857829072 6.21027 35.694960134758844 140.4603357640731 6.21027 35.69507570867792 140.46050484145292 6.21027 35.695018436956204 140.46056361806959 6.21027 35.69503874309687 140.46059327910808 6.21027 35.69499506476905 140.46063801880325 6.21027 35.69493915630275 140.4605562571112 6.21027 35.69496924175446 140.4605254437623 6.21027 35.694889359841234 140.46040857829072 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.694889359841234 140.46040857829072 6.21027 35.69496924175446 140.4605254437623 6.21027 35.69496924175446 140.4605254437623 12.210270000000001 35.694889359841234 140.46040857829072 12.210270000000001 35.694889359841234 140.46040857829072 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69496924175446 140.4605254437623 6.21027 35.69493915630275 140.4605562571112 6.21027 35.69493915630275 140.4605562571112 12.210270000000001 35.69496924175446 140.4605254437623 12.210270000000001 35.69496924175446 140.4605254437623 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69493915630275 140.4605562571112 6.21027 35.69499506476905 140.46063801880325 6.21027 35.69499506476905 140.46063801880325 12.210270000000001 35.69493915630275 140.4605562571112 12.210270000000001 35.69493915630275 140.4605562571112 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69499506476905 140.46063801880325 6.21027 35.69503874309687 140.46059327910808 6.21027 35.69503874309687 140.46059327910808 12.210270000000001 35.69499506476905 140.46063801880325 12.210270000000001 35.69499506476905 140.46063801880325 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69503874309687 140.46059327910808 6.21027 35.695018436956204 140.46056361806959 6.21027 35.695018436956204 140.46056361806959 12.210270000000001 35.69503874309687 140.46059327910808 12.210270000000001 35.69503874309687 140.46059327910808 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695018436956204 140.46056361806959 6.21027 35.69507570867792 140.46050484145292 6.21027 35.69507570867792 140.46050484145292 12.210270000000001 35.695018436956204 140.46056361806959 12.210270000000001 35.695018436956204 140.46056361806959 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69507570867792 140.46050484145292 6.21027 35.694960134758844 140.4603357640731 6.21027 35.694960134758844 140.4603357640731 12.210270000000001 35.69507570867792 140.46050484145292 12.210270000000001 35.69507570867792 140.46050484145292 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.694960134758844 140.4603357640731 6.21027 35.694889359841234 140.46040857829072 6.21027 35.694889359841234 140.46040857829072 12.210270000000001 35.694960134758844 140.4603357640731 12.210270000000001 35.694960134758844 140.4603357640731 6.21027</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.694889359841234 140.46040857829072 12.210270000000001 35.69496924175446 140.4605254437623 12.210270000000001 35.69493915630275 140.4605562571112 12.210270000000001 35.69499506476905 140.46063801880325 12.210270000000001 35.69503874309687 140.46059327910808 12.210270000000001 35.695018436956204 140.46056361806959 12.210270000000001 35.69507570867792 140.46050484145292 12.210270000000001 35.694960134758844 140.4603357640731 12.210270000000001 35.694889359841234 140.46040857829072 12.210270000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">4</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">253.5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8315</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_a20d4fa0-28d7-4b1e-818c-5658abac8e6f">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.2</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698582983215296 140.45715268058365 0 35.69853181814434 140.45709438040805 0 35.69850813003994 140.45712579872512 0 35.69848402608289 140.4570983176687 0 35.698436288765066 140.45716126193142 0 35.69846702407409 140.4571961980938 0 35.6984753745915 140.45718499221414 0 35.69851999896009 140.45723572749924 0 35.698582983215296 140.45715268058365 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698582983215296 140.45715268058365 5.75972 35.69851999896009 140.45723572749924 5.75972 35.6984753745915 140.45718499221414 5.75972 35.69846702407409 140.4571961980938 5.75972 35.698436288765066 140.45716126193142 5.75972 35.69848402608289 140.4570983176687 5.75972 35.69850813003994 140.45712579872512 5.75972 35.69853181814434 140.45709438040805 5.75972 35.698582983215296 140.45715268058365 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698582983215296 140.45715268058365 5.75972 35.69853181814434 140.45709438040805 5.75972 35.69853181814434 140.45709438040805 9.059719999999999 35.698582983215296 140.45715268058365 9.059719999999999 35.698582983215296 140.45715268058365 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69853181814434 140.45709438040805 5.75972 35.69850813003994 140.45712579872512 5.75972 35.69850813003994 140.45712579872512 9.059719999999999 35.69853181814434 140.45709438040805 9.059719999999999 35.69853181814434 140.45709438040805 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69850813003994 140.45712579872512 5.75972 35.69848402608289 140.4570983176687 5.75972 35.69848402608289 140.4570983176687 9.059719999999999 35.69850813003994 140.45712579872512 9.059719999999999 35.69850813003994 140.45712579872512 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69848402608289 140.4570983176687 5.75972 35.698436288765066 140.45716126193142 5.75972 35.698436288765066 140.45716126193142 9.059719999999999 35.69848402608289 140.4570983176687 9.059719999999999 35.69848402608289 140.4570983176687 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698436288765066 140.45716126193142 5.75972 35.69846702407409 140.4571961980938 5.75972 35.69846702407409 140.4571961980938 9.059719999999999 35.698436288765066 140.45716126193142 9.059719999999999 35.698436288765066 140.45716126193142 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69846702407409 140.4571961980938 5.75972 35.6984753745915 140.45718499221414 5.75972 35.6984753745915 140.45718499221414 9.059719999999999 35.69846702407409 140.4571961980938 9.059719999999999 35.69846702407409 140.4571961980938 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6984753745915 140.45718499221414 5.75972 35.69851999896009 140.45723572749924 5.75972 35.69851999896009 140.45723572749924 9.059719999999999 35.6984753745915 140.45718499221414 9.059719999999999 35.6984753745915 140.45718499221414 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851999896009 140.45723572749924 5.75972 35.698582983215296 140.45715268058365 5.75972 35.698582983215296 140.45715268058365 9.059719999999999 35.69851999896009 140.45723572749924 9.059719999999999 35.69851999896009 140.45723572749924 5.75972</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698582983215296 140.45715268058365 9.059719999999999 35.69853181814434 140.45709438040805 9.059719999999999 35.69850813003994 140.45712579872512 9.059719999999999 35.69848402608289 140.4570983176687 9.059719999999999 35.698436288765066 140.45716126193142 9.059719999999999 35.69846702407409 140.4571961980938 9.059719999999999 35.6984753745915 140.45718499221414 9.059719999999999 35.69851999896009 140.45723572749924 9.059719999999999 35.698582983215296 140.45715268058365 9.059719999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">109.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8194</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_ba646d41-1963-4e74-8317-07704548e7a1">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.695441283023634 140.46092261276726 0 35.69543458532552 140.46094521177488 0 35.69546839759053 140.460960283048 0 35.695475095867586 140.4609375735416 0 35.695441283023634 140.46092261276726 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695441283023634 140.46092261276726 6.90478 35.695475095867586 140.4609375735416 6.90478 35.69546839759053 140.460960283048 6.90478 35.69543458532552 140.46094521177488 6.90478 35.695441283023634 140.46092261276726 6.90478</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695441283023634 140.46092261276726 6.90478 35.69543458532552 140.46094521177488 6.90478 35.69543458532552 140.46094521177488 12.904779999999999 35.695441283023634 140.46092261276726 12.904779999999999 35.695441283023634 140.46092261276726 6.90478</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69543458532552 140.46094521177488 6.90478 35.69546839759053 140.460960283048 6.90478 35.69546839759053 140.460960283048 12.904779999999999 35.69543458532552 140.46094521177488 12.904779999999999 35.69543458532552 140.46094521177488 6.90478</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69546839759053 140.460960283048 6.90478 35.695475095867586 140.4609375735416 6.90478 35.695475095867586 140.4609375735416 12.904779999999999 35.69546839759053 140.460960283048 12.904779999999999 35.69546839759053 140.460960283048 6.90478</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695475095867586 140.4609375735416 6.90478 35.695441283023634 140.46092261276726 6.90478 35.695441283023634 140.46092261276726 12.904779999999999 35.695475095867586 140.4609375735416 12.904779999999999 35.695475095867586 140.4609375735416 6.90478</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695441283023634 140.46092261276726 12.904779999999999 35.69543458532552 140.46094521177488 12.904779999999999 35.69546839759053 140.460960283048 12.904779999999999 35.695475095867586 140.4609375735416 12.904779999999999 35.695441283023634 140.46092261276726 12.904779999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">3</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">8.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8203</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_d816dc06-060d-484c-8f67-e7746ba10c99">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.697483421830505 140.460483473103 0 35.6974699734866 140.46046966606596 0 35.697457984591615 140.46048714134977 0 35.69747152306506 140.4605009490934 0 35.697483421830505 140.460483473103 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697483421830505 140.460483473103 6.43141 35.69747152306506 140.4605009490934 6.43141 35.697457984591615 140.46048714134977 6.43141 35.6974699734866 140.46046966606596 6.43141 35.697483421830505 140.460483473103 6.43141</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697483421830505 140.460483473103 6.43141 35.6974699734866 140.46046966606596 6.43141 35.6974699734866 140.46046966606596 12.43141 35.697483421830505 140.460483473103 12.43141 35.697483421830505 140.460483473103 6.43141</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6974699734866 140.46046966606596 6.43141 35.697457984591615 140.46048714134977 6.43141 35.697457984591615 140.46048714134977 12.43141 35.6974699734866 140.46046966606596 12.43141 35.6974699734866 140.46046966606596 6.43141</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697457984591615 140.46048714134977 6.43141 35.69747152306506 140.4605009490934 6.43141 35.69747152306506 140.4605009490934 12.43141 35.697457984591615 140.46048714134977 12.43141 35.697457984591615 140.46048714134977 6.43141</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69747152306506 140.4605009490934 6.43141 35.697483421830505 140.460483473103 6.43141 35.697483421830505 140.460483473103 12.43141 35.69747152306506 140.4605009490934 12.43141 35.69747152306506 140.4605009490934 6.43141</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697483421830505 140.460483473103 12.43141 35.6974699734866 140.46046966606596 12.43141 35.697457984591615 140.46048714134977 12.43141 35.69747152306506 140.4605009490934 12.43141 35.697483421830505 140.460483473103 12.43141</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">4</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">214</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8267</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_9ee61a2e-17bf-4183-a9e0-b53ebcabe6c0">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69932377495566 140.45634041587394 0 35.699331194680596 140.45635229714023 0 35.699289460435956 140.45638744311225 0 35.699296694745485 140.45640031743892 0 35.69927922270874 140.45641498856284 0 35.69929994275724 140.45645194639891 0 35.699367658030205 140.4563951234869 0 35.69936586626616 140.45639301003806 0 35.699408302351564 140.45634416742638 0 35.69941161491469 140.45634850271696 0 35.69946765249439 140.45628396426125 0 35.69946425036928 140.45627951777303 0 35.69948592162276 140.45625460268582 0 35.69945064708804 140.456208470566 0 35.69943423549637 140.45622723852938 0 35.6994015576129 140.45618444166234 0 35.69934388799645 140.4562508459498 0 35.69937394460689 140.45629505892995 0 35.699352105663586 140.45631754165674 0 35.69934960028018 140.45631398615282 0 35.69932377495566 140.45634041587394 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69932377495566 140.45634041587394 5.3023 35.69934960028018 140.45631398615282 5.3023 35.699352105663586 140.45631754165674 5.3023 35.69937394460689 140.45629505892995 5.3023 35.69934388799645 140.4562508459498 5.3023 35.6994015576129 140.45618444166234 5.3023 35.69943423549637 140.45622723852938 5.3023 35.69945064708804 140.456208470566 5.3023 35.69948592162276 140.45625460268582 5.3023 35.69946425036928 140.45627951777303 5.3023 35.69946765249439 140.45628396426125 5.3023 35.69941161491469 140.45634850271696 5.3023 35.699408302351564 140.45634416742638 5.3023 35.69936586626616 140.45639301003806 5.3023 35.699367658030205 140.4563951234869 5.3023 35.69929994275724 140.45645194639891 5.3023 35.69927922270874 140.45641498856284 5.3023 35.699296694745485 140.45640031743892 5.3023 35.699289460435956 140.45638744311225 5.3023 35.699331194680596 140.45635229714023 5.3023 35.69932377495566 140.45634041587394 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69932377495566 140.45634041587394 5.3023 35.699331194680596 140.45635229714023 5.3023 35.699331194680596 140.45635229714023 8.1023 35.69932377495566 140.45634041587394 8.1023 35.69932377495566 140.45634041587394 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699331194680596 140.45635229714023 5.3023 35.699289460435956 140.45638744311225 5.3023 35.699289460435956 140.45638744311225 8.1023 35.699331194680596 140.45635229714023 8.1023 35.699331194680596 140.45635229714023 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699289460435956 140.45638744311225 5.3023 35.699296694745485 140.45640031743892 5.3023 35.699296694745485 140.45640031743892 8.1023 35.699289460435956 140.45638744311225 8.1023 35.699289460435956 140.45638744311225 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699296694745485 140.45640031743892 5.3023 35.69927922270874 140.45641498856284 5.3023 35.69927922270874 140.45641498856284 8.1023 35.699296694745485 140.45640031743892 8.1023 35.699296694745485 140.45640031743892 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69927922270874 140.45641498856284 5.3023 35.69929994275724 140.45645194639891 5.3023 35.69929994275724 140.45645194639891 8.1023 35.69927922270874 140.45641498856284 8.1023 35.69927922270874 140.45641498856284 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69929994275724 140.45645194639891 5.3023 35.699367658030205 140.4563951234869 5.3023 35.699367658030205 140.4563951234869 8.1023 35.69929994275724 140.45645194639891 8.1023 35.69929994275724 140.45645194639891 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699367658030205 140.4563951234869 5.3023 35.69936586626616 140.45639301003806 5.3023 35.69936586626616 140.45639301003806 8.1023 35.699367658030205 140.4563951234869 8.1023 35.699367658030205 140.4563951234869 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69936586626616 140.45639301003806 5.3023 35.699408302351564 140.45634416742638 5.3023 35.699408302351564 140.45634416742638 8.1023 35.69936586626616 140.45639301003806 8.1023 35.69936586626616 140.45639301003806 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699408302351564 140.45634416742638 5.3023 35.69941161491469 140.45634850271696 5.3023 35.69941161491469 140.45634850271696 8.1023 35.699408302351564 140.45634416742638 8.1023 35.699408302351564 140.45634416742638 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69941161491469 140.45634850271696 5.3023 35.69946765249439 140.45628396426125 5.3023 35.69946765249439 140.45628396426125 8.1023 35.69941161491469 140.45634850271696 8.1023 35.69941161491469 140.45634850271696 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69946765249439 140.45628396426125 5.3023 35.69946425036928 140.45627951777303 5.3023 35.69946425036928 140.45627951777303 8.1023 35.69946765249439 140.45628396426125 8.1023 35.69946765249439 140.45628396426125 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69946425036928 140.45627951777303 5.3023 35.69948592162276 140.45625460268582 5.3023 35.69948592162276 140.45625460268582 8.1023 35.69946425036928 140.45627951777303 8.1023 35.69946425036928 140.45627951777303 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69948592162276 140.45625460268582 5.3023 35.69945064708804 140.456208470566 5.3023 35.69945064708804 140.456208470566 8.1023 35.69948592162276 140.45625460268582 8.1023 35.69948592162276 140.45625460268582 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69945064708804 140.456208470566 5.3023 35.69943423549637 140.45622723852938 5.3023 35.69943423549637 140.45622723852938 8.1023 35.69945064708804 140.456208470566 8.1023 35.69945064708804 140.456208470566 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69943423549637 140.45622723852938 5.3023 35.6994015576129 140.45618444166234 5.3023 35.6994015576129 140.45618444166234 8.1023 35.69943423549637 140.45622723852938 8.1023 35.69943423549637 140.45622723852938 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6994015576129 140.45618444166234 5.3023 35.69934388799645 140.4562508459498 5.3023 35.69934388799645 140.4562508459498 8.1023 35.6994015576129 140.45618444166234 8.1023 35.6994015576129 140.45618444166234 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69934388799645 140.4562508459498 5.3023 35.69937394460689 140.45629505892995 5.3023 35.69937394460689 140.45629505892995 8.1023 35.69934388799645 140.4562508459498 8.1023 35.69934388799645 140.4562508459498 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69937394460689 140.45629505892995 5.3023 35.699352105663586 140.45631754165674 5.3023 35.699352105663586 140.45631754165674 8.1023 35.69937394460689 140.45629505892995 8.1023 35.69937394460689 140.45629505892995 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699352105663586 140.45631754165674 5.3023 35.69934960028018 140.45631398615282 5.3023 35.69934960028018 140.45631398615282 8.1023 35.699352105663586 140.45631754165674 8.1023 35.699352105663586 140.45631754165674 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69934960028018 140.45631398615282 5.3023 35.69932377495566 140.45634041587394 5.3023 35.69932377495566 140.45634041587394 8.1023 35.69934960028018 140.45631398615282 8.1023 35.69934960028018 140.45631398615282 5.3023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69932377495566 140.45634041587394 8.1023 35.699331194680596 140.45635229714023 8.1023 35.699289460435956 140.45638744311225 8.1023 35.699296694745485 140.45640031743892 8.1023 35.69927922270874 140.45641498856284 8.1023 35.69929994275724 140.45645194639891 8.1023 35.699367658030205 140.4563951234869 8.1023 35.69936586626616 140.45639301003806 8.1023 35.699408302351564 140.45634416742638 8.1023 35.69941161491469 140.45634850271696 8.1023 35.69946765249439 140.45628396426125 8.1023 35.69946425036928 140.45627951777303 8.1023 35.69948592162276 140.45625460268582 8.1023 35.69945064708804 140.456208470566 8.1023 35.69943423549637 140.45622723852938 8.1023 35.6994015576129 140.45618444166234 8.1023 35.69934388799645 140.4562508459498 8.1023 35.69937394460689 140.45629505892995 8.1023 35.699352105663586 140.45631754165674 8.1023 35.69934960028018 140.45631398615282 8.1023 35.69932377495566 140.45634041587394 8.1023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">204.9</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8233</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_72d5408d-20bb-4283-8f69-77d410ed43c4">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69837045521908 140.45767213908798 0 35.69832655167195 140.4576388681269 0 35.69830505883059 140.45768146371685 0 35.69834896236596 140.45771473469196 0 35.69837045521908 140.45767213908798 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69837045521908 140.45767213908798 6.42148 35.69834896236596 140.45771473469196 6.42148 35.69830505883059 140.45768146371685 6.42148 35.69832655167195 140.4576388681269 6.42148 35.69837045521908 140.45767213908798 6.42148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69837045521908 140.45767213908798 6.42148 35.69832655167195 140.4576388681269 6.42148 35.69832655167195 140.4576388681269 9.421479999999999 35.69837045521908 140.45767213908798 9.421479999999999 35.69837045521908 140.45767213908798 6.42148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69832655167195 140.4576388681269 6.42148 35.69830505883059 140.45768146371685 6.42148 35.69830505883059 140.45768146371685 9.421479999999999 35.69832655167195 140.4576388681269 9.421479999999999 35.69832655167195 140.4576388681269 6.42148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69830505883059 140.45768146371685 6.42148 35.69834896236596 140.45771473469196 6.42148 35.69834896236596 140.45771473469196 9.421479999999999 35.69830505883059 140.45768146371685 9.421479999999999 35.69830505883059 140.45768146371685 6.42148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69834896236596 140.45771473469196 6.42148 35.69837045521908 140.45767213908798 6.42148 35.69837045521908 140.45767213908798 9.421479999999999 35.69834896236596 140.45771473469196 9.421479999999999 35.69834896236596 140.45771473469196 6.42148</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69837045521908 140.45767213908798 9.421479999999999 35.69832655167195 140.4576388681269 9.421479999999999 35.69830505883059 140.45768146371685 9.421479999999999 35.69834896236596 140.45771473469196 9.421479999999999 35.69837045521908 140.45767213908798 9.421479999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">26</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-9095</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_5cce18d1-51b4-49b7-8256-a65b6d1468c6">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">7.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69916745746445 140.45536889736303 0 35.69911869512508 140.4554733832949 0 35.69917868770645 140.45551550791635 0 35.69917130537237 140.45553125209992 0 35.699207588303125 140.45555672804127 0 35.699214607258355 140.4555415335314 0 35.6992346352033 140.45555550166844 0 35.699283670864254 140.45545046525797 0 35.69916745746445 140.45536889736303 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69916745746445 140.45536889736303 7.65184 35.699283670864254 140.45545046525797 7.65184 35.6992346352033 140.45555550166844 7.65184 35.699214607258355 140.4555415335314 7.65184 35.699207588303125 140.45555672804127 7.65184 35.69917130537237 140.45553125209992 7.65184 35.69917868770645 140.45551550791635 7.65184 35.69911869512508 140.4554733832949 7.65184 35.69916745746445 140.45536889736303 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69916745746445 140.45536889736303 7.65184 35.69911869512508 140.4554733832949 7.65184 35.69911869512508 140.4554733832949 12.85184 35.69916745746445 140.45536889736303 12.85184 35.69916745746445 140.45536889736303 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69911869512508 140.4554733832949 7.65184 35.69917868770645 140.45551550791635 7.65184 35.69917868770645 140.45551550791635 12.85184 35.69911869512508 140.4554733832949 12.85184 35.69911869512508 140.4554733832949 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69917868770645 140.45551550791635 7.65184 35.69917130537237 140.45553125209992 7.65184 35.69917130537237 140.45553125209992 12.85184 35.69917868770645 140.45551550791635 12.85184 35.69917868770645 140.45551550791635 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69917130537237 140.45553125209992 7.65184 35.699207588303125 140.45555672804127 7.65184 35.699207588303125 140.45555672804127 12.85184 35.69917130537237 140.45553125209992 12.85184 35.69917130537237 140.45553125209992 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699207588303125 140.45555672804127 7.65184 35.699214607258355 140.4555415335314 7.65184 35.699214607258355 140.4555415335314 12.85184 35.699207588303125 140.45555672804127 12.85184 35.699207588303125 140.45555672804127 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699214607258355 140.4555415335314 7.65184 35.6992346352033 140.45555550166844 7.65184 35.6992346352033 140.45555550166844 12.85184 35.699214607258355 140.4555415335314 12.85184 35.699214607258355 140.4555415335314 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6992346352033 140.45555550166844 7.65184 35.699283670864254 140.45545046525797 7.65184 35.699283670864254 140.45545046525797 12.85184 35.6992346352033 140.45555550166844 12.85184 35.6992346352033 140.45555550166844 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699283670864254 140.45545046525797 7.65184 35.69916745746445 140.45536889736303 7.65184 35.69916745746445 140.45536889736303 12.85184 35.699283670864254 140.45545046525797 12.85184 35.699283670864254 140.45545046525797 7.65184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69916745746445 140.45536889736303 12.85184 35.69911869512508 140.4554733832949 12.85184 35.69917868770645 140.45551550791635 12.85184 35.69917130537237 140.45553125209992 12.85184 35.699207588303125 140.45555672804127 12.85184 35.699214607258355 140.4555415335314 12.85184 35.6992346352033 140.45555550166844 12.85184 35.699283670864254 140.45545046525797 12.85184 35.69916745746445 140.45536889736303 12.85184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">169.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8265</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_dada1197-ce8b-4ef5-87c9-d355628466e8">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">2.9</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698791937803996 140.45608677044152 0 35.69881354575422 140.4560566615282 0 35.698789345132695 140.45603039543084 0 35.69876764648574 140.45606061413676 0 35.698791937803996 140.45608677044152 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698791937803996 140.45608677044152 6.27241 35.69876764648574 140.45606061413676 6.27241 35.698789345132695 140.45603039543084 6.27241 35.69881354575422 140.4560566615282 6.27241 35.698791937803996 140.45608677044152 6.27241</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698791937803996 140.45608677044152 6.27241 35.69881354575422 140.4560566615282 6.27241 35.69881354575422 140.4560566615282 7.57241 35.698791937803996 140.45608677044152 7.57241 35.698791937803996 140.45608677044152 6.27241</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69881354575422 140.4560566615282 6.27241 35.698789345132695 140.45603039543084 6.27241 35.698789345132695 140.45603039543084 7.57241 35.69881354575422 140.4560566615282 7.57241 35.69881354575422 140.4560566615282 6.27241</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698789345132695 140.45603039543084 6.27241 35.69876764648574 140.45606061413676 6.27241 35.69876764648574 140.45606061413676 7.57241 35.698789345132695 140.45603039543084 7.57241 35.698789345132695 140.45603039543084 6.27241</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69876764648574 140.45606061413676 6.27241 35.698791937803996 140.45608677044152 6.27241 35.698791937803996 140.45608677044152 7.57241 35.69876764648574 140.45606061413676 7.57241 35.69876764648574 140.45606061413676 6.27241</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698791937803996 140.45608677044152 7.57241 35.69881354575422 140.4560566615282 7.57241 35.698789345132695 140.45603039543084 7.57241 35.69876764648574 140.45606061413676 7.57241 35.698791937803996 140.45608677044152 7.57241</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">13</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8238</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_5fb9c8f4-260f-43b2-9c43-5eede6710b02">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69930188601043 140.45544894924404 0 35.69932415868885 140.45546459231733 0 35.69934229589747 140.4554258370405 0 35.69932002321398 140.45541019397427 0 35.69930188601043 140.45544894924404 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69930188601043 140.45544894924404 7.61956 35.69932002321398 140.45541019397427 7.61956 35.69934229589747 140.4554258370405 7.61956 35.69932415868885 140.45546459231733 7.61956 35.69930188601043 140.45544894924404 7.61956</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69930188601043 140.45544894924404 7.61956 35.69932415868885 140.45546459231733 7.61956 35.69932415868885 140.45546459231733 10.21956 35.69930188601043 140.45544894924404 10.21956 35.69930188601043 140.45544894924404 7.61956</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69932415868885 140.45546459231733 7.61956 35.69934229589747 140.4554258370405 7.61956 35.69934229589747 140.4554258370405 10.21956 35.69932415868885 140.45546459231733 10.21956 35.69932415868885 140.45546459231733 7.61956</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69934229589747 140.4554258370405 7.61956 35.69932002321398 140.45541019397427 7.61956 35.69932002321398 140.45541019397427 10.21956 35.69934229589747 140.4554258370405 10.21956 35.69934229589747 140.4554258370405 7.61956</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69932002321398 140.45541019397427 7.61956 35.69930188601043 140.45544894924404 7.61956 35.69930188601043 140.45544894924404 10.21956 35.69932002321398 140.45541019397427 10.21956 35.69932002321398 140.45541019397427 7.61956</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69930188601043 140.45544894924404 10.21956 35.69932415868885 140.45546459231733 10.21956 35.69934229589747 140.4554258370405 10.21956 35.69932002321398 140.45541019397427 10.21956 35.69930188601043 140.45544894924404 10.21956</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">11.5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8229</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_4fee9632-bad1-494d-9449-734d03aece7f">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69851145323379 140.45672791656196 0 35.6984678475614 140.4566718855277 0 35.698451252201615 140.45669131473346 0 35.698434239355905 140.45666952447507 0 35.698396514587714 140.45671365153336 0 35.69841200491722 140.45673355145385 0 35.698394956600595 140.45675341910427 0 35.69843999461116 140.4568113397679 0 35.69851145323379 140.45672791656196 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851145323379 140.45672791656196 6.01606 35.69843999461116 140.4568113397679 6.01606 35.698394956600595 140.45675341910427 6.01606 35.69841200491722 140.45673355145385 6.01606 35.698396514587714 140.45671365153336 6.01606 35.698434239355905 140.45666952447507 6.01606 35.698451252201615 140.45669131473346 6.01606 35.6984678475614 140.4566718855277 6.01606 35.69851145323379 140.45672791656196 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851145323379 140.45672791656196 6.01606 35.6984678475614 140.4566718855277 6.01606 35.6984678475614 140.4566718855277 8.616060000000001 35.69851145323379 140.45672791656196 8.616060000000001 35.69851145323379 140.45672791656196 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6984678475614 140.4566718855277 6.01606 35.698451252201615 140.45669131473346 6.01606 35.698451252201615 140.45669131473346 8.616060000000001 35.6984678475614 140.4566718855277 8.616060000000001 35.6984678475614 140.4566718855277 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698451252201615 140.45669131473346 6.01606 35.698434239355905 140.45666952447507 6.01606 35.698434239355905 140.45666952447507 8.616060000000001 35.698451252201615 140.45669131473346 8.616060000000001 35.698451252201615 140.45669131473346 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698434239355905 140.45666952447507 6.01606 35.698396514587714 140.45671365153336 6.01606 35.698396514587714 140.45671365153336 8.616060000000001 35.698434239355905 140.45666952447507 8.616060000000001 35.698434239355905 140.45666952447507 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698396514587714 140.45671365153336 6.01606 35.69841200491722 140.45673355145385 6.01606 35.69841200491722 140.45673355145385 8.616060000000001 35.698396514587714 140.45671365153336 8.616060000000001 35.698396514587714 140.45671365153336 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69841200491722 140.45673355145385 6.01606 35.698394956600595 140.45675341910427 6.01606 35.698394956600595 140.45675341910427 8.616060000000001 35.69841200491722 140.45673355145385 8.616060000000001 35.69841200491722 140.45673355145385 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698394956600595 140.45675341910427 6.01606 35.69843999461116 140.4568113397679 6.01606 35.69843999461116 140.4568113397679 8.616060000000001 35.698394956600595 140.45675341910427 8.616060000000001 35.698394956600595 140.45675341910427 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69843999461116 140.4568113397679 6.01606 35.69851145323379 140.45672791656196 6.01606 35.69851145323379 140.45672791656196 8.616060000000001 35.69843999461116 140.4568113397679 8.616060000000001 35.69843999461116 140.4568113397679 6.01606</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851145323379 140.45672791656196 8.616060000000001 35.6984678475614 140.4566718855277 8.616060000000001 35.698451252201615 140.45669131473346 8.616060000000001 35.698434239355905 140.45666952447507 8.616060000000001 35.698396514587714 140.45671365153336 8.616060000000001 35.69841200491722 140.45673355145385 8.616060000000001 35.698394956600595 140.45675341910427 8.616060000000001 35.69843999461116 140.4568113397679 8.616060000000001 35.69851145323379 140.45672791656196 8.616060000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">93.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8206</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_2945db21-8af8-411e-94f9-990cbdce77fe">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69887232689954 140.4562109343202 0 35.69885089237499 140.45619010415834 0 35.69882127104536 140.45623595219104 0 35.698842705562036 140.45625678235726 0 35.69887232689954 140.4562109343202 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69887232689954 140.4562109343202 6.1456 35.698842705562036 140.45625678235726 6.1456 35.69882127104536 140.45623595219104 6.1456 35.69885089237499 140.45619010415834 6.1456 35.69887232689954 140.4562109343202 6.1456</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69887232689954 140.4562109343202 6.1456 35.69885089237499 140.45619010415834 6.1456 35.69885089237499 140.45619010415834 9.1456 35.69887232689954 140.4562109343202 9.1456 35.69887232689954 140.4562109343202 6.1456</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69885089237499 140.45619010415834 6.1456 35.69882127104536 140.45623595219104 6.1456 35.69882127104536 140.45623595219104 9.1456 35.69885089237499 140.45619010415834 9.1456 35.69885089237499 140.45619010415834 6.1456</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69882127104536 140.45623595219104 6.1456 35.698842705562036 140.45625678235726 6.1456 35.698842705562036 140.45625678235726 9.1456 35.69882127104536 140.45623595219104 9.1456 35.69882127104536 140.45623595219104 6.1456</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698842705562036 140.45625678235726 6.1456 35.69887232689954 140.4562109343202 6.1456 35.69887232689954 140.4562109343202 9.1456 35.698842705562036 140.45625678235726 9.1456 35.698842705562036 140.45625678235726 6.1456</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69887232689954 140.4562109343202 9.1456 35.69885089237499 140.45619010415834 9.1456 35.69882127104536 140.45623595219104 9.1456 35.698842705562036 140.45625678235726 9.1456 35.69887232689954 140.4562109343202 9.1456</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">16.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8240</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_74374a4a-0d21-45cc-8cdd-d25cac478dfd">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69864122201354 140.45659367524198 0 35.69862716747369 140.45657500195784 0 35.698628254773745 140.45657390542408 0 35.69858224350898 140.45651244120108 0 35.698563657270874 140.45653340196768 0 35.698560523832555 140.45652928910852 0 35.69851346979347 140.4565821836508 0 35.6985765788548 140.45666643330344 0 35.69864122201354 140.45659367524198 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69864122201354 140.45659367524198 6.09628 35.6985765788548 140.45666643330344 6.09628 35.69851346979347 140.4565821836508 6.09628 35.698560523832555 140.45652928910852 6.09628 35.698563657270874 140.45653340196768 6.09628 35.69858224350898 140.45651244120108 6.09628 35.698628254773745 140.45657390542408 6.09628 35.69862716747369 140.45657500195784 6.09628 35.69864122201354 140.45659367524198 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69864122201354 140.45659367524198 6.09628 35.69862716747369 140.45657500195784 6.09628 35.69862716747369 140.45657500195784 8.79628 35.69864122201354 140.45659367524198 8.79628 35.69864122201354 140.45659367524198 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69862716747369 140.45657500195784 6.09628 35.698628254773745 140.45657390542408 6.09628 35.698628254773745 140.45657390542408 8.79628 35.69862716747369 140.45657500195784 8.79628 35.69862716747369 140.45657500195784 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698628254773745 140.45657390542408 6.09628 35.69858224350898 140.45651244120108 6.09628 35.69858224350898 140.45651244120108 8.79628 35.698628254773745 140.45657390542408 8.79628 35.698628254773745 140.45657390542408 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69858224350898 140.45651244120108 6.09628 35.698563657270874 140.45653340196768 6.09628 35.698563657270874 140.45653340196768 8.79628 35.69858224350898 140.45651244120108 8.79628 35.69858224350898 140.45651244120108 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698563657270874 140.45653340196768 6.09628 35.698560523832555 140.45652928910852 6.09628 35.698560523832555 140.45652928910852 8.79628 35.698563657270874 140.45653340196768 8.79628 35.698563657270874 140.45653340196768 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698560523832555 140.45652928910852 6.09628 35.69851346979347 140.4565821836508 6.09628 35.69851346979347 140.4565821836508 8.79628 35.698560523832555 140.45652928910852 8.79628 35.698560523832555 140.45652928910852 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69851346979347 140.4565821836508 6.09628 35.6985765788548 140.45666643330344 6.09628 35.6985765788548 140.45666643330344 8.79628 35.69851346979347 140.4565821836508 8.79628 35.69851346979347 140.4565821836508 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985765788548 140.45666643330344 6.09628 35.69864122201354 140.45659367524198 6.09628 35.69864122201354 140.45659367524198 8.79628 35.6985765788548 140.45666643330344 8.79628 35.6985765788548 140.45666643330344 6.09628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69864122201354 140.45659367524198 8.79628 35.69862716747369 140.45657500195784 8.79628 35.698628254773745 140.45657390542408 8.79628 35.69858224350898 140.45651244120108 8.79628 35.698563657270874 140.45653340196768 8.79628 35.698560523832555 140.45652928910852 8.79628 35.69851346979347 140.4565821836508 8.79628 35.6985765788548 140.45666643330344 8.79628 35.69864122201354 140.45659367524198 8.79628</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">100.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8246</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_f9501705-1c62-4bf9-af61-06e8301c0345">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69548746069642 140.46083071114762 0 35.695588637196806 140.46089128169365 0 35.6955903715855 140.46088709646213 0 35.69562658284551 140.46090881625554 0 35.69566000991466 140.46082488069231 0 35.69562622470371 140.46080461637123 0 35.69563024290352 140.4607945927745 0 35.69552664086492 140.46073245638698 0 35.69548746069642 140.46083071114762 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69548746069642 140.46083071114762 7.67266 35.69552664086492 140.46073245638698 7.67266 35.69563024290352 140.4607945927745 7.67266 35.69562622470371 140.46080461637123 7.67266 35.69566000991466 140.46082488069231 7.67266 35.69562658284551 140.46090881625554 7.67266 35.6955903715855 140.46088709646213 7.67266 35.695588637196806 140.46089128169365 7.67266 35.69548746069642 140.46083071114762 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69548746069642 140.46083071114762 7.67266 35.695588637196806 140.46089128169365 7.67266 35.695588637196806 140.46089128169365 13.67266 35.69548746069642 140.46083071114762 13.67266 35.69548746069642 140.46083071114762 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695588637196806 140.46089128169365 7.67266 35.6955903715855 140.46088709646213 7.67266 35.6955903715855 140.46088709646213 13.67266 35.695588637196806 140.46089128169365 13.67266 35.695588637196806 140.46089128169365 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6955903715855 140.46088709646213 7.67266 35.69562658284551 140.46090881625554 7.67266 35.69562658284551 140.46090881625554 13.67266 35.6955903715855 140.46088709646213 13.67266 35.6955903715855 140.46088709646213 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69562658284551 140.46090881625554 7.67266 35.69566000991466 140.46082488069231 7.67266 35.69566000991466 140.46082488069231 13.67266 35.69562658284551 140.46090881625554 13.67266 35.69562658284551 140.46090881625554 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69566000991466 140.46082488069231 7.67266 35.69562622470371 140.46080461637123 7.67266 35.69562622470371 140.46080461637123 13.67266 35.69566000991466 140.46082488069231 13.67266 35.69566000991466 140.46082488069231 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69562622470371 140.46080461637123 7.67266 35.69563024290352 140.4607945927745 7.67266 35.69563024290352 140.4607945927745 13.67266 35.69562622470371 140.46080461637123 13.67266 35.69562622470371 140.46080461637123 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69563024290352 140.4607945927745 7.67266 35.69552664086492 140.46073245638698 7.67266 35.69552664086492 140.46073245638698 13.67266 35.69563024290352 140.4607945927745 13.67266 35.69563024290352 140.4607945927745 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69552664086492 140.46073245638698 7.67266 35.69548746069642 140.46083071114762 7.67266 35.69548746069642 140.46083071114762 13.67266 35.69552664086492 140.46073245638698 13.67266 35.69552664086492 140.46073245638698 7.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69548746069642 140.46083071114762 13.67266 35.695588637196806 140.46089128169365 13.67266 35.6955903715855 140.46088709646213 13.67266 35.69562658284551 140.46090881625554 13.67266 35.69566000991466 140.46082488069231 13.67266 35.69562622470371 140.46080461637123 13.67266 35.69563024290352 140.4607945927745 13.67266 35.69552664086492 140.46073245638698 13.67266 35.69548746069642 140.46083071114762 13.67266</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">4</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">161.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8219</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_41187c58-6030-4e5e-b4f8-647f0824b6e4">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.695473122012906 140.46081479802024 0 35.69551277182999 140.46071290060758 0 35.69540934194692 140.46065231269733 0 35.69540248994879 140.46066993823533 0 35.69538784284455 140.46066131540712 0 35.69535504450741 140.46074569766049 0 35.695473122012906 140.46081479802024 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695473122012906 140.46081479802024 7.61573 35.69535504450741 140.46074569766049 7.61573 35.69538784284455 140.46066131540712 7.61573 35.69540248994879 140.46066993823533 7.61573 35.69540934194692 140.46065231269733 7.61573 35.69551277182999 140.46071290060758 7.61573 35.695473122012906 140.46081479802024 7.61573</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695473122012906 140.46081479802024 7.61573 35.69551277182999 140.46071290060758 7.61573 35.69551277182999 140.46071290060758 13.61573 35.695473122012906 140.46081479802024 13.61573 35.695473122012906 140.46081479802024 7.61573</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69551277182999 140.46071290060758 7.61573 35.69540934194692 140.46065231269733 7.61573 35.69540934194692 140.46065231269733 13.61573 35.69551277182999 140.46071290060758 13.61573 35.69551277182999 140.46071290060758 7.61573</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69540934194692 140.46065231269733 7.61573 35.69540248994879 140.46066993823533 7.61573 35.69540248994879 140.46066993823533 13.61573 35.69540934194692 140.46065231269733 13.61573 35.69540934194692 140.46065231269733 7.61573</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69540248994879 140.46066993823533 7.61573 35.69538784284455 140.46066131540712 7.61573 35.69538784284455 140.46066131540712 13.61573 35.69540248994879 140.46066993823533 13.61573 35.69540248994879 140.46066993823533 7.61573</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69538784284455 140.46066131540712 7.61573 35.69535504450741 140.46074569766049 7.61573 35.69535504450741 140.46074569766049 13.61573 35.69538784284455 140.46066131540712 13.61573 35.69538784284455 140.46066131540712 7.61573</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69535504450741 140.46074569766049 7.61573 35.695473122012906 140.46081479802024 7.61573 35.695473122012906 140.46081479802024 13.61573 35.69535504450741 140.46074569766049 13.61573 35.69535504450741 140.46074569766049 7.61573</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.695473122012906 140.46081479802024 13.61573 35.69551277182999 140.46071290060758 13.61573 35.69540934194692 140.46065231269733 13.61573 35.69540248994879 140.46066993823533 13.61573 35.69538784284455 140.46066131540712 13.61573 35.69535504450741 140.46074569766049 13.61573 35.695473122012906 140.46081479802024 13.61573</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">4</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">145.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8207</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b4213f4b-9578-4bf9-b769-b3dd2fc3baea">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69970356632523 140.45526720132725 0 35.699725672541355 140.45526273195463 0 35.69971772455127 140.4552309564482 0 35.69969552135379 140.45523675108765 0 35.69970356632523 140.45526720132725 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69970356632523 140.45526720132725 6.69145 35.69969552135379 140.45523675108765 6.69145 35.69971772455127 140.4552309564482 6.69145 35.699725672541355 140.45526273195463 6.69145 35.69970356632523 140.45526720132725 6.69145</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69970356632523 140.45526720132725 6.69145 35.699725672541355 140.45526273195463 6.69145 35.699725672541355 140.45526273195463 8.29145 35.69970356632523 140.45526720132725 8.29145 35.69970356632523 140.45526720132725 6.69145</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699725672541355 140.45526273195463 6.69145 35.69971772455127 140.4552309564482 6.69145 35.69971772455127 140.4552309564482 8.29145 35.699725672541355 140.45526273195463 8.29145 35.699725672541355 140.45526273195463 6.69145</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69971772455127 140.4552309564482 6.69145 35.69969552135379 140.45523675108765 6.69145 35.69969552135379 140.45523675108765 8.29145 35.69971772455127 140.4552309564482 8.29145 35.69971772455127 140.4552309564482 6.69145</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69969552135379 140.45523675108765 6.69145 35.69970356632523 140.45526720132725 6.69145 35.69970356632523 140.45526720132725 8.29145 35.69969552135379 140.45523675108765 8.29145 35.69969552135379 140.45523675108765 6.69145</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69970356632523 140.45526720132725 8.29145 35.699725672541355 140.45526273195463 8.29145 35.69971772455127 140.4552309564482 8.29145 35.69969552135379 140.45523675108765 8.29145 35.69970356632523 140.45526720132725 8.29145</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">7.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8313</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_9f846fb9-f5c5-448b-b0e0-e2432d8521ff">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69991702718219 140.4553504089388 0 35.69987986718742 140.45537255207307 0 35.69989863939703 140.45542021341205 0 35.69993588953183 140.45539807099507 0 35.69991702718219 140.4553504089388 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69991702718219 140.4553504089388 6.07759 35.69993588953183 140.45539807099507 6.07759 35.69989863939703 140.45542021341205 6.07759 35.69987986718742 140.45537255207307 6.07759 35.69991702718219 140.4553504089388 6.07759</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69991702718219 140.4553504089388 6.07759 35.69987986718742 140.45537255207307 6.07759 35.69987986718742 140.45537255207307 8.47759 35.69991702718219 140.4553504089388 8.47759 35.69991702718219 140.4553504089388 6.07759</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69987986718742 140.45537255207307 6.07759 35.69989863939703 140.45542021341205 6.07759 35.69989863939703 140.45542021341205 8.47759 35.69987986718742 140.45537255207307 8.47759 35.69987986718742 140.45537255207307 6.07759</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69989863939703 140.45542021341205 6.07759 35.69993588953183 140.45539807099507 6.07759 35.69993588953183 140.45539807099507 8.47759 35.69989863939703 140.45542021341205 8.47759 35.69989863939703 140.45542021341205 6.07759</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69993588953183 140.45539807099507 6.07759 35.69991702718219 140.4553504089388 6.07759 35.69991702718219 140.4553504089388 8.47759 35.69993588953183 140.45539807099507 8.47759 35.69993588953183 140.45539807099507 6.07759</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69991702718219 140.4553504089388 8.47759 35.69987986718742 140.45537255207307 8.47759 35.69989863939703 140.45542021341205 8.47759 35.69993588953183 140.45539807099507 8.47759 35.69991702718219 140.4553504089388 8.47759</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">22</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8317</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_2f3637b0-1f07-4f37-a1d7-0e871a67b2dc">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69888765919707 140.45685327863458 0 35.69890618626558 140.45682635040566 0 35.698912372719356 140.45683269705938 0 35.698938892027826 140.456794118044 0 35.69889020983904 140.45674368264864 0 35.69884507335627 140.4568091891812 0 35.69888765919707 140.45685327863458 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69888765919707 140.45685327863458 5.94332 35.69884507335627 140.4568091891812 5.94332 35.69889020983904 140.45674368264864 5.94332 35.698938892027826 140.456794118044 5.94332 35.698912372719356 140.45683269705938 5.94332 35.69890618626558 140.45682635040566 5.94332 35.69888765919707 140.45685327863458 5.94332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69888765919707 140.45685327863458 5.94332 35.69890618626558 140.45682635040566 5.94332 35.69890618626558 140.45682635040566 9.64332 35.69888765919707 140.45685327863458 9.64332 35.69888765919707 140.45685327863458 5.94332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69890618626558 140.45682635040566 5.94332 35.698912372719356 140.45683269705938 5.94332 35.698912372719356 140.45683269705938 9.64332 35.69890618626558 140.45682635040566 9.64332 35.69890618626558 140.45682635040566 5.94332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698912372719356 140.45683269705938 5.94332 35.698938892027826 140.456794118044 5.94332 35.698938892027826 140.456794118044 9.64332 35.698912372719356 140.45683269705938 9.64332 35.698912372719356 140.45683269705938 5.94332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698938892027826 140.456794118044 5.94332 35.69889020983904 140.45674368264864 5.94332 35.69889020983904 140.45674368264864 9.64332 35.698938892027826 140.456794118044 9.64332 35.698938892027826 140.456794118044 5.94332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69889020983904 140.45674368264864 5.94332 35.69884507335627 140.4568091891812 5.94332 35.69884507335627 140.4568091891812 9.64332 35.69889020983904 140.45674368264864 9.64332 35.69889020983904 140.45674368264864 5.94332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69884507335627 140.4568091891812 5.94332 35.69888765919707 140.45685327863458 5.94332 35.69888765919707 140.45685327863458 9.64332 35.69884507335627 140.4568091891812 9.64332 35.69884507335627 140.4568091891812 5.94332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69888765919707 140.45685327863458 9.64332 35.69890618626558 140.45682635040566 9.64332 35.698912372719356 140.45683269705938 9.64332 35.698938892027826 140.456794118044 9.64332 35.69889020983904 140.45674368264864 9.64332 35.69884507335627 140.4568091891812 9.64332 35.69888765919707 140.45685327863458 9.64332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">52.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8228</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b7794d65-cc84-4586-b70f-9e498ecb7af2">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">6.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.699047664499396 140.45644998359865 0 35.69908460968493 140.45650419509656 0 35.699104545031126 140.4564837971863 0 35.69910785415723 140.4564887954401 0 35.69918904562376 140.45640566797746 0 35.699148790691 140.45634656869768 0 35.699047664499396 140.45644998359865 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699047664499396 140.45644998359865 5.33401 35.699148790691 140.45634656869768 5.33401 35.69918904562376 140.45640566797746 5.33401 35.69910785415723 140.4564887954401 5.33401 35.699104545031126 140.4564837971863 5.33401 35.69908460968493 140.45650419509656 5.33401 35.699047664499396 140.45644998359865 5.33401</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699047664499396 140.45644998359865 5.33401 35.69908460968493 140.45650419509656 5.33401 35.69908460968493 140.45650419509656 9.03401 35.699047664499396 140.45644998359865 9.03401 35.699047664499396 140.45644998359865 5.33401</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69908460968493 140.45650419509656 5.33401 35.699104545031126 140.4564837971863 5.33401 35.699104545031126 140.4564837971863 9.03401 35.69908460968493 140.45650419509656 9.03401 35.69908460968493 140.45650419509656 5.33401</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699104545031126 140.4564837971863 5.33401 35.69910785415723 140.4564887954401 5.33401 35.69910785415723 140.4564887954401 9.03401 35.699104545031126 140.4564837971863 9.03401 35.699104545031126 140.4564837971863 5.33401</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69910785415723 140.4564887954401 5.33401 35.69918904562376 140.45640566797746 5.33401 35.69918904562376 140.45640566797746 9.03401 35.69910785415723 140.4564887954401 9.03401 35.69910785415723 140.4564887954401 5.33401</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69918904562376 140.45640566797746 5.33401 35.699148790691 140.45634656869768 5.33401 35.699148790691 140.45634656869768 9.03401 35.69918904562376 140.45640566797746 9.03401 35.69918904562376 140.45640566797746 5.33401</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699148790691 140.45634656869768 5.33401 35.699047664499396 140.45644998359865 5.33401 35.699047664499396 140.45644998359865 9.03401 35.699148790691 140.45634656869768 9.03401 35.699148790691 140.45634656869768 5.33401</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699047664499396 140.45644998359865 9.03401 35.69908460968493 140.45650419509656 9.03401 35.699104545031126 140.4564837971863 9.03401 35.69910785415723 140.4564887954401 9.03401 35.69918904562376 140.45640566797746 9.03401 35.699148790691 140.45634656869768 9.03401 35.699047664499396 140.45644998359865 9.03401</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">100.2</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8321</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_d6e412ad-7935-4a0c-9d93-7eb3d8335b97">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">8.8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69782251820047 140.45944313440506 0 35.697780439334856 140.45954435339425 0 35.697837220237936 140.45957982532113 0 35.69784123603827 140.4595702433619 0 35.69790107163166 140.45960761769408 0 35.6979392243122 140.45951609176075 0 35.69782251820047 140.45944313440506 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69782251820047 140.45944313440506 6.59331 35.6979392243122 140.45951609176075 6.59331 35.69790107163166 140.45960761769408 6.59331 35.69784123603827 140.4595702433619 6.59331 35.697837220237936 140.45957982532113 6.59331 35.697780439334856 140.45954435339425 6.59331 35.69782251820047 140.45944313440506 6.59331</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69782251820047 140.45944313440506 6.59331 35.697780439334856 140.45954435339425 6.59331 35.697780439334856 140.45954435339425 12.89331 35.69782251820047 140.45944313440506 12.89331 35.69782251820047 140.45944313440506 6.59331</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697780439334856 140.45954435339425 6.59331 35.697837220237936 140.45957982532113 6.59331 35.697837220237936 140.45957982532113 12.89331 35.697780439334856 140.45954435339425 12.89331 35.697780439334856 140.45954435339425 6.59331</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.697837220237936 140.45957982532113 6.59331 35.69784123603827 140.4595702433619 6.59331 35.69784123603827 140.4595702433619 12.89331 35.697837220237936 140.45957982532113 12.89331 35.697837220237936 140.45957982532113 6.59331</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69784123603827 140.4595702433619 6.59331 35.69790107163166 140.45960761769408 6.59331 35.69790107163166 140.45960761769408 12.89331 35.69784123603827 140.4595702433619 12.89331 35.69784123603827 140.4595702433619 6.59331</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69790107163166 140.45960761769408 6.59331 35.6979392243122 140.45951609176075 6.59331 35.6979392243122 140.45951609176075 12.89331 35.69790107163166 140.45960761769408 12.89331 35.69790107163166 140.45960761769408 6.59331</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6979392243122 140.45951609176075 6.59331 35.69782251820047 140.45944313440506 6.59331 35.69782251820047 140.45944313440506 12.89331 35.6979392243122 140.45951609176075 12.89331 35.6979392243122 140.45951609176075 6.59331</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69782251820047 140.45944313440506 12.89331 35.697780439334856 140.45954435339425 12.89331 35.697837220237936 140.45957982532113 12.89331 35.69784123603827 140.4595702433619 12.89331 35.69790107163166 140.45960761769408 12.89331 35.6979392243122 140.45951609176075 12.89331 35.69782251820047 140.45944313440506 12.89331</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">142.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8209</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_49a56b59-5acc-439f-83cc-476fb0ab8a1b">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69913532310094 140.45549992200753 0 35.69909261132191 140.4554452241411 0 35.6990354794702 140.45551229588906 0 35.69907819121918 140.45556699375132 0 35.69913532310094 140.45549992200753 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69913532310094 140.45549992200753 7.79973 35.69907819121918 140.45556699375132 7.79973 35.6990354794702 140.45551229588906 7.79973 35.69909261132191 140.4554452241411 7.79973 35.69913532310094 140.45549992200753 7.79973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69913532310094 140.45549992200753 7.79973 35.69909261132191 140.4554452241411 7.79973 35.69909261132191 140.4554452241411 11.69973 35.69913532310094 140.45549992200753 11.69973 35.69913532310094 140.45549992200753 7.79973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69909261132191 140.4554452241411 7.79973 35.6990354794702 140.45551229588906 7.79973 35.6990354794702 140.45551229588906 11.69973 35.69909261132191 140.4554452241411 11.69973 35.69909261132191 140.4554452241411 7.79973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6990354794702 140.45551229588906 7.79973 35.69907819121918 140.45556699375132 7.79973 35.69907819121918 140.45556699375132 11.69973 35.6990354794702 140.45551229588906 11.69973 35.6990354794702 140.45551229588906 7.79973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69907819121918 140.45556699375132 7.79973 35.69913532310094 140.45549992200753 7.79973 35.69913532310094 140.45549992200753 11.69973 35.69907819121918 140.45556699375132 11.69973 35.69907819121918 140.45556699375132 7.79973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69913532310094 140.45549992200753 11.69973 35.69909261132191 140.4554452241411 11.69973 35.6990354794702 140.45551229588906 11.69973 35.69907819121918 140.45556699375132 11.69973 35.69913532310094 140.45549992200753 11.69973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">60.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8230</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_62e91cac-6119-4916-8a8e-3c49162ca806">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.699111282165475 140.4561225141141 0 35.69913107125939 140.45616521056652 0 35.6991456411279 140.45617140138327 0 35.69916255862438 140.4561593779229 0 35.699167245668775 140.45614195533628 0 35.699161461440205 140.45612754532692 0 35.69916498743411 140.45612547324163 0 35.69915368650752 140.45609720780942 0 35.699111282165475 140.4561225141141 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699111282165475 140.4561225141141 5.65065 35.69915368650752 140.45609720780942 5.65065 35.69916498743411 140.45612547324163 5.65065 35.699161461440205 140.45612754532692 5.65065 35.699167245668775 140.45614195533628 5.65065 35.69916255862438 140.4561593779229 5.65065 35.6991456411279 140.45617140138327 5.65065 35.69913107125939 140.45616521056652 5.65065 35.699111282165475 140.4561225141141 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699111282165475 140.4561225141141 5.65065 35.69913107125939 140.45616521056652 5.65065 35.69913107125939 140.45616521056652 8.25065 35.699111282165475 140.4561225141141 8.25065 35.699111282165475 140.4561225141141 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69913107125939 140.45616521056652 5.65065 35.6991456411279 140.45617140138327 5.65065 35.6991456411279 140.45617140138327 8.25065 35.69913107125939 140.45616521056652 8.25065 35.69913107125939 140.45616521056652 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6991456411279 140.45617140138327 5.65065 35.69916255862438 140.4561593779229 5.65065 35.69916255862438 140.4561593779229 8.25065 35.6991456411279 140.45617140138327 8.25065 35.6991456411279 140.45617140138327 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69916255862438 140.4561593779229 5.65065 35.699167245668775 140.45614195533628 5.65065 35.699167245668775 140.45614195533628 8.25065 35.69916255862438 140.4561593779229 8.25065 35.69916255862438 140.4561593779229 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699167245668775 140.45614195533628 5.65065 35.699161461440205 140.45612754532692 5.65065 35.699161461440205 140.45612754532692 8.25065 35.699167245668775 140.45614195533628 8.25065 35.699167245668775 140.45614195533628 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699161461440205 140.45612754532692 5.65065 35.69916498743411 140.45612547324163 5.65065 35.69916498743411 140.45612547324163 8.25065 35.699161461440205 140.45612754532692 8.25065 35.699161461440205 140.45612754532692 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69916498743411 140.45612547324163 5.65065 35.69915368650752 140.45609720780942 5.65065 35.69915368650752 140.45609720780942 8.25065 35.69916498743411 140.45612547324163 8.25065 35.69916498743411 140.45612547324163 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69915368650752 140.45609720780942 5.65065 35.699111282165475 140.4561225141141 5.65065 35.699111282165475 140.4561225141141 8.25065 35.69915368650752 140.45609720780942 8.25065 35.69915368650752 140.45609720780942 5.65065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699111282165475 140.4561225141141 8.25065 35.69913107125939 140.45616521056652 8.25065 35.6991456411279 140.45617140138327 8.25065 35.69916255862438 140.4561593779229 8.25065 35.699167245668775 140.45614195533628 8.25065 35.699161461440205 140.45612754532692 8.25065 35.69916498743411 140.45612547324163 8.25065 35.69915368650752 140.45609720780942 8.25065 35.699111282165475 140.4561225141141 8.25065</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">25.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8227</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_2385119e-b885-4d2c-b535-073ccdc5ce32">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.6993512505353 140.45608216897784 0 35.69933560403558 140.45609243435808 0 35.69935288603428 140.45613190691338 0 35.69936853310897 140.45612153104267 0 35.6993512505353 140.45608216897784 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6993512505353 140.45608216897784 5.80951 35.69936853310897 140.45612153104267 5.80951 35.69935288603428 140.45613190691338 5.80951 35.69933560403558 140.45609243435808 5.80951 35.6993512505353 140.45608216897784 5.80951</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6993512505353 140.45608216897784 5.80951 35.69933560403558 140.45609243435808 5.80951 35.69933560403558 140.45609243435808 11.80951 35.6993512505353 140.45608216897784 11.80951 35.6993512505353 140.45608216897784 5.80951</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69933560403558 140.45609243435808 5.80951 35.69935288603428 140.45613190691338 5.80951 35.69935288603428 140.45613190691338 11.80951 35.69933560403558 140.45609243435808 11.80951 35.69933560403558 140.45609243435808 5.80951</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69935288603428 140.45613190691338 5.80951 35.69936853310897 140.45612153104267 5.80951 35.69936853310897 140.45612153104267 11.80951 35.69935288603428 140.45613190691338 11.80951 35.69935288603428 140.45613190691338 5.80951</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69936853310897 140.45612153104267 5.80951 35.6993512505353 140.45608216897784 5.80951 35.6993512505353 140.45608216897784 11.80951 35.69936853310897 140.45612153104267 11.80951 35.69936853310897 140.45612153104267 5.80951</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6993512505353 140.45608216897784 11.80951 35.69933560403558 140.45609243435808 11.80951 35.69935288603428 140.45613190691338 11.80951 35.69936853310897 140.45612153104267 11.80951 35.6993512505353 140.45608216897784 11.80951</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8260</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cb7bb7fa-eb21-4c31-9985-7412b03b8937">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69844700489505 140.45830998483353 0 35.698378088557355 140.4582681203865 0 35.698336176589464 140.45837210447726 0 35.69840509289079 140.45841396899075 0 35.69844700489505 140.45830998483353 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69844700489505 140.45830998483353 5.6165 35.69840509289079 140.45841396899075 5.6165 35.698336176589464 140.45837210447726 5.6165 35.698378088557355 140.4582681203865 5.6165 35.69844700489505 140.45830998483353 5.6165</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69844700489505 140.45830998483353 5.6165 35.698378088557355 140.4582681203865 5.6165 35.698378088557355 140.4582681203865 10.9165 35.69844700489505 140.45830998483353 10.9165 35.69844700489505 140.45830998483353 5.6165</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698378088557355 140.4582681203865 5.6165 35.698336176589464 140.45837210447726 5.6165 35.698336176589464 140.45837210447726 10.9165 35.698378088557355 140.4582681203865 10.9165 35.698378088557355 140.4582681203865 5.6165</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698336176589464 140.45837210447726 5.6165 35.69840509289079 140.45841396899075 5.6165 35.69840509289079 140.45841396899075 10.9165 35.698336176589464 140.45837210447726 10.9165 35.698336176589464 140.45837210447726 5.6165</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69840509289079 140.45841396899075 5.6165 35.69844700489505 140.45830998483353 5.6165 35.69844700489505 140.45830998483353 10.9165 35.69840509289079 140.45841396899075 10.9165 35.69840509289079 140.45841396899075 5.6165</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69844700489505 140.45830998483353 10.9165 35.698378088557355 140.4582681203865 10.9165 35.698336176589464 140.45837210447726 10.9165 35.69840509289079 140.45841396899075 10.9165 35.69844700489505 140.45830998483353 10.9165</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">89.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8199</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_e8be90d2-d4be-4c81-b89a-f07c5007652b">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69900151371864 140.457320253995 0 35.699052983815704 140.45735446802368 0 35.69909083462171 140.4572686833504 0 35.699072600346284 140.45725649678673 0 35.699075791606475 140.4572494496416 0 35.699042556331804 140.45722731172125 0 35.69900151371864 140.457320253995 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69900151371864 140.457320253995 5.19985 35.699042556331804 140.45722731172125 5.19985 35.699075791606475 140.4572494496416 5.19985 35.699072600346284 140.45725649678673 5.19985 35.69909083462171 140.4572686833504 5.19985 35.699052983815704 140.45735446802368 5.19985 35.69900151371864 140.457320253995 5.19985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69900151371864 140.457320253995 5.19985 35.699052983815704 140.45735446802368 5.19985 35.699052983815704 140.45735446802368 9.59985 35.69900151371864 140.457320253995 9.59985 35.69900151371864 140.457320253995 5.19985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699052983815704 140.45735446802368 5.19985 35.69909083462171 140.4572686833504 5.19985 35.69909083462171 140.4572686833504 9.59985 35.699052983815704 140.45735446802368 9.59985 35.699052983815704 140.45735446802368 5.19985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69909083462171 140.4572686833504 5.19985 35.699072600346284 140.45725649678673 5.19985 35.699072600346284 140.45725649678673 9.59985 35.69909083462171 140.4572686833504 9.59985 35.69909083462171 140.4572686833504 5.19985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699072600346284 140.45725649678673 5.19985 35.699075791606475 140.4572494496416 5.19985 35.699075791606475 140.4572494496416 9.59985 35.699072600346284 140.45725649678673 9.59985 35.699072600346284 140.45725649678673 5.19985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699075791606475 140.4572494496416 5.19985 35.699042556331804 140.45722731172125 5.19985 35.699042556331804 140.45722731172125 9.59985 35.699075791606475 140.4572494496416 9.59985 35.699075791606475 140.4572494496416 5.19985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699042556331804 140.45722731172125 5.19985 35.69900151371864 140.457320253995 5.19985 35.69900151371864 140.457320253995 9.59985 35.699042556331804 140.45722731172125 9.59985 35.699042556331804 140.45722731172125 5.19985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69900151371864 140.457320253995 9.59985 35.699052983815704 140.45735446802368 9.59985 35.69909083462171 140.4572686833504 9.59985 35.699072600346284 140.45725649678673 9.59985 35.699075791606475 140.4572494496416 9.59985 35.699042556331804 140.45722731172125 9.59985 35.69900151371864 140.457320253995 9.59985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">60.4</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8210</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cfecd606-15a8-4c94-b540-34ca60a2221f">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">6.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.699036923502696 140.45723588684504 0 35.69898851326233 140.45720258073425 0 35.6989854144273 140.4572091865937 0 35.69894876960749 140.4571840386593 0 35.69892953193185 140.45722632076877 0 35.69893519017476 140.45723023234254 0 35.698910118195116 140.4572851764662 0 35.69892610490723 140.4572962405019 0 35.698923552011536 140.457301856107 0 35.69897824966626 140.45733941023542 0 35.698980164196165 140.45733522615362 0 35.69898878646585 140.45734114983082 0 35.699036923502696 140.45723588684504 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699036923502696 140.45723588684504 5.13025 35.69898878646585 140.45734114983082 5.13025 35.698980164196165 140.45733522615362 5.13025 35.69897824966626 140.45733941023542 5.13025 35.698923552011536 140.457301856107 5.13025 35.69892610490723 140.4572962405019 5.13025 35.698910118195116 140.4572851764662 5.13025 35.69893519017476 140.45723023234254 5.13025 35.69892953193185 140.45722632076877 5.13025 35.69894876960749 140.4571840386593 5.13025 35.6989854144273 140.4572091865937 5.13025 35.69898851326233 140.45720258073425 5.13025 35.699036923502696 140.45723588684504 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699036923502696 140.45723588684504 5.13025 35.69898851326233 140.45720258073425 5.13025 35.69898851326233 140.45720258073425 10.23025 35.699036923502696 140.45723588684504 10.23025 35.699036923502696 140.45723588684504 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69898851326233 140.45720258073425 5.13025 35.6989854144273 140.4572091865937 5.13025 35.6989854144273 140.4572091865937 10.23025 35.69898851326233 140.45720258073425 10.23025 35.69898851326233 140.45720258073425 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6989854144273 140.4572091865937 5.13025 35.69894876960749 140.4571840386593 5.13025 35.69894876960749 140.4571840386593 10.23025 35.6989854144273 140.4572091865937 10.23025 35.6989854144273 140.4572091865937 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69894876960749 140.4571840386593 5.13025 35.69892953193185 140.45722632076877 5.13025 35.69892953193185 140.45722632076877 10.23025 35.69894876960749 140.4571840386593 10.23025 35.69894876960749 140.4571840386593 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69892953193185 140.45722632076877 5.13025 35.69893519017476 140.45723023234254 5.13025 35.69893519017476 140.45723023234254 10.23025 35.69892953193185 140.45722632076877 10.23025 35.69892953193185 140.45722632076877 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69893519017476 140.45723023234254 5.13025 35.698910118195116 140.4572851764662 5.13025 35.698910118195116 140.4572851764662 10.23025 35.69893519017476 140.45723023234254 10.23025 35.69893519017476 140.45723023234254 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698910118195116 140.4572851764662 5.13025 35.69892610490723 140.4572962405019 5.13025 35.69892610490723 140.4572962405019 10.23025 35.698910118195116 140.4572851764662 10.23025 35.698910118195116 140.4572851764662 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69892610490723 140.4572962405019 5.13025 35.698923552011536 140.457301856107 5.13025 35.698923552011536 140.457301856107 10.23025 35.69892610490723 140.4572962405019 10.23025 35.69892610490723 140.4572962405019 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698923552011536 140.457301856107 5.13025 35.69897824966626 140.45733941023542 5.13025 35.69897824966626 140.45733941023542 10.23025 35.698923552011536 140.457301856107 10.23025 35.698923552011536 140.457301856107 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69897824966626 140.45733941023542 5.13025 35.698980164196165 140.45733522615362 5.13025 35.698980164196165 140.45733522615362 10.23025 35.69897824966626 140.45733941023542 10.23025 35.69897824966626 140.45733941023542 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698980164196165 140.45733522615362 5.13025 35.69898878646585 140.45734114983082 5.13025 35.69898878646585 140.45734114983082 10.23025 35.698980164196165 140.45733522615362 10.23025 35.698980164196165 140.45733522615362 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69898878646585 140.45734114983082 5.13025 35.699036923502696 140.45723588684504 5.13025 35.699036923502696 140.45723588684504 10.23025 35.69898878646585 140.45734114983082 10.23025 35.69898878646585 140.45734114983082 5.13025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699036923502696 140.45723588684504 10.23025 35.69898851326233 140.45720258073425 10.23025 35.6989854144273 140.4572091865937 10.23025 35.69894876960749 140.4571840386593 10.23025 35.69892953193185 140.45722632076877 10.23025 35.69893519017476 140.45723023234254 10.23025 35.698910118195116 140.4572851764662 10.23025 35.69892610490723 140.4572962405019 10.23025 35.698923552011536 140.457301856107 10.23025 35.69897824966626 140.45733941023542 10.23025 35.698980164196165 140.45733522615362 10.23025 35.69898878646585 140.45734114983082 10.23025 35.699036923502696 140.45723588684504 10.23025</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">113.5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">215</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8216</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_db748a65-a4ad-4588-9347-c6537e10fe7b">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.6936396385879 140.45966335659068 0 35.69355438834439 140.45967727504114 0 35.693572884054156 140.45984702562407 0 35.693658134315214 140.45983310735164 0 35.6936396385879 140.45966335659068 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6936396385879 140.45966335659068 5.27643 35.693658134315214 140.45983310735164 5.27643 35.693572884054156 140.45984702562407 5.27643 35.69355438834439 140.45967727504114 5.27643 35.6936396385879 140.45966335659068 5.27643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6936396385879 140.45966335659068 5.27643 35.69355438834439 140.45967727504114 5.27643 35.69355438834439 140.45967727504114 11.276430000000001 35.6936396385879 140.45966335659068 11.276430000000001 35.6936396385879 140.45966335659068 5.27643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69355438834439 140.45967727504114 5.27643 35.693572884054156 140.45984702562407 5.27643 35.693572884054156 140.45984702562407 11.276430000000001 35.69355438834439 140.45967727504114 11.276430000000001 35.69355438834439 140.45967727504114 5.27643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693572884054156 140.45984702562407 5.27643 35.693658134315214 140.45983310735164 5.27643 35.693658134315214 140.45983310735164 11.276430000000001 35.693572884054156 140.45984702562407 11.276430000000001 35.693572884054156 140.45984702562407 5.27643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.693658134315214 140.45983310735164 5.27643 35.6936396385879 140.45966335659068 5.27643 35.6936396385879 140.45966335659068 11.276430000000001 35.693658134315214 140.45983310735164 11.276430000000001 35.693658134315214 140.45983310735164 5.27643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6936396385879 140.45966335659068 11.276430000000001 35.69355438834439 140.45967727504114 11.276430000000001 35.693572884054156 140.45984702562407 11.276430000000001 35.693658134315214 140.45983310735164 11.276430000000001 35.6936396385879 140.45966335659068 11.276430000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">147.9</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">231</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8270</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_da4167d5-793b-4634-8dc5-64d41a6873ea">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69975899687679 140.4558953882427 0 35.69980443764017 140.45587551977513 0 35.69976254826044 140.45573154278884 0 35.6997171075198 140.45575141132844 0 35.69975899687679 140.4558953882427 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69975899687679 140.4558953882427 5.62643 35.6997171075198 140.45575141132844 5.62643 35.69976254826044 140.45573154278884 5.62643 35.69980443764017 140.45587551977513 5.62643 35.69975899687679 140.4558953882427 5.62643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69975899687679 140.4558953882427 5.62643 35.69980443764017 140.45587551977513 5.62643 35.69980443764017 140.45587551977513 8.62643 35.69975899687679 140.4558953882427 8.62643 35.69975899687679 140.4558953882427 5.62643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69980443764017 140.45587551977513 5.62643 35.69976254826044 140.45573154278884 5.62643 35.69976254826044 140.45573154278884 8.62643 35.69980443764017 140.45587551977513 8.62643 35.69980443764017 140.45587551977513 5.62643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69976254826044 140.45573154278884 5.62643 35.6997171075198 140.45575141132844 5.62643 35.6997171075198 140.45575141132844 8.62643 35.69976254826044 140.45573154278884 8.62643 35.69976254826044 140.45573154278884 5.62643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6997171075198 140.45575141132844 5.62643 35.69975899687679 140.4558953882427 5.62643 35.69975899687679 140.4558953882427 8.62643 35.6997171075198 140.45575141132844 8.62643 35.6997171075198 140.45575141132844 5.62643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69975899687679 140.4558953882427 8.62643 35.69980443764017 140.45587551977513 8.62643 35.69976254826044 140.45573154278884 8.62643 35.6997171075198 140.45575141132844 8.62643 35.69975899687679 140.4558953882427 8.62643</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">74.1</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-9089</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_6a1c3533-1861-4875-b90b-6c3683244a14">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.6992968135136 140.4554537718599 0 35.699312761862785 140.4554199721115 0 35.69927414576099 140.45539248906886 0 35.69925810728791 140.4554262881065 0 35.6992968135136 140.4554537718599 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6992968135136 140.4554537718599 7.8646 35.69925810728791 140.4554262881065 7.8646 35.69927414576099 140.45539248906886 7.8646 35.699312761862785 140.4554199721115 7.8646 35.6992968135136 140.4554537718599 7.8646</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6992968135136 140.4554537718599 7.8646 35.699312761862785 140.4554199721115 7.8646 35.699312761862785 140.4554199721115 11.2646 35.6992968135136 140.4554537718599 11.2646 35.6992968135136 140.4554537718599 7.8646</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699312761862785 140.4554199721115 7.8646 35.69927414576099 140.45539248906886 7.8646 35.69927414576099 140.45539248906886 11.2646 35.699312761862785 140.4554199721115 11.2646 35.699312761862785 140.4554199721115 7.8646</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69927414576099 140.45539248906886 7.8646 35.69925810728791 140.4554262881065 7.8646 35.69925810728791 140.4554262881065 11.2646 35.69927414576099 140.45539248906886 11.2646 35.69927414576099 140.45539248906886 7.8646</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69925810728791 140.4554262881065 7.8646 35.6992968135136 140.4554537718599 7.8646 35.6992968135136 140.4554537718599 11.2646 35.69925810728791 140.4554262881065 11.2646 35.69925810728791 140.4554262881065 7.8646</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6992968135136 140.4554537718599 11.2646 35.699312761862785 140.4554199721115 11.2646 35.69927414576099 140.45539248906886 11.2646 35.69925810728791 140.4554262881065 11.2646 35.6992968135136 140.4554537718599 11.2646</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">17.5</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8264</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_a0cde9d0-d257-4750-aace-910751ce0065">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698041699292986 140.45898042196623 0 35.698062487071674 140.45891748960193 0 35.6979818440817 140.45887741172763 0 35.69796105632423 140.4589403440396 0 35.698041699292986 140.45898042196623 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698041699292986 140.45898042196623 6.96962 35.69796105632423 140.4589403440396 6.96962 35.6979818440817 140.45887741172763 6.96962 35.698062487071674 140.45891748960193 6.96962 35.698041699292986 140.45898042196623 6.96962</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698041699292986 140.45898042196623 6.96962 35.698062487071674 140.45891748960193 6.96962 35.698062487071674 140.45891748960193 9.969619999999999 35.698041699292986 140.45898042196623 9.969619999999999 35.698041699292986 140.45898042196623 6.96962</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698062487071674 140.45891748960193 6.96962 35.6979818440817 140.45887741172763 6.96962 35.6979818440817 140.45887741172763 9.969619999999999 35.698062487071674 140.45891748960193 9.969619999999999 35.698062487071674 140.45891748960193 6.96962</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6979818440817 140.45887741172763 6.96962 35.69796105632423 140.4589403440396 6.96962 35.69796105632423 140.4589403440396 9.969619999999999 35.6979818440817 140.45887741172763 9.969619999999999 35.6979818440817 140.45887741172763 6.96962</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69796105632423 140.4589403440396 6.96962 35.698041699292986 140.45898042196623 6.96962 35.698041699292986 140.45898042196623 9.969619999999999 35.69796105632423 140.4589403440396 9.969619999999999 35.69796105632423 140.4589403440396 6.96962</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698041699292986 140.45898042196623 9.969619999999999 35.698062487071674 140.45891748960193 9.969619999999999 35.6979818440817 140.45887741172763 9.969619999999999 35.69796105632423 140.4589403440396 9.969619999999999 35.698041699292986 140.45898042196623 9.969619999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">0</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">59.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8251</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_c10f55f5-2e71-4f83-85bd-bc89bf4b478b">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">4.4</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.6990958542301 140.45612537764728 0 35.69906563973912 140.45616392822424 0 35.69910613277817 140.45621186868817 0 35.69913643741588 140.4561733188112 0 35.6990958542301 140.45612537764728 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6990958542301 140.45612537764728 5.61883 35.69913643741588 140.4561733188112 5.61883 35.69910613277817 140.45621186868817 5.61883 35.69906563973912 140.45616392822424 5.61883 35.6990958542301 140.45612537764728 5.61883</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6990958542301 140.45612537764728 5.61883 35.69906563973912 140.45616392822424 5.61883 35.69906563973912 140.45616392822424 9.21883 35.6990958542301 140.45612537764728 9.21883 35.6990958542301 140.45612537764728 5.61883</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906563973912 140.45616392822424 5.61883 35.69910613277817 140.45621186868817 5.61883 35.69910613277817 140.45621186868817 9.21883 35.69906563973912 140.45616392822424 9.21883 35.69906563973912 140.45616392822424 5.61883</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69910613277817 140.45621186868817 5.61883 35.69913643741588 140.4561733188112 5.61883 35.69913643741588 140.4561733188112 9.21883 35.69910613277817 140.45621186868817 9.21883 35.69910613277817 140.45621186868817 5.61883</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69913643741588 140.4561733188112 5.61883 35.6990958542301 140.45612537764728 5.61883 35.6990958542301 140.45612537764728 9.21883 35.69913643741588 140.4561733188112 9.21883 35.69913643741588 140.4561733188112 5.61883</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6990958542301 140.45612537764728 9.21883 35.69906563973912 140.45616392822424 9.21883 35.69910613277817 140.45621186868817 9.21883 35.69913643741588 140.4561733188112 9.21883 35.6990958542301 140.45612537764728 9.21883</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">30.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8236</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_8222b934-7aa7-4648-9542-1dd7a1715fdf">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.7</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.6985352734349 140.45674058839506 0 35.698569833879816 140.45669831513536 0 35.69856455045062 140.45669164404973 0 35.698569811842866 140.45668516553516 0 35.698540165973974 140.45664869101 0 35.69853617500854 140.4566535219216 0 35.69852318832435 140.4566375089464 0 35.698500874055064 140.45666473911484 0 35.69850553114248 140.4566705213241 0 35.698491924430336 140.45668721131776 0 35.6985352734349 140.45674058839506 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985352734349 140.45674058839506 6.15763 35.698491924430336 140.45668721131776 6.15763 35.69850553114248 140.4566705213241 6.15763 35.698500874055064 140.45666473911484 6.15763 35.69852318832435 140.4566375089464 6.15763 35.69853617500854 140.4566535219216 6.15763 35.698540165973974 140.45664869101 6.15763 35.698569811842866 140.45668516553516 6.15763 35.69856455045062 140.45669164404973 6.15763 35.698569833879816 140.45669831513536 6.15763 35.6985352734349 140.45674058839506 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985352734349 140.45674058839506 6.15763 35.698569833879816 140.45669831513536 6.15763 35.698569833879816 140.45669831513536 9.35763 35.6985352734349 140.45674058839506 9.35763 35.6985352734349 140.45674058839506 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698569833879816 140.45669831513536 6.15763 35.69856455045062 140.45669164404973 6.15763 35.69856455045062 140.45669164404973 9.35763 35.698569833879816 140.45669831513536 9.35763 35.698569833879816 140.45669831513536 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69856455045062 140.45669164404973 6.15763 35.698569811842866 140.45668516553516 6.15763 35.698569811842866 140.45668516553516 9.35763 35.69856455045062 140.45669164404973 9.35763 35.69856455045062 140.45669164404973 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698569811842866 140.45668516553516 6.15763 35.698540165973974 140.45664869101 6.15763 35.698540165973974 140.45664869101 9.35763 35.698569811842866 140.45668516553516 9.35763 35.698569811842866 140.45668516553516 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698540165973974 140.45664869101 6.15763 35.69853617500854 140.4566535219216 6.15763 35.69853617500854 140.4566535219216 9.35763 35.698540165973974 140.45664869101 9.35763 35.698540165973974 140.45664869101 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69853617500854 140.4566535219216 6.15763 35.69852318832435 140.4566375089464 6.15763 35.69852318832435 140.4566375089464 9.35763 35.69853617500854 140.4566535219216 9.35763 35.69853617500854 140.4566535219216 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69852318832435 140.4566375089464 6.15763 35.698500874055064 140.45666473911484 6.15763 35.698500874055064 140.45666473911484 9.35763 35.69852318832435 140.4566375089464 9.35763 35.69852318832435 140.4566375089464 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698500874055064 140.45666473911484 6.15763 35.69850553114248 140.4566705213241 6.15763 35.69850553114248 140.4566705213241 9.35763 35.698500874055064 140.45666473911484 9.35763 35.698500874055064 140.45666473911484 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69850553114248 140.4566705213241 6.15763 35.698491924430336 140.45668721131776 6.15763 35.698491924430336 140.45668721131776 9.35763 35.69850553114248 140.4566705213241 9.35763 35.69850553114248 140.4566705213241 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698491924430336 140.45668721131776 6.15763 35.6985352734349 140.45674058839506 6.15763 35.6985352734349 140.45674058839506 9.35763 35.698491924430336 140.45668721131776 9.35763 35.698491924430336 140.45668721131776 6.15763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985352734349 140.45674058839506 9.35763 35.698569833879816 140.45669831513536 9.35763 35.69856455045062 140.45669164404973 9.35763 35.698569811842866 140.45668516553516 9.35763 35.698540165973974 140.45664869101 9.35763 35.69853617500854 140.4566535219216 9.35763 35.69852318832435 140.4566375089464 9.35763 35.698500874055064 140.45666473911484 9.35763 35.69850553114248 140.4566705213241 9.35763 35.698491924430336 140.45668721131776 9.35763 35.6985352734349 140.45674058839506 9.35763</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">43.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8205</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_4b7520dc-a63b-4391-8981-c45c5a363372">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">6.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69896284487801 140.45624147328502 0 35.699019737610804 140.45616843325942 0 35.69896687785789 140.45610658421063 0 35.69896969080255 140.4561029595854 0 35.698925342639114 140.45605101128797 0 35.69891899055532 140.45605924939053 0 35.69889426343621 140.4560302166675 0 35.69884000232433 140.45609985164887 0 35.698857114550975 140.45611987467467 0 35.698851126419086 140.4561274525962 0 35.69887361394698 140.45615381592125 0 35.698880510257005 140.45614491906395 0 35.69896284487801 140.45624147328502 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69896284487801 140.45624147328502 5.84128 35.698880510257005 140.45614491906395 5.84128 35.69887361394698 140.45615381592125 5.84128 35.698851126419086 140.4561274525962 5.84128 35.698857114550975 140.45611987467467 5.84128 35.69884000232433 140.45609985164887 5.84128 35.69889426343621 140.4560302166675 5.84128 35.69891899055532 140.45605924939053 5.84128 35.698925342639114 140.45605101128797 5.84128 35.69896969080255 140.4561029595854 5.84128 35.69896687785789 140.45610658421063 5.84128 35.699019737610804 140.45616843325942 5.84128 35.69896284487801 140.45624147328502 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69896284487801 140.45624147328502 5.84128 35.699019737610804 140.45616843325942 5.84128 35.699019737610804 140.45616843325942 10.24128 35.69896284487801 140.45624147328502 10.24128 35.69896284487801 140.45624147328502 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699019737610804 140.45616843325942 5.84128 35.69896687785789 140.45610658421063 5.84128 35.69896687785789 140.45610658421063 10.24128 35.699019737610804 140.45616843325942 10.24128 35.699019737610804 140.45616843325942 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69896687785789 140.45610658421063 5.84128 35.69896969080255 140.4561029595854 5.84128 35.69896969080255 140.4561029595854 10.24128 35.69896687785789 140.45610658421063 10.24128 35.69896687785789 140.45610658421063 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69896969080255 140.4561029595854 5.84128 35.698925342639114 140.45605101128797 5.84128 35.698925342639114 140.45605101128797 10.24128 35.69896969080255 140.4561029595854 10.24128 35.69896969080255 140.4561029595854 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698925342639114 140.45605101128797 5.84128 35.69891899055532 140.45605924939053 5.84128 35.69891899055532 140.45605924939053 10.24128 35.698925342639114 140.45605101128797 10.24128 35.698925342639114 140.45605101128797 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69891899055532 140.45605924939053 5.84128 35.69889426343621 140.4560302166675 5.84128 35.69889426343621 140.4560302166675 10.24128 35.69891899055532 140.45605924939053 10.24128 35.69891899055532 140.45605924939053 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69889426343621 140.4560302166675 5.84128 35.69884000232433 140.45609985164887 5.84128 35.69884000232433 140.45609985164887 10.24128 35.69889426343621 140.4560302166675 10.24128 35.69889426343621 140.4560302166675 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69884000232433 140.45609985164887 5.84128 35.698857114550975 140.45611987467467 5.84128 35.698857114550975 140.45611987467467 10.24128 35.69884000232433 140.45609985164887 10.24128 35.69884000232433 140.45609985164887 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698857114550975 140.45611987467467 5.84128 35.698851126419086 140.4561274525962 5.84128 35.698851126419086 140.4561274525962 10.24128 35.698857114550975 140.45611987467467 10.24128 35.698857114550975 140.45611987467467 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698851126419086 140.4561274525962 5.84128 35.69887361394698 140.45615381592125 5.84128 35.69887361394698 140.45615381592125 10.24128 35.698851126419086 140.4561274525962 10.24128 35.698851126419086 140.4561274525962 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69887361394698 140.45615381592125 5.84128 35.698880510257005 140.45614491906395 5.84128 35.698880510257005 140.45614491906395 10.24128 35.69887361394698 140.45615381592125 10.24128 35.69887361394698 140.45615381592125 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698880510257005 140.45614491906395 5.84128 35.69896284487801 140.45624147328502 5.84128 35.69896284487801 140.45624147328502 10.24128 35.698880510257005 140.45614491906395 10.24128 35.698880510257005 140.45614491906395 5.84128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69896284487801 140.45624147328502 10.24128 35.699019737610804 140.45616843325942 10.24128 35.69896687785789 140.45610658421063 10.24128 35.69896969080255 140.4561029595854 10.24128 35.698925342639114 140.45605101128797 10.24128 35.69891899055532 140.45605924939053 10.24128 35.69889426343621 140.4560302166675 10.24128 35.69884000232433 140.45609985164887 10.24128 35.698857114550975 140.45611987467467 10.24128 35.698851126419086 140.4561274525962 10.24128 35.69887361394698 140.45615381592125 10.24128 35.698880510257005 140.45614491906395 10.24128 35.69896284487801 140.45624147328502 10.24128</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">176</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8237</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_3035f88a-5170-4874-a12c-c30b67f44495">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">9.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.698592920441875 140.45671341224647 0 35.698564001371004 140.45674545298024 0 35.698570625325125 140.4567543444919 0 35.6985317339668 140.4567974680088 0 35.6985740702963 140.4568549257665 0 35.698563553789675 140.45686666732854 0 35.69860463740236 140.4569223474008 0 35.69867453295645 140.45684487893206 0 35.69865475238925 140.45681798405587 0 35.69866318327379 140.45680865722815 0 35.698592920441875 140.45671341224647 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698592920441875 140.45671341224647 5.86157 35.69866318327379 140.45680865722815 5.86157 35.69865475238925 140.45681798405587 5.86157 35.69867453295645 140.45684487893206 5.86157 35.69860463740236 140.4569223474008 5.86157 35.698563553789675 140.45686666732854 5.86157 35.6985740702963 140.4568549257665 5.86157 35.6985317339668 140.4567974680088 5.86157 35.698570625325125 140.4567543444919 5.86157 35.698564001371004 140.45674545298024 5.86157 35.698592920441875 140.45671341224647 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698592920441875 140.45671341224647 5.86157 35.698564001371004 140.45674545298024 5.86157 35.698564001371004 140.45674545298024 11.36157 35.698592920441875 140.45671341224647 11.36157 35.698592920441875 140.45671341224647 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698564001371004 140.45674545298024 5.86157 35.698570625325125 140.4567543444919 5.86157 35.698570625325125 140.4567543444919 11.36157 35.698564001371004 140.45674545298024 11.36157 35.698564001371004 140.45674545298024 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698570625325125 140.4567543444919 5.86157 35.6985317339668 140.4567974680088 5.86157 35.6985317339668 140.4567974680088 11.36157 35.698570625325125 140.4567543444919 11.36157 35.698570625325125 140.4567543444919 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985317339668 140.4567974680088 5.86157 35.6985740702963 140.4568549257665 5.86157 35.6985740702963 140.4568549257665 11.36157 35.6985317339668 140.4567974680088 11.36157 35.6985317339668 140.4567974680088 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6985740702963 140.4568549257665 5.86157 35.698563553789675 140.45686666732854 5.86157 35.698563553789675 140.45686666732854 11.36157 35.6985740702963 140.4568549257665 11.36157 35.6985740702963 140.4568549257665 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698563553789675 140.45686666732854 5.86157 35.69860463740236 140.4569223474008 5.86157 35.69860463740236 140.4569223474008 11.36157 35.698563553789675 140.45686666732854 11.36157 35.698563553789675 140.45686666732854 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69860463740236 140.4569223474008 5.86157 35.69867453295645 140.45684487893206 5.86157 35.69867453295645 140.45684487893206 11.36157 35.69860463740236 140.4569223474008 11.36157 35.69860463740236 140.4569223474008 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69867453295645 140.45684487893206 5.86157 35.69865475238925 140.45681798405587 5.86157 35.69865475238925 140.45681798405587 11.36157 35.69867453295645 140.45684487893206 11.36157 35.69867453295645 140.45684487893206 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69865475238925 140.45681798405587 5.86157 35.69866318327379 140.45680865722815 5.86157 35.69866318327379 140.45680865722815 11.36157 35.69865475238925 140.45681798405587 11.36157 35.69865475238925 140.45681798405587 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69866318327379 140.45680865722815 5.86157 35.698592920441875 140.45671341224647 5.86157 35.698592920441875 140.45671341224647 11.36157 35.69866318327379 140.45680865722815 11.36157 35.69866318327379 140.45680865722815 5.86157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698592920441875 140.45671341224647 11.36157 35.698564001371004 140.45674545298024 11.36157 35.698570625325125 140.4567543444919 11.36157 35.6985317339668 140.4567974680088 11.36157 35.6985740702963 140.4568549257665 11.36157 35.698563553789675 140.45686666732854 11.36157 35.69860463740236 140.4569223474008 11.36157 35.69867453295645 140.45684487893206 11.36157 35.69865475238925 140.45681798405587 11.36157 35.69866318327379 140.45680865722815 11.36157 35.698592920441875 140.45671341224647 11.36157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">151.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8247</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_cda94f7d-1c42-4f37-a5f8-1e2f2510999c">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">5.5</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69906791890776 140.45732497036877 0 35.699151187302775 140.45738031672755 0 35.6991813760029 140.45731204180615 0 35.6990981971352 140.45725680669437 0 35.69906791890776 140.45732497036877 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906791890776 140.45732497036877 4.88589 35.6990981971352 140.45725680669437 4.88589 35.6991813760029 140.45731204180615 4.88589 35.699151187302775 140.45738031672755 4.88589 35.69906791890776 140.45732497036877 4.88589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906791890776 140.45732497036877 4.88589 35.699151187302775 140.45738031672755 4.88589 35.699151187302775 140.45738031672755 9.18589 35.69906791890776 140.45732497036877 9.18589 35.69906791890776 140.45732497036877 4.88589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699151187302775 140.45738031672755 4.88589 35.6991813760029 140.45731204180615 4.88589 35.6991813760029 140.45731204180615 9.18589 35.699151187302775 140.45738031672755 9.18589 35.699151187302775 140.45738031672755 4.88589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6991813760029 140.45731204180615 4.88589 35.6990981971352 140.45725680669437 4.88589 35.6990981971352 140.45725680669437 9.18589 35.6991813760029 140.45731204180615 9.18589 35.6991813760029 140.45731204180615 4.88589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6990981971352 140.45725680669437 4.88589 35.69906791890776 140.45732497036877 4.88589 35.69906791890776 140.45732497036877 9.18589 35.6990981971352 140.45725680669437 9.18589 35.6990981971352 140.45725680669437 4.88589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69906791890776 140.45732497036877 9.18589 35.699151187302775 140.45738031672755 9.18589 35.6991813760029 140.45731204180615 9.18589 35.6990981971352 140.45725680669437 9.18589 35.69906791890776 140.45732497036877 9.18589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">73.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8223</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_d80eddd3-14dc-4cf3-a254-298eb4f70c41">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">3.3</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69884031525152 140.4566314685048 0 35.69880449928224 140.45667251666592 0 35.69886063352267 140.45674599392598 0 35.69889645008887 140.4567048352744 0 35.69884031525152 140.4566314685048 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69884031525152 140.4566314685048 5.65983 35.69889645008887 140.4567048352744 5.65983 35.69886063352267 140.45674599392598 5.65983 35.69880449928224 140.45667251666592 5.65983 35.69884031525152 140.4566314685048 5.65983</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69884031525152 140.4566314685048 5.65983 35.69880449928224 140.45667251666592 5.65983 35.69880449928224 140.45667251666592 8.759830000000001 35.69884031525152 140.4566314685048 8.759830000000001 35.69884031525152 140.4566314685048 5.65983</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69880449928224 140.45667251666592 5.65983 35.69886063352267 140.45674599392598 5.65983 35.69886063352267 140.45674599392598 8.759830000000001 35.69880449928224 140.45667251666592 8.759830000000001 35.69880449928224 140.45667251666592 5.65983</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69886063352267 140.45674599392598 5.65983 35.69889645008887 140.4567048352744 5.65983 35.69889645008887 140.4567048352744 8.759830000000001 35.69886063352267 140.45674599392598 8.759830000000001 35.69886063352267 140.45674599392598 5.65983</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69889645008887 140.4567048352744 5.65983 35.69884031525152 140.4566314685048 5.65983 35.69884031525152 140.4566314685048 8.759830000000001 35.69889645008887 140.4567048352744 8.759830000000001 35.69889645008887 140.4567048352744 5.65983</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69884031525152 140.4566314685048 8.759830000000001 35.69880449928224 140.45667251666592 8.759830000000001 35.69886063352267 140.45674599392598 8.759830000000001 35.69889645008887 140.4567048352744 8.759830000000001 35.69884031525152 140.4566314685048 8.759830000000001</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">49.6</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8226</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_e974f2ef-e665-4243-9d42-b08235d6019f">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">6.1</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.699071435188706 140.4557405369189 0 35.69899249919255 140.4556838999819 0 35.69897463915291 140.45572133118588 0 35.6989187824115 140.45568122765636 0 35.69888698140448 140.4557477224495 0 35.69902177407096 140.45584446301172 0 35.699071435188706 140.4557405369189 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699071435188706 140.4557405369189 7.22992 35.69902177407096 140.45584446301172 7.22992 35.69888698140448 140.4557477224495 7.22992 35.6989187824115 140.45568122765636 7.22992 35.69897463915291 140.45572133118588 7.22992 35.69899249919255 140.4556838999819 7.22992 35.699071435188706 140.4557405369189 7.22992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699071435188706 140.4557405369189 7.22992 35.69899249919255 140.4556838999819 7.22992 35.69899249919255 140.4556838999819 12.22992 35.699071435188706 140.4557405369189 12.22992 35.699071435188706 140.4557405369189 7.22992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69899249919255 140.4556838999819 7.22992 35.69897463915291 140.45572133118588 7.22992 35.69897463915291 140.45572133118588 12.22992 35.69899249919255 140.4556838999819 12.22992 35.69899249919255 140.4556838999819 7.22992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69897463915291 140.45572133118588 7.22992 35.6989187824115 140.45568122765636 7.22992 35.6989187824115 140.45568122765636 12.22992 35.69897463915291 140.45572133118588 12.22992 35.69897463915291 140.45572133118588 7.22992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6989187824115 140.45568122765636 7.22992 35.69888698140448 140.4557477224495 7.22992 35.69888698140448 140.4557477224495 12.22992 35.6989187824115 140.45568122765636 12.22992 35.6989187824115 140.45568122765636 7.22992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69888698140448 140.4557477224495 7.22992 35.69902177407096 140.45584446301172 7.22992 35.69902177407096 140.45584446301172 12.22992 35.69888698140448 140.4557477224495 12.22992 35.69888698140448 140.4557477224495 7.22992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69902177407096 140.45584446301172 7.22992 35.699071435188706 140.4557405369189 7.22992 35.699071435188706 140.4557405369189 12.22992 35.69902177407096 140.45584446301172 12.22992 35.69902177407096 140.45584446301172 7.22992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.699071435188706 140.4557405369189 12.22992 35.69899249919255 140.4556838999819 12.22992 35.69897463915291 140.45572133118588 12.22992 35.6989187824115 140.45568122765636 12.22992 35.69888698140448 140.4557477224495 12.22992 35.69902177407096 140.45584446301172 12.22992 35.699071435188706 140.4557405369189 12.22992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
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
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">160.7</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8231</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_d4d490c5-b73c-463c-a1ff-bd01ac5dbb73">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69834946779741 140.45690433828852 0 35.69833649946299 140.45688478948307 0 35.698314210324895 140.45690715775714 0 35.69832717922817 140.456926596066 0 35.69834946779741 140.45690433828852 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69834946779741 140.45690433828852 6.40548 35.69832717922817 140.456926596066 6.40548 35.698314210324895 140.45690715775714 6.40548 35.69833649946299 140.45688478948307 6.40548 35.69834946779741 140.45690433828852 6.40548</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69834946779741 140.45690433828852 6.40548 35.69833649946299 140.45688478948307 6.40548 35.69833649946299 140.45688478948307 12.40548 35.69834946779741 140.45690433828852 12.40548 35.69834946779741 140.45690433828852 6.40548</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69833649946299 140.45688478948307 6.40548 35.698314210324895 140.45690715775714 6.40548 35.698314210324895 140.45690715775714 12.40548 35.69833649946299 140.45688478948307 12.40548 35.69833649946299 140.45688478948307 6.40548</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.698314210324895 140.45690715775714 6.40548 35.69832717922817 140.456926596066 6.40548 35.69832717922817 140.456926596066 12.40548 35.698314210324895 140.45690715775714 12.40548 35.698314210324895 140.45690715775714 6.40548</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69832717922817 140.456926596066 6.40548 35.69834946779741 140.45690433828852 6.40548 35.69834946779741 140.45690433828852 12.40548 35.69832717922817 140.456926596066 12.40548 35.69832717922817 140.456926596066 6.40548</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69834946779741 140.45690433828852 12.40548 35.69833649946299 140.45688478948307 12.40548 35.698314210324895 140.45690715775714 12.40548 35.69832717922817 140.456926596066 12.40548 35.69834946779741 140.45690433828852 12.40548</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">7.3</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">211</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8198</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_defa6ebd-8001-4d94-8eb3-c00b52f5e060">
			<core:creationDate>2026-03-20</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:measuredHeight uom="m">-9999</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.694752437796595 140.4603752421233 0 35.69481969869508 140.46047521342516 0 35.69494963690879 140.4603438584845 0 35.69494516428641 140.46033730430605 0 35.69496491771038 140.46031734894694 0 35.69491008945135 140.46023581695093 0 35.69488761770753 140.46025851339908 0 35.69487965757031 140.46024662820972 0 35.694752437796595 140.4603752421233 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.694752437796595 140.4603752421233 5.96174 35.69487965757031 140.46024662820972 5.96174 35.69488761770753 140.46025851339908 5.96174 35.69491008945135 140.46023581695093 5.96174 35.69496491771038 140.46031734894694 5.96174 35.69494516428641 140.46033730430605 5.96174 35.69494963690879 140.4603438584845 5.96174 35.69481969869508 140.46047521342516 5.96174 35.694752437796595 140.4603752421233 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.694752437796595 140.4603752421233 5.96174 35.69481969869508 140.46047521342516 5.96174 35.69481969869508 140.46047521342516 11.961739999999999 35.694752437796595 140.4603752421233 11.961739999999999 35.694752437796595 140.4603752421233 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69481969869508 140.46047521342516 5.96174 35.69494963690879 140.4603438584845 5.96174 35.69494963690879 140.4603438584845 11.961739999999999 35.69481969869508 140.46047521342516 11.961739999999999 35.69481969869508 140.46047521342516 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69494963690879 140.4603438584845 5.96174 35.69494516428641 140.46033730430605 5.96174 35.69494516428641 140.46033730430605 11.961739999999999 35.69494963690879 140.4603438584845 11.961739999999999 35.69494963690879 140.4603438584845 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69494516428641 140.46033730430605 5.96174 35.69496491771038 140.46031734894694 5.96174 35.69496491771038 140.46031734894694 11.961739999999999 35.69494516428641 140.46033730430605 11.961739999999999 35.69494516428641 140.46033730430605 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69496491771038 140.46031734894694 5.96174 35.69491008945135 140.46023581695093 5.96174 35.69491008945135 140.46023581695093 11.961739999999999 35.69496491771038 140.46031734894694 11.961739999999999 35.69496491771038 140.46031734894694 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69491008945135 140.46023581695093 5.96174 35.69488761770753 140.46025851339908 5.96174 35.69488761770753 140.46025851339908 11.961739999999999 35.69491008945135 140.46023581695093 11.961739999999999 35.69491008945135 140.46023581695093 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69488761770753 140.46025851339908 5.96174 35.69487965757031 140.46024662820972 5.96174 35.69487965757031 140.46024662820972 11.961739999999999 35.69488761770753 140.46025851339908 11.961739999999999 35.69488761770753 140.46025851339908 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69487965757031 140.46024662820972 5.96174 35.694752437796595 140.4603752421233 5.96174 35.694752437796595 140.4603752421233 11.961739999999999 35.69487965757031 140.46024662820972 11.961739999999999 35.69487965757031 140.46024662820972 5.96174</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.694752437796595 140.4603752421233 11.961739999999999 35.69481969869508 140.46047521342516 11.961739999999999 35.69494963690879 140.4603438584845 11.961739999999999 35.69494516428641 140.46033730430605 11.961739999999999 35.69496491771038 140.46031734894694 11.961739999999999 35.69491008945135 140.46023581695093 11.961739999999999 35.69488761770753 140.46025851339908 11.961739999999999 35.69487965757031 140.46024662820972 11.961739999999999 35.694752437796595 140.4603752421233 11.961739999999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
			<uro:bldgDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">901</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">202</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2>99</uro:appearanceSrcDescLod2>
					<uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">11</uro:lod1HeightType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">9</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:bldgDataQualityAttribute>
			<uro:bldgDisasterRiskAttribute>
				<uro:LandSlideRiskAttribute>
					<uro:description codeSpace="../../codelists/LandSlideRiskAttribute_description.xml">1</uro:description>
					<uro:areaType codeSpace="../../codelists/LandSlideRiskAttribute_areaType.xml">4</uro:areaType>
				</uro:LandSlideRiskAttribute>
			</uro:bldgDisasterRiskAttribute>
			<uro:buildingDetailAttribute>
				<uro:BuildingDetailAttribute>
					<uro:buildingRoofEdgeArea uom="m2">245.8</uro:buildingRoofEdgeArea>
					<uro:urbanPlanType codeSpace="../../codelists/Common_urbanPlanType.xml">21</uro:urbanPlanType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">0</uro:districtsAndZonesType>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">41</uro:districtsAndZonesType>
					<uro:landUseType codeSpace="../../codelists/Common_landUseType.xml">213</uro:landUseType>
					<uro:specifiedBuildingCoverageRate>-9999</uro:specifiedBuildingCoverageRate>
					<uro:specifiedFloorAreaRate>-9999</uro:specifiedFloorAreaRate>
					<uro:surveyYear>0001</uro:surveyYear>
				</uro:BuildingDetailAttribute>
			</uro:buildingDetailAttribute>
			<uro:buildingIDAttribute>
				<uro:BuildingIDAttribute>
					<uro:buildingID>12347-bldg-8322</uro:buildingID>
					<uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12347</uro:city>
				</uro:BuildingIDAttribute>
			</uro:buildingIDAttribute>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>