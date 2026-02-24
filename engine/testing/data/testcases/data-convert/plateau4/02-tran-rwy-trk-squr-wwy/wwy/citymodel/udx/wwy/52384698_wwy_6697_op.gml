<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd ">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.0778981774801 138.85003657347107 0</gml:lowerCorner>
			<gml:upperCorner>35.08483590828251 138.8628279051643 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_3e4c67ac-52ed-41f1-8665-770dc0279915">
			<gml:name>大型船泊地</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設終了年度">
				<gen:value>昭和56年度</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.083108600023145 138.85003657347107 0 35.08212454323818 138.85058363299726 0 35.08207190875669 138.85044604940646 0 35.08172297104372 138.85063820120777 0 35.080159563938544 138.85118330166645 0 35.080315221855855 138.85128665653875 0 35.081749831809006 138.8507881796504 0 35.08204145673941 138.85062898824393 0 35.08208014189639 138.85073010949344 0 35.083098719585905 138.85016386124067 0 35.083108600023145 138.85003657347107 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-2-6</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">57</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">4</uro:geologicalType>
					<uro:plannedDepth uom="m">-5.5</uro:plannedDepth>
					<uro:innerArea uom="m2">1260</uro:innerArea>
					<uro:note>耐震岸壁築造に伴い、前面泊地-5.5(捨石基礎)</uro:note>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_4ab5185f-bf41-417a-af6f-6946144cc3c9">
			<gml:name>大型船泊地</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設開始年度">
				<gen:value>昭和38年度</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設終了年度">
				<gen:value>昭和44年度</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.08136159520416 138.85392071169494 0 35.082693926025115 138.85306943692714 0 35.08257021255562 138.852773104589 0 35.081514734888486 138.85342521355042 0 35.08136159520416 138.85392071169494 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-2-4</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">57</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">4</uro:geologicalType>
					<uro:plannedDepth uom="m">-5.5</uro:plannedDepth>
					<uro:innerArea uom="m2">4700</uro:innerArea>
					<uro:totalCost>71520000</uro:totalCost>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_02dd7186-9bda-4a60-8511-5c9fb1d7bd19">
			<gml:name>大型船泊地</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設開始年度">
				<gen:value>昭和38年度</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設終了年度">
				<gen:value>昭和44年度</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.0799984480634 138.8519611609151 0 35.079996414662354 138.85197263026842 0 35.07987567198491 138.8520673812066 0 35.07987200054038 138.85207083276518 0 35.0806038313394 138.85345274587328 0 35.081514734888486 138.85342521355042 0 35.08028209327721 138.8521239822704 0 35.0799984480634 138.8519611609151 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-2-3</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">57</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">4</uro:geologicalType>
					<uro:plannedDepth uom="m">-5.5</uro:plannedDepth>
					<uro:innerArea uom="m2">7840</uro:innerArea>
					<uro:totalCost>71520000</uro:totalCost>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_731be388-a9ce-444d-8dee-c2dae83698ae">
			<gml:name>大型船泊地</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設開始年度">
				<gen:value>昭和43年度</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設終了年度">
				<gen:value>昭和47年度</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.081514734888486 138.85342521355042 0 35.080604191317825 138.85345273499294 0 35.08090432927188 138.85402099380033 0 35.08136159520416 138.85392071169494 0 35.081514734888486 138.85342521355042 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-2-2</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">57</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">4</uro:geologicalType>
					<uro:plannedDepth uom="m">-4</uro:plannedDepth>
					<uro:innerArea uom="m2">3580</uro:innerArea>
					<uro:totalCost>1500000</uro:totalCost>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_5e04e2c1-6311-4e04-93fa-12542d7e8205">
			<gml:name>内港航路</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設開始年度">
				<gen:value>昭和45年度</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設終了年度">
				<gen:value>昭和47年度</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.08202545055105 138.85592767426874 0 35.08174098494225 138.85558788484897 0 35.08136159520416 138.85392071169494 0 35.08090432927188 138.85402099380033 0 35.08108548976673 138.85436169510623 0 35.08143539026648 138.85588401929073 0 35.08202545055105 138.85592767426874 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-1-1</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">58</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">3</uro:geologicalType>
					<uro:length uom="m">180</uro:length>
					<uro:minimumWidth uom="m">40</uro:minimumWidth>
					<uro:plannedDepth uom="m">-3.5</uro:plannedDepth>
					<uro:currentDepth uom="m">-3.5</uro:currentDepth>
					<uro:isDredged>0</uro:isDredged>
					<uro:areaType codeSpace="../../codelists/HarborFacility_areaType.xml">2</uro:areaType>
					<uro:totalCost>206100000</uro:totalCost>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_dae8ec58-5d36-4d04-97f1-dae84d22a4ff">
			<gml:name>大型船泊地</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設開始年度">
				<gen:value>昭和44年度</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設終了年度">
				<gen:value>平成14年度</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.080014315417166 138.8510945073778 0 35.07953397402667 138.85132987881008 0 35.078400178729986 138.8506804700885 0 35.0778981774801 138.85075676239163 0 35.07997659871654 138.85194863106605 0 35.07999111224077 138.85193724007712 0 35.08000217444485 138.85194014232624 0 35.0799984480634 138.8519611609151 0 35.08028209327721 138.8521239822704 0 35.081514734888486 138.85342521355042 0 35.08257021255562 138.852773104589 0 35.08268399796391 138.8530540416439 0 35.083685038215954 138.8524245445475 0 35.08481203483023 138.85173309581114 0 35.08483590828251 138.85152709235177 0 35.0845178096273 138.85125259452047 0 35.08425422493508 138.85141061551863 0 35.08406785606535 138.8508643193401 0 35.083108600023145 138.85003657347107 0 35.083098719585905 138.85016386124067 0 35.08208014189639 138.85073010949344 0 35.08204145673941 138.85062898824393 0 35.081749831809006 138.8507881796504 0 35.080315221855855 138.85128665653875 0 35.080159563938544 138.85118330166645 0 35.080014315417166 138.8510945073778 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-2-5</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">57</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">4</uro:geologicalType>
					<uro:plannedDepth uom="m">-7.5</uro:plannedDepth>
					<uro:innerArea uom="m2">96500</uro:innerArea>
					<uro:totalCost>61100000</uro:totalCost>
					<uro:note>平成14年度　外港東岸壁施工に伴う埋立により530㎡減</uro:note>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_4a91b0a1-a847-4de5-ae25-da18d15c2b51">
			<gml:name>小型船泊地</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="建設終了年度">
				<gen:value>昭和54年度</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.08138684593848 138.85588021343773 0 35.081573954889414 138.85701895125987 0 35.083800113498825 138.85720097712635 0 35.083842458104506 138.8571154064503 0 35.0835063314968 138.85598120918146 0 35.08208252224719 138.85586128208018 0 35.08207778340816 138.85593601715857 0 35.08202545055105 138.85592767426874 0 35.08143539026648 138.85588401929073 0 35.08138684593848 138.85588021343773 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-2-1</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">57</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">4</uro:geologicalType>
					<uro:plannedDepth uom="m">-3.5</uro:plannedDepth>
					<uro:innerArea uom="m2">26150</uro:innerArea>
					<uro:totalCost>14707000</uro:totalCost>
					<uro:note>平成14年度　内港南物揚場施工に伴う埋立により3,060㎡減</uro:note>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_78346e51-3b51-4b18-b149-28d33e342197">
			<gml:name>狩野川航路</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.07860422424014 138.85180852539497 0 35.07844014709661 138.85205166877913 0 35.079719479918715 138.85384578258834 0 35.080093847544696 138.85462537798765 0 35.07938806215378 138.85814082775374 0 35.07926475277769 138.85839708665355 0 35.07911588662498 138.85913085119128 0 35.07944431556674 138.8603093570711 0 35.079473665232115 138.86039520761517 0 35.0796457175909 138.86069660749362 0 35.07966655410535 138.86074541115778 0 35.079697713162616 138.860833093481 0 35.07971694262215 138.8609031660165 0 35.07973065414513 138.86100424831855 0 35.07973670959123 138.86112898101513 0 35.07974210066914 138.86116004715927 0 35.07976336857987 138.86121433704835 0 35.079783281430004 138.86124361384609 0 35.07987302683944 138.86133598670665 0 35.079904655815255 138.861289209104 0 35.07991325939308 138.86129791390613 0 35.07994258771067 138.86129631497928 0 35.079962478663795 138.86130606941717 0 35.0800154115851 138.8613548202372 0 35.080028658534566 138.8613824175196 0 35.080046948091336 138.86146334357042 0 35.08006717468677 138.86152922538125 0 35.080426364193734 138.86189632269432 0 35.080447724461514 138.86186935799876 0 35.080470820798084 138.86186487126997 0 35.080501361349405 138.86187150047837 0 35.08053456705264 138.86188944020418 0 35.080562323765974 138.86191864843215 0 35.08058214925712 138.8619519211057 0 35.08058615798224 138.86196088790146 0 35.08058868188923 138.86198678475952 0 35.0805841949202 138.86205846292444 0 35.080700705996755 138.86217919068457 0 35.0807266758227 138.86217800925309 0 35.080791993890344 138.86224283817964 0 35.080799910993306 138.8622733695209 0 35.080830637411545 138.8623041288627 0 35.080850401579866 138.86227286368623 0 35.080865475482476 138.86226789672216 0 35.08090396134376 138.86227127700874 0 35.0809478108876 138.8622877367527 0 35.080971651724845 138.86230233781967 0 35.08098620602461 138.86231700043612 0 35.08099550312824 138.86233964222544 0 35.08099669726334 138.86235906133618 0 35.08098038093264 138.86243553155185 0 35.0811059570816 138.86254257048938 0 35.08128136893862 138.8626690934149 0 35.08130145555209 138.86263549534718 0 35.08132142565075 138.86262265704465 0 35.08134271962186 138.8626179417575 0 35.08136659429779 138.86262289155079 0 35.08142191072652 138.86265985365552 0 35.08159250672813 138.86280985941772 0 35.081645997595835 138.8628279051643 0 35.0816984353568 138.86268598602106 0 35.08003004838411 138.86116965439578 0 35.07994924830742 138.86067537301597 0 35.07928719765306 138.8595411655623 0 35.07924424923342 138.85902894265837 0 35.07966378346484 138.8583562918884 0 35.08039474501535 138.85459826344226 0 35.079992230573886 138.8537550397679 0 35.07860422424014 138.85180852539497 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-1-2</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">58</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">3</uro:geologicalType>
					<uro:length uom="m">1175</uro:length>
					<uro:minimumWidth uom="m">7</uro:minimumWidth>
					<uro:maximumWidth uom="m">30</uro:maximumWidth>
					<uro:plannedDepth uom="m">-1.5</uro:plannedDepth>
					<uro:isDredged>1</uro:isDredged>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Waterway gml:id="wwy_24086a36-e741-4037-8fa0-d8b8725faf43">
			<gml:name>小型船我入道泊地</gml:name>
			<core:creationDate>2021-03-26</core:creationDate>
			<gen:stringAttribute name="管理者名等">
				<gen:value>静岡県</gen:value>
			</gen:stringAttribute>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.07926475277769 138.85839708665355 0 35.07907233996126 138.85832251333386 0 35.07895466637441 138.85880355959912 0 35.078952567332536 138.85888865639825 0 35.07895172098761 138.8589246253421 0 35.07896617446865 138.85909798589992 0 35.07901435788749 138.85947540572434 0 35.079034471757495 138.85957651990654 0 35.07906026008586 138.85965331557404 0 35.07915970804452 138.85983358561015 0 35.07920388938582 138.85991014732053 0 35.07922221444033 138.85995531830213 0 35.07923512707723 138.85997545546726 0 35.07926988124997 138.86004067213977 0 35.079473665232115 138.86039520761517 0 35.07944431556674 138.8603093570711 0 35.07911588662498 138.85913085119128 0 35.07926475277769 138.85839708665355 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranFacilityIdAttribute>
				<uro:FacilityIdAttribute>
					<uro:id>A-2-7</uro:id>
				</uro:FacilityIdAttribute>
			</uro:tranFacilityIdAttribute>
			<uro:tranFacilityAttribute>
				<uro:HarborFacility>
					<uro:portFacilityDetailsType codeSpace="../../codelists/PortAttribute_facilityDetailType.xml">57</uro:portFacilityDetailsType>
					<uro:portName>沼津港</uro:portName>
					<uro:geologicalType codeSpace="../../codelists/PortAttribute_geologicalType.xml">4</uro:geologicalType>
					<uro:plannedDepth uom="m">-1.5</uro:plannedDepth>
					<uro:innerArea uom="m2">500</uro:innerArea>
				</uro:HarborFacility>
			</uro:tranFacilityAttribute>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
		</uro:Waterway>
	</core:cityObjectMember>
</core:CityModel>
