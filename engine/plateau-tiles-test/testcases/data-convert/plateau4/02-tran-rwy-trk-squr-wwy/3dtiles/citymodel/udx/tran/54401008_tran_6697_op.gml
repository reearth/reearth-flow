<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsDimension="3" srsName="http://www.opengis.net/def/crs/EPSG/0/6697">
			<gml:lowerCorner>36.0821455602610 140.0997260017590 0.0000000000000</gml:lowerCorner>
			<gml:upperCorner>36.0921280621680 140.1129533346330 25.2788066900000</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_dde0287d-c067-4ded-9df4-fd6bccfab2a7">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08333799495752 140.11130091947854 0 36.083458832005135 140.1112183178866 0 36.08339704178814 140.11108112728465 0 36.08339345176743 140.11107456346815 0 36.08337285575951 140.1110295387771 0 36.083355854074206 140.1110413421863 0 36.08326845381911 140.11114651989791 0 36.08333799495752 140.11130091947854 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9c94cb00-fd90-4335-a98f-e6b544055127">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.083498248765856 140.10510978834986 0 36.08352300633909 140.10500305567209 0 36.08347072445509 140.1048863977476 0 36.083383723188255 140.10485766195652 0 36.08332015618664 140.10498791187814 0 36.083310685471574 140.10503075068527 0 36.08333714221692 140.1050889029848 0 36.08337172446137 140.10510124718144 0 36.083498248765856 140.10510978834986 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c75fb4e8-7034-4387-849c-8e76e807b4cf">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.083510704848 140.10541962524894 0 36.08360369463399 140.10535132453666 0 36.083498248765856 140.10510978834986 0 36.08337172446137 140.10510124718144 0 36.08341906711698 140.10520989341933 0 36.0834391522492 140.10525580982613 0 36.083510704848 140.10541962524894 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8de9a16b-fdd3-4825-b8d9-00b305b8d043">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.083707191864086 140.11084337643283 0 36.08371078123749 140.11081129919478 0 36.08370926470803 140.11080452057521 0 36.08359663949326 140.1105542906947 0 36.08344446497101 140.11021798003176 0 36.083390025458826 140.1102572073161 0 36.08354148217188 140.11059207181628 0 36.083628013655435 140.1107842483316 0 36.08362110592994 140.11080920748134 0 36.08361283570694 140.11083871437256 0 36.08359309864377 140.110876952909 0 36.08346851854314 140.1109631240637 0 36.08337285575951 140.1110295387771 0 36.08339345176743 140.11107456346815 0 36.083707191864086 140.11084337643283 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_64783a19-463d-4776-80e9-89fd4441eb58">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">2</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08386506176597 140.10591207740418 0 36.08387507329888 140.10586936241154 0 36.083671978479515 140.10536688293794 0 36.083667194022425 140.10537019758064 0 36.08365655640081 140.10537116027098 0 36.083642227225155 140.10537044468134 0 36.08362781648861 140.10536595353767 0 36.083616381557746 140.1053608064225 0 36.08360846180789 140.10535522726843 0 36.08360369463399 140.10535132453666 0 36.083510704848 140.10541962524894 0 36.0835140051257 140.10543484872937 0 36.083515054165524 140.10544917615485 0 36.0835140292025 140.1054639405824 0 36.08351012839454 140.10547503086786 0 36.08350244565082 140.105484553596 0 36.08349340969385 140.1054946268446 0 36.083487091256124 140.10549871344583 0 36.08373623373213 140.10596537489891 0 36.08377090629005 140.10597759771306 0 36.08386506176597 140.10591207740418 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_41e41829-315f-415b-b113-3f4936b0f9a1">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084060496888476 140.10153688740436 0 36.08406410588068 140.101501777209 0 36.083933996814636 140.10148356928352 0 36.08392841163494 140.10148266201395 0 36.083882641673334 140.10147528912808 0 36.083772812036514 140.10145748326187 0 36.08371902570606 140.10144775139742 0 36.08365884264305 140.1014368763492 0 36.08365073093029 140.10143684880805 0 36.08363811271043 140.1014368059663 0 36.08361431835289 140.10143672517904 0 36.08357114325736 140.10143781110492 0 36.08354436442054 140.10144227271567 0 36.083514070128174 140.101446489209 0 36.083485400179455 140.10145062238456 0 36.08336233452406 140.10146719320502 0 36.08335502953549 140.10146915596405 0 36.083336271509516 140.10147398900324 0 36.0833169670133 140.10148136293236 0 36.083294232850626 140.10149083491413 0 36.083270515636556 140.1015046140068 0 36.08328205701945 140.10153476420882 0 36.08328891610443 140.10153067913674 0 36.083304347609555 140.1015217264467 0 36.083325278308905 140.10151303671077 0 36.083343139507235 140.1015062019687 0 36.083366587233755 140.10150007462533 0 36.08348848070464 140.10148372194934 0 36.083518683991784 140.10147949405888 0 36.08354780459201 140.10147536242238 0 36.083573232105906 140.10147101837515 0 36.08361451422023 140.10147003707695 0 36.08363794715582 140.10147011664696 0 36.08365687538687 140.1014701809207 0 36.083769042916984 140.1014904486489 0 36.08392491265026 140.10151573942 0 36.084060496888476 140.10153688740436 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c699c62b-2c8b-4595-8a1f-ee1f323de0b6">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08406729870524 140.10007754810553 0 36.084096701968974 140.1000213055769 0 36.08407348693557 140.10000390522893 0 36.084044460227645 140.10005942727224 0 36.08406729870524 140.10007754810553 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1a3c5c49-b3d1-4dab-9d20-5ad2cb060e91">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08408574002812 140.10154082505588 0 36.08410720518893 140.10153455768702 0 36.08410412465435 140.10150146896456 0 36.08410434302584 140.1014941045469 0 36.08407566268756 140.10149281126792 0 36.08406410588068 140.101501777209 0 36.084060496888476 140.10153688740436 0 36.08408574002812 140.10154082505588 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2bcf9926-3ec0-404b-9c52-58ace3ab59fe">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08411184024436 140.10182535234296 0 36.08412342458122 140.10173771267162 0 36.08397081152212 140.10170710264376 0 36.08393171346788 140.10169874185402 0 36.08375378971042 140.1016609506629 0 36.083515955017276 140.1016115081466 0 36.08349649587926 140.10160743358995 0 36.08330080500772 140.10157469016858 0 36.08329107545015 140.1015726473484 0 36.083279092093896 140.10166022480544 0 36.0832899730384 140.10166248362916 0 36.08348557373039 140.10169523806059 0 36.083641697653945 140.1017277474416 0 36.08374169545076 140.10174851824868 0 36.083958898389774 140.10179467107864 0 36.08411184024436 140.10182535234296 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_27fdebb1-621e-4cc7-a226-bef00640fb94">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08412500517596 140.10054637850325 0 36.08413496947122 140.10048356461408 0 36.08411482273009 140.10047781352083 0 36.0841044856821 140.10053277351642 0 36.084102661541124 140.1005424276613 0 36.08409885240178 140.10059304815704 0 36.084097350781406 140.10066055425423 0 36.08409335581024 140.10095633517238 0 36.084093102622646 140.1009889906246 0 36.08408163629285 140.10132006737447 0 36.08407566268756 140.10149281126792 0 36.08410434302584 140.1014941045469 0 36.08411426146166 140.10116017212917 0 36.084114608522896 140.10112586251887 0 36.0841158875762 140.1009567557554 0 36.08411988210293 140.10066117462003 0 36.08412138147042 140.10059467894322 0 36.08412500517596 140.10054637850325 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c976e3fe-82da-44fc-9e03-e7d69a072fae">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08412342458122 140.10173771267162 0 36.084147657650185 140.10174312487555 0 36.08411068472926 140.10155044795943 0 36.084109257780774 140.10154368088988 0 36.08410720518893 140.10153455768702 0 36.08408574002812 140.10154082505588 0 36.08412342458122 140.10173771267162 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a9e92e2a-8f34-4365-9ac0-383c9a8071f3">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08413622055399 140.10183021432422 0 36.084147657650185 140.10174312487555 0 36.08412342458122 140.10173771267162 0 36.08411184024436 140.10182535234296 0 36.08411430331511 140.1018258459546 0 36.08413622055399 140.10183021432422 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_60d54c45-ecda-477d-aac2-df9e56ef89db">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08415872959457 140.1001348739776 0 36.08417139730689 140.10007219128335 0 36.08412603571827 140.10004327916636 0 36.084096701968974 140.1000213055769 0 36.08406729870524 140.10007754810553 0 36.08407286256915 140.10008196289868 0 36.0841006678897 140.10010182161582 0 36.08413279754859 140.10012336053003 0 36.08415872959457 140.1001348739776 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5e11ea6a-104f-4fc9-8d0a-3319dba825af">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084169670293406 140.10048336000384 0 36.08420208042528 140.1004190674568 0 36.08419793858172 140.1003767478679 0 36.08413622914844 140.10036320348925 0 36.08413549969553 140.10036697632054 0 36.08412455291367 140.10042580063165 0 36.08412053895527 140.1004474283914 0 36.08411482273009 140.10047781352083 0 36.08413496947122 140.10048356461408 0 36.084169670293406 140.10048336000384 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_78ccb521-2e66-44f6-9c1c-ddb675f17007">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08419793858172 140.1003767478679 0 36.08421996046863 140.10016096389248 0 36.08415708451375 140.10015155968287 0 36.08413622914844 140.10036320348925 0 36.08419793858172 140.1003767478679 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c5144660-c139-4976-ad91-bc7739aa86c2">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084205769271016 140.10419693506307 0 36.084216861253296 140.10415444544927 0 36.08417022955534 140.10412963503168 0 36.08413881177823 140.10411287156177 0 36.08412496394039 140.1041384627846 0 36.08384826128298 140.1042186935472 0 36.08363008075269 140.1042864551969 0 36.08357489164376 140.10429970134876 0 36.08329722733405 140.1043667031935 0 36.0832001089594 140.10436737029784 0 36.08320025985161 140.10440068076102 0 36.08329994603104 140.1044000236226 0 36.08351746328942 140.10434580683838 0 36.08363623270096 140.1043161233095 0 36.08371632287408 140.1042926361132 0 36.0839700290792 140.10421822296772 0 36.084061953228435 140.1041824509429 0 36.08412012840796 140.1041639960786 0 36.0841442801651 140.1041658555259 0 36.084205769271016 140.10419693506307 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_3bdbd577-226a-4013-a442-ddb987f33762">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08421996046863 140.10016096389248 0 36.084241670806875 140.10012550498973 0 36.084207806919345 140.100073646735 0 36.08417139730689 140.10007219128335 0 36.08415872959457 140.1001348739776 0 36.08415708451375 140.10015155968287 0 36.08421996046863 140.10016096389248 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f0b65a14-6666-4744-bb1e-d1a59d2b5e94">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084331321069335 140.1043167320655 0 36.084337609479725 140.104297605065 0 36.0842187447747 140.10423828581438 0 36.08421040109741 140.10426113107326 0 36.08418775208723 140.10431257510686 0 36.08417815812399 140.1043303082995 0 36.084155370283554 140.10436341940962 0 36.08412309536543 140.10440684668924 0 36.084099955498324 140.1044361923534 0 36.08407953560426 140.10445844090216 0 36.08406210833838 140.10447259393698 0 36.083860980582195 140.10461236611505 0 36.083736765468146 140.10469854871124 0 36.08369262093737 140.10472970959387 0 36.08347072445509 140.1048863977476 0 36.08352300633909 140.10500305567209 0 36.083682883958154 140.10489012535885 0 36.08379022098368 140.10481443367735 0 36.08387011238768 140.1047589673743 0 36.08389674301749 140.1047404045571 0 36.08411872364835 140.10458626932137 0 36.084145542326354 140.10456425374645 0 36.08417463606337 140.10453260785295 0 36.08420175298433 140.10449827912552 0 36.084236468707644 140.10445165121376 0 36.08426468248729 140.104410552858 0 36.08428043027514 140.10438140382809 0 36.084297734669015 140.10434226667357 0 36.084309281366714 140.10433786476662 0 36.08431717089309 140.1043165724667 0 36.08432069475469 140.10431269821484 0 36.0843274535087 140.10431316556284 0 36.084331321069335 140.1043167320655 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_37655a32-fa56-4510-88c6-b03e6cc1880c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0843550988324 140.10424441657915 0 36.08437057302073 140.10417918236328 0 36.084249441129295 140.10413556963505 0 36.084216861253296 140.10415444544927 0 36.084205769271016 140.10419693506307 0 36.0842187447747 140.10423828581438 0 36.084337609479725 140.104297605065 0 36.0843550988324 140.10424441657915 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_35f3d6f2-60d8-4ffc-9b50-eaf3795f5f1c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08444669604065 140.10359493291455 0 36.08444687880486 140.103593823154 0 36.084321424922415 140.1035757952899 0 36.08429970568996 140.10374899527125 0 36.08427189575471 140.10397063253592 0 36.084249441129295 140.10413556963505 0 36.08437057302073 140.10417918236328 0 36.08437220642184 140.1041723002477 0 36.084379628651654 140.10411847202772 0 36.08439649050856 140.10399549915473 0 36.08444669604065 140.10359493291455 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e5b48ed3-903e-4add-ba97-699d1533b083">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08444687880486 140.103593823154 0 36.084457571495 140.10352845800261 0 36.08432772695124 140.10350979895347 0 36.084321641512894 140.1035740738251 0 36.084321424922415 140.1035757952899 0 36.08444687880486 140.103593823154 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0e5bd9df-32bb-42bb-8bb0-c264c8dbe9d9">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08447486791028 140.10188825426013 0 36.0844877217992 140.10182334035505 0 36.08442556039427 140.10181045948997 0 36.084307993532356 140.10178641967772 0 36.084281597840416 140.10178077802027 0 36.08418673582907 140.10176035759915 0 36.084147657650185 140.10174312487555 0 36.08413622055399 140.10183021432422 0 36.08415907923954 140.10183477022096 0 36.084168567033224 140.1018240317752 0 36.08429883235615 140.1018521232674 0 36.08441666957016 140.1018761751979 0 36.08447486791028 140.10188825426013 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_b4e49365-902e-4753-930a-2de56d043bac">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084512877037746 140.1029435843315 0 36.08457727688921 140.10236140574824 0 36.08444777705265 140.10233967288247 0 36.084387640167925 140.1028835290329 0 36.08438322425893 140.10292337678717 0 36.084375467966225 140.10300574087958 0 36.08435487904037 140.10322330598422 0 36.08434398507519 140.10333808252068 0 36.08432772695124 140.10350979895347 0 36.084457571495 140.10352845800261 0 36.084512877037746 140.1029435843315 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ec600147-2aec-463b-b8a1-d875adcbe7e5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084555696044056 140.10809959304527 0 36.08458150305631 140.108086680205 0 36.08455400627649 140.1078930439947 0 36.08436090453217 140.10802918306604 0 36.08430150529808 140.1080698274516 0 36.084010015613885 140.10826924793213 0 36.08396271190826 140.1083016173387 0 36.08395485900887 140.10830703085048 0 36.08386368366492 140.10836944969574 0 36.08382423463796 140.10839640549605 0 36.08363249578492 140.10852752794543 0 36.083300281631956 140.10875944790604 0 36.08325929557434 140.10878806351303 0 36.083249185572015 140.1087951346087 0 36.083087139764444 140.10890827089656 0 36.08307757155673 140.10891489970567 0 36.08303830164308 140.1089422997389 0 36.082987566584144 140.10897775428919 0 36.08296283084244 140.10899510067685 0 36.08289340890101 140.10904350338498 0 36.08287797071981 140.10905433104892 0 36.08268703767553 140.10918746291108 0 36.08255676970855 140.10927872353346 0 36.0825182215177 140.10930579262111 0 36.08229569021086 140.10946157508454 0 36.082256509142425 140.10948908592303 0 36.08214556026165 140.10956675578709 0 36.082209199368876 140.10972020730895 0 36.082640811602204 140.1094179122068 0 36.082710775959534 140.10936886786803 0 36.082781913547315 140.10931915022198 0 36.08300381170181 140.10916425248737 0 36.08303513744129 140.1091423766605 0 36.08370083061575 140.108677666779 0 36.08404350524112 140.1084433611243 0 36.08409071795278 140.108410991453 0 36.08447013357783 140.1081512613896 0 36.084555696044056 140.10809959304527 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1aef5f1d-5603-47bf-b55d-2e7d3f11c241">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08460884130068 140.1023667587648 0 36.08461625929976 140.10230080031312 0 36.084455059676834 140.10227378738708 0 36.08445351220902 140.10228780962544 0 36.08444777705265 140.10233967288247 0 36.08457727688921 140.10236140574824 0 36.084604035564695 140.1023659384699 0 36.08460884130068 140.1023667587648 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ac24caed-145c-40a0-9995-81afa88faa81">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084669473503915 140.1020830121679 0 36.08468389935228 140.10205187819994 0 36.084494268335035 140.10191852276267 0 36.08447945530415 140.10205282915302 0 36.084455059676834 140.10227378738708 0 36.08461625929976 140.10230080031312 0 36.08461983417983 140.10227416316516 0 36.08463215870713 140.10220425065273 0 36.08464670244615 140.1021491027418 0 36.084669473503915 140.1020830121679 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9f9828ee-af7d-4ee5-8d21-f4cf95bd4708">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08467374554243 140.1036304652166 0 36.08468101481096 140.10356442193302 0 36.08448441987975 140.1035327582383 0 36.084457571495 140.10352845800261 0 36.08444687880486 140.103593823154 0 36.084476520844994 140.1035986992703 0 36.08467374554243 140.1036304652166 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4d9638c7-3551-412c-a087-fdf896160466">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084586902419055 140.1077756904399 0 36.08468437599214 140.1077175117066 0 36.08465173843924 140.1076432243199 0 36.08459443054925 140.10751877281373 0 36.084540711595366 140.1074009962683 0 36.08452752851574 140.10737206934297 0 36.084477126971436 140.10726185513258 0 36.08445945847911 140.1072232745433 0 36.0844104020394 140.1071163963429 0 36.08435499176594 140.10698917621377 0 36.08430450657875 140.10687596416173 0 36.08425562335828 140.10677164086795 0 36.08424683316905 140.1067529559914 0 36.084240375239055 140.10673915377592 0 36.08423436664076 140.1067259305176 0 36.0842020821503 140.10665464321843 0 36.08416432637437 140.10657123388026 0 36.08407114776056 140.1063665902989 0 36.08400181908239 140.10621623834348 0 36.08397401553963 140.10615605963326 0 36.083945768088505 140.10609290361643 0 36.0839239768982 140.10604430479756 0 36.0838900797602 140.10596857097616 0 36.08386506176597 140.10591207740418 0 36.08377090629005 140.10597759771306 0 36.08379520559634 140.10603264526628 0 36.083824889214966 140.10609881522413 0 36.083846590253636 140.10614741368536 0 36.083879679757416 140.10622114606983 0 36.0839769920157 140.10643212133732 0 36.0840699023222 140.10663619752808 0 36.084108553894175 140.10672160858476 0 36.08409547895038 140.1068028432088 0 36.08410614958921 140.10682730855007 0 36.08416774535897 140.10685083991393 0 36.08421026070957 140.10694137232935 0 36.0842454119492 140.10702022029392 0 36.08425993872298 140.10705280487608 0 36.0842669320967 140.1070689296781 0 36.08431561756986 140.10718079199256 0 36.084365303201615 140.1072887937757 0 36.084388979252665 140.1073406198664 0 36.08436657787133 140.10740104700142 0 36.08437634491494 140.10742606456333 0 36.08443453782784 140.10744026852606 0 36.08444646544522 140.10746640400117 0 36.08450027421207 140.10758429180052 0 36.08455713431994 140.1077078533392 0 36.084586902419055 140.1077756904399 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_da40835c-5556-4734-b97b-168c571acca4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084697007411435 140.10202358899633 0 36.08471860923341 140.10198491655632 0 36.084517934794285 140.10181478206673 0 36.0844877217992 140.10182334035505 0 36.08447486791028 140.10188825426013 0 36.084494268335035 140.10191852276267 0 36.08468389935228 140.10205187819994 0 36.084697007411435 140.10202358899633 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4dbea4b3-4a4b-44dd-8017-74c5a2cfc175">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08470171763824 140.1016510692773 0 36.08474892877715 140.10161858422742 0 36.08465178373853 140.10141005622114 0 36.084646581525 140.10139871259227 0 36.08456029875356 140.1012090985652 0 36.08454047793032 140.1011656151742 0 36.08448917515626 140.1010524037901 0 36.08442217766362 140.10090493948047 0 36.08441939726418 140.1008988229371 0 36.08430084072201 140.10063115175478 0 36.08429868890133 140.10062648085753 0 36.08420208042528 140.1004190674568 0 36.084169670293406 140.10048336000384 0 36.08425363074683 140.10066363721717 0 36.08429470374688 140.10075638229748 0 36.08431344678597 140.100798640377 0 36.08437209693018 140.1009309748658 0 36.08438151425425 140.10095175988027 0 36.08449335681852 140.10119820065054 0 36.08451703498905 140.10125025819585 0 36.08460475192165 140.10144298612045 0 36.084627625820005 140.10149190978507 0 36.08470171763824 140.1016510692773 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_7b3d08bb-42c6-4965-9f3d-f8c51db56fd1">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08477789838249 140.1036474776083 0 36.08478516766062 140.10358143423926 0 36.084763255282084 140.1035461598638 0 36.084709647143804 140.10353742641124 0 36.08468101481096 140.10356442193302 0 36.08467374554243 140.1036304652166 0 36.08469565792161 140.10366572847136 0 36.08474926627572 140.10367436207497 0 36.08477789838249 140.1036474776083 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0e6870ea-712c-4a8c-bb36-09430533deab">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08477931480315 140.10189494159664 0 36.08480621109555 140.10186823489505 0 36.084692158008 140.10169389783064 0 36.08452136496281 140.1018124619167 0 36.084517934794285 140.10181478206673 0 36.08471860923341 140.10198491655632 0 36.084730128206886 140.10196429582913 0 36.08477931480315 140.10189494159664 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0ba0344e-f1c8-46ac-bc9c-f018c7cb3321">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08480981547988 140.1024010617174 0 36.084830238084166 140.10233728379797 0 36.08461625929976 140.10230080031312 0 36.08460884130068 140.1023667587648 0 36.08480981547988 140.1024010617174 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ad6dfc23-e420-4d76-939d-3e8b93d3f4c6">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084802620026856 140.10451376720442 0 36.08484965265338 140.10448085016168 0 36.08470703776595 140.10417433702872 0 36.0846991210695 140.10412789558868 0 36.084705285722784 140.10407217516735 0 36.08474926627572 140.10367436207497 0 36.08469565792161 140.10366572847136 0 36.08464431619418 140.10413026153302 0 36.08465178776004 140.10417425855854 0 36.08465578940996 140.1041982566866 0 36.084802620026856 140.10451376720442 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_75f022e3-c092-4be0-b721-247b8b1e42f1">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08488070562112 140.10246026510845 0 36.08488161586891 140.10245628412358 0 36.08482956083518 140.10243821607716 0 36.084828103544 140.1024449844986 0 36.08482334873027 140.1024752708748 0 36.084821217607995 140.10250114683367 0 36.08480363457192 140.10266496952104 0 36.0847831054216 140.1028562309409 0 36.08476893114621 140.10298698657448 0 36.084748128117056 140.10317956825358 0 36.08473330857065 140.10331653974853 0 36.084709647143804 140.10353742641124 0 36.084763255282084 140.1035461598638 0 36.08478691672543 140.103325273054 0 36.08479630633403 140.1032381394665 0 36.08481250598591 140.10308862514148 0 36.08483137442672 140.10291458021473 0 36.08483671363576 140.10286495283592 0 36.08484766804926 140.10276317841047 0 36.08487491819838 140.102508880558 0 36.08487686454851 140.10248501376174 0 36.08488070562112 140.10246026510845 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_fa8441fd-8b72-4ffe-b70d-2466208c977c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.084879402357295 140.1017955798965 0 36.084887550977385 140.1017891895336 0 36.08478369290441 140.10163036144866 0 36.08474892877715 140.10161858422742 0 36.08470171763824 140.1016510692773 0 36.084692158008 140.10169389783064 0 36.08480621109555 140.10186823489505 0 36.084838482120055 140.10183619214365 0 36.084879402357295 140.1017955798965 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_18341375-afe5-4e61-a36e-36bb95a8ac4c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08491034182471 140.10234732814968 0 36.084913696464334 140.1023393103292 0 36.08486228038279 140.1023168507081 0 36.084830238084166 140.10233728379797 0 36.08480981547988 140.1024010617174 0 36.08482956083518 140.10243821607716 0 36.08488161586891 140.10245628412358 0 36.08489217471173 140.102410114464 0 36.08490152950628 140.1023779449599 0 36.08491034182471 140.10234732814968 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_18e14111-a3c4-4699-acf7-a8efd26f6e5c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08489217349083 140.1080519066468 0 36.0849319438473 140.10788389934757 0 36.0848555775915 140.10785646142415 0 36.08481252961315 140.10784098836297 0 36.0848059563599 140.10783863369005 0 36.08478094241643 140.10782011424553 0 36.084775453662466 140.10781608665545 0 36.08473650540781 140.10778186229123 0 36.084707207137924 140.1077450176662 0 36.08468437599214 140.1077175117066 0 36.084586902419055 140.1077756904399 0 36.08455400627649 140.1078930439947 0 36.08458150305631 140.108086680205 0 36.08461686049526 140.10807526629878 0 36.08474554636725 140.108045067285 0 36.08477024786203 140.10804259932485 0 36.084806659674335 140.10804304801616 0 36.08489217349083 140.1080519066468 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_6e3261e0-aae2-4bc5-9588-e4aee71df666">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08492511737887 140.10367174400096 0 36.08508972872087 140.10369729147237 0 36.085081681168674 140.10362886358524 0 36.08493211653084 140.10360558854728 0 36.08490202384283 140.1036007108567 0 36.08478516766062 140.10358143423926 0 36.08477789838249 140.1036474776083 0 36.08489718731526 140.10366709576817 0 36.08492511737887 140.10367174400096 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8a138522-20e0-413c-9d57-8e55cb2f6bc3">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085163001311095 140.1036190373537 0 36.085200406467735 140.10359307107205 0 36.08518270552434 140.10355436869096 0 36.085171640552055 140.1035450034953 0 36.085118082820216 140.10353550177535 0 36.0851103132642 140.1036019789896 0 36.085081681168674 140.10362886358524 0 36.08508972872087 140.10369729147237 0 36.08511968643205 140.10372204482965 0 36.08516429737692 140.10368377788646 0 36.08515923384994 140.1036510038649 0 36.085163001311095 140.1036190373537 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f8a9d671-5837-4052-9f7f-26bf1e8f55be">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0851948553365 140.1038854661306 0 36.08524187955765 140.10385260182136 0 36.08522654184803 140.10381912632334 0 36.085191832291265 140.10374372253662 0 36.08516429737692 140.10368377788646 0 36.08511968643205 140.10372204482965 0 36.08517942040066 140.1038519436765 0 36.0851948553365 140.1038854661306 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4df9d34b-3870-42b7-91fc-6db4cc654be5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08525731065125 140.10276402099964 0 36.08525985154377 140.10275635682285 0 36.08520823035594 140.10273496211158 0 36.08520480149325 140.10274751899368 0 36.08519884892265 140.10278936063793 0 36.08519746835711 140.1028016813295 0 36.08519213334616 140.1028498543341 0 36.085169217083006 140.1030604071784 0 36.08514507418399 140.10329518454458 0 36.08513668857369 140.10337632580627 0 36.085118082820216 140.10353550177535 0 36.085171640552055 140.1035450034953 0 36.08519020701594 140.10338526986718 0 36.085212136017375 140.10317271497473 0 36.0852229156081 140.1030690407331 0 36.08522788634044 140.10302297623508 0 36.085228161228464 140.10302097845923 0 36.085245741282236 140.10285869841454 0 36.085252272456124 140.10279964756543 0 36.08525731065125 140.10276402099964 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_32d01295-2374-49cf-bb55-05ac1f7a2a7f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08525263064419 140.10127351853305 0 36.0852998414301 140.1012411440712 0 36.08520261628292 140.10102784009027 0 36.085186470330605 140.1009926968206 0 36.08503587561599 140.10066384270226 0 36.085020000352216 140.10062892257636 0 36.0850085199109 140.10060367776802 0 36.084929231639656 140.1004295218546 0 36.08491927659855 140.10040772448133 0 36.08477845510394 140.10010066926594 0 36.084766615512514 140.1000747571639 0 36.08476320761977 140.10006708395633 0 36.084710747839225 140.0999472067165 0 36.084682231217016 140.09988203050236 0 36.084630267332635 140.09990385189099 0 36.08471922504708 140.10010700935754 0 36.084882201487744 140.10046223038063 0 36.084987947434726 140.1006946493182 0 36.084988844252344 140.10069666216756 0 36.08507495016015 140.10088471041306 0 36.085155494939464 140.10106054806243 0 36.08525263064419 140.10127351853305 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e8e62949-e9ef-44ca-9ba0-e3bcac750ea0">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085293630921356 140.103965151398 0 36.08530328126701 140.103922423043 0 36.08527664320735 140.1038644911386 0 36.08524187955765 140.10385260182136 0 36.0851948553365 140.1038854661306 0 36.08521117103836 140.10392089705704 0 36.0852465040671 140.1039979509066 0 36.085293630921356 140.103965151398 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5138fa14-1236-4883-80ca-5848d7662f38">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08520167709739 140.10156881110066 0 36.08532000736108 140.10148815399717 0 36.08524280100623 140.1013162354748 0 36.085116063874295 140.10140274931405 0 36.08485175740058 140.10158306795017 0 36.08478369290441 140.10163036144866 0 36.084887550977385 140.1017891895336 0 36.08494919605159 140.10174085272757 0 36.08520167709739 140.10156881110066 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_07dedfe8-aaff-41ce-bcf1-7a191ed59b62">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08531774219266 140.10266429135578 0 36.08532063553569 140.10262010736906 0 36.08527814035683 140.1025789265817 0 36.08526091071017 140.1026059126731 0 36.08523066328225 140.10266987943626 0 36.08521678979657 140.1027070193063 0 36.08521343499076 140.10271590214035 0 36.08520823035594 140.10273496211158 0 36.08525985154377 140.10275635682285 0 36.08529590955284 140.1027538260068 0 36.08531774219266 140.10266429135578 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_41621c84-b135-457a-9c9f-e461e4a3b3fc">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08528476736233 140.1081767487861 0 36.08532261892088 140.108011139164 0 36.08514531488965 140.10794963160572 0 36.08511064229106 140.10793751874888 0 36.08509974497389 140.1079338054429 0 36.0850122074631 140.107903420444 0 36.084945454149015 140.1078887532146 0 36.0849319438473 140.10788389934757 0 36.08489217349083 140.1080519066468 0 36.08498090783164 140.1080713141363 0 36.084988022507225 140.1080737817571 0 36.08528476736233 140.1081767487861 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8dbe21f5-b5c5-49ae-b93d-34356f9bfc10">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085200406467735 140.10359307107205 0 36.08533379611272 140.1035004762324 0 36.08531612758686 140.1034617739173 0 36.08518270552434 140.10355436869096 0 36.085200406467735 140.10359307107205 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_b12201d2-cebb-4177-99a7-72b28c1aeb39">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08534130440187 140.1042070485807 0 36.085388336966744 140.10417412002832 0 36.0853317489008 140.10404821771033 0 36.085293630921356 140.103965151398 0 36.0852465040671 140.1039979509066 0 36.0852706349006 140.104050573199 0 36.085284536837136 140.10408082368357 0 36.08534130440187 140.1042070485807 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_123db83b-5c38-40a2-a78c-ec86abcad9c0">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08532000736108 140.10148815399717 0 36.085411989114576 140.10142545674483 0 36.08533460509515 140.1012531432883 0 36.0852998414301 140.1012411440712 0 36.08525263064419 140.10127351853305 0 36.08524280100623 140.1013162354748 0 36.08532000736108 140.10148815399717 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_156b0129-a48b-498b-ad91-be544364c086">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08536561520575 140.1086477282771 0 36.085435486320904 140.10832506762927 0 36.085328979900744 140.10827350733732 0 36.085275799585304 140.10851050308852 0 36.08519540549381 140.10886244066972 0 36.085175013887074 140.1089504239296 0 36.08516017506841 140.10901455301956 0 36.08515407504928 140.10904118122278 0 36.08510782900231 140.10924089088238 0 36.08506640044616 140.10942285083897 0 36.085041906540894 140.10953079545385 0 36.085030528162626 140.1095794020193 0 36.08501706118316 140.1096347635614 0 36.08500515163006 140.1096787934152 0 36.08499689485131 140.1097029822035 0 36.084983562250954 140.10973913436456 0 36.084973765970375 140.10976608262968 0 36.084958633567524 140.10980100703537 0 36.084941691759745 140.10983847899513 0 36.08491978113334 140.10988137446014 0 36.08490457433104 140.1099098582837 0 36.08488530794895 140.10993954930663 0 36.084864504222296 140.1099714557126 0 36.084840451190885 140.11000534942121 0 36.0848167650794 140.11003646841726 0 36.08477880753538 140.11008051784347 0 36.08473986978439 140.11012047755622 0 36.08471855369742 140.1101400567339 0 36.084699948045724 140.110156425274 0 36.08468188907863 140.11017101910036 0 36.0846675319796 140.11018240577133 0 36.08457923932556 140.11024505519285 0 36.084313738981336 140.11042922513033 0 36.08413517462432 140.1105527391665 0 36.0839331380317 140.11069271484723 0 36.08378364181026 140.11079623202798 0 36.083756710038294 140.1108276720788 0 36.083715729794775 140.1108539550063 0 36.08339704178814 140.11108112728465 0 36.083458832005135 140.1112183178866 0 36.08377938535518 140.1109991938921 0 36.08380953780412 140.11097853587063 0 36.08383725328984 140.110958968539 0 36.0838646077119 140.1109396331036 0 36.08387210057008 140.11093445177374 0 36.08393908717591 140.11088715195496 0 36.08463818022733 140.11040304810908 0 36.0847008288003 140.1103607400215 0 36.084754276649704 140.1103216197239 0 36.084803946978134 140.1102790439124 0 36.08480448878486 140.11027860165635 0 36.08484893463313 140.11023467499976 0 36.08487486173389 140.1102088937897 0 36.084903515559404 140.11017323963154 0 36.08493950721963 140.11012106631793 0 36.08496519301312 140.11008306987037 0 36.0850362261999 140.10996117547518 0 36.085063568622466 140.10990830544492 0 36.08508884071598 140.10985387357476 0 36.08511042385625 140.1097966527654 0 36.08513100811179 140.10974264857512 0 36.08514744994508 140.10968707530438 0 36.08516282555158 140.1096248359037 0 36.085192508924216 140.109494357154 0 36.08525340592143 140.1092340709445 0 36.08529248416839 140.10905498928497 0 36.08529174149062 140.10898603098818 0 36.08536561520575 140.1086477282771 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_36ff740a-6a00-40ae-a7aa-ed74bb06c569">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08536679501168 140.1018513134509 0 36.085458058100066 140.10178844202994 0 36.08543169440365 140.10173028847066 0 36.085340336730965 140.10179314962298 0 36.08531271363464 140.10181237663616 0 36.08512025441765 140.10194641322573 0 36.08507311888642 140.10198533877846 0 36.08503788450089 140.1020225281515 0 36.08499884870391 140.10206681107496 0 36.08498736482989 140.10208342789676 0 36.084938988823815 140.10215299592772 0 36.08491102761901 140.10220198008966 0 36.08489354207216 140.10224222779394 0 36.08486228038279 140.1023168507081 0 36.084913696464334 140.1023393103292 0 36.08494133279802 140.10227325947682 0 36.08495718679486 140.10223711465412 0 36.08498179934499 140.10219389312115 0 36.08499093190523 140.10218081046702 0 36.08503786183594 140.1021130252203 0 36.085073735128 140.10207228479845 0 36.085105896485736 140.10203852719908 0 36.08514463542588 140.10200644644317 0 36.085148698724055 140.10200314018508 0 36.08536679501168 140.1018513134509 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_354b2262-f5ae-493c-abf4-6cad8bea51de">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08553994252848 140.10176805585837 0 36.085549772302585 140.10172534991727 0 36.085523404092015 140.10166718630228 0 36.08543169440365 140.10173028847066 0 36.085458058100066 140.10178844202994 0 36.08549273132942 140.1018005522449 0 36.08553994252848 140.10176805585837 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_81c6b7b0-a2e5-400a-9733-369f95b535b7">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08548691770338 140.10048981540993 0 36.085562283860675 140.10044143480837 0 36.08553429135753 140.10038426547874 0 36.0854155093657 140.10046048132827 0 36.085498469229044 140.1006448664665 0 36.085526001266025 140.1006259718116 0 36.08547909546803 140.1005216574431 0 36.08548691770338 140.10048981540993 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_36d00572-246f-4dee-824a-b6209e8d4115">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08551276707397 140.10826892840768 0 36.08555321503778 140.1080909086575 0 36.08544278037621 140.10805282296477 0 36.08532261892088 140.108011139164 0 36.08528476736233 140.1081767487861 0 36.085328979900744 140.10827350733732 0 36.085435486320904 140.10832506762927 0 36.08551276707397 140.10826892840768 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_fb6c5704-583f-46a0-a010-f9bd9f09ce25">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">2</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085463969083825 140.10476622284392 0 36.08556626844998 140.1046945749495 0 36.08534130440187 140.1042070485807 0 36.085085649284764 140.10438604444658 0 36.08506624014683 140.10439975781867 0 36.08487449784128 140.10453434533662 0 36.08482755530533 140.1045672737711 0 36.08447747136849 140.10481313222505 0 36.08408984455847 140.10507973447955 0 36.08405834095434 140.10510127848355 0 36.08404227261526 140.10511232698474 0 36.08384502841138 140.10524788033104 0 36.08372731493269 140.10532887664135 0 36.083671978479515 140.10536688293794 0 36.08387507329888 140.10586936241154 0 36.084059768650654 140.1057424171436 0 36.08423408456021 140.1056225419449 0 36.08426170750915 140.1056036496412 0 36.08428120638596 140.1055901701939 0 36.08431271097752 140.10556862632842 0 36.084344757606765 140.10554652912214 0 36.084387726870254 140.1055170299685 0 36.08441038508176 140.10550145163566 0 36.084423926030475 140.10549206003023 0 36.084701060001024 140.10530158406368 0 36.08531086445671 140.10487340515414 0 36.085463969083825 140.10476622284392 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2e14154e-cc7a-47c3-bc1f-73889e894e08">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">2</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08556626844998 140.1046945749495 0 36.0856132975611 140.1046616376999 0 36.085388336966744 140.10417412002832 0 36.08534130440187 140.1042070485807 0 36.08556626844998 140.1046945749495 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_903bddf5-57ed-4c7b-970f-6e40bcafee6c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08561406775455 140.10046037579 0 36.08566112186245 140.10042743052762 0 36.085613110370794 140.1003233488745 0 36.08559660334195 140.1002875525381 0 36.08554957547381 140.10032046907554 0 36.08553429135753 140.10038426547874 0 36.085562283860675 140.10044143480837 0 36.08561406775455 140.10046037579 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_350b94ec-dccf-422b-897c-b7fcbeb83448">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08564017828647 140.10368807223048 0 36.08567795758606 140.10366166738734 0 36.08565127437283 140.10360371959462 0 36.08553256430503 140.10368670440087 0 36.08543010520658 140.10375830749916 0 36.08527664320735 140.1038644911386 0 36.08530328126701 140.103922423043 0 36.08545665291078 140.10381636131532 0 36.08550756697664 140.10378078081476 0 36.08564017828647 140.10368807223048 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_41314414-717a-4dab-b26e-5c4b65aad4fd">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085475175656725 140.10247483882455 0 36.08570075407353 140.1023227051415 0 36.085674025136115 140.10226476206842 0 36.085658859203484 140.10227503716305 0 36.08552327832672 140.10236651658616 0 36.0854445652132 140.10241954759888 0 36.08539003611961 140.10245966926647 0 36.08536330309302 140.102483451728 0 36.08532790126393 140.1025148663711 0 36.085293381923385 140.10255505613924 0 36.08527814035683 140.1025789265817 0 36.08532063553569 140.10262010736906 0 36.08533338881443 140.10260015258493 0 36.08536248677174 140.10256639574928 0 36.08541983440138 140.1025153907565 0 36.08547147458689 140.10247738013024 0 36.085475175656725 140.10247483882455 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_cab8d596-3c38-4a42-8601-4f5c26fe0bcd">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0856613752155 140.1035610036476 0 36.08570804858424 140.10352740710618 0 36.08566662348754 140.10343188171348 0 36.08562303646749 140.10333578272287 0 36.08559729017259 140.10328196231737 0 36.085577015806905 140.1032396977274 0 36.08555262219499 140.103185537783 0 36.08552410382132 140.10312192536747 0 36.085506973705016 140.10308422424342 0 36.0854693057328 140.1030011486424 0 36.0854238360968 140.10290039112672 0 36.08541997923415 140.1028920499611 0 36.08538213061754 140.1028090849666 0 36.08531774219266 140.10266429135578 0 36.08529590955284 140.1027538260068 0 36.08533482953215 140.10284134717497 0 36.085376623466765 140.102932986719 0 36.085409269589476 140.10300516312162 0 36.08541061479482 140.10300816579496 0 36.08542209360065 140.1030335110107 0 36.085476802403385 140.10315429848458 0 36.08553007312744 140.10327284930267 0 36.08557618389207 140.10336893456824 0 36.08561923285859 140.10346381023632 0 36.08563178574429 140.10349283469316 0 36.0856613752155 140.1035610036476 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0caf49bc-967f-469e-8783-acf36db6184f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08568403539709 140.10222203452383 0 36.08573117473363 140.10218948032596 0 36.085688553785516 140.1020958081283 0 36.08561563763686 140.10193555092619 0 36.08560299200321 140.1019075257684 0 36.08553994252848 140.10176805585837 0 36.08549273132942 140.1018005522449 0 36.08556842644305 140.1019680250353 0 36.08559192385625 140.10201974978557 0 36.08568403539709 140.10222203452383 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_32e367c3-46d3-47eb-8e72-8db92eacd90d">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08567795758606 140.10366166738734 0 36.08576935908795 140.10359778671145 0 36.08574263100221 140.1035397398934 0 36.08570804858424 140.10352740710618 0 36.0856613752155 140.1035610036476 0 36.08565127437283 140.10360371959462 0 36.08567795758606 140.10366166738734 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4c003272-0d57-4c3d-a93e-de624992eff4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08573542699711 140.10233492666822 0 36.08578254650519 140.10230238577415 0 36.08573117473363 140.10218948032596 0 36.08568403539709 140.10222203452383 0 36.085674025136115 140.10226476206842 0 36.08570075407353 140.1023227051415 0 36.08573542699711 140.10233492666822 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_3094bd5f-4e88-471e-b57f-b5dd2e17652b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08573571598803 140.10834098488652 0 36.0857810464157 140.10812917428424 0 36.08558525691728 140.10806164669214 0 36.08557678903403 140.1080990377711 0 36.08555321503778 140.1080909086575 0 36.08551276707397 140.10826892840768 0 36.08552555637229 140.10827297038395 0 36.085713339719106 140.10833369709997 0 36.08573090208581 140.10833942132726 0 36.08573571598803 140.10834098488652 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1bff6c2a-777a-4c79-a97a-7f424e81922e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08581032276669 140.10088676914933 0 36.085857354424775 140.1008538384195 0 36.08572900030133 140.10057458105015 0 36.08566112186245 140.10042743052762 0 36.08561406775455 140.10046037579 0 36.085681968718475 140.10060751189377 0 36.0856970366189 140.10064031980866 0 36.08581032276669 140.10088676914933 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d8d2e060-b605-457a-be2a-d050daceeb48">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085847281440806 140.1011266310063 0 36.08587806419021 140.10110508248616 0 36.08580067352623 140.1009294870523 0 36.08565190574417 140.10103347180024 0 36.08544807422057 140.1011745788475 0 36.08542514633573 140.1011904908316 0 36.08533460509515 140.1012531432883 0 36.085411989114576 140.10142545674483 0 36.08570221552657 140.10122762991284 0 36.08571683894356 140.10121768589173 0 36.085847281440806 140.1011266310063 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_14a4325d-947e-4d16-86ed-aa6d6efe700a">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08593466449365 140.10102521408587 0 36.085944314016245 140.10098238508252 0 36.085892118373614 140.1008657265128 0 36.085857354424775 140.1008538384195 0 36.08581032276669 140.10088676914933 0 36.08580067352623 140.1009294870523 0 36.08587806419021 140.10110508248616 0 36.08593466449365 140.10102521408587 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8c490d43-86d9-478f-8ad3-992e16d4b43f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08594299570311 140.1084086832494 0 36.08598043874772 140.1082370225489 0 36.085935853955185 140.10814536979336 0 36.0857810464157 140.10812917428424 0 36.08573571598803 140.10834098488652 0 36.08576476601663 140.1083504212363 0 36.08579178518375 140.10835916539946 0 36.08582564934167 140.10837006538995 0 36.08585167781545 140.10837848409804 0 36.08586770898793 140.10838375884924 0 36.08594299570311 140.1084086832494 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0b02caf8-3591-411d-af86-9ded084eb494">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085935853955185 140.10814536979336 0 36.08598316186936 140.1079141255138 0 36.08583464107899 140.1078679602571 0 36.0857810464157 140.10812917428424 0 36.085935853955185 140.10814536979336 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_90e142f2-dc6e-4f46-9c34-01ca396778d4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08587754581347 140.10149815343968 0 36.08601476141636 140.10140124777107 0 36.085986948734906 140.10134407838427 0 36.0859562555768 140.10136584922617 0 36.0858509063344 140.1014403329628 0 36.08581064572781 140.10146816736585 0 36.0855524709865 140.10164718681244 0 36.085523404092015 140.10166718630228 0 36.085549772302585 140.10172534991727 0 36.08557892936153 140.1017053396447 0 36.08587754581347 140.10149815343968 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_dc91a18e-e719-4eaf-aaed-12282e7ceb4d">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085986948734906 140.10134407838427 0 36.086057698243096 140.10129510965717 0 36.0860322475068 140.10123929854723 0 36.085994668048286 140.10115677877238 0 36.08593466449365 140.10102521408587 0 36.08587806419021 140.10110508248616 0 36.085986948734906 140.10134407838427 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2ed89625-d5ed-4693-bbe5-2cea2c0d6f6a">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.085944314016245 140.10098238508252 0 36.08606518741856 140.10089807061624 0 36.08600838812127 140.10078450547198 0 36.085892118373614 140.1008657265128 0 36.085944314016245 140.10098238508252 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_771eb938-836c-408f-a964-251220c8f45a">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08604952530884 140.10141313612917 0 36.08609665201836 140.10138048687074 0 36.086074492880066 140.10133193895916 0 36.086057698243096 140.10129510965717 0 36.085986948734906 140.10134407838427 0 36.08601476141636 140.10140124777107 0 36.08604952530884 140.10141313612917 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_6aeef6e4-d27b-4cb2-b5f2-5c734eeb9dc1">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.086086740883545 140.10845614322966 0 36.08609340605259 140.10826173382162 0 36.0860295224392 140.10825340540094 0 36.08598043874772 140.1082370225489 0 36.08594299570311 140.1084086832494 0 36.08599109349192 140.1084246074981 0 36.086086740883545 140.10845614322966 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a9624146-8f9c-4763-9f21-3c0a0d015145">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08609912045187 140.1033699457176 0 36.08615947424557 140.10332775666896 0 36.08613278367865 140.10326980194654 0 36.08607257160932 140.10331190267726 0 36.086024819143894 140.10334482956483 0 36.08590918209145 140.1034244944418 0 36.08580094741761 140.10349896551094 0 36.08574263100221 140.1035397398934 0 36.08576935908795 140.10359778671145 0 36.08578262922828 140.1035885125054 0 36.08582749524502 140.1035570195142 0 36.08592914019493 140.1034870786221 0 36.086029972931264 140.10341757893573 0 36.08609912045187 140.1033699457176 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f850323e-0e9b-4f2a-ad5e-83082b9ce27e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08614243490369 140.10322697311847 0 36.08618946694686 140.10319415468973 0 36.08606972882981 140.10293357664128 0 36.086045870618484 140.10288175013784 0 36.085969090267696 140.10271648146812 0 36.0859389518013 140.1026517529517 0 36.085917877021394 140.1026045998198 0 36.08582434068481 140.10239530252244 0 36.08579537373454 140.10233057823825 0 36.08578254650519 140.10230238577415 0 36.08573542699711 140.10233492666822 0 36.08574807253003 140.10236295194096 0 36.08589183029777 140.1026843381103 0 36.085931745472074 140.1027702087965 0 36.085998838409424 140.10291467973403 0 36.08600870476056 140.10293603325485 0 36.08603785441473 140.10299952600616 0 36.08614243490369 140.10322697311847 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_efb51895-01cb-4ba7-b0be-647f9c782930">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08615947424557 140.10332775666896 0 36.08625083406057 140.10326389506494 0 36.08622414077698 140.10320593251458 0 36.08618946694686 140.10319415468973 0 36.08614243490369 140.10322697311847 0 36.08613278367865 140.10326980194654 0 36.08615947424557 140.10332775666896 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_19863a40-91e2-416c-95dc-0aa809989ae6">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08632380408999 140.10852625899568 0 36.08634712389975 140.10841896336942 0 36.08629528104506 140.1082275120727 0 36.08626353138803 140.10824056213116 0 36.08622736656726 140.1082503188729 0 36.086180656784116 140.1082599278476 0 36.08613774494428 140.1082641090282 0 36.08609340605259 140.10826173382162 0 36.086086740883545 140.10845614322966 0 36.0861125717549 140.1084721120896 0 36.08612903751167 140.1084843839724 0 36.086163402688236 140.10851259710316 0 36.08619110690381 140.10853701164606 0 36.08620746135818 140.10855849957548 0 36.086229392413124 140.10859284441554 0 36.08632380408999 140.10852625899568 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e5775cec-e061-4617-a20f-c80dea2264e8">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08634428444994 140.1007939713473 0 36.08636858778696 140.10068712088386 0 36.08631837794369 140.10056902506466 0 36.08623137523638 140.10054008198782 0 36.086032781579426 140.10067765576463 0 36.08600838812127 140.10078450547198 0 36.08606518741856 140.10089807061624 0 36.086152006895404 140.1009279017323 0 36.08634428444994 140.1007939713473 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f5409235-bcc7-4a60-9d8d-1494a3b38b10">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08636858778696 140.10068712088386 0 36.086488942572466 140.10060368209864 0 36.086435909827 140.1004875742354 0 36.08631837794369 140.10056902506466 0 36.08636858778696 140.10068712088386 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5d9ffee9-302f-40e2-bea5-226596db0eb4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.086566730954026 140.10054987815138 0 36.086580568488195 140.1005402865703 0 36.086527533926926 140.10042415751028 0 36.086492770185025 140.100412158473 0 36.08644564901596 140.1004448673432 0 36.086435909827 140.1004875742354 0 36.086488942572466 140.10060368209864 0 36.08650056307606 140.1005956253984 0 36.086566730954026 140.10054987815138 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_3d9c69bb-11d1-46af-aac0-dcb19b80487c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.086628471870895 140.1030037751218 0 36.08671842859456 140.10294092733926 0 36.086691745092125 140.10288295570336 0 36.086602014160654 140.10294560997914 0 36.086467331194044 140.10303798073016 0 36.08637218637546 140.10310328114738 0 36.08634140427195 140.10312438485812 0 36.086315677449356 140.1031419525078 0 36.08622414077698 140.10320593251458 0 36.08625083406057 140.10326389506494 0 36.086274430496324 140.10324740061733 0 36.08634213531622 140.10320000646365 0 36.086424551845965 140.10314354600536 0 36.086609154297996 140.1030169231 0 36.086628471870895 140.1030037751218 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_30615795-7397-492a-9344-736b9773515f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08670130514812 140.1028401262136 0 36.086748426963325 140.10280741867223 0 36.08664383918248 140.1025828685258 0 36.08661765165775 140.1025247044334 0 36.08651021057864 140.10228615408246 0 36.08634347156703 140.10192314737984 0 36.08626704991977 140.10175810231527 0 36.0862033781402 140.10161430973497 0 36.08609665201836 140.10138048687074 0 36.08604952530884 140.10141313612917 0 36.08611652341076 140.10156003760954 0 36.08615607737848 140.1016465729936 0 36.08621983900146 140.10179047686 0 36.08622737336428 140.10180682552118 0 36.08629643962076 140.10195607767173 0 36.08632953632311 140.10202814500758 0 36.08646299902206 140.10231875049047 0 36.086502190943776 140.10240571846185 0 36.086596807582374 140.102615565362 0 36.08670130514812 140.1028401262136 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a017b13e-267c-4b3f-a411-8de59df00762">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.086798816385645 140.10288476478283 0 36.08680980521104 140.10287711486532 0 36.08678319120835 140.10281908567902 0 36.086748426963325 140.10280741867223 0 36.08670130514812 140.1028401262136 0 36.086691745092125 140.10288295570336 0 36.08671842859456 140.10294092733926 0 36.086798816385645 140.10288476478283 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ab3c523f-87d3-4f84-80de-875b808dc4a5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.086765949455 140.10158522947816 0 36.08689702250748 140.10149485350033 0 36.08687059290392 140.1014366898685 0 36.08673922104448 140.10152729697555 0 36.086765949455 140.10158522947816 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a03e42f7-c426-42e3-ab9d-590896c67534">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.086872255461635 140.10281239402025 0 36.086917926698504 140.10278345687271 0 36.08690976929921 140.10276366354492 0 36.086864007936946 140.1027926003934 0 36.08685378805736 140.1027681251719 0 36.08681424489548 140.10279742744933 0 36.08678319120835 140.10281908567902 0 36.08680980521104 140.10287711486532 0 36.086841515095635 140.10285504026297 0 36.086878530633776 140.10282750604955 0 36.086872255461635 140.10281239402025 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_58850dae-15cc-4f26-97f9-390bac73fc8e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08697899722282 140.10147436703085 0 36.08698873678811 140.10143164886517 0 36.086962289215364 140.10137344626094 0 36.08687059290392 140.1014366898685 0 36.08689702250748 140.10149485350033 0 36.08693169575778 140.10150696381132 0 36.08697899722282 140.10147436703085 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_aeb0ca7c-99c4-4ee9-be07-d8c7bf7e6e05">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.086755033534786 140.10041948604103 0 36.087083255574065 140.10019251540373 0 36.08697238389921 140.1001164099104 0 36.08659622958908 140.10037653094548 0 36.086527533926926 140.10042415751028 0 36.086580568488195 140.1005402865703 0 36.08664914772974 140.10049274851443 0 36.086755033534786 140.10041948604103 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1f9bac0a-ea76-4443-bca1-c81d9b29943b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">2</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08682770304114 140.1038249334994 0 36.08714670356601 140.10361250292524 0 36.08690297983002 140.10313740796158 0 36.08659453953268 140.10334278036257 0 36.08653387744221 140.10338499078387 0 36.08638384524153 140.1034891896117 0 36.08618425364836 140.10362775217692 0 36.086135777402376 140.1036614537282 0 36.08603855424048 140.10372896682102 0 36.08597816200099 140.10377095549345 0 36.08589258711177 140.10382895872885 0 36.085748788284874 140.1039266258502 0 36.085615280007524 140.1040173323601 0 36.08551490058201 140.10408549999102 0 36.085388336966744 140.10417412002832 0 36.0856132975611 140.1046616376999 0 36.085737228291066 140.10457484020705 0 36.08619976802262 140.10426073939527 0 36.086404055532824 140.10411886326978 0 36.08681452397448 140.1038337828087 0 36.08682770304114 140.1038249334994 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1585c1dd-afbb-4c60-9310-2561d8d38cd4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08706113238997 140.1122823794567 0 36.0871336125867 140.11212096895767 0 36.08698914142774 140.11207859593978 0 36.0868872730813 140.11204869912524 0 36.08668508457769 140.11198202660685 0 36.08654367803542 140.1119395538616 0 36.08644496471212 140.11190944644773 0 36.08643478724695 140.11190630138077 0 36.08635823015541 140.11188304573093 0 36.08628986921281 140.11186226194502 0 36.08620565621933 140.11183675855568 0 36.086067856877094 140.11179341072878 0 36.08600453107113 140.111777863771 0 36.085951556026934 140.11176846057685 0 36.08590019431873 140.11176293840245 0 36.08586387679216 140.1117607116694 0 36.08580790088332 140.11176273509395 0 36.08576967726236 140.11176625353752 0 36.08571639490849 140.11177273917343 0 36.0856540838387 140.11178595531442 0 36.085609443439594 140.11179700184593 0 36.08556334878015 140.11181328433148 0 36.085484224231855 140.1118476497619 0 36.08543874437346 140.11187047461596 0 36.08540038635375 140.1118934245048 0 36.085366085035005 140.11191563361302 0 36.08532906796246 140.11194292984763 0 36.08528383278557 140.11197719256123 0 36.08516538025469 140.11206360751433 0 36.085149489983515 140.11211407442647 0 36.08493199494942 140.11227308093152 0 36.084666825546456 140.112468238875 0 36.084600797402935 140.1125155023075 0 36.08465966570718 140.11264032382996 0 36.08472629317747 140.11259271281395 0 36.08479048546583 140.11254541512022 0 36.08480980756657 140.11253115940147 0 36.08499200369246 140.11239711289852 0 36.0852001997567 140.11224483608407 0 36.08525834298384 140.11220229117038 0 36.08526728110163 140.11219577139335 0 36.08535828772595 140.11212923569914 0 36.085403883698106 140.11209487437856 0 36.08543665772003 140.11207056129066 0 36.085465452587044 140.11205200821772 0 36.0854975831686 140.11203268962487 0 36.08553539296709 140.11201372418103 0 36.085607209283744 140.111982664311 0 36.08564365202308 140.11196980120778 0 36.08566078679762 140.11196553110304 0 36.08568062702189 140.1119606043015 0 36.08573563396349 140.11194891708618 0 36.08578215448298 140.111943307136 0 36.085815419950265 140.11194020434067 0 36.08586247221417 140.11193848267635 0 36.085890318779256 140.11194013550192 0 36.08593483217141 140.11194495627765 0 36.08598014898875 140.11195311111618 0 36.08603590823825 140.11196674381384 0 36.086079319672066 140.11198033295466 0 36.086127684382454 140.11199560520615 0 36.08617055529026 140.11200908143022 0 36.086323669681185 140.11205548186624 0 36.08650929660556 140.11211209094589 0 36.08664935304481 140.11215412615084 0 36.086726715822934 140.11217971687398 0 36.08677895169969 140.11219689075955 0 36.08685190145532 140.11222102243406 0 36.08706113238997 140.1122823794567 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d6d902b7-a9f3-4ccc-b156-e126c0bec0f7">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08716720230069 140.1001768090241 0 36.08717694167617 140.10013396844792 0 36.087155225780506 140.09996945233718 0 36.087091551232405 140.09994791717452 0 36.086996867141316 140.10000944795323 0 36.08697238389921 140.1001164099104 0 36.087083255574065 140.10019251540373 0 36.0871179287099 140.10020473621424 0 36.08716720230069 140.1001768090241 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e23ddfda-6406-492d-bbc2-c178b3095472">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08690976929921 140.10276366354492 0 36.08720956450381 140.10252844367773 0 36.087202173690414 140.10251403847684 0 36.08709342419039 140.1023359999838 0 36.08699795347717 140.10218564559614 0 36.086927360286 140.1020743631173 0 36.086909062860386 140.10203455887967 0 36.08680824792625 140.10181500790623 0 36.08671900264386 140.10162083654305 0 36.086519808048585 140.1011800910111 0 36.08648904476621 140.10111226244922 0 36.08634428444994 140.1007939713473 0 36.086152006895404 140.1009279017323 0 36.08632080405586 140.10129890741408 0 36.086526455285075 140.1017540983749 0 36.08673257036627 140.10220263077522 0 36.08679700073208 140.10240927710342 0 36.086899104729355 140.1027365328904 0 36.08690179290568 140.10274353771263 0 36.08690976929921 140.10276366354492 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a96089a3-2fb8-4634-a098-f522cf0bcfac">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.086917926698504 140.10278345687271 0 36.0872206504986 140.1025501503111 0 36.087215813849625 140.10254062413912 0 36.08720956450381 140.10252844367773 0 36.08690976929921 140.10276366354492 0 36.086917926698504 140.10278345687271 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8beb315f-9123-450a-8d72-b9314d2e583b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08718441597729 140.10035341695053 0 36.08723177470765 140.10032127151143 0 36.08716720230069 140.1001768090241 0 36.0871179287099 140.10020473621424 0 36.08718441597729 140.10035341695053 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1ecc7af9-3a59-4e5e-9209-e0c9c4152baf">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08720182093437 140.1003923362434 0 36.08724917335595 140.100360195241 0 36.08723177470765 140.10032127151143 0 36.08718441597729 140.10035341695053 0 36.08720182093437 140.1003923362434 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5e8033ee-142c-4c2c-abaa-cde049f6a40e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0869546023766 140.10286697468013 0 36.087282223683964 140.10266832747647 0 36.08726077018362 140.10262927835137 0 36.08723896538097 140.10258611947557 0 36.08722541536205 140.10255953410507 0 36.0872206504986 140.1025501503111 0 36.086917926698504 140.10278345687271 0 36.08692743205084 140.10280503147862 0 36.08694150956211 140.1028375038232 0 36.0869546023766 140.10286697468013 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ebd17264-5b40-4da8-8edc-94295d4c73d8">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0872883648219 140.1025425371526 0 36.08735252347553 140.1024996169368 0 36.08732671804105 140.10244106389086 0 36.087262628680925 140.10248404100832 0 36.087271864968116 140.10250504843836 0 36.08728038367793 140.10252450993465 0 36.0872883648219 140.1025425371526 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_cedcf770-a999-41f9-9abf-3ec306f992b4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08733672809155 140.10239845762203 0 36.08738689834795 140.1023724225081 0 36.087278560823044 140.10213197925643 0 36.087271923574 140.10211752112397 0 36.08712375271561 140.10179399443322 0 36.08711541141058 140.10177575512327 0 36.087054693233824 140.1016416316944 0 36.08699415417173 140.10150795324884 0 36.08697899722282 140.10147436703085 0 36.08693169575778 140.10150696381132 0 36.08700819966271 140.1016756747 0 36.08706829015023 140.1018083412872 0 36.08710829258713 140.10189576770566 0 36.08718237811364 140.10205747544177 0 36.0872314394428 140.10216457637802 0 36.08733672809155 140.10239845762203 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f740bb4f-30a8-42a3-acef-392098afab21">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08733573453337 140.11237108443305 0 36.087366072506214 140.11223171093283 0 36.0871723442275 140.1120927899027 0 36.0871336125867 140.11212096895767 0 36.08706113238997 140.1122823794567 0 36.08710913347908 140.11237639091593 0 36.08733573453337 140.11237108443305 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_7178a81d-f8e6-49e7-b66a-ec00d1bcde1d">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.087376555095574 140.1024835410366 0 36.087442889257645 140.1024475449979 0 36.087421117019055 140.10238664154477 0 36.08738689834795 140.1023724225081 0 36.08733672809155 140.10239845762203 0 36.08732671804105 140.10244106389086 0 36.08735252347553 140.1024996169368 0 36.087376555095574 140.1024835410366 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_edb1fea8-38e9-4c9a-99f9-cff1802175a3">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08733652363337 140.1110224742608 0 36.08743516514678 140.1109669666711 0 36.08732766495404 140.11072229485333 0 36.087192082840005 140.1104212266072 0 36.08705334670083 140.1101203704219 0 36.087031105748814 140.11007210012832 0 36.087022675577224 140.11005385965768 0 36.086884933389264 140.10975112052805 0 36.08685112340841 140.10967770335625 0 36.08683775967615 140.10964879677965 0 36.08674879588398 140.10945571682632 0 36.086633375446205 140.10920480318268 0 36.08663122331641 140.1092000208716 0 36.086607278463696 140.10914785868573 0 36.08659741399104 140.1091261711114 0 36.08656512839988 140.10905599110714 0 36.08655624997637 140.10903663887171 0 36.08653158719665 140.10898269761398 0 36.08641609212029 140.10872500046358 0 36.08632380408999 140.10852625899568 0 36.086229392413124 140.10859284441554 0 36.08623197797625 140.10859689421505 0 36.086321845532396 140.10879029710983 0 36.08642138022324 140.10901229403413 0 36.08643716187868 140.10904766029847 0 36.086471150747634 140.10912184362635 0 36.086528725160655 140.1092476326876 0 36.086539218220715 140.1092703218287 0 36.08665472814224 140.10952145766396 0 36.08669051163635 140.1095989789316 0 36.08671920962868 140.10966137378747 0 36.08679077587099 140.10981663872735 0 36.086928517922324 140.11011936650397 0 36.08706976484689 140.1104257942399 0 36.087097746117436 140.11048652154497 0 36.08710079512995 140.11049319478707 0 36.08723278921713 140.1107863661802 0 36.08733652363337 140.1110224742608 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e21ba0ed-4174-4ae4-9c19-b2ee1652a467">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08729402913657 140.10122103828866 0 36.087485856336755 140.10108710517198 0 36.08746228722881 140.10102695105783 0 36.0873290463598 140.1011199861261 0 36.087275876337415 140.10115712720625 0 36.08726748108663 140.10116298398515 0 36.087252856527606 140.10117303924855 0 36.087072316617416 140.1012975603735 0 36.086962289215364 140.10137344626094 0 36.08698873678811 140.10143164886517 0 36.08703251794295 140.1014013719514 0 36.087130821850884 140.1013336367301 0 36.08722434162542 140.10126910533884 0 36.08723923617795 140.10125882892802 0 36.08729402913657 140.10122103828866 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_154911fb-0e8f-49ea-8cef-5c327854c108">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08747202663517 140.1009842325748 0 36.08751896842754 140.10095118986888 0 36.087426679413824 140.10074711350143 0 36.08736658233057 140.10061687892767 0 36.087330881749416 140.10053958341328 0 36.087283432803595 140.10043670842074 0 36.08724917335595 140.100360195241 0 36.08720182093437 140.1003923362434 0 36.087219630348415 140.10043216191815 0 36.08723622175014 140.10046930625415 0 36.087276496075106 140.10055649991435 0 36.08730771078708 140.10062411948633 0 36.08737955840768 140.10077971151253 0 36.08747202663517 140.1009842325748 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d450aac3-e548-40bb-99d4-89ab6800eb02">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.087482812106884 140.11241657658897 0 36.08751146857923 140.11227669277164 0 36.087473792243216 140.1122646381016 0 36.087366072506214 140.11223171093283 0 36.08733573453337 140.11237108443305 0 36.087444985174834 140.11240456129983 0 36.087482812106884 140.11241657658897 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_dd3968f3-8e94-447c-b821-0744ed43c803">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08748887342333 140.1115152631069 0 36.08752602319665 140.11135304925688 0 36.087338438087286 140.11129731044826 0 36.08733029198361 140.11133892386982 0 36.08729518325941 140.1115149137972 0 36.08722848029929 140.11182804070512 0 36.0871942016517 140.11199559388987 0 36.0871723442275 140.1120927899027 0 36.087366072506214 140.11223171093283 0 36.08742676018888 140.11187015905426 0 36.08742803981657 140.11186250161705 0 36.087486230783774 140.11152769059618 0 36.08748887342333 140.1115152631069 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_73ccf6a9-f75c-4158-a2a7-66a2793ed0f1">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0874290762177 140.10023799033453 0 36.08756465833033 140.10014561672455 0 36.08754716923642 140.10010680362342 0 36.087337206694585 140.10024978342085 0 36.08723177470765 140.10032127151143 0 36.08724917335595 140.100360195241 0 36.08735460536564 140.10028870718696 0 36.0874290762177 140.10023799033453 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_602265d1-7ab8-47d2-8e06-a8be3f4bedb7">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08756783287036 140.10106617396428 0 36.08757739227861 140.10102334377584 0 36.08755373188466 140.10096328923996 0 36.08751896842754 140.10095118986888 0 36.08747202663517 140.1009842325748 0 36.08746228722881 140.10102695105783 0 36.087485856336755 140.10108710517198 0 36.087520621649034 140.1010987715365 0 36.08756783287036 140.10106617396428 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8d37286e-da24-40f6-bb07-031e3ebd1dc4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.087495000480864 140.11244637916104 0 36.08754877240071 140.1124618931798 0 36.087572245279055 140.11244519757105 0 36.087595371887126 140.11230353958726 0 36.08758380828601 140.11228023300077 0 36.08753178457502 140.11226484964286 0 36.08751146857923 140.11227669277164 0 36.087482812106884 140.11241657658897 0 36.087495000480864 140.11244637916104 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_865c21be-26b6-4fb2-a5ff-85b48bcca555">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08737445257936 140.1037969483422 0 36.0876784378515 140.10354947399415 0 36.087641950427155 140.1034626244543 0 36.0876301255895 140.10343049254547 0 36.08730512003802 140.10366767819505 0 36.08737445257936 140.1037969483422 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2f7c040e-99ad-4458-9325-d218f568e85c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08749360414758 140.1076251147289 0 36.08767187536471 140.1075015245463 0 36.08760337830396 140.10735155292664 0 36.087425087554635 140.1074750799799 0 36.08704900531218 140.1077361559115 0 36.08673205367059 140.10795515034314 0 36.086419804470765 140.1081679299717 0 36.086360060195865 140.10820181168685 0 36.08631630764482 140.10821887076307 0 36.08629528104506 140.1082275120727 0 36.08634712389975 140.10841896336942 0 36.086488771373325 140.10831774288854 0 36.086800028155764 140.10810550432086 0 36.08711752155924 140.10788620127224 0 36.08720102538864 140.10782819434888 0 36.08724814854933 140.1077954895883 0 36.087395927632244 140.1076928445942 0 36.08740540628473 140.1076863260093 0 36.08749360414758 140.1076251147289 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_06bffc7d-c69e-4a0c-a55e-d4077bcc35af">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0876194295719 140.1073086349712 0 36.087683713847106 140.1072599990587 0 36.08765770303048 140.1072043872581 0 36.08759844343645 140.10706571104177 0 36.087536208628585 140.10692735785233 0 36.08746624175475 140.10697796191778 0 36.087476823319115 140.1070015507827 0 36.08750830883005 140.10706739731629 0 36.08755109718269 140.10715670205997 0 36.08755970879123 140.10717462094976 0 36.0876194295719 140.1073086349712 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0052cbe5-bb51-457a-9c36-2af71de57977">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08765337347123 140.1110828856757 0 36.087678943055714 140.11105588110308 0 36.08758069493685 140.11085754363083 0 36.08743516514678 140.1109669666711 0 36.08733652363337 140.1110224742608 0 36.08734249724445 140.11105058906318 0 36.087346650748536 140.11108624836817 0 36.08735252887384 140.1111555485701 0 36.087352513683875 140.1112009760738 0 36.08734634519807 140.11125691986697 0 36.087338438087286 140.11129731044826 0 36.08752602319665 140.11135304925688 0 36.08757578530136 140.11123218763055 0 36.08759408682911 140.11119116613375 0 36.087611391764355 140.11115236197188 0 36.08762995812067 140.1111133401449 0 36.08763303296543 140.11110924236962 0 36.08765337347123 140.1110828856757 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c7c3f6cf-2332-409b-aa89-17af9b8603f3">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">2</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08768012077881 140.1032422233437 0 36.08769737578619 140.10322484855246 0 36.087480815765694 140.1027333986978 0 36.0874702577913 140.10273903696284 0 36.0874483447003 140.10274407016124 0 36.0874290538687 140.10274533684952 0 36.08740192871007 140.10274346760679 0 36.08737057542571 140.10273803057098 0 36.08734032367087 140.10272371388382 0 36.087317739307665 140.10270620313057 0 36.08729848906147 140.102689814188 0 36.087282223683964 140.10266832747647 0 36.0869546023766 140.10286697468013 0 36.08696157358148 140.1028932044246 0 36.08696772886385 140.1029215411887 0 36.08696756753932 140.10295318765182 0 36.086965692833594 140.10298482826317 0 36.086959296426166 140.10302400429413 0 36.08694949115603 140.10305561780842 0 36.086939615492724 140.10307845874732 0 36.08692188151082 140.1031088236929 0 36.08691084814534 140.10312544230504 0 36.08690297983002 140.10313740796158 0 36.08714670356601 140.10361250292524 0 36.08717258206583 140.10360803877316 0 36.0871966442909 140.10360923156708 0 36.087219975207546 140.10361497460113 0 36.08724176865333 140.10362304426866 0 36.08726832707105 140.10363601612676 0 36.08728407844389 140.10364595284338 0 36.08730512003802 140.10366767819505 0 36.0876301255895 140.10343049254547 0 36.08762958883839 140.1034287029156 0 36.087629502937766 140.10342682599364 0 36.0876308230921 140.10340094636652 0 36.08763260795255 140.10336842795124 0 36.08763918728642 140.10332891911105 0 36.08765170753678 140.10329242874153 0 36.0876652900785 140.1032644923135 0 36.08768012077881 140.1032422233437 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_fd5e70f9-ae4e-4945-94d6-cb1297b2a7bb">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0874040378388 140.10385212801137 0 36.08770327068966 140.1036086228557 0 36.0876784378515 140.10354947399415 0 36.08737445257936 140.1037969483422 0 36.0873791655137 140.10380573575205 0 36.0874040378388 140.10385212801137 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_b2b5226b-0f15-46c8-821e-29fc7b2fddfb">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08767187536471 140.1075015245463 0 36.087787006193565 140.1074217077798 0 36.0877184776442 140.1072716681641 0 36.087683713847106 140.1072599990587 0 36.0876194295719 140.1073086349712 0 36.08760337830396 140.10735155292664 0 36.08767187536471 140.1075015245463 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2ef7c726-16d0-4ab5-86a7-8f23d72cb1f0">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08782442378968 140.11252571823943 0 36.08785640380832 140.1123868672036 0 36.08774713542881 140.11235210646066 0 36.087601682638294 140.11230556288064 0 36.087595371887126 140.11230353958726 0 36.087572245279055 140.11244519757105 0 36.08769464142435 140.1124844953637 0 36.08771769760633 140.11249191683703 0 36.08782442378968 140.11252571823943 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a1784cc9-eb10-407e-a210-a3fe2ebf1d0a">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08786276584814 140.10350479914638 0 36.08788009910756 140.10349231055167 0 36.0878527218703 140.10343473969937 0 36.08783639877148 140.10344652219126 0 36.0877939748174 140.10347424892228 0 36.0876784378515 140.10354947399415 0 36.08770327068966 140.1036086228557 0 36.08786276584814 140.10350479914638 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_fc6358cf-348c-451d-8d3e-d7da508c6dca">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.087839230329166 140.1125536313756 0 36.08788993649205 140.1125697010152 0 36.08791376729117 140.1125541281718 0 36.08794389198899 140.11241470188975 0 36.087928709198586 140.11238578802127 0 36.08787505012882 140.11236825560766 0 36.08785640380832 140.1123868672036 0 36.08782442378968 140.11252571823943 0 36.087839230329166 140.1125536313756 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_374795a1-f721-40d6-b5f0-d24346bc3c38">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.087959564268125 140.1034657099271 0 36.08796687278301 140.10342220585804 0 36.08793464089024 140.1033687147579 0 36.08789887088206 140.10340143018894 0 36.0878527218703 140.10343473969937 0 36.08788009910756 140.10349231055167 0 36.08791486377803 140.1035037559448 0 36.087959564268125 140.1034657099271 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d53a20e3-0eb0-419a-8cb6-023450bc85b3">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08793224259606 140.1021885050229 0 36.08797249224643 140.1021668286858 0 36.08795081494339 140.1021057297867 0 36.08783061483026 140.10217039201282 0 36.08782538105848 140.10217315027566 0 36.08778874580459 140.102192347066 0 36.087755989499584 140.10220965820878 0 36.08758462825734 140.1022996857425 0 36.08750386601104 140.10234250636773 0 36.08742797538103 140.1023828894609 0 36.087421117019055 140.10238664154477 0 36.087442889257645 140.1024475449979 0 36.0874495598517 140.1024439255297 0 36.087545573467374 140.1023929398334 0 36.08760594275701 140.1023609541806 0 36.08785201979862 140.10223153876788 0 36.08788387433222 140.10221443552487 0 36.08793224259606 140.1021885050229 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4b659348-34a2-4a78-837a-159044822137">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08796235584149 140.10206368356708 0 36.088012437315776 140.10203741454265 0 36.08788821980823 140.10176361390958 0 36.08784768070505 140.10167430823424 0 36.08783826329223 140.10165362215756 0 36.087816737572574 140.10160602244764 0 36.08773000834737 140.10141551069748 0 36.08756783287036 140.10106617396428 0 36.087520621649034 140.1010987715365 0 36.08753748521748 140.10113502873347 0 36.087560448335594 140.1011844097109 0 36.087617855792615 140.10130797332206 0 36.08768100364794 140.1014439823351 0 36.08771965961191 140.10152907276878 0 36.08776952713345 140.10163860869673 0 36.08784881050704 140.10181333840725 0 36.08796235584149 140.10206368356708 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_da0c9206-5dfe-4cbe-930b-9ec0d51c2d72">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08785948889468 140.1008271981663 0 36.08802224184704 140.10071602857138 0 36.087995980805 140.10065769060003 0 36.08783303013423 140.1007690328367 0 36.08755373188466 140.10096328923996 0 36.08757739227861 140.10102334377584 0 36.08785948889468 140.1008271981663 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d6bb015d-e430-4840-93c5-dc16e9c8d2b7">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08793968254666 140.11086512609532 0 36.088037817882565 140.11079463632416 0 36.08797183277493 140.11064294103798 0 36.087946176605065 140.11059054943132 0 36.087894897206134 140.11062734681755 0 36.08784325650474 140.11066447601715 0 36.087775993743755 140.11071432026253 0 36.08775279023546 140.11073156148763 0 36.08766972744337 140.11079312066707 0 36.08760156589911 140.11084185110477 0 36.08758069493685 140.11085754363083 0 36.087678943055714 140.11105588110308 0 36.08769835175731 140.11104229106604 0 36.08776606195962 140.1109938922849 0 36.08793968254666 140.11086512609532 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_15904bcd-ba47-4ad8-8dfb-6951d9f13800">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08803394124141 140.10213443994527 0 36.08807290758147 140.10210827079734 0 36.0880470198501 140.102049758149 0 36.088012437315776 140.10203741454265 0 36.08796235584149 140.10206368356708 0 36.08795081494339 140.1021057297867 0 36.08797249224643 140.1021668286858 0 36.088001907705056 140.1021509874252 0 36.0880248274192 140.1021391726885 0 36.08803394124141 140.10213443994527 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9ce52ff7-b017-4e31-9213-5e84783191f5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08801130666235 140.11258512116018 0 36.08806605301841 140.1124531728279 0 36.088028405319704 140.1124416022387 0 36.08794389198899 140.11241470188975 0 36.08791376729117 140.1125541281718 0 36.08799950817209 140.11258141499107 0 36.08801130666235 140.11258512116018 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ef423a86-03f4-435d-b9cb-0652f8602c35">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08810430643416 140.1006959965922 0 36.08811404572312 140.10065327772074 0 36.08808779182439 140.1005949563829 0 36.087995980805 140.10065769060003 0 36.08802224184704 140.10071602857138 0 36.08805691629455 140.1007280387811 0 36.08810430643416 140.1006959965922 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_20c47ed3-fe02-4bdf-b281-e4b26b923bcb">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08809704872116 140.11078818185658 0 36.08810634123499 140.1107453405271 0 36.08803562840458 140.1105968932301 0 36.08801733460857 140.11061012090593 0 36.08797183277493 140.11064294103798 0 36.088037817882565 140.11079463632416 0 36.08805002879585 140.11082110651634 0 36.08809704872116 140.11078818185658 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_3556b3fd-f935-4713-b44d-ad246e335449">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088300171110824 140.1043503260352 0 36.08834718678796 140.10431730078986 0 36.08833848660618 140.10429850447892 0 36.088303691622414 140.10422076528366 0 36.08813052309988 140.10383384979764 0 36.088063521193995 140.1036894855253 0 36.087959564268125 140.1034657099271 0 36.08791486377803 140.1035037559448 0 36.08803819501615 140.10376912814687 0 36.08808340101558 140.10386655729184 0 36.088158641247965 140.1040346023295 0 36.088211281253436 140.10415226726292 0 36.08825334109469 140.10424623257757 0 36.08829127451858 140.1043311004278 0 36.088300171110824 140.1043503260352 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9224b277-1322-43fe-9b01-9b5405acd89f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088048371811524 140.10525913808516 0 36.088354280025364 140.1050457744301 0 36.08825302225446 140.10482522578337 0 36.08815983606236 140.10462225035982 0 36.088144768615884 140.10458932961959 0 36.08805050623603 140.10438379705323 0 36.088034720947526 140.10434943037245 0 36.08802243339585 140.10432273772122 0 36.0880098782098 140.10429482267998 0 36.087956159644044 140.1041754885999 0 36.08783625675502 140.1039090173579 0 36.087788097797315 140.1038021396973 0 36.08773527527721 140.10368491917683 0 36.08770327068966 140.1036086228557 0 36.0874040378388 140.10385212801137 0 36.08743265911947 140.10390551329792 0 36.087592111604906 140.10425962074493 0 36.087715780813944 140.10453422188547 0 36.087839998493905 140.10480515023576 0 36.08802888097171 140.10521666116776 0 36.08804663904551 140.105255365516 0 36.088048371811524 140.10525913808516 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_02bc9616-ae4c-46d3-b8b6-fe64e963930e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08829859557758 140.1031236328392 0 36.08838159256922 140.10304951687968 0 36.08834981929309 140.1029956185732 0 36.08834655629751 140.1029985389923 0 36.08826672038728 140.10306977862928 0 36.08808554996928 140.10323328229202 0 36.0880162801477 140.10329522992268 0 36.08795983478289 140.10334567277667 0 36.08793464089024 140.1033687147579 0 36.08796687278301 140.10342220585804 0 36.08799180025659 140.10339940504832 0 36.08811742446116 140.10328701429313 0 36.088179651754956 140.10323093893084 0 36.08827655884403 140.10314354545488 0 36.08829859557758 140.1031236328392 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c52d08fc-2c4d-46ef-8d84-41ebd6bf0bd5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0883928397213 140.10441617573838 0 36.088399762443956 140.10438399670028 0 36.088373305419005 140.10432582979064 0 36.08834718678796 140.10431730078986 0 36.088300171110824 140.1043503260352 0 36.08834590075109 140.10444914679547 0 36.0883928397213 140.10441617573838 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4ed8d464-39e3-4bd1-804f-63aa3bd68587">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08809179452796 140.10535386913753 0 36.08839858546749 140.10514241318018 0 36.088354280025364 140.1050457744301 0 36.088048371811524 140.10525913808516 0 36.08808287291921 140.1053342203057 0 36.08809179452796 140.10535386913753 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_059712df-108c-4e67-b2ff-16e3bff5d286">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088433430694785 140.11175367175522 0 36.088444716644034 140.11174561976387 0 36.088417630136 140.11168798893087 0 36.08834992095143 140.11173583229473 0 36.088307126867804 140.11176687365474 0 36.088274172374945 140.1117913092567 0 36.08825186229928 140.11181187363752 0 36.08823261300128 140.11183401452269 0 36.088215246338194 140.11186050385442 0 36.08819977513713 140.11188543414357 0 36.08818908949143 140.1119070388888 0 36.08818293235867 140.11191957622194 0 36.088175860425686 140.11193752926383 0 36.08816878766388 140.1119558376398 0 36.08816005104181 140.11199199598423 0 36.088112655181305 140.11221892425095 0 36.08811092191208 140.1122274684961 0 36.08810618049108 140.1122513261489 0 36.08806605301841 140.1124531728279 0 36.08811847117135 140.11246945948608 0 36.08816334125566 140.11224364397432 0 36.08821219458392 140.11200973606174 0 36.08821938521186 140.11197989063652 0 36.088230537552825 140.11195139171346 0 36.08824448365868 140.1119230136844 0 36.08825778258661 140.1119016291303 0 36.08827225477503 140.1118795713402 0 36.08828662387373 140.11186308758636 0 36.08830441793573 140.1118466048042 0 36.088334753999185 140.11182416987725 0 36.08837709579784 140.1117934489703 0 36.088433430694785 140.11175367175522 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8b58b92b-b6a0-43e5-b5d7-73bcedb94b43">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088461064124644 140.10302002851463 0 36.08847098452617 140.10297729925372 0 36.088444811306175 140.10291887610728 0 36.08843206985903 140.10292754068757 0 36.08837437257395 140.10297364907177 0 36.08834981929309 140.1029956185732 0 36.08838159256922 140.10304951687968 0 36.08841681217863 140.1030590759551 0 36.088461064124644 140.10302002851463 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d888b4e9-5783-4521-9374-af9988d3fa18">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08842719336127 140.11164514848892 0 36.08847432464316 140.11161249439488 0 36.08846176213228 140.111584973331 0 36.08839943971073 140.11144818049172 0 36.088305184759186 140.11124452688196 0 36.08820071947946 140.11101285478975 0 36.08818699854059 140.11098304678114 0 36.08809704872116 140.11078818185658 0 36.08805002879585 140.11082110651634 0 36.088129919750216 140.11099428355965 0 36.08814337209763 140.11102331329738 0 36.08815359545492 140.11104555801498 0 36.08825806005465 140.11127710786084 0 36.08827070564434 140.1113045913423 0 36.08835240454069 140.1114809948954 0 36.08842719336127 140.11164514848892 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_43a6edde-dfe6-479a-a3de-1540937e6b30">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08839104847338 140.10188913773575 0 36.08851794132847 140.10180002654707 0 36.08849115258105 140.10174212245948 0 36.08836440972494 140.10183108209105 0 36.08826773040886 140.10189770172258 0 36.08805604540117 140.10204379250874 0 36.0880470198501 140.102049758149 0 36.08807290758147 140.10210827079734 0 36.088082143671386 140.102102068245 0 36.08812673715449 140.10207134987803 0 36.08839104847338 140.10188913773575 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_b5d2b751-c9e4-4e4a-9786-73bc08e4c161">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088450313122834 140.11174162747187 0 36.08851338642615 140.11169863811088 0 36.08849781218169 140.1116639529293 0 36.08847432464316 140.11161249439488 0 36.08842719336127 140.11164514848892 0 36.088417630136 140.11168798893087 0 36.088444716644034 140.11174561976387 0 36.088450313122834 140.11174162747187 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4d49f126-464a-4772-899e-d7970741a52e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088399762443956 140.10438399670028 0 36.08853527314094 140.10428474409994 0 36.08850809402428 140.10422701882294 0 36.088373305419005 140.10432582979064 0 36.088399762443956 140.10438399670028 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_66ba8977-06fe-48c3-b3bf-fc3b37eace88">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08850071223116 140.10169929185426 0 36.08854765423753 140.1016662380438 0 36.088414279639046 140.10137585945498 0 36.08840665608527 140.10135906590102 0 36.088300646627005 140.101124181017 0 36.08824584755478 140.1010029572334 0 36.08823185645657 140.10097192854056 0 36.08810430643416 140.1006959965922 0 36.08805691629455 140.1007280387811 0 36.0881847353948 140.1010045267465 0 36.08821693169171 140.10107603714087 0 36.08836482626545 140.10140345258927 0 36.08836706746299 140.1014084460803 0 36.08850071223116 140.10169929185426 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8888f464-4af1-4963-b3a3-d4c59cdf2a51">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08854249142169 140.10487621114694 0 36.08858961922926 140.10484348409952 0 36.08851293615605 140.10467587659284 0 36.0883928397213 140.10441617573838 0 36.08834590075109 140.10444914679547 0 36.08836168209847 140.10448325044388 0 36.08846590437487 140.10470880613957 0 36.08854249142169 140.10487621114694 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5a526660-e3c9-47cf-9eae-86db262f6cf8">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088598999449175 140.10499964484526 0 36.08859983715487 140.10499907473843 0 36.08855469417093 140.10490288378062 0 36.088354280025364 140.1050457744301 0 36.08839858546749 140.10514241318018 0 36.088598999449175 140.10499964484526 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5c783685-e0c0-4abb-8e84-ba982066fa06">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08851794132847 140.10180002654707 0 36.08860920798481 140.10173593332487 0 36.0885824183107 140.10167802696776 0 36.08854765423753 140.1016662380438 0 36.08850071223116 140.10169929185426 0 36.08849115258105 140.10174212245948 0 36.08851794132847 140.10180002654707 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_559a5590-e5b8-4b93-a7b9-8a7967bbe6b8">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08853003371211 140.1104389144446 0 36.088604427431 140.110384419546 0 36.088532995598314 140.11023645770464 0 36.088458902540275 140.11029075386915 0 36.08822732940255 140.1104585061677 0 36.0880795387416 140.11056514458073 0 36.08803562840458 140.1105968932301 0 36.08810634123499 140.1107453405271 0 36.088298009207115 140.1106068869605 0 36.08835741378153 140.11056389943752 0 36.088517213683375 140.11044819717029 0 36.08853003371211 140.1104389144446 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0f3e9430-9dbc-4d16-8936-a424af67b731">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088586268655156 140.11204268189715 0 36.08864908395143 140.11199906328122 0 36.08863688824099 140.11197236956045 0 36.08857492720089 140.11183501106078 0 36.08851753890119 140.1117078849756 0 36.08851338642615 140.11169863811088 0 36.088450313122834 140.11174162747187 0 36.08848160777913 140.1118109072966 0 36.08857396616435 140.11201578793415 0 36.088586268655156 140.11204268189715 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9b918d93-0295-4847-a11c-04c78c8cecaf">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08859983715487 140.10499907473843 0 36.088667851918494 140.10495281664765 0 36.08862429122266 140.10485559614028 0 36.08858961922926 140.10484348409952 0 36.08854249142169 140.10487621114694 0 36.08855469417093 140.10490288378062 0 36.08859983715487 140.10499907473843 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9bc97f06-a28f-42d6-a5f4-2c83ce688d69">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08860616659824 140.1120861771765 0 36.08866899183984 140.11204255195676 0 36.08864908395143 140.11199906328122 0 36.088586268655156 140.11204268189715 0 36.08860616659824 140.1120861771765 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_992208df-1727-4b12-8222-715de3ce1f5b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08869064797174 140.10055640036558 0 36.08870481570555 140.10049208797696 0 36.088596325073446 140.1004558079707 0 36.088487158790684 140.10042301368077 0 36.08845859646576 140.10041891946136 0 36.08842057363577 140.10041378273894 0 36.08839686296621 140.10041660078176 0 36.08835484081889 140.1004261194629 0 36.08832787147237 140.1004348007071 0 36.08829845531776 140.1004502473533 0 36.08816584953128 140.10054162086416 0 36.08808779182439 140.1005949563829 0 36.08811404572312 140.10065327772074 0 36.08819212786735 140.1005999079013 0 36.08832238687425 140.10051007002923 0 36.08833177156742 140.10050509369975 0 36.08834539702805 140.1004980440906 0 36.08836650281435 140.10049123077508 0 36.08840428662515 140.10048269718354 0 36.08842024410449 140.10048075238382 0 36.08845259139553 140.1004850815195 0 36.08847754950524 140.10048871938932 0 36.088582841751105 140.10052039021087 0 36.08869064797174 140.10055640036558 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_00c6918a-b14f-4caa-9abc-0bcff193a334">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08862526652071 140.10684485026508 0 36.0886992084503 140.10679013935996 0 36.08863213544544 140.10663899758688 0 36.08855638704219 140.10669503486545 0 36.08834164556749 140.1068358725354 0 36.08803235479804 140.10705611107304 0 36.08772434523425 140.10726769096712 0 36.0877184776442 140.1072716681641 0 36.087787006193565 140.1074217077798 0 36.08779259143769 140.10741783621197 0 36.088101323342 140.1072058039497 0 36.08840953076706 140.10698646151155 0 36.08862526652071 140.10684485026508 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_947ff0ad-8b57-46f5-8bce-b40dc9d514cc">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08868532869291 140.11036106205947 0 36.088694711159086 140.11031823178055 0 36.0886232891118 140.11017029096186 0 36.088532995598314 140.11023645770464 0 36.088604427431 140.110384419546 0 36.08863919114297 140.11039599022467 0 36.08868532869291 140.11036106205947 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_6b0409d0-1f39-426a-811e-7151c12788b5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088778837025984 140.10057835257868 0 36.088788486391046 140.10053552197903 0 36.08876195961587 140.10047748242113 0 36.08872916890721 140.10050023111603 0 36.08870481570555 140.10049208797696 0 36.08869064797174 140.10055640036558 0 36.08871906865477 140.10058359144972 0 36.0887318433426 140.1006112725938 0 36.088778837025984 140.10057835257868 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f2cd3f12-51f3-4785-9a27-5a53b285f117">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08874056979496 140.11238504416445 0 36.088803222004124 140.11234106981746 0 36.08873965519801 140.11219715343495 0 36.088721719248724 140.11215800238486 0 36.08866899183984 140.11204255195676 0 36.08860616659824 140.1120861771765 0 36.08867664368689 140.11224023823567 0 36.08874056979496 140.11238504416445 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9c05ee2b-ba6a-4f9e-b035-b96d0f17d5d5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08876396558344 140.11197537208318 0 36.0888593907129 140.11190786050702 0 36.08883912199854 140.1118644815318 0 36.08874378702303 140.1119319934615 0 36.08864908395143 140.11199906328122 0 36.08866899183984 140.11204255195676 0 36.08876396558344 140.11197537208318 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c54f0a91-39eb-4566-887d-eab45cffcc6c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.088877232065336 140.11249079768731 0 36.08888679555167 140.1124479681999 0 36.088803222004124 140.11234106981746 0 36.08874056979496 140.11238504416445 0 36.088785590678576 140.1125202336286 0 36.088877232065336 140.11249079768731 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5094f925-1fac-488e-b4b4-0d3bdc655dd4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08880940504702 140.1027459384949 0 36.08891393889137 140.10267367185344 0 36.088887502471806 140.10261559293025 0 36.088783037357814 140.10268777213386 0 36.08859852568227 140.1028142885288 0 36.08855573811059 140.10284345820077 0 36.08845400518597 140.10291262461735 0 36.088444811306175 140.10291887610728 0 36.08847098452617 140.10297729925372 0 36.08862480293147 140.10287256553127 0 36.08880940504702 140.1027459384949 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c8c8d14d-cd1e-4ade-b49a-3aa0e3498b37">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08864616233455 140.1065746403317 0 36.08895147357956 140.1063600477384 0 36.08894385072746 140.10634325359308 0 36.08885192945854 140.1061400568855 0 36.08883793854271 140.10610913809285 0 36.08882475654805 140.1060799988218 0 36.08874817028703 140.10591061323854 0 36.08867660330006 140.10575346020258 0 36.08850234912417 140.10537053339124 0 36.08839858546749 140.10514241318018 0 36.08809179452796 140.10535386913753 0 36.088118476838595 140.10541262882177 0 36.08843200846506 140.1061012839254 0 36.088441334302445 140.1061218704301 0 36.08862723983575 140.10653293316284 0 36.08864616233455 140.1065746403317 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a4a57064-e537-42c6-b418-5a2ecd9ce208">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0889960039077 140.10265318640685 0 36.08900565376385 140.10261046699213 0 36.08897915991041 140.1025522634561 0 36.088887502471806 140.10261559293025 0 36.08891393889137 140.10267367185344 0 36.088948702411955 140.10268566115545 0 36.0889960039077 140.10265318640685 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_fe831368-791c-46f4-a3a3-1506de412bdd">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08899026999824 140.10147220325425 0 36.089075661227206 140.1014151939893 0 36.08904866098837 140.10135746990696 0 36.088964172577356 140.1014138160109 0 36.08892490516286 140.10144087757328 0 36.08879699255935 140.10152906799107 0 36.08866736470854 140.10161846268116 0 36.0885824183107 140.10167802696776 0 36.08860920798481 140.10173593332487 0 36.08864534686525 140.1017105548107 0 36.088693912526615 140.1016765181688 0 36.08878499522452 140.10161364330793 0 36.08888997994434 140.101541376719 0 36.08899026999824 140.10147220325425 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_602de9e8-cf5f-40bf-aba2-5e3e2811d188">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08906106121451 140.10660228292448 0 36.08907644028789 140.1065378189477 0 36.08900352774838 140.10637821704648 0 36.08895147357956 140.1063600477384 0 36.08864616233455 140.1065746403317 0 36.08863213544544 140.10663899758688 0 36.0886992084503 140.10679013935996 0 36.0887514448424 140.10680742116625 0 36.08906106121451 140.10660228292448 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d031a3a7-c87d-442b-9a2b-f3e40dc7fb07">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08905840051341 140.10131475060837 0 36.08911018681549 140.10129182899414 0 36.08898953543555 140.10103478338854 0 36.08896639371599 140.10098463474208 0 36.088778837025984 140.10057835257868 0 36.0887318433426 140.1006112725938 0 36.08894259247989 140.10106793769552 0 36.08895829143951 140.10110142657143 0 36.08905840051341 140.10131475060837 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ed4c4f3d-841b-448f-abf3-0a5cf165c651">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08907834209826 140.10451001956585 0 36.08911532862195 140.1044556345343 0 36.089052001474606 140.10432071937524 0 36.08904410704469 140.1043039244724 0 36.088881600013174 140.10394591336495 0 36.08873146533733 140.10361615141912 0 36.08872563618546 140.10360313924204 0 36.08856420454658 140.10324613409526 0 36.088551918151744 140.10321921935065 0 36.088461064124644 140.10302002851463 0 36.08841681217863 140.1030590759551 0 36.08851699364523 140.10327873077074 0 36.08855008702694 140.10335191112407 0 36.08868425368648 140.10364863689426 0 36.088791249099835 140.1038837517658 0 36.08883438802389 140.1039785097474 0 36.088963354219004 140.10426254957258 0 36.0889837125336 140.10430738166988 0 36.08899707536798 140.10433685446088 0 36.08907834209826 140.10451001956585 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_62ba6924-2cf1-480f-b3f9-f5feaf1d240c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08896233546099 140.10475259856946 0 36.08911796181524 140.10464618599156 0 36.089068691289455 140.10455286093196 0 36.08893581235792 140.10464368336662 0 36.0889186610019 140.10465539518455 0 36.088916134239554 140.10465716321917 0 36.08877441080969 140.10475350713307 0 36.08875473202888 140.10476687588394 0 36.08862429122266 140.10485559614028 0 36.088667851918494 140.10495281664765 0 36.088774123027896 140.10488054126887 0 36.08879777385813 140.1048643989817 0 36.08896233546099 140.10475259856946 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f5bc227d-0f8c-4c65-90be-755aa7ff675a">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089096744942275 140.11002315178013 0 36.08913231615131 140.10999729187375 0 36.08906157240253 140.1098489586872 0 36.08902561326616 140.10987499050893 0 36.08869959857322 140.11011437109764 0 36.0886232891118 140.11017029096186 0 36.088694711159086 140.11031823178055 0 36.08877109099218 140.11026231114582 0 36.0889307121511 140.11014504130682 0 36.089096744942275 140.11002315178013 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_bc88e24a-0353-4036-adb9-9c6675782dcc">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08915548147395 140.10139081298792 0 36.08916267352763 140.10135874533125 0 36.0891363054141 140.1013004681238 0 36.08911018681549 140.10129182899414 0 36.08905840051341 140.10131475060837 0 36.08904866098837 140.10135746990696 0 36.089075661227206 140.1014151939893 0 36.08911024362119 140.1014276263282 0 36.08915548147395 140.10139081298792 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_59ea53c5-e67a-486b-9c16-f2c29dcb0b6c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08921647218865 140.101564361819 0 36.08923001257471 140.10155508001446 0 36.08915548147395 140.10139081298792 0 36.08911024362119 140.1014276263282 0 36.08918271135033 140.10158756662273 0 36.0892007652007 140.1015751908932 0 36.08921647218865 140.101564361819 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_7f1ced93-847e-428b-bc19-059be602a7d5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08921899336844 140.10997061161348 0 36.08922837569566 140.10992778099813 0 36.089157621100696 140.10977942433504 0 36.08906157240253 140.1098489586872 0 36.08913231615131 140.10999729187375 0 36.08917572573743 140.11001176883218 0 36.08921899336844 140.10997061161348 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1f295b30-3f9b-45db-8288-b0c03434515c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08921823187199 140.10466586356642 0 36.08924107285184 140.10456988784165 0 36.08915009268818 140.10446730262234 0 36.08911532862195 140.1044556345343 0 36.08907834209826 140.10451001956585 0 36.089068691289455 140.10455286093196 0 36.08911796181524 140.10464618599156 0 36.08921823187199 140.10466586356642 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_53a7cf86-fd26-4731-87d0-08d5e0bef97d">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08923306019014 140.100230430431 0 36.089315658530055 140.10017263288947 0 36.08928894471423 140.10011472679722 0 36.08920425291036 140.10017403307273 0 36.08914892050123 140.10021137934106 0 36.088940491822896 140.10035358960667 0 36.08890862642156 140.10037569081084 0 36.08885924856073 140.10040994768784 0 36.08879750356432 140.10045282416976 0 36.08876195961587 140.10047748242113 0 36.088788486391046 140.10053552197903 0 36.08896686064717 140.1004117553117 0 36.08898699033893 140.1003980538285 0 36.08916581138986 140.10027595373137 0 36.08917781609128 140.10026777697186 0 36.08921744345171 140.1002412600783 0 36.08921988042599 140.10023971368256 0 36.08923306019014 140.100230430431 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a32bba42-2e9d-4a92-9833-2688a15ea503">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089388807912684 140.1001486721957 0 36.08939590977643 140.1001164818753 0 36.08936918068849 140.1000585412672 0 36.08928894471423 140.10011472679722 0 36.089315658530055 140.10017263288947 0 36.08934177696267 140.100181382717 0 36.089388807912684 140.1001486721957 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2003c9a3-7fbf-4aa4-b327-e725ff6afe37">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08924134244067 140.11218870932765 0 36.08937947588099 140.11208836790087 0 36.089344023785806 140.11194654870363 0 36.08921825944646 140.1120378282871 0 36.08918124313911 140.112064792651 0 36.089111902431256 140.11211651696848 0 36.08907416158125 140.1121445891516 0 36.08905691768729 140.11215740948185 0 36.08892393374013 140.11225254946507 0 36.08880999377074 140.11233631882592 0 36.088803222004124 140.11234106981746 0 36.08888679555167 140.1124479681999 0 36.08898340090717 140.11237690782445 0 36.089038833273456 140.11233734962428 0 36.08911665559954 140.1122816578915 0 36.0891341713147 140.11226861644883 0 36.08924134244067 140.11218870932765 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_11ecd41b-6a7f-42b7-a181-9b8fa701ec0e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089344023785806 140.11194654870363 0 36.089390520234474 140.11191251076767 0 36.08932954063723 140.1117790410518 0 36.08926490191927 140.11163067856435 0 36.08926104695969 140.111621781347 0 36.089251992334354 140.11160086180107 0 36.08918885452372 140.11146550829162 0 36.089187150584536 140.11146182669088 0 36.08917943661992 140.11144537592418 0 36.08912617006781 140.11132892399357 0 36.0890685000348 140.11120646028627 0 36.089049844396285 140.11116697362877 0 36.089025724534714 140.11111292086488 0 36.08896403470438 140.11097478599726 0 36.08891121088019 140.11086077924475 0 36.08886072058705 140.11075176675553 0 36.088826826455644 140.1106760372878 0 36.088775446866784 140.11056136973633 0 36.0887566161472 140.11051955082533 0 36.08868532869291 140.11036106205947 0 36.08863919114297 140.11039599022467 0 36.088719266594026 140.11057383154653 0 36.088728143735395 140.1105936286598 0 36.08881359686844 140.11078437046845 0 36.08882139884236 140.11080138773184 0 36.088905430271446 140.1109827972223 0 36.088916910977765 140.1110073674101 0 36.08897456523213 140.11113672657282 0 36.089002720077765 140.11119978815418 0 36.08905985242981 140.11132091732017 0 36.089079225192904 140.11136207222083 0 36.08913240302598 140.11147829059684 0 36.08913957718735 140.11149354016703 0 36.08920468864512 140.11163324244148 0 36.089219212334605 140.11166660722753 0 36.089282058342064 140.1118110767461 0 36.089344023785806 140.11194654870363 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0e53af97-f17f-4695-b6fc-37b32c2919c5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08937696947764 140.10362236895222 0 36.089424179838005 140.10358965108927 0 36.08938417889215 140.10350212130822 0 36.08929861601313 140.1033149390421 0 36.08912058682736 140.10292389537383 0 36.08909708815367 140.1028725122727 0 36.0889960039077 140.10265318640685 0 36.088948702411955 140.10268566115545 0 36.08903902302005 140.10288141972853 0 36.08905005556396 140.10290522104373 0 36.08921508164499 140.1032674591194 0 36.089251494326476 140.10334753628183 0 36.08937696947764 140.10362236895222 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_81cc7a2e-6321-4d22-9e87-676b20c138b6">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08916267352763 140.10135874533125 0 36.08943828101729 140.10116268605117 0 36.08935907377938 140.10099340605632 0 36.08935607607048 140.10096352464845 0 36.08932002231265 140.1008834496907 0 36.08925701461217 140.10092664380142 0 36.089293068564935 140.1010066187745 0 36.089311234902844 140.10102455868568 0 36.08935482996398 140.10111787369325 0 36.08934790811658 140.10115005341214 0 36.08923984969036 140.10122697441258 0 36.0891363054141 140.1013004681238 0 36.08916267352763 140.10135874533125 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_adbb7893-9047-4d51-bc21-4f8151754524">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08934110552544 140.10237630574758 0 36.08947516176118 140.1022818185327 0 36.08944716876943 140.1022248678005 0 36.08913473266226 140.1024447725225 0 36.08897915991041 140.1025522634561 0 36.08900565376385 140.10261046699213 0 36.08908834187681 140.10255333868614 0 36.089161370180435 140.10250284004184 0 36.08920966747398 140.10246880275272 0 36.08934110552544 140.10237630574758 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_bebdf268-8d00-4074-ac4c-d1e16aa89d91">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08946920144719 140.1036889689925 0 36.0894753135566 140.10365645353926 0 36.089450571411085 140.10359707044609 0 36.089424179838005 140.10358965108927 0 36.08937696947764 140.10362236895222 0 36.089397326776016 140.1036669579275 0 36.08942182866651 140.1037213575632 0 36.08946920144719 140.1036889689925 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_153ce9a3-af9a-4d9f-9cfd-3805302c4591">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089460556968845 140.11206533460478 0 36.089469850172115 140.11202239274957 0 36.089390520234474 140.11191251076767 0 36.089344023785806 140.11194654870363 0 36.08937947588099 140.11208836790087 0 36.089414329834256 140.11209981761374 0 36.089460556968845 140.11206533460478 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2893f3f0-2433-4a56-9699-bdf8281678ff">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08945636832143 140.10218192446436 0 36.089502872791314 140.10214790073636 0 36.089423734859025 140.10198370796408 0 36.0894105542138 140.10199332407856 0 36.089395116582175 140.1020045981926 0 36.089377241454635 140.10201764073545 0 36.08945636832143 140.10218192446436 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_70781345-d408-4408-9adf-4e86377f8367">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089510016918545 140.10229315289146 0 36.089556498911826 140.10225911802502 0 36.089529775979415 140.10220371732433 0 36.089502872791314 140.10214790073636 0 36.08945636832143 140.10218192446436 0 36.08944716876943 140.1022248678005 0 36.08947516176118 140.1022818185327 0 36.089510016918545 140.10229315289146 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e99fd5dc-3932-421d-9e63-0512666a7b40">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08941008151868 140.1001065467656 0 36.089591343349746 140.09998145536497 0 36.08956425204505 140.09992384213098 0 36.08938353294321 140.10004849127284 0 36.08936918068849 140.1000585412672 0 36.08939590977643 140.1001164818753 0 36.08941008151868 140.1001065467656 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1b06d481-3856-476f-b7e5-4cccbd339084">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089600185162 140.10411716617676 0 36.08964694919128 140.1040837907064 0 36.08951852590793 140.10379862900317 0 36.08951745013674 140.10379606016113 0 36.089507315789334 140.10377371635647 0 36.08946920144719 140.1036889689925 0 36.08942182866651 140.1037213575632 0 36.08956538946913 140.1040400920874 0 36.089591755992345 140.1040984815639 0 36.089600185162 140.10411716617676 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4c0788f1-d215-4c81-bc3c-d59565573a1b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08952209234173 140.10437430235825 0 36.08964859219447 140.10428521254423 0 36.08959044554038 140.104159885419 0 36.08936150952708 140.10432111547658 0 36.08915009268818 140.10446730262234 0 36.08924107285184 140.10456988784165 0 36.08941926970912 140.1044466842182 0 36.08947514962632 140.10440734385338 0 36.08952209234173 140.10437430235825 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_bb456789-5504-49f1-a29e-be81029dea4c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089598083708985 140.10965745816333 0 36.08967392130119 140.1096024227243 0 36.089603027278045 140.1094541660757 0 36.089526771718866 140.10950951773702 0 36.089398748548035 140.10960401383457 0 36.08933717466865 140.10964943805058 0 36.089157621100696 140.10977942433504 0 36.08922837569566 140.10992778099813 0 36.08940848713442 140.10979748929353 0 36.08944911530707 140.10976742715752 0 36.089598083708985 140.10965745816333 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2bf7a8ec-73fd-4e54-99ea-b16349afcd43">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08964886316094 140.10614103397324 0 36.08970079853899 140.10610528784116 0 36.08962830786968 140.1059454111346 0 36.08958949110554 140.1059720280383 0 36.089576401764 140.10598098865094 0 36.08953911945082 140.1060066225123 0 36.08941625942891 140.10609103700375 0 36.08927804513361 140.1061895009349 0 36.08927325975614 140.10619281575765 0 36.08922595779244 140.10622507750776 0 36.08914489574132 140.10628020886938 0 36.08903548347469 140.10635611855452 0 36.08900352774838 140.10637821704648 0 36.08907644028789 140.1065378189477 0 36.08910821527677 140.10651594196514 0 36.089217356746445 140.10644025358636 0 36.089306633605105 140.1063794874339 0 36.08931006501332 140.10637705630194 0 36.089353484706415 140.10634733523014 0 36.08937740786605 140.1063304279932 0 36.08948971388221 140.1062503082141 0 36.0896011997073 140.10617372773598 0 36.08964453009481 140.10614401723964 0 36.08964886316094 140.10614103397324 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_66ff9bc1-ec9f-4cfb-b775-286a966536c6">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0896540699993 140.10583323266385 0 36.089725381811405 140.1057854958217 0 36.089598214047115 140.10550489939118 0 36.08959103844795 140.10548955032775 0 36.08956404018155 140.10543182448674 0 36.08947147440067 140.10523394410245 0 36.089254186574024 140.10474916011535 0 36.08921823187199 140.10466586356642 0 36.08911796181524 140.10464618599156 0 36.08918278531299 140.1047967751554 0 36.089304029313595 140.1050671435041 0 36.08932582086753 140.1051157453772 0 36.0894008805681 140.10528312735707 0 36.08952761987851 140.10555418242137 0 36.08962411679476 140.10576705697517 0 36.08964052803393 140.10580342561875 0 36.0896540699993 140.10583323266385 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0643777f-2e05-4cc6-9039-0c0c2a75131d">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08964859219447 140.10428521254423 0 36.089739770302984 140.10422099772475 0 36.08968162268425 140.1040956693876 0 36.08964694919128 140.1040837907064 0 36.089600185162 140.10411716617676 0 36.08959044554038 140.104159885419 0 36.08964859219447 140.10428521254423 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ebfa0281-8e4d-48da-ba3e-247c7b3088c3">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08975500143242 140.10957959777377 0 36.08976447372494 140.10953677824048 0 36.0896935500225 140.10938845919046 0 36.089603027278045 140.1094541660757 0 36.08967392130119 140.1096024227243 0 36.08970868485288 140.1096140931596 0 36.08975500143242 140.10957959777377 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_11ce32ad-4007-4362-b023-63927c3a80a1">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08952334265526 140.10056643682697 0 36.08978611713256 140.10038565484717 0 36.0897584847491 140.1003282615948 0 36.089559892503104 140.10046495363252 0 36.08953386471775 140.10045598190177 0 36.089486947037024 140.1003565482938 0 36.089432944437306 140.1002419886442 0 36.089388807912684 140.1001486721957 0 36.08934177696267 140.100181382717 0 36.08952334265526 140.10056643682697 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_befaec85-a6da-47a9-bf0a-833bd8e73dcf">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08976576667394 140.10029618279518 0 36.08985975563019 140.10023035038492 0 36.089823770644756 140.1001523631488 0 36.08972020585012 140.09992763018067 0 36.0896261071156 140.0999934546925 0 36.08966709934066 140.10008220782743 0 36.08972019995978 140.10019754198228 0 36.08976576667394 140.10029618279518 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_58f25693-c013-436e-a38d-0bf13b09b48e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089870386280424 140.10599022136407 0 36.08989021857371 140.10597678330245 0 36.08981634064285 140.1058176910786 0 36.089725381811405 140.1057854958217 0 36.0896540699993 140.10583323266385 0 36.08962830786968 140.1059454111346 0 36.08970079853899 140.10610528784116 0 36.089743739188584 140.10607573360906 0 36.089826877558664 140.1060194983331 0 36.089870386280424 140.10599022136407 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_bb7f2313-b13f-419e-9caf-d50a5f1f0b80">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08981214541701 140.10039440449336 0 36.08990552914824 140.10032717870496 0 36.08986503165573 140.1002417837749 0 36.08985975563019 140.10023035038492 0 36.08976576667394 140.10029618279518 0 36.0897584847491 140.1003282615948 0 36.08978611713256 140.10038565484717 0 36.08981214541701 140.10039440449336 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2fd38026-5ec1-4cde-91bf-da54d187b0a4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08979354446342 140.1095156701403 0 36.089907744667734 140.10943545005895 0 36.08984040387546 140.10928463593072 0 36.089723495547794 140.10936672337164 0 36.0896935500225 140.10938845919046 0 36.08976447372494 140.10953677824048 0 36.08979354446342 140.1095156701403 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_47293d7f-3224-43a2-8bd4-7bfc81368922">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0898020045454 140.11177959696323 0 36.08991548830626 140.1116980450716 0 36.0898795847182 140.11155677872367 0 36.089742628059504 140.11165512703187 0 36.08964439950802 140.1117267384746 0 36.08952603735136 140.11181315885176 0 36.089390520234474 140.11191251076767 0 36.089469850172115 140.11202239274957 0 36.08958586600189 140.11193729701716 0 36.089692491219346 140.11185949693976 0 36.0898020045454 140.11177959696323 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_68ca0b59-6174-4171-a5f0-6183572f2703">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0898795847182 140.11155677872367 0 36.08992626002507 140.1115232961907 0 36.089873342382496 140.11141149716818 0 36.0898610589916 140.11138369235172 0 36.08981210533953 140.11127259595656 0 36.089778486082864 140.11119507847638 0 36.08972378249179 140.11107595566764 0 36.08967849354916 140.11097796486058 0 36.08962011678342 140.1108491684505 0 36.089569897777956 140.11073937861948 0 36.08952846664744 140.11064896396152 0 36.08947806704031 140.11053885171856 0 36.08946667922512 140.11051382646235 0 36.089368485477834 140.11029783133128 0 36.089340236975836 140.11023609088969 0 36.08921899336844 140.10997061161348 0 36.08917572573743 140.11001176883218 0 36.08932136193594 140.11033041331473 0 36.089415699275534 140.11053807750204 0 36.08941946546999 140.1105464191446 0 36.089522772651534 140.11077220471634 0 36.08957299306889 140.11088176131122 0 36.089596397393194 140.11093347990217 0 36.08963136928568 140.11101077975798 0 36.08966033664901 140.1110732894497 0 36.0896767480746 140.11110888188716 0 36.089731092916196 140.1112272260625 0 36.089757629746835 140.11128839480244 0 36.089764532922835 140.11130429869425 0 36.089783449782914 140.11134732903284 0 36.08982621823456 140.1114442119455 0 36.0898795847182 140.11155677872367 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_48491562-45a3-497e-9d76-48c46ad8aaea">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089762473919826 140.1034548887482 0 36.08996405643819 140.10331221761714 0 36.08994246610111 140.1032543885829 0 36.08973583594255 140.1033968315686 0 36.089450571411085 140.10359707044609 0 36.0894753135566 140.10365645353926 0 36.089762473919826 140.1034548887482 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_dd5d061e-bd1c-40a5-9e72-72cc4a72c26d">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089945424767315 140.10322131799347 0 36.08999279537716 140.1031886312977 0 36.0899914707055 140.10318571846432 0 36.0898334119285 140.10285215068504 0 36.0897285654362 140.10262247157365 0 36.089672061147155 140.10249869578925 0 36.089556498911826 140.10225911802502 0 36.089510016918545 140.10229315289146 0 36.089625298456816 140.10253207221498 0 36.08963821332166 140.10256054400614 0 36.089786469630816 140.10288519321708 0 36.08990640498772 140.1031382330232 0 36.089945424767315 140.10322131799347 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e44fd26d-8e3a-4abe-bb98-67211aa5a867">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.089996658823814 140.11167566671068 0 36.09000631212433 140.11163284793847 0 36.08992626002507 140.1115232961907 0 36.0898795847182 140.11155677872367 0 36.08991548830626 140.1116980450716 0 36.08995025203365 140.11170993855876 0 36.089996658823814 140.11167566671068 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e12ff430-83d1-4564-9d91-fade7fe554c4">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08999072307031 140.10331797217776 0 36.09003705649347 140.1032860004456 0 36.08999279537716 140.1031886312977 0 36.089945424767315 140.10322131799347 0 36.08994246610111 140.1032543885829 0 36.08996405643819 140.10331221761714 0 36.08999072307031 140.10331797217776 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_3a137d02-f363-4077-b4d7-73c29d78ba59">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08995245331367 140.10069573369796 0 36.09004667960365 140.1006302721982 0 36.089973848962295 140.10047124037547 0 36.08990552914824 140.10032717870496 0 36.08981214541701 140.10039440449336 0 36.08985520587748 140.1004851638761 0 36.08986525390243 140.10050607457066 0 36.0898797863906 140.10053688356498 0 36.08989359906846 140.10056713490275 0 36.08995245331367 140.10069573369796 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d675686a-c8d7-4845-b35d-7a5dda586286">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090098885211056 140.10074295023176 0 36.09010835437565 140.100700118327 0 36.09008135404687 140.10064228268396 0 36.09004667960365 140.1006302721982 0 36.08995245331367 140.10069573369796 0 36.089987866972876 140.1007731119489 0 36.090005667411596 140.10081043610163 0 36.090098885211056 140.10074295023176 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_45cdbd28-e324-42ad-beeb-26e894614b74">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09007300253461 140.10095161892883 0 36.090168949605804 140.10088976915287 0 36.090098885211056 140.10074295023176 0 36.090005667411596 140.10081043610163 0 36.09007300253461 140.10095161892883 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_23e739cc-8239-4c93-ad52-6ffbcb7ef07f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08985497245341 140.10922050215177 0 36.09015616331147 140.10899679633974 0 36.090112403091496 140.10889958884383 0 36.09008343893452 140.10883519180743 0 36.08990944776015 140.10845980344325 0 36.08973931214766 140.1080926475576 0 36.08972971584573 140.108071848553 0 36.089513945863146 140.10759837603175 0 36.08948300511479 140.10753053067904 0 36.08945045217242 140.10745901527562 0 36.08943960079697 140.10743521383475 0 36.08940435670712 140.10735769270812 0 36.089386689451246 140.1073189875366 0 36.089362744811545 140.10726637991124 0 36.08930588787727 140.1071412565202 0 36.08919889888971 140.1069061357385 0 36.089131996927115 140.10675910217222 0 36.08906106121451 140.10660228292448 0 36.0887514448424 140.10680742116625 0 36.0887945799527 140.10690283584103 0 36.08882516037752 140.10697035707673 0 36.088894662918314 140.10712306255397 0 36.0890041619252 140.10736396577192 0 36.08900882532509 140.10737419809965 0 36.08902120040447 140.10740133604222 0 36.08902622319079 140.10741245798226 0 36.08903797130048 140.10743826121936 0 36.08905160250774 140.10746827961097 0 36.08908011989847 140.107530908135 0 36.08909070213643 140.1075541533198 0 36.08909572516254 140.10756516423507 0 36.089097967486246 140.1075699469648 0 36.089133301571515 140.1076474681967 0 36.08924611855049 140.10789526921923 0 36.08936054881136 140.1081464079303 0 36.08942377348711 140.10828521291273 0 36.08960511992053 140.10867628169314 0 36.08977713702298 140.1090475534811 0 36.08985497245341 140.10922050215177 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1163f3df-51de-4371-b016-b9f366f03e84">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">2</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09006606280706 140.10147563502346 0 36.09017691764687 140.1013977236415 0 36.09004968964227 140.10105881081475 0 36.089777065887 140.10125034030258 0 36.08976668569543 140.10125752307226 0 36.08974709718526 140.10127100421465 0 36.089729314254015 140.1012831589261 0 36.08969004569823 140.1013103429601 0 36.08968616479943 140.10131310593638 0 36.08965511188632 140.10133454344202 0 36.08960708819317 140.10136769417824 0 36.08959923470854 140.10137310876203 0 36.08959120071515 140.10137863377776 0 36.08958343760844 140.10138393762037 0 36.089578111490304 140.10138769509484 0 36.0895372189865 140.10141598396848 0 36.0894150834568 140.10150029702726 0 36.089280850397806 140.10159311921785 0 36.08925530496885 140.10161068863147 0 36.08924564588871 140.1016174295757 0 36.08924212534738 140.10161986061001 0 36.089226417204614 140.10163080071803 0 36.089002184156556 140.10178672321442 0 36.088897379209044 140.1018595454142 0 36.08886109019191 140.10188474020566 0 36.08875818090162 140.10195623605998 0 36.08847220141493 140.10215480959494 0 36.08823704433138 140.10231813185257 0 36.088218719264795 140.10233083947 0 36.08813269321906 140.1023893884758 0 36.088009656909925 140.10247315135632 0 36.087975746058184 140.10248280759214 0 36.08774260767991 140.10254974920306 0 36.08762832212986 140.10262986574514 0 36.087480815765694 140.1027333986978 0 36.08769737578619 140.10322484855246 0 36.0878934558053 140.10308449364354 0 36.087965406206486 140.1030328710263 0 36.08816567794384 140.10279193486488 0 36.08820566722921 140.1027646435045 0 36.08834495183072 140.10266995404444 0 36.08835713810618 140.10266166730895 0 36.0883658031668 140.1026557004858 0 36.08844091005657 140.10260354382 0 36.088769046835665 140.10237579987032 0 36.089009439745794 140.1022088300403 0 36.08925723438132 140.1020364431664 0 36.089373324261075 140.10195577503794 0 36.08938903153164 140.10194483497153 0 36.089393635382315 140.1019416303141 0 36.08972212940526 140.10171477046933 0 36.08975480721302 140.10169222815804 0 36.08975706419422 140.10169057013883 0 36.08977322233892 140.10167952045336 0 36.08977863859603 140.1016757632958 0 36.08983938972219 140.10163376113104 0 36.08987865832505 140.10160658831202 0 36.08992568891154 140.10157421155824 0 36.09006606280706 140.10147563502346 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_dfde73ec-fdfe-40e1-b370-3275700bc05e">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09017540077623 140.10371780540547 0 36.09022279194502 140.10368565311893 0 36.09013550761559 140.10350258259854 0 36.09003705649347 140.1032860004456 0 36.08999072307031 140.10331797217776 0 36.0900290146766 140.1034041641353 0 36.09004201773527 140.10343340273698 0 36.09006677144818 140.10348768922987 0 36.09008856489166 140.10353573591365 0 36.09009322961949 140.10354552397555 0 36.09017540077623 140.10371780540547 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_31badab6-affe-49f1-977c-cd193a299a1b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090134863414285 140.10394787728785 0 36.090224413059644 140.10388588703432 0 36.09016593025892 140.103760759128 0 36.09007755412186 140.1038219761732 0 36.089734794294785 140.10405821824455 0 36.08968162268425 140.1040956693876 0 36.089739770302984 140.10422099772475 0 36.08979246439108 140.10418388702266 0 36.08979607508859 140.10418146749998 0 36.089835433266835 140.1041543962136 0 36.090134863414285 140.10394787728785 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_a9f3b7d3-9edc-4e13-83fe-6a246694184a">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09026108004879 140.10922979456632 0 36.09027501817364 140.10916544724427 0 36.09020839987583 140.10901385783765 0 36.09015616331147 140.10899679633974 0 36.08985497245341 140.10922050215177 0 36.08984040387546 140.10928463593072 0 36.089907744667734 140.10943545005895 0 36.08995979886391 140.10945339952508 0 36.09026108004879 140.10922979456632 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_6b472ce3-9888-4f1d-a2b6-305f6ab50d35">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090306208372596 140.1038651794994 0 36.09031594879111 140.10382245983646 0 36.090257556055484 140.10369733213983 0 36.09022279194502 140.10368565311893 0 36.09017540077623 140.10371780540547 0 36.09016593025892 140.103760759128 0 36.090224413059644 140.10388588703432 0 36.0902590862931 140.1038978878678 0 36.090306208372596 140.1038651794994 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_753c71c1-7335-4305-ab0a-88bd6e2598d1">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">2</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09040599771976 140.10133154016106 0 36.09043057193224 140.1012246852945 0 36.09025586088442 140.10091882482325 0 36.090168949605804 140.10088976915287 0 36.09007300253461 140.10095161892883 0 36.09004968964227 140.10105881081475 0 36.09017691764687 140.1013977236415 0 36.09026373693081 140.10142755688886 0 36.09040599771976 140.10133154016106 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_d49ea13f-0de1-4fd9-8c65-2ce49eef33f2">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09035636418341 140.10910498880844 0 36.09041802558566 140.10906099639786 0 36.090348114180465 140.1089119771485 0 36.09032594808582 140.10892776281617 0 36.09028532245059 140.10895681519875 0 36.09020839987583 140.10901385783765 0 36.09027501817364 140.10916544724427 0 36.09035636418341 140.10910498880844 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0594e2bc-c101-4088-826f-acf10488585f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090312010072786 140.1114108324974 0 36.0904500625072 140.11130626828685 0 36.09038875956787 140.11118330003268 0 36.09025154894741 140.11128713554393 0 36.09017173766231 140.11134526507348 0 36.090004985329095 140.11146671784414 0 36.08992626002507 140.1115232961907 0 36.09000631212433 140.11163284793847 0 36.09006427252037 140.11159129878112 0 36.090264160017284 140.1114456436818 0 36.090312010072786 140.1114108324974 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_6ad47c5e-c58e-4c27-9578-e589e8893df9">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09041461130183 140.11116367018576 0 36.09046155440442 140.11113048130173 0 36.090458775985816 140.11112451496155 0 36.09042693839394 140.1110559979816 0 36.090371884701206 140.1109323201325 0 36.0903599594056 140.11090551589533 0 36.090316465217256 140.11081086210692 0 36.090264271808856 140.11069739984848 0 36.09018006906601 140.1105115561654 0 36.09002007777493 140.11016464063405 0 36.089842613617414 140.10977280265888 0 36.08975500143242 140.10957959777377 0 36.08970868485288 140.1096140931596 0 36.08981154122069 140.10984109827498 0 36.08997295399065 140.11019734511356 0 36.089992235466276 140.11023916617538 0 36.0899938498516 140.11024261427596 0 36.09013294544579 140.11054414945718 0 36.09021714785636 140.1107301152138 0 36.09031122159821 140.11093453874562 0 36.090312746064875 140.110937886616 0 36.09037972449864 140.11108859073124 0 36.09041461130183 140.11116367018576 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_95999bdd-df5a-4f97-b5d7-1da67b43ca9f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09049937415755 140.1090389711575 0 36.09050893641327 140.10899614036504 0 36.09043888953742 140.1088472060341 0 36.090369101234735 140.10889703131306 0 36.090348114180465 140.1089119771485 0 36.09041802558566 140.10906099639786 0 36.09045278890525 140.109072788858 0 36.09049937415755 140.1090389711575 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_6463efa9-630c-4483-a0fe-4b982d521b3c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090423914212835 140.10374753988248 0 36.09053774693153 140.10366829787193 0 36.09048009797646 140.10354261058373 0 36.090340425704824 140.10363986045257 0 36.090257556055484 140.10369733213983 0 36.09031594879111 140.10382245983646 0 36.0903979147691 140.1037656625296 0 36.090408206806394 140.10375846862664 0 36.090423914212835 140.10374753988248 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_15f1234c-c2be-4de1-9bec-4dde6151a038">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.0904849162722 140.1113178288595 0 36.090532451796534 140.1112860990624 0 36.09051877091691 140.11125498435024 0 36.09050989363224 140.11123529775915 0 36.09049455943062 140.11120159651088 0 36.09048334789777 140.1111775708804 0 36.09047680200836 140.11116344486484 0 36.090467116627956 140.1111424228861 0 36.09046155440442 140.11113048130173 0 36.09041461130183 140.11116367018576 0 36.09039673390148 140.1111772660454 0 36.09038875956787 140.11118330003268 0 36.0904500625072 140.11130626828685 0 36.0904849162722 140.1113178288595 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_9c318cbd-e241-4223-8048-bb61a5c773b2">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090517524062676 140.11295333463224 0 36.0905642000114 140.1129197416133 0 36.09053603727972 140.1128604537453 0 36.09042619250744 140.11261798236814 0 36.0903448596125 140.11243957544553 0 36.09031248794108 140.1123685020719 0 36.09021429072713 140.1121550563849 0 36.090187567752416 140.11209699564924 0 36.090155911649745 140.11202780177135 0 36.090076819087614 140.11185496740018 0 36.09007233530053 140.11184517948976 0 36.089996658823814 140.11167566671068 0 36.08995025203365 140.11170993855876 0 36.090025121706695 140.11187754952996 0 36.09014053331849 140.1121298104045 0 36.09019891337301 140.11225659867574 0 36.09026536268746 140.1124011942441 0 36.090270205755196 140.11241188295887 0 36.09034750354643 140.11258139165986 0 36.090378978191396 140.1126505741805 0 36.09039368446614 140.1126828187225 0 36.0904890922527 140.11289349059928 0 36.090517524062676 140.11295333463224 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2b044024-373a-4027-8131-d5fcc4222ad5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09058687457336 140.1003694855762 0 36.090616084889746 140.10034932824567 0 36.090589662769325 140.10029120669188 0 36.090560325233184 140.10031142912487 0 36.09033952266855 140.10046525970097 0 36.09032110701782 140.10047785673098 0 36.09008135404687 140.10064228268396 0 36.09010835437565 140.100700118327 0 36.09036598155799 140.1005234267818 0 36.090576493558636 140.10037665742738 0 36.09058687457336 140.1003694855762 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_76d22b44-fa8d-4036-be43-335e8953ce18">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09061080658041 140.1036444619005 0 36.09061809026836 140.1036123941744 0 36.090560426895095 140.10348668009786 0 36.09048009797646 140.10354261058373 0 36.09053774693153 140.10366829787193 0 36.09056377509277 140.10367739293054 0 36.09061080658041 140.1036444619005 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ee429393-3570-4080-834c-40893b952a46">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090616084889746 140.10034932824567 0 36.09070774003686 140.1002860769303 0 36.09068128649418 140.1002278863743 0 36.09064661291382 140.10021587598035 0 36.09059940191582 140.10024848642666 0 36.090589662769325 140.10029120669188 0 36.090616084889746 140.10034932824567 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0f934a8e-3014-4f3f-a765-96a6bc5ec4c9">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090702633201786 140.1048498527571 0 36.090749572402814 140.10481682099865 0 36.09069199037717 140.10469236095128 0 36.09054075317298 140.10437280256247 0 36.090375355837345 140.10401721762923 0 36.09033858460078 140.1039364715055 0 36.090306208372596 140.1038651794994 0 36.0902590862931 140.1038978878678 0 36.09032832359634 140.10405003729008 0 36.09036841702032 140.10413634719845 0 36.09039093073929 140.10418461880974 0 36.090406537812676 140.10421809749906 0 36.0904335356548 140.1042762677314 0 36.09049389984061 140.10440617792918 0 36.090645137208995 140.10472562514462 0 36.09064890405939 140.10473385559638 0 36.090671237275124 140.10478212692874 0 36.090702633201786 140.1048498527571 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_770e767a-691e-438d-b58e-906b35892d32">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09079531527323 140.10491558851461 0 36.090802598399634 140.10488352083007 0 36.09077569000485 140.10482578351107 0 36.090749572402814 140.10481682099865 0 36.090702633201786 140.1048498527571 0 36.09074840854517 140.104948597025 0 36.09079531527323 140.10491558851461 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_96845359-e575-4095-89b2-887fc9ea7a74">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09081249214156 140.10509142478358 0 36.090859524888785 140.1050585056035 0 36.09084625513897 140.10502781079026 0 36.09083065433601 140.10499166663408 0 36.09079531527323 140.10491558851461 0 36.09074840854517 140.104948597025 0 36.09076765648143 140.10499011727887 0 36.090775728659295 140.10500757951456 0 36.090783262597334 140.1050239183215 0 36.090791690481296 140.10504350279615 0 36.09081249214156 140.10509142478358 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5f1df710-d580-4a52-ab0a-295411203034">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09078521450654 140.10535541298404 0 36.090874609168225 140.1052941173374 0 36.09080230087285 140.10513403195705 0 36.09071293259286 140.10519525563967 0 36.0905967533602 140.105275031668 0 36.09055911017745 140.10530088704255 0 36.090460172702954 140.10536882903685 0 36.09037603864842 140.10542707196365 0 36.09033767172562 140.10545370202757 0 36.090214179318856 140.10553933755978 0 36.08997323905004 140.10570718624075 0 36.08996069078649 140.10571602665632 0 36.08981886832315 140.10581592306437 0 36.08981634064285 140.1058176910786 0 36.08989021857371 140.10597678330245 0 36.08989160060408 140.1059758475135 0 36.09004678413338 140.1058664585355 0 36.09015231313105 140.1057929768314 0 36.0901651329437 140.10578402629733 0 36.0901950128148 140.10576313048787 0 36.09022001933612 140.1057457935512 0 36.0902500805889 140.10572479839664 0 36.090287273213704 140.10569894177024 0 36.09042656436283 140.1056023672449 0 36.09042746627538 140.1056017040734 0 36.090427828201314 140.10560148322756 0 36.09045608289946 140.10558181429784 0 36.09051214307852 140.10554303008303 0 36.09053281564071 140.10552866523318 0 36.09055032745307 140.1055166214693 0 36.09066388969042 140.1054387246282 0 36.09078521450654 140.10535541298404 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0d3c0509-f06e-4c25-8e92-1b669f0cdea6">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090891278789044 140.10438313636584 0 36.09093801139371 140.104349735974 0 36.09080211577846 140.10406110106317 0 36.090660132743935 140.10375377923302 0 36.0906369944314 140.10370262931443 0 36.09061080658041 140.1036444619005 0 36.09056377509277 140.10367739293054 0 36.090613010671674 140.10378649891803 0 36.09075526303547 140.1040942546317 0 36.090891278789044 140.10438313636584 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_0a4e82e2-c3b2-40b6-a545-4ecd4ee85ade">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09090033846648 140.10440237872083 0 36.09094707107641 140.10436897833662 0 36.09093801139371 140.104349735974 0 36.090891278789044 140.10438313636584 0 36.09090033846648 140.10440237872083 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_2198796d-5d37-4ebd-a7c2-d8df62c5e9c0">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09083621242415 140.10875597678225 0 36.09094509520617 140.10867540286128 0 36.09087309798476 140.10852783151728 0 36.090764356896 140.10860836608623 0 36.09063209145065 140.10870584859464 0 36.09048826937591 140.1088119521249 0 36.09043888953742 140.1088472060341 0 36.09050893641327 140.10899614036504 0 36.09055913144041 140.1089603363493 0 36.09072236454944 140.10883996451952 0 36.09083621242415 140.10875597678225 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_7fcbe5c3-c4e0-40b9-80c3-bb539418c315">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090874609168225 140.1052941173374 0 36.090966423061246 140.1052311641683 0 36.090894106609504 140.10507106197338 0 36.090859524888785 140.1050585056035 0 36.09081249214156 140.10509142478358 0 36.09080230087285 140.10513403195705 0 36.090874609168225 140.1052941173374 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_b35246c2-87fb-4c66-a304-6baea4288e50">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09065869928028 140.11098300014837 0 36.09096433086593 140.11076926429845 0 36.09079680869999 140.11040533919143 0 36.09064641387738 140.11007887639445 0 36.09060058973223 140.10997811805618 0 36.09057234156201 140.1099161657734 0 36.09038626329583 140.10950730528575 0 36.09036276822955 140.10945558627924 0 36.09029345151651 140.1093017660908 0 36.09026108004879 140.10922979456632 0 36.08995979886391 140.10945339952508 0 36.090011897978606 140.10956918106595 0 36.0900430138509 140.10963836084042 0 36.09005574752903 140.1096665001664 0 36.09034028901289 140.110291690649 0 36.09042369168445 140.11047287879916 0 36.09065869928028 140.11098300014837 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_95ea1cb2-2c8f-4520-a418-8d77bf9d0522">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09071942433436 140.1111150625294 0 36.09102522384922 140.11090130318857 0 36.09096433086593 140.11076926429845 0 36.09065869928028 140.11098300014837 0 36.09066170265587 140.1109895180847 0 36.09067515445433 140.111018781953 0 36.090686364087844 140.11104325180236 0 36.09069721525535 140.11106683201854 0 36.09071942433436 140.1111150625294 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_18d03411-2c30-492d-a763-5143578e26e7">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09102617716485 140.10865169932762 0 36.091035108628006 140.10860875495496 0 36.09096529864784 140.10845967865498 0 36.09096090571824 140.10846280155474 0 36.09087309798476 140.10852783151728 0 36.09094509520617 140.10867540286128 0 36.090980039756815 140.1086863963201 0 36.09102617716485 140.10865169932762 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_356d9ceb-567f-4bc3-8471-6e1beb31565b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09104871768364 140.10429847876503 0 36.091061355070515 140.104289638309 0 36.09105211551448 140.10427061740597 0 36.091039657989825 140.1042792363856 0 36.09093801139371 140.104349735974 0 36.09094707107641 140.10436897833662 0 36.09104871768364 140.10429847876503 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5c6312d1-1667-465a-9906-944c448588d8">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.090914900934585 140.10340797210682 0 36.09108063436241 140.10329549229982 0 36.091011589742315 140.1031774344281 0 36.09085804248658 140.10328173853347 0 36.09070539518683 140.10338660055038 0 36.09064112225579 140.10343091066048 0 36.09059354964568 140.10346361788433 0 36.090560426895095 140.10348668009786 0 36.09061809026836 140.1036123941744 0 36.090650949437276 140.10358951987513 0 36.09070050794632 140.10355548693255 0 36.09071052817857 140.10354851413928 0 36.09074988528414 140.10352145332732 0 36.09076405863053 140.10351172965662 0 36.090914900934585 140.10340797210682 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_23e8e5c6-73cf-4029-8002-ec5dc23b3737">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09089413389259 140.10482042747262 0 36.09109120635882 140.1046804074559 0 36.09095675777579 140.10438954432487 0 36.09094707107641 140.10436897833662 0 36.09090033846648 140.10440237872083 0 36.09090990584502 140.10442269776533 0 36.09100811678539 140.10463536958034 0 36.09100110501652 140.10466743830403 0 36.09088916184833 140.1047470078672 0 36.09087354510719 140.10475805894012 0 36.09086740620594 140.10476247973872 0 36.09077569000485 140.10482578351107 0 36.090802598399634 140.10488352083007 0 36.09089413389259 140.10482042747262 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e735dff2-5383-42a3-8969-6789417cfabb">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09105029516676 140.11256311169294 0 36.09109670216643 140.1125288509795 0 36.09103598850575 140.11239759979324 0 36.0909664097237 140.11224222012314 0 36.09093852376711 140.11218004592988 0 36.09086679103568 140.11202020602977 0 36.09082312277136 140.11192288526485 0 36.0907916416161 140.1118567120071 0 36.09074033809835 140.11174859289522 0 36.09071881835154 140.11170054452157 0 36.090668425273385 140.11158843111582 0 36.0906225988189 140.11148921542602 0 36.09060197230349 140.11144460170107 0 36.09055544089288 140.11133839882814 0 36.09053383342566 140.11128923989835 0 36.090532451796534 140.1112860990624 0 36.0904849162722 140.1113178288595 0 36.090554759052 140.111476872255 0 36.0906213013661 140.11162102396253 0 36.09069330350422 140.11178151913273 0 36.09077608900579 140.11195582254024 0 36.09084002084389 140.11209830012334 0 36.09089122021744 140.1122123047558 0 36.090988775154074 140.11243019199028 0 36.09105029516676 140.11256311169294 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_6fdb751b-e041-437c-81d3-da3dc84d61c2">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091032801543435 140.1030835604348 0 36.09114349351919 140.10291825501432 0 36.09108591326769 140.1027921300097 0 36.09090740341203 140.10241372533022 0 36.09074478881563 140.1020605953441 0 36.0906256718035 140.10180334856298 0 36.09046716487386 140.1014665478596 0 36.09040599771976 140.10133154016106 0 36.09026373693081 140.10142755688886 0 36.09032615956078 140.10156535595988 0 36.09048502477227 140.10190303474843 0 36.09060378277323 140.1021595137709 0 36.090766665977235 140.10251331052814 0 36.090945086193834 140.1028913590963 0 36.091032801543435 140.1030835604348 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f9a5ab01-3a12-4472-9ef0-88ee9fff94e5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091164109464394 140.11063127850582 0 36.09121410312922 140.11060480257058 0 36.091153928906984 140.11047322141138 0 36.09097312780903 140.11008314216684 0 36.090807862206724 140.10971688326595 0 36.090780691109494 140.10965682249054 0 36.09061173809325 140.10928654367777 0 36.09049937415755 140.1090389711575 0 36.09045278890525 140.109072788858 0 36.09056461503583 140.10931903793784 0 36.0907334778493 140.10968930517762 0 36.0907748163293 140.10978073092622 0 36.09092600405256 140.11011584708623 0 36.09108877948282 140.11046709605066 0 36.09110689584629 140.11050602643337 0 36.09111146907547 140.11051615895948 0 36.091164109464394 140.11063127850582 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_59499fad-c596-4b6c-8a37-cf57b3f78b2b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09102522384922 140.11090130318857 0 36.09122311683448 140.11076030159452 0 36.091164109464394 140.11063127850582 0 36.09112420687785 140.1106589003372 0 36.09096433086593 140.11076926429845 0 36.09102522384922 140.11090130318857 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_82b2fcad-2563-4064-8c19-d8aa75e9b09f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09126092212971 140.09990169859725 0 36.09127167646643 140.0998940270551 0 36.09124457356114 140.0998363470634 0 36.091234192060284 140.09984375224343 0 36.09122579712438 140.09984949840606 0 36.091091837643496 140.09994188224877 0 36.09098279253367 140.100017026491 0 36.090968529992736 140.10002686156318 0 36.090861651162996 140.1001005692726 0 36.090769479803406 140.10016699736073 0 36.09068128649418 140.1002278863743 0 36.09070774003686 140.1002860769303 0 36.09074412405474 140.10026096886781 0 36.09079584870479 140.1002251643555 0 36.090888110473294 140.10015862557128 0 36.09101945215022 140.10006812130172 0 36.091084444973745 140.10002336647975 0 36.09126092212971 140.09990169859725 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_ea17e0dd-8808-43ae-b639-3c2c997b2c34">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091283961239625 140.11075762816606 0 36.091293163846544 140.11071468484673 0 36.09121410312922 140.11060480257058 0 36.091164109464394 140.11063127850582 0 36.09122311683448 140.11076030159452 0 36.091283961239625 140.11075762816606 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_63f5411e-bf01-4c70-a0c4-238cde1de9c0">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091293643774065 140.10324191814428 0 36.091318129558914 140.10313506237566 0 36.09123040328144 140.1029477574695 0 36.09114349351919 140.10291825501432 0 36.091032801543435 140.1030835604348 0 36.091011589742315 140.1031774344281 0 36.09108063436241 140.10329549229982 0 36.09114133521083 140.10331646585954 0 36.091293643774065 140.10324191814428 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_32b79a0a-d5a2-4f49-8571-82ddef1896e7">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091324269731054 140.10840299139602 0 36.091489848680666 140.1082813039491 0 36.09141826613127 140.10813349352034 0 36.091352898595574 140.10818154977835 0 36.09133737000614 140.10819293368695 0 36.091275796926695 140.1082382490653 0 36.09125358724951 140.10825460685246 0 36.09115888458153 140.10832201634784 0 36.0910621073745 140.10839086207662 0 36.09096529864784 140.10845967865498 0 36.091035108628006 140.10860875495496 0 36.091223700000654 140.10847448910522 0 36.09124744296252 140.10845768143236 0 36.091324269731054 140.10840299139602 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_7e1262ad-6240-4d7c-9e70-5c206d5fd7d5">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091570839662204 140.10825793267955 0 36.091580131504806 140.1082149892209 0 36.09150852914076 140.1080671375255 0 36.09141826613127 140.10813349352034 0 36.091489848680666 140.1082813039491 0 36.09152470299137 140.10829275229582 0 36.091570839662204 140.10825793267955 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_c37a8861-a720-4d36-a343-61abf7ff7259">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09162919792043 140.1029226899002 0 36.0916443613442 140.10291274729812 0 36.091561260241804 140.10271300909528 0 36.09151991351808 140.10274275117143 0 36.09123040328144 140.1029477574695 0 36.091318129558914 140.10313506237566 0 36.09145551576365 140.10304258435588 0 36.09162919792043 140.1029226899002 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4ad47753-ccfe-49b8-b2b5-276b035bbf2f">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">3</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09137392126216 140.11257736010165 0 36.09169521750778 140.1123536228655 0 36.09167673505095 140.112317344541 0 36.09159428038868 140.11215603261985 0 36.09156558061384 140.11209497667795 0 36.09144243896313 140.11183357804387 0 36.09136999116148 140.1116709698093 0 36.09134703730243 140.11161947351974 0 36.091327669797266 140.11157609642348 0 36.091274499592856 140.11145664326475 0 36.09123029486078 140.1113576546947 0 36.091167081383865 140.11121595686706 0 36.0911520180845 140.11118203419855 0 36.09110072991806 140.11106703010952 0 36.09107822413087 140.1110165351783 0 36.09102522384922 140.11090130318857 0 36.09071942433436 140.1111150625294 0 36.090721697705156 140.11111999881285 0 36.090739723470364 140.11115903989173 0 36.09075066422988 140.11118284254994 0 36.09077129037864 140.11122765617526 0 36.0908267029652 140.11135190246344 0 36.090920222445874 140.11156166775672 0 36.090991415035695 140.11172149434103 0 36.091011230174345 140.1117657612488 0 36.0910642211045 140.11188465834317 0 36.091134516892694 140.11204236132704 0 36.09116321031451 140.11210654844746 0 36.09118141239782 140.1121471452651 0 36.09135536087606 140.1125358734323 0 36.09137392126216 140.11257736010165 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_6a9c1db7-23b5-4519-a506-aebcce33f46b">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09155198649148 140.10482546607793 0 36.091720072172684 140.1047096538685 0 36.09163840103942 140.10455602593163 0 36.091634790335284 140.10455844549304 0 36.09147934260909 140.10466563955777 0 36.09135052491319 140.1047544796431 0 36.0913335538118 140.1047661924115 0 36.09129419552947 140.10479325279616 0 36.091207263730546 140.10485325298907 0 36.09093147884792 140.1050454275433 0 36.090894106609504 140.10507106197338 0 36.090966423061246 140.1052311641683 0 36.091004212363686 140.10520525360758 0 36.091012156400346 140.1051997285741 0 36.091059188949124 140.1051669093186 0 36.09106984116617 140.105159505788 0 36.09117762767427 140.10508435313966 0 36.09118159968978 140.10508159061092 0 36.09125950551043 140.105027334131 0 36.09128017823598 140.1050128690684 0 36.09134806247876 140.10496612925618 0 36.091372165195814 140.10494944389137 0 36.09140357976595 140.10492778649524 0 36.09155198649148 140.10482546607793 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_50482045-d877-4099-869f-18bc0231074a">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09168956739555 140.1104377894807 0 36.091734165415986 140.1104064081616 0 36.091698710332956 140.11026502966087 0 36.09168020341005 140.1102780684912 0 36.09163154380262 140.11031232287715 0 36.09153819772095 140.11037762505833 0 36.09153485752128 140.11037994535567 0 36.091449910949336 140.11043772556877 0 36.09132470268953 140.11052290427034 0 36.09121410312922 140.11060480257058 0 36.091293163846544 140.11071468484673 0 36.09138326851837 140.11064803929855 0 36.09148608918357 140.11057810680344 0 36.09157600093223 140.11051690164248 0 36.091592159879106 140.1105058535168 0 36.09168956739555 140.1104377894807 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_38ffe09f-4dea-4ff2-9600-f1224e0e6114">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091698710332956 140.11026502966087 0 36.09174592374182 140.11023254692577 0 36.09167902130572 140.1100875046708 0 36.091616345059876 140.10994669731954 0 36.091567297686495 140.10983658733812 0 36.09144039945016 140.10956050956327 0 36.091418607388626 140.10951291570169 0 36.091292430180104 140.10923651944657 0 36.091278261483694 140.10920526538303 0 36.09117621154513 140.10897981435053 0 36.09113720880676 140.10889106185826 0 36.09110663366659 140.10882132806248 0 36.09109030914371 140.1087868462344 0 36.09102617716485 140.10865169932762 0 36.090980039756815 140.1086863963201 0 36.09105951117887 140.1088540226906 0 36.09106327695294 140.1088625976265 0 36.09112881923099 140.10901185278703 0 36.09117536034272 140.10911473476574 0 36.0911969720233 140.1091624498934 0 36.09124530600883 140.1092691139535 0 36.091296064466626 140.10938045064728 0 36.091367627652794 140.10953716798033 0 36.0913932755562 140.1095933371395 0 36.091520084664715 140.1098689480855 0 36.09153039605048 140.10989219335409 0 36.09163180793539 140.11011998746565 0 36.091698710332956 140.11026502966087 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_998c2bee-3264-43a0-a055-a59b5f5cd1c9">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091663898267115 140.10444084492522 0 36.09181315891223 140.10435929234512 0 36.091798894897366 140.10432937120643 0 36.09168059941804 140.10407011040587 0 36.091580576443214 140.1038607738283 0 36.09147626067411 140.10363643153454 0 36.091446931418965 140.10357281139403 0 36.09141571841243 140.10350518711093 0 36.09136844993213 140.10340297191212 0 36.091293643774065 140.10324191814428 0 36.09114133521083 140.10331646585954 0 36.09122753270791 140.10350198895935 0 36.09133534210885 140.1037355593421 0 36.09134054526755 140.10374678193833 0 36.09135032165166 140.1037675925669 0 36.09140118012782 140.103876816311 0 36.091408804073374 140.10389327760205 0 36.091440286748764 140.10396112505333 0 36.09153995053751 140.10416979386054 0 36.09155932273829 140.10421228089223 0 36.0916582469578 140.10442894330097 0 36.091663898267115 140.10444084492522 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_8c412cb5-b701-4726-a4cc-a428a7b2e091">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09179683288725 140.11202738715343 0 36.091848889253214 140.11200547218405 0 36.09172648728948 140.11173596879252 0 36.091712138861354 140.11170438032983 0 36.091643723404175 140.1115522244455 0 36.09158005925822 140.1114106351795 0 36.091488418612265 140.111206760489 0 36.09148285913665 140.11119441453835 0 36.09131328538447 140.11082180605575 0 36.091283961239625 140.11075762816606 0 36.09122311683448 140.11076030159452 0 36.09124679107581 140.11081224427906 0 36.09126616143041 140.11085451067243 0 36.09129996861306 140.11092880973564 0 36.09135215898113 140.11104348405533 0 36.091431341376065 140.11121733127464 0 36.09143564546797 140.11122689658922 0 36.09147707227438 140.11131897945202 0 36.09153006593554 140.11143688862919 0 36.09157301651283 140.11153254174235 0 36.09166483582856 140.1117368618535 0 36.091753252573845 140.11193139813122 0 36.09179683288725 140.11202738715343 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_1107c756-af09-4d78-b692-03973b550fe1">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">2</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.091691008352974 140.10035489744328 0 36.09212806216777 140.1000557583621 0 36.091979384196634 140.09972600175968 0 36.09187955002822 140.0997932937453 0 36.091793255824896 140.0998514142132 0 36.0916695914346 140.0999347274807 0 36.091488427183265 140.10005692421456 0 36.09145005960584 140.1000847898996 0 36.09119331238641 140.10027137153875 0 36.09118374301865 140.10027833521062 0 36.09113219861553 140.1003141404719 0 36.09091211982709 140.10046675306884 0 36.09085209078863 140.10050807064684 0 36.090685544046366 140.1006226742427 0 36.09067227461992 140.10063173519958 0 36.09063932615293 140.1006544882834 0 36.09063093115307 140.10066024542132 0 36.090467000650854 140.1007732803721 0 36.09025586088442 140.10091882482325 0 36.09043057193224 140.1012246852945 0 36.09081963457775 140.100956493078 0 36.091059751109036 140.10079129045363 0 36.09133299961819 140.10060176838422 0 36.091691008352974 140.10035489744328 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_5264d77e-1359-42b5-b3fd-ba776d5b135a">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.087928709198586 140.11238578802127 0 36.087967405968065 140.11220611199897 0 36.08799401801877 140.11207204537519 0 36.08800924505303 140.1120122133241 0 36.088034972185284 140.11193304119104 0 36.08807626396729 140.11183873313755 0 36.08809885861464 140.11180431155722 0 36.08818193227237 140.11172277636376 0 36.088227189757184 140.11169051112645 0 36.08799151717607 140.11117339425076 0 36.087948475716615 140.1112178933793 0 36.088144628895925 140.11164830101526 0 36.088143227829406 140.11167332534498 0 36.088059521850056 140.11175548016556 0 36.08802985399713 140.11180065913203 0 36.08798461154061 140.11190400326137 0 36.087956914483286 140.1119892424816 0 36.08794052310474 140.11205366758793 0 36.08791361020928 140.11218920994267 0 36.08787505012882 140.11236825560766 0 36.087928709198586 140.11238578802127 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_4a16c526-702d-4c7f-b310-bf3a2c702e1c">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08799151717607 140.11117339425076 0 36.08812845926206 140.11107729029945 0 36.0881409683026 140.11107776737572 0 36.088122021670145 140.11103159313546 0 36.0879725433336 140.11113119755933 0 36.08799151717607 140.11117339425076 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_eac50f9e-05b7-4b75-882c-56b56b940798">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.08758380828601 140.11228023300077 0 36.08765996783374 140.11188926613787 0 36.08766948177676 140.11171638114303 0 36.087677097773444 140.1116310291569 0 36.08768149911063 140.1115948224008 0 36.08768643563338 140.11156878685742 0 36.087697923707886 140.11153539446855 0 36.08772550889644 140.11146326695362 0 36.08774730546566 140.1114153863885 0 36.08776124636987 140.1113891959711 0 36.08778034978908 140.1113599700341 0 36.08780096375324 140.11133231511226 0 36.08784148019898 140.11128841797796 0 36.087923312143246 140.11120971862638 0 36.08789656852696 140.11115335990175 0 36.08789038167477 140.11115775654926 0 36.08780732122016 140.1112376486449 0 36.087763479659 140.11128513410276 0 36.08773976456169 140.11131695333026 0 36.087717938528066 140.1113503560038 0 36.087701544422565 140.11138114384488 0 36.087677908837996 140.11143306432592 0 36.08764907224922 140.1115084720536 0 36.08763537008433 140.11154828601767 0 36.087628900535655 140.1115824333906 0 36.087624030462756 140.1116225471866 0 36.08761617396961 140.11171055219697 0 36.08760688600538 140.111879282643 0 36.08753178457502 140.11226484964286 0 36.08758380828601 140.11228023300077 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">1</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_e8e5ab62-b4d0-421e-b692-8df2a38bc039">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">9999</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.087948475716615 140.1112178933793 0 36.08799151717607 140.11117339425076 0 36.0879725433336 140.11113119755933 0 36.087963291357816 140.11110594701972 0 36.08789656852696 140.11115335990175 0 36.087923312143246 140.11120971862638 0 36.087948475716615 140.1112178933793 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f3bcc49b-555c-4a8d-9541-b760a202be41">
			<core:creationDate>2024-03-19</core:creationDate>
			<tran:class codeSpace="../../codelists/Road_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:trafficArea>
				<tran:TrafficArea gml:id="tran_dd088f6f-9d3e-4003-95d3-7dba94ece5d6">
					<tran:function codeSpace="../../codelists/TrafficArea_function.xml">1010</tran:function>
					<tran:lod3MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_4b48ef55-2fb6-400b-8053-30841a072954">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3a0e121b-6167-47f1-8257-42e955fd75ee">
											<gml:posList>36.09144568715436 140.10149242939048 24.9856472 36.09144791380706 140.10150827002158 24.95467567 36.09145165170068 140.10150561500234 24.96961975 36.09144568715436 140.10149242939048 24.9856472</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_5f894980-ecef-4f9b-b64d-0982f2b0ff70">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_347dcbc9-77c1-4d9a-a8df-58ecdee58196">
											<gml:posList>36.09142989814035 140.10150358909448 24.91503334 36.09143603890374 140.10151668846328 24.9071331 36.09144791380706 140.10150827002158 24.95467567 36.09142989814035 140.10150358909448 24.91503334</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_e20af9cd-7950-4d13-95c9-9cdba1803173">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_82918cb0-c955-4eda-a2eb-dbc3665018f6">
											<gml:posList>36.09142989814035 140.10150358909448 24.91503334 36.09144791380706 140.10150827002158 24.95467567 36.09144568715436 140.10149242939048 24.9856472 36.09142989814035 140.10150358909448 24.91503334</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_401376b3-ffff-4dc8-9c52-9e257fb85389">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ce3d144e-d955-4b5b-8890-5fd11618c8b8">
											<gml:posList>36.09141752913487 140.10147712965727 24.91936493 36.09142405524843 140.1014910979523 24.91890526 36.091433593938156 140.1014843416299 24.96018028 36.09141752913487 140.10147712965727 24.91936493</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_de73375f-5f33-42c9-baa1-ce567d79e4de">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_188c6108-484b-4384-a296-fede71570c08">
											<gml:posList>36.09143355737228 140.10146527670952 24.9965744 36.091431779078526 140.1014614967993 24.99559975 36.09141574259118 140.10147308942206 24.92325592 36.09143355737228 140.10146527670952 24.9965744</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f74f49b6-522a-4a27-a573-0746a3108ce7">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9392f7b1-e0a1-46c4-bef4-79842fe0f525">
											<gml:posList>36.091433593938156 140.1014843416299 24.96018028 36.09142989814035 140.10150358909448 24.91503334 36.09144568715436 140.10149242939048 24.9856472 36.091433593938156 140.1014843416299 24.96018028</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_6f6ef5ad-d410-4e1e-9796-c08f8a45c505">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_20286c77-f5a2-426e-af79-0d768a3df41a">
											<gml:posList>36.09143355737228 140.10146527670952 24.9965744 36.09141752913487 140.10147712965727 24.91936493 36.091433593938156 140.1014843416299 24.96018028 36.09143355737228 140.10146527670952 24.9965744</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_0b4934db-8566-43c8-8a3b-0a841bb11463">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c7682871-b64b-429b-88a3-7fd73d3244b1">
											<gml:posList>36.09144568715436 140.10149242939048 24.9856472 36.09144004717161 140.1014797654782 24.9923954 36.091433593938156 140.1014843416299 24.96018028 36.09144568715436 140.10149242939048 24.9856472</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_02ebd709-d551-472e-bcd1-9f89a064a57c">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9d3c7da8-04fc-4692-acd4-d2efd1082e27">
											<gml:posList>36.09143355737228 140.10146527670952 24.9965744 36.09141574259118 140.10147308942206 24.92325592 36.09141752913487 140.10147712965727 24.91936493 36.09143355737228 140.10146527670952 24.9965744</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_07d10b09-a1d1-4638-a212-8e2cc4fa8484">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_1e25d3b2-bca6-4033-a546-549abce48f76">
											<gml:posList>36.091433593938156 140.1014843416299 24.96018028 36.09142405524843 140.1014910979523 24.91890526 36.09142989814035 140.10150358909448 24.91503334 36.091433593938156 140.1014843416299 24.96018028</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_b6a7658f-d5f1-4491-a65c-eb111f21a4ef">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6566277f-c417-474a-9f5f-e0c267112895">
											<gml:posList>36.091433593938156 140.1014843416299 24.96018028 36.09144004717161 140.1014797654782 24.9923954 36.09143355737228 140.10146527670952 24.9965744 36.091433593938156 140.1014843416299 24.96018028</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod3MultiSurface>
				</tran:TrafficArea>
			</tran:trafficArea>
			<tran:trafficArea>
				<tran:TrafficArea gml:id="tran_1d8d52eb-0b81-4a7c-9281-c7726319ba41">
					<tran:function codeSpace="../../codelists/TrafficArea_function.xml">2000</tran:function>
					<tran:lod3MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f0169b43-be98-4a04-bbdc-e2ddaf220c77">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_999b2c64-c1f0-46a8-8c2f-35df701638a7">
											<gml:posList>36.0911750527325 140.10169504032262 25.20619202 36.09117784285245 140.1016832727144 25.20619202 36.09117166335038 140.10168748108532 25.20619202 36.0911750527325 140.10169504032262 25.20619202</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_953e3ca7-e5ac-4545-a643-966eb7c04b86">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_cf2ff595-0230-40d7-b0c6-af2f7a5365d7">
											<gml:posList>36.0911750527325 140.10169504032262 25.20619202 36.09118123223459 140.10169083206318 25.20619202 36.09117784285245 140.1016832727144 25.20619202 36.0911750527325 140.10169504032262 25.20619202</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f955c045-bac9-4ef0-a284-e9ac8eaeed4d">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_b80cda33-2930-4ead-916f-911ff3c3e5ec">
											<gml:posList>36.091209815346275 140.10169708884578 25.16922188 36.0911750527325 140.10169504032262 25.20619202 36.09117547308328 140.10169597444437 25.20522118 36.091209815346275 140.10169708884578 25.16922188</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_fbd79cba-2ac5-4ade-8aac-8698d252d710">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_32abfb60-805b-4a7f-92f6-f4c0619f3ea0">
											<gml:posList>36.09130062619289 140.10158203319028 24.95639801 36.091281468849445 140.1015958707758 24.98280144 36.09129025674413 140.10161388090327 24.99821091 36.09130062619289 140.10158203319028 24.95639801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_549dc592-c27a-4444-989b-da32dc1d96de">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9d2d1e36-1358-4feb-8bc5-b713c0483ef8">
											<gml:posList>36.091281468849445 140.1015958707758 24.98280144 36.09130062619289 140.10158203319028 24.95639801 36.091280960683484 140.10159482797184 24.98251343 36.091281468849445 140.1015958707758 24.98280144</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_756da669-74ed-4d0d-b704-d73fd7b678aa">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f2772d42-6261-4935-987e-bda57d966940">
											<gml:posList>36.09125362238905 140.10161440714654 25.04195404 36.091267300045594 140.1016302866602 25.03454971 36.09129025674413 140.10161388090327 24.99821091 36.09125362238905 140.10161440714654 25.04195404</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_e033426e-368e-4777-86b8-53630fb92889">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_31146f7b-689e-4db4-b523-d52859f06280">
											<gml:posList>36.09137790794659 140.1015261860574 24.81175232 36.09137673432996 140.10152357931878 24.81012917 36.09135858312723 140.10154015324957 24.83866882 36.09137790794659 140.1015261860574 24.81175232</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_e77d1b67-a97a-4d5e-82ac-f2d4dc5f50e6">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c00b4fdc-1a0f-43e7-84e6-85f04f81a483">
											<gml:posList>36.09137673432996 140.10152357931878 24.81012917 36.091369193682404 140.10150672299102 24.81363487 36.09135858312723 140.10154015324957 24.83866882 36.09137673432996 140.10152357931878 24.81012917</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_c5ef8a7c-28e0-4533-8311-e921aa4696a3">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_db58a5e3-a2fa-4844-b846-bca9170962db">
											<gml:posList>36.09131497470301 140.1015459261839 24.91418648 36.09131994227462 140.1015680659889 24.91345215 36.09133926704883 140.1015541204646 24.87486839 36.09131497470301 140.1015459261839 24.91418648</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_42367d28-6247-4ebf-8246-361370d6c662">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2d358add-ed7d-4bfc-b170-efc2f7a93f29">
											<gml:posList>36.09138240085487 140.10153622165973 24.80320358 36.09135858312723 140.10154015324957 24.83866882 36.09136737675596 140.1015595515123 24.84247589 36.09138240085487 140.10153622165973 24.80320358</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3cbeb25c-eed6-4709-803f-84fef17e73bc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e1960ee7-45c8-4cb2-afc8-2091bc54da1f">
											<gml:posList>36.091369193682404 140.10150672299102 24.81363487 36.09134208419446 140.10152632453864 24.86391068 36.09135858312723 140.10154015324957 24.83866882 36.091369193682404 140.10150672299102 24.81363487</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_827996a0-7200-4163-8369-11db9518c922">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_1d13e0d5-e9cd-45c3-9a1f-583c73bcffd0">
											<gml:posList>36.09135455566663 140.10157384439975 24.86423874 36.09136737675596 140.1015595515123 24.84247589 36.091348140045284 140.1015734539195 24.88210297 36.09135455566663 140.10157384439975 24.86423874</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d42013ae-8aa3-4559-9d59-703df386c728">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_bf722f32-b4e4-4cb8-b2ba-67d351466d2a">
											<gml:posList>36.09136737675596 140.1015595515123 24.84247589 36.0913803428504 140.10157482129816 24.87458992 36.09138224682043 140.1015735914663 24.87450027 36.09136737675596 140.1015595515123 24.84247589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_7974cee0-db54-4c81-a36e-06321b04971c">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_a3be5f20-0bde-4509-9c5f-d430c3e4b769">
											<gml:posList>36.09138870671137 140.10155029769933 24.80227089 36.09138224682043 140.1015735914663 24.87450027 36.091394922622946 140.10156517587117 24.85140228 36.09138870671137 140.10155029769933 24.80227089</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_922b59b5-90e1-4e81-82a9-fda2e7b5585d">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_566fddeb-6517-4f5a-879a-0a69e204c840">
											<gml:posList>36.09136737675596 140.1015595515123 24.84247589 36.0913866135137 140.10154562733305 24.80430794 36.09138240085487 140.10153622165973 24.80320358 36.09136737675596 140.1015595515123 24.84247589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_74a6f2a5-6041-473a-bc85-e4f0c7a8c24a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c60fa74f-d430-47ce-acbd-12d2c3c9dd52">
											<gml:posList>36.09121010274613 140.10161985496325 25.1380806 36.09117464968434 140.10161951759946 25.15528679 36.091183619615734 140.10163878638755 25.14565659 36.09121010274613 140.10161985496325 25.1380806</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f69f6d0b-bd1b-4b01-998e-4bcc61ac1217">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_8724539f-3b4a-4a23-a6d8-9d4f72438cc2">
											<gml:posList>36.091183619615734 140.10163878638755 25.14565659 36.09117464968434 140.10161951759946 25.15528679 36.09114908700122 140.101640165504 25.1935215 36.091183619615734 140.10163878638755 25.14565659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_4e99e9e6-868d-493f-8dfe-6bcced39ecd6">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2d58d4e2-16f6-490e-b932-8378a89c1799">
											<gml:posList>36.09117464968434 140.10161951759946 25.15528679 36.091201027033456 140.10160065099655 25.13873291 36.0911777819941 140.10159250369486 25.16017151 36.09117464968434 140.10161951759946 25.15528679</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_9a82b3e2-46ec-4a63-892d-19aab2911965">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_55db1fc0-39a5-47e3-a293-70e225c68b45">
											<gml:posList>36.0911777819941 140.10159250369486 25.16017151 36.091201027033456 140.10160065099655 25.13873291 36.091178304187736 140.1015912258623 25.16076469 36.0911777819941 140.10159250369486 25.16017151</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_123aac70-57eb-4f05-9f72-8ce19c470577">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_fd157cf4-9492-471e-a66c-e7c75e475e70">
											<gml:posList>36.0911777819941 140.10159250369486 25.16017151 36.091162391228174 140.1016027103476 25.17502022 36.09117464968434 140.10161951759946 25.15528679 36.0911777819941 140.10159250369486 25.16017151</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_96817b57-2ff0-44a9-b707-ce6f7438c16c">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c58d371c-1425-4d4b-8438-0c7153319e2e">
											<gml:posList>36.09114826358959 140.10163838416005 25.19202232 36.09114908700122 140.101640165504 25.1935215 36.09117464968434 140.10161951759946 25.15528679 36.09114826358959 140.10163838416005 25.19202232</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_06145d06-fb85-413f-8d8a-a02233d05ccc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e06179ea-846f-4fbf-971b-87a52f782dec">
											<gml:posList>36.09121075975082 140.10162124529765 25.13861656 36.09119892822236 140.10165352186476 25.13913727 36.091226275307335 140.1016339645124 25.10983276 36.09121075975082 140.10162124529765 25.13861656</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_de5594d8-d4ed-440d-ad07-59596f3da1f4">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_29c3d852-e649-498c-b8a0-f4ac93ce7378">
											<gml:posList>36.091178304187736 140.1015912258623 25.16076469 36.091201027033456 140.10160065099655 25.13873291 36.09119196010192 140.1015814687185 25.13285637 36.091178304187736 140.1015912258623 25.16076469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_780c84b6-ea2a-4d43-8857-8cb248cdb079">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ec5585df-0729-409b-a516-bd46c2e562a0">
											<gml:posList>36.091183619615734 140.10163878638755 25.14565659 36.09121075975082 140.10162124529765 25.13861656 36.09121010274613 140.10161985496325 25.1380806 36.091183619615734 140.10163878638755 25.14565659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_9fe9a4fd-c912-4687-91ec-f2d23135381a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_80a2e138-e7cb-44be-b82b-59f8ac1b86d8">
											<gml:posList>36.09125362238905 140.10161440714654 25.04195404 36.091260755849135 140.10158510766914 25.02545166 36.0912426828048 140.10159816815198 25.04432106 36.09125362238905 140.10161440714654 25.04195404</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_a34a6b66-34da-4b13-b611-0816af47e25e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e0594b5a-377e-4c32-8811-7aed279ae93b">
											<gml:posList>36.09118680902693 140.10157056328276 25.13362885 36.091178304187736 140.1015912258623 25.16076469 36.09119196010192 140.1015814687185 25.13285637 36.09118680902693 140.10157056328276 25.13362885</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d5548820-027d-4e90-b559-744ad8477ab0">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6ef53ccf-406b-4d6b-ab34-674eb6cb8173">
											<gml:posList>36.091201027033456 140.10160065099655 25.13873291 36.09117464968434 140.10161951759946 25.15528679 36.09121010274613 140.10161985496325 25.1380806 36.091201027033456 140.10160065099655 25.13873291</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_4082bdfa-055a-48d9-ae7d-927cbeddf23d">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6e18467c-5591-4cb6-a5a7-f4796da544f0">
											<gml:posList>36.0912426828048 140.10159816815198 25.04432106 36.091226275307335 140.1016339645124 25.10983276 36.09125362238905 140.10161440714654 25.04195404 36.0912426828048 140.10159816815198 25.04432106</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_57d76baf-432e-4e82-be86-ac4085d01a04">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_1be647fc-7279-4122-bb7d-6976acf5f9c2">
											<gml:posList>36.09125362238905 140.10161440714654 25.04195404 36.091226275307335 140.1016339645124 25.10983276 36.09124434329628 140.10164671406187 25.09251785 36.09125362238905 140.10161440714654 25.04195404</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_b11f0f4a-d226-402a-a591-82d960cfd3f3">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_5d3d7608-ec10-42d5-b599-a4c94b58bb77">
											<gml:posList>36.091140439606455 140.10161833860747 25.17980766 36.09114826358959 140.10163838416005 25.19202232 36.091162391228174 140.1016027103476 25.17502022 36.091140439606455 140.10161833860747 25.17980766</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_5792e886-6b85-4193-89dd-30de9ae5d257">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_76513c44-9b3b-4f1e-b544-1ea88080f786">
											<gml:posList>36.09117166335038 140.10168748108532 25.20619202 36.09117158113415 140.10167307920358 25.21131706 36.09116882567956 140.10168115991112 25.20949554 36.09117166335038 140.10168748108532 25.20619202</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d9f5cea2-1ce2-4af4-9682-20503865375d">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_b432fd11-1c42-4f80-9d85-1aa6f1e51e28">
											<gml:posList>36.09114908700122 140.101640165504 25.1935215 36.09115714540538 140.10165771771847 25.21895981 36.091183619615734 140.10163878638755 25.14565659 36.09114908700122 140.101640165504 25.1935215</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_ce534719-fbfb-4d50-9334-72c2a33f7569">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9fbbf041-eeb3-40ef-b32b-b289e99565d3">
											<gml:posList>36.091162391228174 140.1016027103476 25.17502022 36.09114826358959 140.10163838416005 25.19202232 36.09117464968434 140.10161951759946 25.15528679 36.091162391228174 140.1016027103476 25.17502022</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_54c06e5e-8b0c-464d-bc30-da16b311dbda">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c73a53ff-1da9-41b1-a757-26edf870c4b8">
											<gml:posList>36.091140439606455 140.10161833860747 25.17980766 36.091139390603125 140.101619050747 25.17941093 36.09114826358959 140.10163838416005 25.19202232 36.091140439606455 140.10161833860747 25.17980766</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_b48d257c-1d1a-4802-a83e-fc347a6dec08">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_7e2be6bc-fbf4-44ee-9c84-59a48db9dd68">
											<gml:posList>36.09116882567956 140.10168115991112 25.20949554 36.09117158113415 140.10167307920358 25.21131706 36.091144234132784 140.10169263652924 25.25620842 36.09116882567956 140.10168115991112 25.20949554</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_4a93259c-538d-4431-975b-0e8323293425">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_a568ceb9-d30e-45d2-8b13-05f30b5e886c">
											<gml:posList>36.09115714540538 140.10165771771847 25.21895981 36.09115872198955 140.10166117156348 25.22476768 36.091183619615734 140.10163878638755 25.14565659 36.09115714540538 140.10165771771847 25.21895981</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_ef1ee3ec-bd3b-4969-a5a2-5cf29502dce6">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_0264b94f-0806-4bfb-95a1-4d5a96a8d7f6">
											<gml:posList>36.091183619615734 140.10163878638755 25.14565659 36.09115872198955 140.10166117156348 25.22476768 36.09117158113415 140.10167307920358 25.21131706 36.091183619615734 140.10163878638755 25.14565659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_e0f6abc6-c0d3-46f5-ac00-c0df4a55d7da">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_0b4ab25c-f9fa-44c2-a754-c53c1caee095">
											<gml:posList>36.091183619615734 140.10163878638755 25.14565659 36.09117158113415 140.10167307920358 25.21131706 36.09119892822236 140.10165352186476 25.13913727 36.091183619615734 140.10163878638755 25.14565659</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_8720812c-ae1f-49d9-91c5-6b7fc5354111">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_a90abcd1-58b7-4d9c-8d63-6fb229fd1931">
											<gml:posList>36.091147050187004 140.1016692870366 25.26788521 36.091144234132784 140.10169263652924 25.25620842 36.09117158113415 140.10167307920358 25.21131706 36.091147050187004 140.1016692870366 25.26788521</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_fafcfe27-57f9-4e3b-adfe-56665cfde0b7">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3738c7b4-b4f9-4b24-9eac-1df746d4cea6">
											<gml:posList>36.091147050187004 140.1016692870366 25.26788521 36.091136471516975 140.10167662533345 25.27880669 36.091144234132784 140.10169263652924 25.25620842 36.091147050187004 140.1016692870366 25.26788521</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_765b3ccf-804a-4164-83ed-6a717cde6780">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_390cda28-afe5-41de-a7e3-e6fd1eb42ee4">
											<gml:posList>36.09115872198955 140.10166117156348 25.22476768 36.091147050187004 140.1016692870366 25.26788521 36.09117158113415 140.10167307920358 25.21131706 36.09115872198955 140.10166117156348 25.22476768</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d67fcdb2-858f-49c7-8f9b-84d70f81511a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_0bdfcfd2-c893-4652-aecd-db6c8dcd9460">
											<gml:posList>36.09116882567956 140.10168115991112 25.20949554 36.091144234132784 140.10169263652924 25.25620842 36.09114639820232 140.1016970901381 25.24954414 36.09116882567956 140.10168115991112 25.20949554</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_88a8affd-d5e2-46b4-86de-904090b7f368">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_0b625c25-6bd5-4033-9641-c32a8a9d6ba3">
											<gml:posList>36.09119892822236 140.10165352186476 25.13913727 36.09117158113415 140.10167307920358 25.21131706 36.09119842983949 140.10167954707129 25.16604805 36.09119892822236 140.10165352186476 25.13913727</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d46bdd0d-710b-438d-a6db-4d5fc7a83bad">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_46912905-7144-4248-8c59-2a0d496371a7">
											<gml:posList>36.09125362238905 140.10161440714654 25.04195404 36.091280960683484 140.10159482797184 24.98251343 36.091260755849135 140.10158510766914 25.02545166 36.09125362238905 140.10161440714654 25.04195404</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3e2766d3-6af0-425d-9c01-e1d608c8b83b">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_08b0914c-8e5a-418a-9a50-679178361a00">
											<gml:posList>36.09122138654466 140.10166314145397 25.1413002 36.09124434329628 140.10164671406187 25.09251785 36.091226275307335 140.1016339645124 25.10983276 36.09122138654466 140.10166314145397 25.1413002</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_8aa582a7-c01e-4e43-9bbb-0cb8940cbf5f">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_18b409e9-d073-4f01-b265-3d2c206ee7e4">
											<gml:posList>36.09123548740448 140.1016787186914 25.15231514 36.09122138654466 140.10166314145397 25.1413002 36.091209815346275 140.10169708884578 25.16922188 36.09123548740448 140.1016787186914 25.15231514</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_5e1974bc-560a-4a15-94f5-6489022afd6e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6f98af6d-ed0e-4983-bf47-38a8bed6c4c9">
											<gml:posList>36.09122740361519 140.10166071096256 25.13858414 36.09124434329628 140.10164671406187 25.09251785 36.09122138654466 140.10166314145397 25.1413002 36.09122740361519 140.10166071096256 25.13858414</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2df9fda1-b8a7-454e-a433-f1eb26e002df">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_8aab9d4f-f528-4c90-9969-b4822b8beb96">
											<gml:posList>36.0912426828048 140.10159816815198 25.04432106 36.09121075975082 140.10162124529765 25.13861656 36.091226275307335 140.1016339645124 25.10983276 36.0912426828048 140.10159816815198 25.04432106</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_686d2ca8-8039-40f6-861c-ef59b9831d6d">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9621adc6-b6de-49ae-93ec-a26d8134e1e3">
											<gml:posList>36.091209815346275 140.10169708884578 25.16922188 36.09122138654466 140.10166314145397 25.1413002 36.09119842983949 140.10167954707129 25.16604805 36.091209815346275 140.10169708884578 25.16922188</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_66987187-2683-41a4-8447-6c04658910bb">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_77ce934c-255a-436a-9c96-ab5da7714201">
											<gml:posList>36.09121075975082 140.10162124529765 25.13861656 36.091183619615734 140.10163878638755 25.14565659 36.09119892822236 140.10165352186476 25.13913727 36.09121075975082 140.10162124529765 25.13861656</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_356d875b-1c62-43fa-ba95-35d7a0afc2ae">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_4fa23dee-d54a-4a63-b0ab-1fd6452b9a75">
											<gml:posList>36.09124434329628 140.10164671406187 25.09251785 36.09124901848716 140.1016461009752 25.08164406 36.091267300045594 140.1016302866602 25.03454971 36.09124434329628 140.10164671406187 25.09251785</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_072f2c5d-fe75-437e-bb2b-1b3c9b78d9fd">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_d17ac897-e24f-45ad-8cde-3a8184ed3647">
											<gml:posList>36.09122740361519 140.10166071096256 25.13858414 36.09124901848716 140.1016461009752 25.08164406 36.09124434329628 140.10164671406187 25.09251785 36.09122740361519 140.10166071096256 25.13858414</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_ec915550-684f-4f96-a6d6-2e642ca64fd8">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_063121ca-1e2e-4a37-b692-d9d68a4dcbfa">
											<gml:posList>36.09124901848716 140.1016461009752 25.08164406 36.09127063326743 140.1016314908685 25.02470398 36.091267300045594 140.1016302866602 25.03454971 36.09124901848716 140.1016461009752 25.08164406</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2bdc2c74-b19e-46d2-a8c0-fe7f240d6eb5">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f44e088e-85ce-47ce-8a97-4569b277fe6f">
											<gml:posList>36.09125362238905 140.10161440714654 25.04195404 36.09124434329628 140.10164671406187 25.09251785 36.091267300045594 140.1016302866602 25.03454971 36.09125362238905 140.10161440714654 25.04195404</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_de6cd6c5-4554-45c0-81fa-1d80e7861ac8">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_5de28bed-0bd8-4c06-808c-87c32807b5bd">
											<gml:posList>36.09118415216643 140.10171543736402 25.20115471 36.091209815346275 140.10169708884578 25.16922188 36.09117547308328 140.10169597444437 25.20522118 36.09118415216643 140.10171543736402 25.20115471</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_c57e8586-0c2b-434f-b0e3-7bf318c8b36c">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_355d845f-ab4f-4d60-afcc-1529fb05e113">
											<gml:posList>36.091226275307335 140.1016339645124 25.10983276 36.09119892822236 140.10165352186476 25.13913727 36.09122138654466 140.10166314145397 25.1413002 36.091226275307335 140.1016339645124 25.10983276</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_6b50beb8-911d-425b-8c54-eaab2a7f0b7a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_30210073-fd8f-4f18-a521-e74cf6cdad5e">
											<gml:posList>36.091209815346275 140.10169708884578 25.16922188 36.091238123614644 140.10168460543187 25.15415001 36.09123548740448 140.1016787186914 25.15231514 36.091209815346275 140.10169708884578 25.16922188</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3d37b5ef-99f3-4c47-b187-48c839916bee">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_657cff35-a8ab-442a-aeb0-ef5b3ec66d67">
											<gml:posList>36.09119892822236 140.10165352186476 25.13913727 36.09119842983949 140.10167954707129 25.16604805 36.09122138654466 140.10166314145397 25.1413002 36.09119892822236 140.10165352186476 25.13913727</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f9ffd753-d6e5-4539-93e6-d7481618f39e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_5eb3fc8e-8cee-4ef6-8fb0-6136787c77c0">
											<gml:posList>36.09123548740448 140.1016787186914 25.15231514 36.09122740361519 140.10166071096256 25.13858414 36.09122138654466 140.10166314145397 25.1413002 36.09123548740448 140.1016787186914 25.15231514</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_eb0c3ce2-8de1-4d70-9e04-c5c0a5f863b9">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_1a54aeac-2fdd-426b-b12b-c08776e9a52b">
											<gml:posList>36.091209815346275 140.10169708884578 25.16922188 36.09120142937852 140.10171183062533 25.17891121 36.091238123614644 140.10168460543187 25.15415001 36.091209815346275 140.10169708884578 25.16922188</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f56dddfa-b33e-41fa-acef-43b08a2c148d">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_5646b322-e284-4a54-ab6f-ca66ea0cd089">
											<gml:posList>36.09134208419446 140.10152632453864 24.86391068 36.09131497470301 140.1015459261839 24.91418648 36.09133926704883 140.1015541204646 24.87486839 36.09134208419446 140.10152632453864 24.86391068</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_e17dd8ea-fe5d-443b-b7a9-8b7e48451921">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_7f377955-14cd-45b6-b75a-08e1b908fdca">
											<gml:posList>36.09135858312723 140.10154015324957 24.83866882 36.09134208419446 140.10152632453864 24.86391068 36.09133926704883 140.1015541204646 24.87486839 36.09135858312723 140.10154015324957 24.83866882</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_ba234ed6-9a10-4648-8fea-24ac5f926dbe">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ac5d958d-2362-4f8c-92a0-b8b846f9132d">
											<gml:posList>36.09131497470301 140.1015459261839 24.91418648 36.091280935883525 140.10157051443392 24.96525574 36.09130062619289 140.10158203319028 24.95639801 36.09131497470301 140.1015459261839 24.91418648</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_9ccb0753-2ae0-433b-98d0-5648b95dbd1a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f816e64b-42b5-43ba-a941-820697dc9364">
											<gml:posList>36.09135858312723 140.10154015324957 24.83866882 36.09133926704883 140.1015541204646 24.87486839 36.091348140045284 140.1015734539195 24.88210297 36.09135858312723 140.10154015324957 24.83866882</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_0ee5a6db-bd96-4823-891e-d6d43c8027cc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3f5df3ac-b305-4286-86a6-5077c93329be">
											<gml:posList>36.09131497470301 140.1015459261839 24.91418648 36.09130062619289 140.10158203319028 24.95639801 36.09131994227462 140.1015680659889 24.91345215 36.09131497470301 140.1015459261839 24.91418648</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2ae161b6-af24-4d77-9166-dfeaeb3f8bce">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_44fee059-5452-403a-9eab-f3e127ae5e2c">
											<gml:posList>36.09127063326743 140.1016314908685 25.02470398 36.09129025674413 140.10161388090327 24.99821091 36.091267300045594 140.1016302866602 25.03454971 36.09127063326743 140.1016314908685 25.02470398</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_5cdf5318-a08a-4923-9325-5ac25fee867f">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_aa65413f-95a2-45e4-9983-25df36c33271">
											<gml:posList>36.091348140045284 140.1015734539195 24.88210297 36.09131994227462 140.1015680659889 24.91345215 36.09132890333298 140.10158735631995 24.91889572 36.091348140045284 140.1015734539195 24.88210297</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_164aaac2-b726-4f28-a750-cf7fcd392297">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ffe7adac-fc7f-4d2f-9329-e5acffd26e6b">
											<gml:posList>36.09133926704883 140.1015541204646 24.87486839 36.09131994227462 140.1015680659889 24.91345215 36.091348140045284 140.1015734539195 24.88210297 36.09133926704883 140.1015541204646 24.87486839</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f71f3930-e64c-47ab-bf3d-3cd3b66dd517">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_219ad0db-4f0b-4147-b1f1-9383060c3b09">
											<gml:posList>36.09132363982232 140.10159523330086 24.93269157 36.091348140045284 140.1015734539195 24.88210297 36.09132890333298 140.10158735631995 24.91889572 36.09132363982232 140.10159523330086 24.93269157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3b5b4931-8ae0-44dd-94d4-22a8d473f321">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_a89f7ca7-c765-4484-a364-41b733ce325f">
											<gml:posList>36.0913803428504 140.10157482129816 24.87458992 36.09136737675596 140.1015595515123 24.84247589 36.09136099491873 140.1015873135673 24.875494 36.0913803428504 140.10157482129816 24.87458992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_7d59592f-c49b-4a9d-b693-fe756777397f">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c88da23c-f950-4202-8958-ca2e66a3741c">
											<gml:posList>36.09135858312723 140.10154015324957 24.83866882 36.091348140045284 140.1015734539195 24.88210297 36.09136737675596 140.1015595515123 24.84247589 36.09135858312723 140.10154015324957 24.83866882</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2fa62caf-5581-44bb-8f4a-5864c3b59c00">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c71543bc-7a4a-40bd-8cd7-4d1a7541d53e">
											<gml:posList>36.09138870671137 140.10155029769933 24.80227089 36.09136737675596 140.1015595515123 24.84247589 36.09138224682043 140.1015735914663 24.87450027 36.09138870671137 140.10155029769933 24.80227089</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_a07b1b30-b2e0-4f29-bf5f-0fa426b3fe22">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6b6e0333-cc5d-433b-9f3f-73a253ac301f">
											<gml:posList>36.09127063326743 140.1016314908685 25.02470398 36.09129042990315 140.10161516121164 24.99909973 36.09129025674413 140.10161388090327 24.99821091 36.09127063326743 140.1016314908685 25.02470398</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f22d9750-840d-456f-83da-42c9e5e641fc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3b1676a5-6fc0-4254-80fc-61fb077c0843">
											<gml:posList>36.09132363982232 140.10159523330086 24.93269157 36.091309666618756 140.10160125882473 24.96043396 36.091291930225864 140.1016172918848 25.0011425 36.09132363982232 140.10159523330086 24.93269157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_e06b9d99-50e0-4630-8a0a-743a309536ae">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2f212046-b7d2-4863-957e-7d08edb0292d">
											<gml:posList>36.091280935883525 140.10157051443392 24.96525574 36.091280960683484 140.10159482797184 24.98251343 36.09130062619289 140.10158203319028 24.95639801 36.091280935883525 140.10157051443392 24.96525574</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_7b2383fe-1be5-4af1-b2b6-73ac4e18bfab">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_655f9fec-c025-4ddb-a6db-00cdb0c8c3c7">
											<gml:posList>36.091280935883525 140.10157051443392 24.96525574 36.09127218147978 140.10157683964363 24.98588371 36.091280960683484 140.10159482797184 24.98251343 36.091280935883525 140.10157051443392 24.96525574</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_5e5c89c2-38d5-4a90-9086-92a0361b5ad0">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ec01dfe5-d70e-4c48-831e-dc4ccce270f4">
											<gml:posList>36.09127218147978 140.10157683964363 24.98588371 36.091260755849135 140.10158510766914 25.02545166 36.091280960683484 140.10159482797184 24.98251343 36.09127218147978 140.10157683964363 24.98588371</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_4c1f2f21-c6e6-48d6-a032-c014473bad5e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_53add22d-86cb-464f-88b4-6a02dd6ec33e">
											<gml:posList>36.09132890333298 140.10158735631995 24.91889572 36.09131994227462 140.1015680659889 24.91345215 36.091309666618756 140.10160125882473 24.96043396 36.09132890333298 140.10158735631995 24.91889572</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d0caaff3-2eb5-45c5-a9bc-ea9fa49fd674">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_50c1d227-1f18-4849-8d25-eb64e49b3a98">
											<gml:posList>36.09131994227462 140.1015680659889 24.91345215 36.09130062619289 140.10158203319028 24.95639801 36.091309666618756 140.10160125882473 24.96043396 36.09131994227462 140.1015680659889 24.91345215</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_973a2bfe-9695-4a11-bb49-19846832c0dc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e7cc17d6-b9fc-4d80-bc40-8bc6d6004c45">
											<gml:posList>36.09127063326743 140.1016314908685 25.02470398 36.091291930225864 140.1016172918848 25.0011425 36.09129042990315 140.10161516121164 24.99909973 36.09127063326743 140.1016314908685 25.02470398</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_7ac7e793-9e74-48d4-975b-4becd3d792e5">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6b42b19b-f015-43f5-939e-e0e1f14a2408">
											<gml:posList>36.091309666618756 140.10160125882473 24.96043396 36.09132363982232 140.10159523330086 24.93269157 36.09132890333298 140.10158735631995 24.91889572 36.091309666618756 140.10160125882473 24.96043396</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_7230b197-0b9b-4ebf-8dc9-05a9fb58befc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_31be3b87-c265-4db5-9e6b-df687e874ec7">
											<gml:posList>36.09135455566663 140.10157384439975 24.86423874 36.091348140045284 140.1015734539195 24.88210297 36.09132363982232 140.10159523330086 24.93269157 36.09135455566663 140.10157384439975 24.86423874</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_c7ecae7d-ca42-485e-95c1-9fd2bce87d1a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_7f90f39a-2a5c-43df-9ced-5c7dc3cca122">
											<gml:posList>36.09136737675596 140.1015595515123 24.84247589 36.09135455566663 140.10157384439975 24.86423874 36.09136099491873 140.1015873135673 24.875494 36.09136737675596 140.1015595515123 24.84247589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_81d03425-3d36-41cf-82b4-2d733cc48587">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_20914adf-c14d-4f18-9a93-a5283624a153">
											<gml:posList>36.09129076490993 140.1016149237075 24.99909973 36.09129042990315 140.10161516121164 24.99909973 36.091291930225864 140.1016172918848 25.0011425 36.09129076490993 140.1016149237075 24.99909973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_fc4b40cc-f329-40b9-ab89-9c12a41379d8">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_254d89da-1590-4588-8cdc-a40ad029983d">
											<gml:posList>36.09129076490993 140.1016149237075 24.99909973 36.09129025674413 140.10161388090327 24.99821091 36.09129042990315 140.10161516121164 24.99909973 36.09129076490993 140.1016149237075 24.99909973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_83d3467c-6313-49b8-bb88-b57923a1986f">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3750c996-5bcd-4eca-b7f3-cd9007ac9ca4">
											<gml:posList>36.09129076490993 140.1016149237075 24.99909973 36.091291930225864 140.1016172918848 25.0011425 36.091309666618756 140.10160125882473 24.96043396 36.09129076490993 140.1016149237075 24.99909973</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_447c03d9-76d9-42ea-b421-0b85b6b1e1cc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_93997d2c-7b95-4f7b-b717-533a6a73e699">
											<gml:posList>36.091281468849445 140.1015958707758 24.98280144 36.09125362238905 140.10161440714654 25.04195404 36.09129025674413 140.10161388090327 24.99821091 36.091281468849445 140.1015958707758 24.98280144</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_96c26909-a3cf-4b3f-9c04-123be0b0ca63">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_84338ce3-8938-48c2-acc5-2983e10b2a19">
											<gml:posList>36.09129025674413 140.10161388090327 24.99821091 36.091309666618756 140.10160125882473 24.96043396 36.09130062619289 140.10158203319028 24.95639801 36.09129025674413 140.10161388090327 24.99821091</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2b514e2f-0a80-44f5-894b-45df9ebfd941">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e67b8a6b-55c8-43ec-a1a2-87dc5b37bab8">
											<gml:posList>36.091309666618756 140.10160125882473 24.96043396 36.09129025674413 140.10161388090327 24.99821091 36.09129076490993 140.1016149237075 24.99909973 36.091309666618756 140.10160125882473 24.96043396</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_942995b3-ede7-4d3c-9427-8a0e2d54b855">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_90e2133c-cf90-4c7e-9f0d-58362894d30d">
											<gml:posList>36.09125362238905 140.10161440714654 25.04195404 36.091281468849445 140.1015958707758 24.98280144 36.091280960683484 140.10159482797184 24.98251343 36.09125362238905 140.10161440714654 25.04195404</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_a2f1116b-168e-4f4e-a55a-71ff75026511">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_bdf73c86-cc3c-4545-8d4b-8066b51947c4">
											<gml:posList>36.09117158113415 140.10167307920358 25.21131706 36.09117784285245 140.1016832727144 25.20619202 36.09119842983949 140.10167954707129 25.16604805 36.09117158113415 140.10167307920358 25.21131706</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_cfe834af-6e28-41eb-9eaa-56b56f1b5707">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_d56b8209-a771-46e4-850c-88b0b1109983">
											<gml:posList>36.09117166335038 140.10168748108532 25.20619202 36.09117784285245 140.1016832727144 25.20619202 36.09117158113415 140.10167307920358 25.21131706 36.09117166335038 140.10168748108532 25.20619202</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_b5259bc5-0d6e-427e-a89c-7337c55efc7e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_a3bf22b4-8f51-43ed-b5ac-db00dbd4478f">
											<gml:posList>36.09118123223459 140.10169083206318 25.20619202 36.0911750527325 140.10169504032262 25.20619202 36.091209815346275 140.10169708884578 25.16922188 36.09118123223459 140.10169083206318 25.20619202</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f9f91a01-3253-4da5-abde-3de8cc89e4be">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f869ab60-f32d-4344-9cc0-d68a458b9579">
											<gml:posList>36.09118123223459 140.10169083206318 25.20619202 36.091209815346275 140.10169708884578 25.16922188 36.09119842983949 140.10167954707129 25.16604805 36.09118123223459 140.10169083206318 25.20619202</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_5f4017c6-d0fd-41d0-8327-f4c53c287684">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_b7fd6055-b471-4d9d-a808-61dc2007b541">
											<gml:posList>36.09117784285245 140.1016832727144 25.20619202 36.09118123223459 140.10169083206318 25.20619202 36.09119842983949 140.10167954707129 25.16604805 36.09117784285245 140.1016832727144 25.20619202</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_bb13e668-dcb5-417e-9503-9a0491472189">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e03e9021-6020-4e98-9c0c-eba4b78dbdc3">
											<gml:posList>36.09118472135596 140.10171671890626 25.20115471 36.09119933598547 140.10170729052464 25.20115471 36.091209815346275 140.10169708884578 25.16922188 36.09118472135596 140.10171671890626 25.20115471</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_87599afa-fa88-4451-8c82-7b6ef7aec9fb">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_38b09aa3-279b-4e02-a9ba-f32594588d55">
											<gml:posList>36.09118415216643 140.10171543736402 25.20115471 36.09118472135596 140.10171671890626 25.20115471 36.091209815346275 140.10169708884578 25.16922188 36.09118415216643 140.10171543736402 25.20115471</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_4711fc82-9c59-4a45-8fce-a660dd4198ac">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3d9908a9-fce3-40d0-a852-02eeee6cd93f">
											<gml:posList>36.09119933598547 140.10170729052464 25.20115471 36.09120142937852 140.10171183062533 25.17891121 36.091209815346275 140.10169708884578 25.16922188 36.09119933598547 140.10170729052464 25.20115471</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2a6eed36-43a8-414a-b835-d4be0df7ff88">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_fc74572a-b0a0-455d-a3b3-7758d18735b3">
											<gml:posList>36.09136737675596 140.1015595515123 24.84247589 36.09138870671137 140.10155029769933 24.80227089 36.0913866135137 140.10154562733305 24.80430794 36.09136737675596 140.1015595515123 24.84247589</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d3529f6e-7730-44f2-aa25-a48ebbf16f0b">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e88fc9d5-3537-4dca-b482-5d3a60468874">
											<gml:posList>36.09135858312723 140.10154015324957 24.83866882 36.09138240085487 140.10153622165973 24.80320358 36.09137790794659 140.1015261860574 24.81175232 36.09135858312723 140.10154015324957 24.83866882</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod3MultiSurface>
				</tran:TrafficArea>
			</tran:trafficArea>
			<tran:trafficArea>
				<tran:TrafficArea gml:id="tran_682cb44e-2085-4974-b62a-54cf8a694b40">
					<tran:function codeSpace="../../codelists/TrafficArea_function.xml">1010</tran:function>
					<tran:lod3MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d66dbf41-987c-4177-b5bb-c79daba09825">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_45c569b4-1e47-499a-b188-f9a396c34643">
											<gml:posList>36.09142465764908 140.10152478320765 24.8629818 36.0913866135137 140.10154562733305 24.80430794 36.09138870671137 140.10155029769933 24.80227089 36.09142465764908 140.10152478320765 24.8629818</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_156b5b4c-50c3-4bb3-91ae-70fbb5a2aa34">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_5bea01ca-b7fe-4c9a-969e-f2f68095427b">
											<gml:posList>36.09140047210315 140.1015239202605 24.83371353 36.0913866135137 140.10154562733305 24.80430794 36.09142465764908 140.10152478320765 24.8629818 36.09140047210315 140.1015239202605 24.83371353</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_1c506cd7-36d6-46a8-b71b-f52af9261442">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_63f3e130-ca40-4a17-b75c-c5371f5d7fac">
											<gml:posList>36.0914044315369 140.1014812711379 24.87598038 36.09137673432996 140.10152357931878 24.81012917 36.0913947350398 140.10151134275222 24.83448982 36.0914044315369 140.1014812711379 24.87598038</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_e0e29d5c-088d-4c02-bb18-713f4ebc1312">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3b93b51d-1d79-4279-b384-046f4340bcec">
											<gml:posList>36.0913947350398 140.10151134275222 24.83448982 36.09137673432996 140.10152357931878 24.81012917 36.09137790794659 140.1015261860574 24.81175232 36.0913947350398 140.10151134275222 24.83448982</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_a981ec94-5429-4d90-8acf-90d9df8da92b">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_bd2ded83-b267-4dff-ac08-c31d3245c8d8">
											<gml:posList>36.091412709381935 140.101499127745 24.86981201 36.0913947350398 140.10151134275222 24.83448982 36.09140047210315 140.1015239202605 24.83371353 36.091412709381935 140.101499127745 24.86981201</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_be1d38dc-9793-431d-9fe5-f7712cdaffd1">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_4e84bd33-c2e3-4fa1-a7b2-01777cebdf36">
											<gml:posList>36.09140619225442 140.1014850510986 24.87500572 36.0914044315369 140.1014812711379 24.87598038 36.0913947350398 140.10151134275222 24.83448982 36.09140619225442 140.1014850510986 24.87500572</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_1e9cc91b-79ca-4f61-948d-8cce0911f61c">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_022ac6df-a07f-4915-ad0e-2fc881e426ef">
											<gml:posList>36.09140619225442 140.1014850510986 24.87500572 36.0913947350398 140.10151134275222 24.83448982 36.091412709381935 140.101499127745 24.86981201 36.09140619225442 140.1014850510986 24.87500572</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_7ba7a630-a738-4792-804f-5a637116ffc3">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_497e1962-5120-49f8-a4bd-f9c3d243288f">
											<gml:posList>36.09137790794659 140.1015261860574 24.81175232 36.09140047210315 140.1015239202605 24.83371353 36.0913947350398 140.10151134275222 24.83448982 36.09137790794659 140.1015261860574 24.81175232</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d159ceee-ded7-449a-8c27-0c4689267618">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e4e352e2-1ae2-4fbe-a5c1-1b062c9a0f6b">
											<gml:posList>36.091369193682404 140.10150672299102 24.81363487 36.09137673432996 140.10152357931878 24.81012917 36.0914044315369 140.1014812711379 24.87598038 36.091369193682404 140.10150672299102 24.81363487</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_85279719-ea8f-4545-93e7-08b3fc9b6976">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_72637e71-ff20-464f-8968-c97d0369bf62">
											<gml:posList>36.091418543391825 140.1015116405102 24.86422348 36.09140047210315 140.1015239202605 24.83371353 36.09142465764908 140.10152478320765 24.8629818 36.091418543391825 140.1015116405102 24.86422348</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_525bfff6-e3e4-4cce-9a72-cd9b9c094e11">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_625893f2-684d-4f9f-bd0f-aed982b8065f">
											<gml:posList>36.09140047210315 140.1015239202605 24.83371353 36.09137790794659 140.1015261860574 24.81175232 36.09138240085487 140.10153622165973 24.80320358 36.09140047210315 140.1015239202605 24.83371353</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_a9d70515-2c1d-4f4d-82f3-01a989dd2b98">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_12dcc8b6-c5a9-4411-8179-6f079f69b1b0">
											<gml:posList>36.09140047210315 140.1015239202605 24.83371353 36.091418543391825 140.1015116405102 24.86422348 36.091412709381935 140.101499127745 24.86981201 36.09140047210315 140.1015239202605 24.83371353</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_27af8fa7-f71c-47ca-a3fa-b465d81ac89b">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_13210c36-77a3-4cb4-898f-14e7fe619a5a">
											<gml:posList>36.09140047210315 140.1015239202605 24.83371353 36.09138240085487 140.10153622165973 24.80320358 36.0913866135137 140.10154562733305 24.80430794 36.09140047210315 140.1015239202605 24.83371353</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod3MultiSurface>
				</tran:TrafficArea>
			</tran:trafficArea>
			<tran:trafficArea>
				<tran:TrafficArea gml:id="tran_546487dc-d1d3-49c1-a42c-88a4b250ac20">
					<tran:function codeSpace="../../codelists/TrafficArea_function.xml">2000</tran:function>
					<tran:lod3MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_dc5784f5-1a90-47db-8328-af91b99a00cd">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ef2632bf-aa88-49b7-afd6-cacad4167f0c">
											<gml:posList>36.09142989814035 140.10150358909448 24.91503334 36.091418543391825 140.1015116405102 24.86422348 36.0914264737072 140.10152348800082 24.86886215 36.09142989814035 140.10150358909448 24.91503334</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_eead2558-67c8-401c-88c4-4df538ab99b1">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_0e84dbec-5eaf-4225-9795-4d5588214be2">
											<gml:posList>36.091418543391825 140.1015116405102 24.86422348 36.09142989814035 140.10150358909448 24.91503334 36.09142405524843 140.1014910979523 24.91890526 36.091418543391825 140.1015116405102 24.86422348</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_c18b2251-fb21-427a-967b-5892c28ad8f0">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9dd9fa61-9c47-4a62-93ae-ec93b78d7c70">
											<gml:posList>36.091412709381935 140.101499127745 24.86981201 36.09142405524843 140.1014910979523 24.91890526 36.09141752913487 140.10147712965727 24.91936493 36.091412709381935 140.101499127745 24.86981201</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f53af968-300e-4a59-8ad5-dacf934b8ae8">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f2f99570-2ce6-45d4-9404-62b932429100">
											<gml:posList>36.091412709381935 140.101499127745 24.86981201 36.09141752913487 140.10147712965727 24.91936493 36.09140619225442 140.1014850510986 24.87500572 36.091412709381935 140.101499127745 24.86981201</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_16e605d7-d549-4a0a-a244-f2296bb113f3">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e0ee6484-89d8-4c1a-869f-ce0a85e3746a">
											<gml:posList>36.0914264737072 140.10152348800082 24.86886215 36.09143603890374 140.10151668846328 24.9071331 36.09142989814035 140.10150358909448 24.91503334 36.0914264737072 140.10152348800082 24.86886215</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_67c2944b-76a7-4d55-b517-6797b2ae5cd6">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_0187fcac-b9b1-4128-a50c-b8d69861c4c3">
											<gml:posList>36.0914264737072 140.10152348800082 24.86886215 36.091418543391825 140.1015116405102 24.86422348 36.09142465764908 140.10152478320765 24.8629818 36.0914264737072 140.10152348800082 24.86886215</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_b5313f96-2470-4e68-89e0-1866bfb1e880">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_381422af-eccc-4f9c-89bd-9b1e641febd6">
											<gml:posList>36.091412709381935 140.101499127745 24.86981201 36.091418543391825 140.1015116405102 24.86422348 36.09142405524843 140.1014910979523 24.91890526 36.091412709381935 140.101499127745 24.86981201</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod3MultiSurface>
				</tran:TrafficArea>
			</tran:trafficArea>
			<tran:auxiliaryTrafficArea>
				<tran:AuxiliaryTrafficArea gml:id="tran_3b28a7b2-a741-4569-bf09-0dadaf5996f4">
					<tran:function codeSpace="../../codelists/AuxiliaryTrafficArea_function.xml">1100</tran:function>
					<tran:lod3MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_08d67e36-31d5-417b-825b-6f8647912a10">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f86dccf7-347e-454b-b188-4f3209f94491">
											<gml:posList>36.09116230360817 140.10160251482736 25.22502136 36.0911776944642 140.10159230817487 25.21017265 36.09117712518321 140.10159102663337 25.21017265 36.09116230360817 140.10160251482736 25.22502136</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_77f4881e-9e65-4654-b2a9-2050584be6d2">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_a08cfd43-e669-4595-a280-09f7bfdae68f">
											<gml:posList>36.09118616090991 140.10156910790522 25.18362999 36.09117712518321 140.10159102663337 25.21017265 36.09118672140661 140.10157036787356 25.18362999 36.09118616090991 140.10156910790522 25.18362999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_fc45a2a7-7d66-491c-b7dc-a6e719933aa4">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_124b0437-836f-4fdc-a6b7-fd29266236f1">
											<gml:posList>36.09114034324386 140.10161814305755 25.22986603 36.09113849718794 140.10161709570855 25.23027992 36.09113930302454 140.10161887699232 25.22941208 36.09114034324386 140.10161814305755 25.22986603</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_1b10b40c-89f4-440b-819a-4d2559d5d553">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2b06bff0-2aef-4c63-93a7-2d175c159b96">
											<gml:posList>36.09117712518321 140.10159102663337 25.21017265 36.09116174311155 140.10160125497026 25.22502136 36.09116230360817 140.10160251482736 25.22502136 36.09117712518321 140.10159102663337 25.21017265</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_f270c6c8-64db-4808-9c6b-fe5c87160af1">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_423a5e5b-dd50-4413-8a26-77b980bbf434">
											<gml:posList>36.091162391228174 140.1016027103476 25.17502022 36.09116230360817 140.10160251482736 25.22502136 36.09114034324386 140.10161814305755 25.22986603 36.091162391228174 140.1016027103476 25.17502022</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3a94a04f-7473-4867-8a0a-b83a300661cc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_81dd46f8-bad9-4699-96d5-b694a9c9316b">
											<gml:posList>36.09116230360817 140.10160251482736 25.22502136 36.091162391228174 140.1016027103476 25.17502022 36.0911776944642 140.10159230817487 25.21017265 36.09116230360817 140.10160251482736 25.22502136</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d750931a-6d61-410b-90ea-edc3bc7f945e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f3c6e597-d5f9-4717-856a-9f379c8f5bea">
											<gml:posList>36.09117712518321 140.10159102663337 25.21017265 36.0911776944642 140.10159230817487 25.21017265 36.09118672140661 140.10157036787356 25.18362999 36.09117712518321 140.10159102663337 25.21017265</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_31645971-31a9-4691-b059-cdac16710365">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2ecb903a-f282-4ef1-a0aa-776f8cb06421">
											<gml:posList>36.091178304187736 140.1015912258623 25.16076469 36.0911776944642 140.10159230817487 25.21017265 36.0911777819941 140.10159250369486 25.16017151 36.091178304187736 140.1015912258623 25.16076469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d2dfc46b-e8d5-45b9-ad40-8100cc525f76">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_b96c7f66-e533-467b-ba1c-bd1c2cad5837">
											<gml:posList>36.0911776944642 140.10159230817487 25.21017265 36.091162391228174 140.1016027103476 25.17502022 36.0911777819941 140.10159250369486 25.16017151 36.0911776944642 140.10159230817487 25.21017265</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_cb2497b5-1f1c-410c-a1f4-c0e1dfba52d0">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2a4150a2-c9c2-4f76-a881-d006e561d566">
											<gml:posList>36.09114034324386 140.10161814305755 25.22986603 36.091140439606455 140.10161833860747 25.17980766 36.091162391228174 140.1016027103476 25.17502022 36.09114034324386 140.10161814305755 25.22986603</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_0a703ede-b162-4e3b-a5f4-1df8a1359f37">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_a9e06e47-5a46-4a8e-aa05-2eca0a862ab7">
											<gml:posList>36.09116174311155 140.10160125497026 25.22502136 36.09113849718794 140.10161709570855 25.23027992 36.09114034324386 140.10161814305755 25.22986603 36.09116174311155 140.10160125497026 25.22502136</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_db19b251-1ade-4825-937e-5a9916f243ab">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_d1249afc-3bd0-4b8f-a041-843a49bc21c5">
											<gml:posList>36.09113930302454 140.10161887699232 25.22941208 36.091139390603125 140.101619050747 25.17941093 36.09114034324386 140.10161814305755 25.22986603 36.09113930302454 140.10161887699232 25.22941208</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_4c0a8226-7200-4b85-b7b5-800d75659428">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9768232e-8584-482b-a4a6-052f66ef95ae">
											<gml:posList>36.091139390603125 140.101619050747 25.17941093 36.091140439606455 140.10161833860747 25.17980766 36.09114034324386 140.10161814305755 25.22986603 36.091139390603125 140.101619050747 25.17941093</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_019dbdef-3f7c-459e-8de7-338dc5e205cf">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2e3f78fd-a29a-4570-89bf-6c0c8c566a49">
											<gml:posList>36.09116230360817 140.10160251482736 25.22502136 36.09116174311155 140.10160125497026 25.22502136 36.09114034324386 140.10161814305755 25.22986603 36.09116230360817 140.10160251482736 25.22502136</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_84ac201f-0df5-4981-8869-02182d1f246b">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_5168dca6-84b5-4eb6-be98-c5c0b0b34ee3">
											<gml:posList>36.091178304187736 140.1015912258623 25.16076469 36.09118680902693 140.10157056328276 25.13362885 36.09118672140661 140.10157036787356 25.18362999 36.091178304187736 140.1015912258623 25.16076469</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_c9311f9f-8ade-4f69-865c-8aae3439a4b1">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_7959e1c7-e4fa-4359-8d82-c29a40e7bead">
											<gml:posList>36.0911776944642 140.10159230817487 25.21017265 36.091178304187736 140.1015912258623 25.16076469 36.09118672140661 140.10157036787356 25.18362999 36.0911776944642 140.10159230817487 25.21017265</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod3MultiSurface>
				</tran:AuxiliaryTrafficArea>
			</tran:auxiliaryTrafficArea>
			<tran:auxiliaryTrafficArea>
				<tran:AuxiliaryTrafficArea gml:id="tran_8a8270ea-3e6a-491a-b98f-b2fd6869d3be">
					<tran:function codeSpace="../../codelists/AuxiliaryTrafficArea_function.xml">1100</tran:function>
					<tran:lod3MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_603fabf3-c263-4667-a179-7db8057046b3">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_a4298c7a-b3a2-4f79-8de8-864e8ff53065">
											<gml:posList>36.09135408449296 140.1015759249549 24.91423988 36.09135448459525 140.10157414787545 24.91423988 36.09132372735221 140.10159542882116 24.98269081 36.09135408449296 140.1015759249549 24.91423988</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_717851b1-d5ef-43e0-9f15-273f22de5a48">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3f8e0d28-a30c-4240-9b51-23f4df1fc5b1">
											<gml:posList>36.09132363982232 140.10159523330086 24.93269157 36.09135448459525 140.10157414787545 24.91423988 36.09135455566663 140.10157384439975 24.86423874 36.09132363982232 140.10159523330086 24.93269157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_cd073381-fa72-4799-a529-d5de1ca0b11b">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_0313cb0c-3a65-4ad3-9345-5956064d4c82">
											<gml:posList>36.0913803428504 140.10157482129816 24.87458992 36.09136099491873 140.1015873135673 24.875494 36.0913804216377 140.1015750167889 24.92458916 36.0913803428504 140.10157482129816 24.87458992</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_6f0328d5-7413-41b5-80cf-1154ba4f99da">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_73624be1-4bfe-4832-8720-a3af8a8fbfd6">
											<gml:posList>36.09135455566663 140.10157384439975 24.86423874 36.09135448459525 140.10157414787545 24.91423988 36.09136099491873 140.1015873135673 24.875494 36.09135455566663 140.10157384439975 24.86423874</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_075ffd22-1ed7-4661-8ac2-c28cafdb2caa">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9aa8d24e-8c7a-444d-bd68-657eeed3297a">
											<gml:posList>36.09136094161706 140.10158753026357 24.92549515 36.0913804216377 140.1015750167889 24.92458916 36.09136099491873 140.1015873135673 24.875494 36.09136094161706 140.10158753026357 24.92549515</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_feea300a-c118-41b3-a939-8a2259caf9d8">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c96d9950-5d74-4cdf-b5d9-5edd0fcce565">
											<gml:posList>36.09138095567775 140.10157629832491 24.92458534 36.0913804216377 140.1015750167889 24.92458916 36.09136094161706 140.10158753026357 24.92549515 36.09138095567775 140.10157629832491 24.92458534</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_788fb896-faa5-48e7-bec8-12c516b3acfc">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_b45815f4-a102-4f1f-9088-52ba4c02f0d6">
											<gml:posList>36.09135448459525 140.10157414787545 24.91423988 36.09136094161706 140.10158753026357 24.92549515 36.09136099491873 140.1015873135673 24.875494 36.09135448459525 140.10157414787545 24.91423988</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_6fd7b288-105e-4ca9-a188-4f1442086678">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9afc8464-1e5d-41cd-82b8-eebf95af793c">
											<gml:posList>36.09135408449296 140.1015759249549 24.91423988 36.09136047986538 140.10158930724467 24.92549515 36.09136094161706 140.10158753026357 24.92549515 36.09135408449296 140.1015759249549 24.91423988</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_57afa94b-d7ff-4dac-9c2d-90f3e9a0af90">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_acdbb446-5d43-48ca-977e-9dfde45d5104">
											<gml:posList>36.091324296681606 140.10159668871086 24.98269081 36.09132372735221 140.10159542882116 24.98269081 36.091292017846094 140.10161748729433 25.05114365 36.091324296681606 140.10159668871086 24.98269081</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_445b7386-f659-46f2-9362-9cb2305e4441">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f0e4a0ca-2cfb-43d4-bcaf-b1a6e4aa4208">
											<gml:posList>36.09132372735221 140.10159542882116 24.98269081 36.09132363982232 140.10159523330086 24.93269157 36.091291930225864 140.1016172918848 25.0011425 36.09132372735221 140.10159542882116 24.98269081</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2c721732-2d83-4222-8b7c-7dcac5e93254">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_5e5ba6c4-732d-44b3-9e71-32bf233b7dc3">
											<gml:posList>36.091291930225864 140.1016172918848 25.0011425 36.091292017846094 140.10161748729433 25.05114365 36.09132372735221 140.10159542882116 24.98269081 36.091291930225864 140.1016172918848 25.0011425</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_8972b1fb-3f12-4386-9dc3-9d96ad085e5a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_30704d06-6fe1-4426-b6e2-31b47c0d8510">
											<gml:posList>36.091324296681606 140.10159668871086 24.98269081 36.09135408449296 140.1015759249549 24.91423988 36.09132372735221 140.10159542882116 24.98269081 36.091324296681606 140.10159668871086 24.98269081</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_303ccad5-e3aa-49fe-8bbe-a449925279af">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_e3d5c185-72f2-47df-8c22-59f9af23e999">
											<gml:posList>36.09138095567775 140.10157629832491 24.92458534 36.09136094161706 140.10158753026357 24.92549515 36.09136047986538 140.10158930724467 24.92549515 36.09138095567775 140.10157629832491 24.92458534</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_82745f2f-b34b-4cbb-b022-a31376b61a81">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_70dc53ff-13e5-47bc-9ce2-d878ec2c2dfa">
											<gml:posList>36.09132363982232 140.10159523330086 24.93269157 36.09132372735221 140.10159542882116 24.98269081 36.09135448459525 140.10157414787545 24.91423988 36.09132363982232 140.10159523330086 24.93269157</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_0675b7eb-e04e-44e5-b9d1-a301f98670cf">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ec6c2126-39eb-444c-bf49-b036d892f037">
											<gml:posList>36.091292017846094 140.10161748729433 25.05114365 36.09129258708513 140.10161874718332 25.05114365 36.091324296681606 140.10159668871086 24.98269081 36.091292017846094 140.10161748729433 25.05114365</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_43a762c4-7cfd-4a7c-adaa-2d0fb51d7c8b">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_d3ca6c9f-6701-4682-b4b3-5e9104149a98">
											<gml:posList>36.09135408449296 140.1015759249549 24.91423988 36.09136094161706 140.10158753026357 24.92549515 36.09135448459525 140.10157414787545 24.91423988 36.09135408449296 140.1015759249549 24.91423988</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod3MultiSurface>
				</tran:AuxiliaryTrafficArea>
			</tran:auxiliaryTrafficArea>
			<tran:auxiliaryTrafficArea>
				<tran:AuxiliaryTrafficArea gml:id="tran_4d448e8a-db1d-48ef-8f04-feb24b49b701">
					<tran:function codeSpace="../../codelists/AuxiliaryTrafficArea_function.xml">1100</tran:function>
					<tran:lod3MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2d7d02c5-e1e9-46d6-b185-632a3843911e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_92075740-c22c-4a4c-9639-3a50dda9657f">
											<gml:posList>36.09141752913487 140.10147712965727 24.91936493 36.09141728291345 140.10147704209209 24.92842484 36.091406263415706 140.10148474762295 24.88406563 36.09141752913487 140.10147712965727 24.91936493</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_a87204ea-b5f2-4140-a1ea-4949cd29e830">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c044936f-5ca8-45ca-af7d-57b6c0403984">
											<gml:posList>36.09141574259118 140.10147308942206 24.92325592 36.09141728291345 140.10147704209209 24.92842484 36.09141752913487 140.10147712965727 24.91936493 36.09141574259118 140.10147308942206 24.92325592</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_e1685670-0047-4cae-9a71-96f67995f415">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f56ebc54-8079-461c-a47e-f79970640bff">
											<gml:posList>36.091406263415706 140.10148474762295 24.88406563 36.09140619225442 140.1014850510986 24.87500572 36.09141752913487 140.10147712965727 24.91936493 36.091406263415706 140.10148474762295 24.88406563</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d10c0210-4156-41f8-8dd2-6306f90cd404">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_22c2130a-d016-4e1f-8822-b674050a4715">
											<gml:posList>36.09141574259118 140.10147308942206 24.92325592 36.091415496369756 140.10147300185693 24.93182755 36.09141728291345 140.10147704209209 24.92842484 36.09141574259118 140.10147308942206 24.92325592</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_199f10ad-28de-435e-aca7-7a13cca30dd0">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_9999adbf-9913-40a8-8141-aaf91252b17c">
											<gml:posList>36.09140635342101 140.1014798895611 24.8860302 36.09140558646299 140.10148042920525 24.88506699 36.091406690222996 140.10148279706388 24.88406563 36.09140635342101 140.1014798895611 24.8860302</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3366410e-ccfc-45ba-bf31-802856b1f89d">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_72274603-32f3-4d29-b097-fbb84e1bacb1">
											<gml:posList>36.09141472776188 140.1014742788571 24.93059921 36.09140635342101 140.1014798895611 24.8860302 36.0914157085638 140.10147649449507 24.93059921 36.09141472776188 140.1014742788571 24.93059921</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_57484b24-9260-4d5b-92bb-b369acf2c6d4">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_d30aaeb3-61a8-45b1-ad8a-67f2cde9d7e7">
											<gml:posList>36.091406690222996 140.10148279706388 24.88406563 36.091406263415706 140.10148474762295 24.88406563 36.09141728291345 140.10147704209209 24.92842484 36.091406690222996 140.10148279706388 24.88406563</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_c80fcea7-2477-4786-9f31-58c022614906">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_12ad05c6-3e27-4cfd-867e-0535a292b973">
											<gml:posList>36.09140558646299 140.10148042920525 24.88506699 36.09140457265264 140.101481163234 24.88529778 36.091406690222996 140.10148279706388 24.88406563 36.09140558646299 140.10148042920525 24.88506699</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_a4f1cde8-416b-4a64-ae72-bee1883a04c2">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ecfa5d6b-d83f-4c8f-b502-6db55ddd451b">
											<gml:posList>36.09140457265264 140.101481163234 24.88529778 36.091406263415706 140.10148474762295 24.88406563 36.091406690222996 140.10148279706388 24.88406563 36.09140457265264 140.101481163234 24.88529778</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_28cf1d74-4a7e-4067-8b90-fa3d07428d18">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_10d0e453-ebe9-40ef-91c3-08244b918041">
											<gml:posList>36.0914044315369 140.1014812711379 24.87598038 36.09140619225442 140.1014850510986 24.87500572 36.091406263415706 140.10148474762295 24.88406563 36.0914044315369 140.1014812711379 24.87598038</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3585dcdc-65b7-4804-ae1f-33c41e76ae2d">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_0c63fdc9-8df0-4903-8e53-ea67bdcd9f8f">
											<gml:posList>36.0914157085638 140.10147649449507 24.93059921 36.091406690222996 140.10148279706388 24.88406563 36.09141728291345 140.10147704209209 24.92842484 36.0914157085638 140.10147649449507 24.93059921</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_92ba41db-fd08-42ff-8be3-cb2420d6a7b0">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6766538a-229b-48f9-af23-a6d7a844ea30">
											<gml:posList>36.0914044315369 140.1014812711379 24.87598038 36.091406263415706 140.10148474762295 24.88406563 36.09140457265264 140.101481163234 24.88529778 36.0914044315369 140.1014812711379 24.87598038</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3e3a21d2-f812-4e79-8b40-ac9c10e0ad8a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_595318b5-8a9e-4373-90fb-312daf77e96a">
											<gml:posList>36.0914157085638 140.10147649449507 24.93059921 36.09140635342101 140.1014798895611 24.8860302 36.091406690222996 140.10148279706388 24.88406563 36.0914157085638 140.10147649449507 24.93059921</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_62cb6bb9-778a-4cc4-a0b0-f290163e2dc7">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6b30c2bb-dc4b-481a-8967-3bd0f1eb7b17">
											<gml:posList>36.091413904541895 140.1014724107807 24.93059921 36.09141472776188 140.1014742788571 24.93059921 36.091415496369756 140.10147300185693 24.93182755 36.091413904541895 140.1014724107807 24.93059921</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_09bb6567-6f85-4452-963d-a6ff6bc4def8">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_65a67a46-3609-4c2d-bd28-28f4eff94d98">
											<gml:posList>36.091413904541895 140.1014724107807 24.93059921 36.091415496369756 140.10147300185693 24.93182755 36.09141490946396 140.10147174179593 24.93182373 36.091413904541895 140.1014724107807 24.93059921</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_3b6e4435-13bd-4f86-81ba-4dafc8596cf3">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_71a8a466-0a2c-419c-9996-b25151eb2d60">
											<gml:posList>36.09141472776188 140.1014742788571 24.93059921 36.0914157085638 140.10147649449507 24.93059921 36.091415496369756 140.10147300185693 24.93182755 36.09141472776188 140.1014742788571 24.93059921</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_ac0313c5-fdb2-4690-889d-3b0ef28c6620">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_340750b8-18c6-42c3-9801-9684511d564e">
											<gml:posList>36.091415496369756 140.10147300185693 24.93182755 36.0914157085638 140.10147649449507 24.93059921 36.09141728291345 140.10147704209209 24.92842484 36.091415496369756 140.10147300185693 24.93182755</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod3MultiSurface>
				</tran:AuxiliaryTrafficArea>
			</tran:auxiliaryTrafficArea>
			<tran:auxiliaryTrafficArea>
				<tran:AuxiliaryTrafficArea gml:id="tran_ddf91fb3-b1db-4bdb-91d9-ae67ba146e62">
					<tran:function codeSpace="../../codelists/AuxiliaryTrafficArea_function.xml">1100</tran:function>
					<tran:lod3MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_fa840ab9-961f-45d9-bb70-86086d8c71d8">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c83755b9-6261-40df-9bd9-dbe3f3a74d6b">
											<gml:posList>36.091280848263466 140.10157031891342 24.97704887 36.09127218147978 140.10157683964363 24.98588371 36.091280935883525 140.10157051443392 24.96525574 36.091280848263466 140.10157031891342 24.97704887</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_eecc9e7b-c2d6-446e-8cb1-f2b4264e6b5c">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_bd9d74e5-24d7-4187-92fb-e819c4547415">
											<gml:posList>36.0913691061039 140.10150654912493 24.82542801 36.09134199666421 140.10152612912938 24.87570381 36.09134208419446 140.10152632453864 24.86391068 36.0913691061039 140.10150654912493 24.82542801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_585d1f61-1d4f-43b9-929f-640221fbf2aa">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_c2352ece-8b7d-44cb-ab40-308fec4bbca6">
											<gml:posList>36.0913691061039 140.10150654912493 24.82542801 36.09136851036547 140.10150528914548 24.82542801 36.09134140087752 140.10152489080448 24.87570381 36.0913691061039 140.10150654912493 24.82542801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_95d09f45-8c45-42c4-944a-e5a5ca86dfe6">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6e76b51a-1359-416f-bda2-a350b57b23ca">
											<gml:posList>36.09134140087752 140.10152489080448 24.87570381 36.0913142913863 140.10154449245005 24.92597961 36.09134199666421 140.10152612912938 24.87570381 36.09134140087752 140.10152489080448 24.87570381</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_319f825f-e372-4297-80b3-c3ee4ab74fc0">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ae0114be-024e-4638-97fa-4e1bdd6bb767">
											<gml:posList>36.0913142913863 140.10154449245005 24.92597961 36.091314887173034 140.10154573066364 24.92597961 36.09134199666421 140.10152612912938 24.87570381 36.0913142913863 140.10154449245005 24.92597961</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_dee19ebc-9421-432e-8534-dc08457cf76e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_1e28b8e3-f817-4367-a35d-f93e00e749ca">
											<gml:posList>36.0913691061039 140.10150654912493 24.82542801 36.09134140087752 140.10152489080448 24.87570381 36.09134199666421 140.10152612912938 24.87570381 36.0913691061039 140.10150654912493 24.82542801</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_8b2e8068-9274-4582-b73f-1614f5c058d6">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ceee0208-02a9-4d09-a5b1-991013a7be1e">
											<gml:posList>36.091314887173034 140.10154573066364 24.92597961 36.09128025247722 140.10156908058914 24.97704887 36.091280848263466 140.10157031891342 24.97704887 36.091314887173034 140.10154573066364 24.92597961</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_d8614724-393a-423b-8158-1f54586453f1">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_bca82883-052a-4ded-892f-ecbfd1d3e424">
											<gml:posList>36.09127218147978 140.10157683964363 24.98588371 36.091280848263466 140.10157031891342 24.97704887 36.09127209381126 140.10157666577734 24.99767494 36.09127218147978 140.10157683964363 24.98588371</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_2005af3a-6c0e-4604-a7df-64bee6f994d6">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_eee2aaa8-4f92-4421-b269-8493d2c06e28">
											<gml:posList>36.091271498163685 140.10157540579925 24.99767494 36.09127209381126 140.10157666577734 24.99767494 36.091280848263466 140.10157031891342 24.97704887 36.091271498163685 140.10157540579925 24.99767494</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_b7859070-6568-4910-a20d-bd4340652bae">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2acc4ff5-cf2b-4cb3-9eb5-4765a78e6901">
											<gml:posList>36.091271498163685 140.10157540579925 24.99767494 36.091280848263466 140.10157031891342 24.97704887 36.09128025247722 140.10156908058914 24.97704887 36.091271498163685 140.10157540579925 24.99767494</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_9c5a9154-5786-4539-90de-b45a991f6634">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_5b27ef17-cbab-4ba9-8c81-ad3ec659b94e">
											<gml:posList>36.091314887173034 140.10154573066364 24.92597961 36.0913142913863 140.10154449245005 24.92597961 36.09128025247722 140.10156908058914 24.97704887 36.091314887173034 140.10154573066364 24.92597961</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_5c350300-91ea-449d-87e6-5b30372132e3">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_2452a508-c3f7-4aad-84a8-5de9239d3acc">
											<gml:posList>36.09134208419446 140.10152632453864 24.86391068 36.091369193682404 140.10150672299102 24.81363487 36.0913691061039 140.10150654912493 24.82542801 36.09134208419446 140.10152632453864 24.86391068</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_0a7ec33c-cfa7-4606-9d3f-37036843f65f">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_ebfc5821-fcdf-4447-8565-3d5a1eb11727">
											<gml:posList>36.09131497470301 140.1015459261839 24.91418648 36.091280848263466 140.10157031891342 24.97704887 36.091280935883525 140.10157051443392 24.96525574 36.09131497470301 140.1015459261839 24.91418648</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_8d7e0065-6ff1-4e1a-a103-ced81cc4bb0a">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_fcc43074-6943-44f7-aa18-36252bf6bb64">
											<gml:posList>36.091314887173034 140.10154573066364 24.92597961 36.091280848263466 140.10157031891342 24.97704887 36.09131497470301 140.1015459261839 24.91418648 36.091314887173034 140.10154573066364 24.92597961</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_1ef5e34d-3c67-4621-8b8e-f69d3efe4da1">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_8d58b2b2-a1f5-434f-bbc9-2322e2e50b79">
											<gml:posList>36.09134208419446 140.10152632453864 24.86391068 36.09134199666421 140.10152612912938 24.87570381 36.091314887173034 140.10154573066364 24.92597961 36.09134208419446 140.10152632453864 24.86391068</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="pol_7f6de06b-a5ce-497e-b344-91ecd361aa1e">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_6efc678d-b91f-4c5e-a117-760bf30bef0a">
											<gml:posList>36.09131497470301 140.1015459261839 24.91418648 36.09134208419446 140.10152632453864 24.86391068 36.091314887173034 140.10154573066364 24.92597961 36.09131497470301 140.1015459261839 24.91418648</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod3MultiSurface>
				</tran:AuxiliaryTrafficArea>
			</tran:auxiliaryTrafficArea>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon gml:id="pol_edfc0129-1621-4aee-9295-68246a0f2859">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.09119196010192 140.1015814687185 0 36.09118680902693 140.10157056328276 0 36.09118672140661 140.10157036787356 0 36.09118616090991 140.10156910790522 0 36.09117712518321 140.10159102663337 0 36.09116174311155 140.10160125497026 0 36.09113849718794 140.10161709570855 0 36.09113930302454 140.10161887699232 0 36.091139390603125 140.101619050747 0 36.09114826358959 140.10163838416005 0 36.09114908700122 140.101640165504 0 36.09115714540538 140.10165771771847 0 36.09115872198955 140.10166117156348 0 36.091147050187004 140.1016692870366 0 36.091136471516975 140.10167662533345 0 36.091144234132784 140.10169263652924 0 36.09114639820232 140.1016970901381 0 36.09116882567956 140.10168115991112 0 36.09117166335038 140.10168748108532 0 36.0911750527325 140.10169504032262 0 36.09117547308328 140.10169597444437 0 36.09118415216643 140.10171543736402 0 36.09118472135596 140.10171671890626 0 36.09119933598547 140.10170729052464 0 36.09120142937852 140.10171183062533 0 36.091238123614644 140.10168460543187 0 36.09123548740448 140.1016787186914 0 36.09122740361519 140.10166071096256 0 36.09124901848716 140.1016461009752 0 36.09127063326743 140.1016314908685 0 36.091291930225864 140.1016172918848 0 36.091292017846094 140.10161748729433 0 36.09129258708513 140.10161874718332 0 36.091324296681606 140.10159668871086 0 36.09135408449296 140.1015759249549 0 36.09136047986538 140.10158930724467 0 36.09138095567775 140.10157629832491 0 36.0913804216377 140.1015750167889 0 36.0913803428504 140.10157482129816 0 36.09138224682043 140.1015735914663 0 36.091394922622946 140.10156517587117 0 36.09138870671137 140.10155029769933 0 36.09142465764908 140.10152478320765 0 36.0914264737072 140.10152348800082 0 36.09143603890374 140.10151668846328 0 36.09144791380706 140.10150827002158 0 36.09145165170068 140.10150561500234 0 36.09144568715436 140.10149242939048 0 36.09144004717161 140.1014797654782 0 36.09143355737228 140.10146527670952 0 36.091431779078526 140.1014614967993 0 36.09141574259118 140.10147308942206 0 36.091415496369756 140.10147300185693 0 36.09141490946396 140.10147174179593 0 36.091413904541895 140.1014724107807 0 36.09141472776188 140.1014742788571 0 36.09140635342101 140.1014798895611 0 36.09140558646299 140.10148042920525 0 36.09140457265264 140.101481163234 0 36.0914044315369 140.1014812711379 0 36.091369193682404 140.10150672299102 0 36.0913691061039 140.10150654912493 0 36.09136851036547 140.10150528914548 0 36.09134140087752 140.10152489080448 0 36.0913142913863 140.10154449245005 0 36.09128025247722 140.10156908058914 0 36.091271498163685 140.10157540579925 0 36.09127209381126 140.10157666577734 0 36.09127218147978 140.10157683964363 0 36.091260755849135 140.10158510766914 0 36.0912426828048 140.10159816815198 0 36.09121075975082 140.10162124529765 0 36.09121010274613 140.10161985496325 0 36.091201027033456 140.10160065099655 0 36.09119196010192 140.1015814687185 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<tran:lod3MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:CompositeSurface>
							<gml:surfaceMember xlink:href="#pol_08d67e36-31d5-417b-825b-6f8647912a10"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_77f4881e-9e65-4654-b2a9-2050584be6d2"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_fc45a2a7-7d66-491c-b7dc-a6e719933aa4"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_1b10b40c-89f4-440b-819a-4d2559d5d553"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f270c6c8-64db-4808-9c6b-fe5c87160af1"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3a94a04f-7473-4867-8a0a-b83a300661cc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d750931a-6d61-410b-90ea-edc3bc7f945e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_31645971-31a9-4691-b059-cdac16710365"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d2dfc46b-e8d5-45b9-ad40-8100cc525f76"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_cb2497b5-1f1c-410c-a1f4-c0e1dfba52d0"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_0a703ede-b162-4e3b-a5f4-1df8a1359f37"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_db19b251-1ade-4825-937e-5a9916f243ab"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_4c0a8226-7200-4b85-b7b5-800d75659428"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_019dbdef-3f7c-459e-8de7-338dc5e205cf"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_84ac201f-0df5-4981-8869-02182d1f246b"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_c9311f9f-8ade-4f69-865c-8aae3439a4b1"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_fa840ab9-961f-45d9-bb70-86086d8c71d8"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_eecc9e7b-c2d6-446e-8cb1-f2b4264e6b5c"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_585d1f61-1d4f-43b9-929f-640221fbf2aa"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_95d09f45-8c45-42c4-944a-e5a5ca86dfe6"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_319f825f-e372-4297-80b3-c3ee4ab74fc0"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_dee19ebc-9421-432e-8534-dc08457cf76e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_8b2e8068-9274-4582-b73f-1614f5c058d6"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d8614724-393a-423b-8158-1f54586453f1"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2005af3a-6c0e-4604-a7df-64bee6f994d6"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_b7859070-6568-4910-a20d-bd4340652bae"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_9c5a9154-5786-4539-90de-b45a991f6634"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_5c350300-91ea-449d-87e6-5b30372132e3"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_0a7ec33c-cfa7-4606-9d3f-37036843f65f"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_8d7e0065-6ff1-4e1a-a103-ced81cc4bb0a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_1ef5e34d-3c67-4621-8b8e-f69d3efe4da1"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_7f6de06b-a5ce-497e-b344-91ecd361aa1e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_603fabf3-c263-4667-a179-7db8057046b3"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_717851b1-d5ef-43e0-9f15-273f22de5a48"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_cd073381-fa72-4799-a529-d5de1ca0b11b"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_6f0328d5-7413-41b5-80cf-1154ba4f99da"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_075ffd22-1ed7-4661-8ac2-c28cafdb2caa"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_feea300a-c118-41b3-a939-8a2259caf9d8"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_788fb896-faa5-48e7-bec8-12c516b3acfc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_6fd7b288-105e-4ca9-a188-4f1442086678"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_57afa94b-d7ff-4dac-9c2d-90f3e9a0af90"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_445b7386-f659-46f2-9362-9cb2305e4441"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2c721732-2d83-4222-8b7c-7dcac5e93254"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_8972b1fb-3f12-4386-9dc3-9d96ad085e5a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_303ccad5-e3aa-49fe-8bbe-a449925279af"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_82745f2f-b34b-4cbb-b022-a31376b61a81"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_0675b7eb-e04e-44e5-b9d1-a301f98670cf"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_43a762c4-7cfd-4a7c-adaa-2d0fb51d7c8b"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2d7d02c5-e1e9-46d6-b185-632a3843911e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_a87204ea-b5f2-4140-a1ea-4949cd29e830"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_e1685670-0047-4cae-9a71-96f67995f415"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d10c0210-4156-41f8-8dd2-6306f90cd404"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_199f10ad-28de-435e-aca7-7a13cca30dd0"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3366410e-ccfc-45ba-bf31-802856b1f89d"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_57484b24-9260-4d5b-92bb-b369acf2c6d4"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_c80fcea7-2477-4786-9f31-58c022614906"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_a4f1cde8-416b-4a64-ae72-bee1883a04c2"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_28cf1d74-4a7e-4067-8b90-fa3d07428d18"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3585dcdc-65b7-4804-ae1f-33c41e76ae2d"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_92ba41db-fd08-42ff-8be3-cb2420d6a7b0"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3e3a21d2-f812-4e79-8b40-ac9c10e0ad8a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_62cb6bb9-778a-4cc4-a0b0-f290163e2dc7"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_09bb6567-6f85-4452-963d-a6ff6bc4def8"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3b6e4435-13bd-4f86-81ba-4dafc8596cf3"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_ac0313c5-fdb2-4690-889d-3b0ef28c6620"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d66dbf41-987c-4177-b5bb-c79daba09825"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_156b5b4c-50c3-4bb3-91ae-70fbb5a2aa34"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_1c506cd7-36d6-46a8-b71b-f52af9261442"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_e0e29d5c-088d-4c02-bb18-713f4ebc1312"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_a981ec94-5429-4d90-8acf-90d9df8da92b"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_be1d38dc-9793-431d-9fe5-f7712cdaffd1"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_1e9cc91b-79ca-4f61-948d-8cce0911f61c"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_7ba7a630-a738-4792-804f-5a637116ffc3"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d159ceee-ded7-449a-8c27-0c4689267618"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_85279719-ea8f-4545-93e7-08b3fc9b6976"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_525bfff6-e3e4-4cce-9a72-cd9b9c094e11"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_a9d70515-2c1d-4f4d-82f3-01a989dd2b98"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_27af8fa7-f71c-47ca-a3fa-b465d81ac89b"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_4b48ef55-2fb6-400b-8053-30841a072954"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_5f894980-ecef-4f9b-b64d-0982f2b0ff70"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_e20af9cd-7950-4d13-95c9-9cdba1803173"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_401376b3-ffff-4dc8-9c52-9e257fb85389"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_de73375f-5f33-42c9-baa1-ce567d79e4de"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f74f49b6-522a-4a27-a573-0746a3108ce7"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_6f6ef5ad-d410-4e1e-9796-c08f8a45c505"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_0b4934db-8566-43c8-8a3b-0a841bb11463"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_02ebd709-d551-472e-bcd1-9f89a064a57c"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_07d10b09-a1d1-4638-a212-8e2cc4fa8484"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_b6a7658f-d5f1-4491-a65c-eb111f21a4ef"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f0169b43-be98-4a04-bbdc-e2ddaf220c77"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_953e3ca7-e5ac-4545-a643-966eb7c04b86"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f955c045-bac9-4ef0-a284-e9ac8eaeed4d"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_fbd79cba-2ac5-4ade-8aac-8698d252d710"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_549dc592-c27a-4444-989b-da32dc1d96de"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_756da669-74ed-4d0d-b704-d73fd7b678aa"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_e033426e-368e-4777-86b8-53630fb92889"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_e77d1b67-a97a-4d5e-82ac-f2d4dc5f50e6"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_c5ef8a7c-28e0-4533-8311-e921aa4696a3"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_42367d28-6247-4ebf-8246-361370d6c662"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3cbeb25c-eed6-4709-803f-84fef17e73bc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_827996a0-7200-4163-8369-11db9518c922"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d42013ae-8aa3-4559-9d59-703df386c728"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_7974cee0-db54-4c81-a36e-06321b04971c"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_922b59b5-90e1-4e81-82a9-fda2e7b5585d"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_74a6f2a5-6041-473a-bc85-e4f0c7a8c24a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f69f6d0b-bd1b-4b01-998e-4bcc61ac1217"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_4e99e9e6-868d-493f-8dfe-6bcced39ecd6"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_9a82b3e2-46ec-4a63-892d-19aab2911965"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_123aac70-57eb-4f05-9f72-8ce19c470577"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_96817b57-2ff0-44a9-b707-ce6f7438c16c"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_06145d06-fb85-413f-8d8a-a02233d05ccc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_de5594d8-d4ed-440d-ad07-59596f3da1f4"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_780c84b6-ea2a-4d43-8857-8cb248cdb079"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_9fe9a4fd-c912-4687-91ec-f2d23135381a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_a34a6b66-34da-4b13-b611-0816af47e25e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d5548820-027d-4e90-b559-744ad8477ab0"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_4082bdfa-055a-48d9-ae7d-927cbeddf23d"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_57d76baf-432e-4e82-be86-ac4085d01a04"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_b11f0f4a-d226-402a-a591-82d960cfd3f3"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_5792e886-6b85-4193-89dd-30de9ae5d257"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d9f5cea2-1ce2-4af4-9682-20503865375d"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_ce534719-fbfb-4d50-9334-72c2a33f7569"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_54c06e5e-8b0c-464d-bc30-da16b311dbda"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_b48d257c-1d1a-4802-a83e-fc347a6dec08"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_4a93259c-538d-4431-975b-0e8323293425"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_ef1ee3ec-bd3b-4969-a5a2-5cf29502dce6"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_e0f6abc6-c0d3-46f5-ac00-c0df4a55d7da"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_8720812c-ae1f-49d9-91c5-6b7fc5354111"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_fafcfe27-57f9-4e3b-adfe-56665cfde0b7"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_765b3ccf-804a-4164-83ed-6a717cde6780"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d67fcdb2-858f-49c7-8f9b-84d70f81511a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_88a8affd-d5e2-46b4-86de-904090b7f368"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d46bdd0d-710b-438d-a6db-4d5fc7a83bad"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3e2766d3-6af0-425d-9c01-e1d608c8b83b"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_8aa582a7-c01e-4e43-9bbb-0cb8940cbf5f"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_5e1974bc-560a-4a15-94f5-6489022afd6e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2df9fda1-b8a7-454e-a433-f1eb26e002df"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_686d2ca8-8039-40f6-861c-ef59b9831d6d"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_66987187-2683-41a4-8447-6c04658910bb"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_356d875b-1c62-43fa-ba95-35d7a0afc2ae"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_072f2c5d-fe75-437e-bb2b-1b3c9b78d9fd"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_ec915550-684f-4f96-a6d6-2e642ca64fd8"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2bdc2c74-b19e-46d2-a8c0-fe7f240d6eb5"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_de6cd6c5-4554-45c0-81fa-1d80e7861ac8"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_c57e8586-0c2b-434f-b0e3-7bf318c8b36c"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_6b50beb8-911d-425b-8c54-eaab2a7f0b7a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3d37b5ef-99f3-4c47-b187-48c839916bee"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f9ffd753-d6e5-4539-93e6-d7481618f39e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_eb0c3ce2-8de1-4d70-9e04-c5c0a5f863b9"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f56dddfa-b33e-41fa-acef-43b08a2c148d"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_e17dd8ea-fe5d-443b-b7a9-8b7e48451921"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_ba234ed6-9a10-4648-8fea-24ac5f926dbe"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_9ccb0753-2ae0-433b-98d0-5648b95dbd1a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_0ee5a6db-bd96-4823-891e-d6d43c8027cc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2ae161b6-af24-4d77-9166-dfeaeb3f8bce"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_5cdf5318-a08a-4923-9325-5ac25fee867f"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_164aaac2-b726-4f28-a750-cf7fcd392297"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f71f3930-e64c-47ab-bf3d-3cd3b66dd517"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_3b5b4931-8ae0-44dd-94d4-22a8d473f321"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_7d59592f-c49b-4a9d-b693-fe756777397f"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2fa62caf-5581-44bb-8f4a-5864c3b59c00"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_a07b1b30-b2e0-4f29-bf5f-0fa426b3fe22"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f22d9750-840d-456f-83da-42c9e5e641fc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_e06b9d99-50e0-4630-8a0a-743a309536ae"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_7b2383fe-1be5-4af1-b2b6-73ac4e18bfab"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_5e5c89c2-38d5-4a90-9086-92a0361b5ad0"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_4c1f2f21-c6e6-48d6-a032-c014473bad5e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d0caaff3-2eb5-45c5-a9bc-ea9fa49fd674"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_973a2bfe-9695-4a11-bb49-19846832c0dc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_7ac7e793-9e74-48d4-975b-4becd3d792e5"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_7230b197-0b9b-4ebf-8dc9-05a9fb58befc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_c7ecae7d-ca42-485e-95c1-9fd2bce87d1a"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_81d03425-3d36-41cf-82b4-2d733cc48587"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_fc4b40cc-f329-40b9-ab89-9c12a41379d8"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_83d3467c-6313-49b8-bb88-b57923a1986f"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_447c03d9-76d9-42ea-b421-0b85b6b1e1cc"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_96c26909-a3cf-4b3f-9c04-123be0b0ca63"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2b514e2f-0a80-44f5-894b-45df9ebfd941"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_942995b3-ede7-4d3c-9427-8a0e2d54b855"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_a2f1116b-168e-4f4e-a55a-71ff75026511"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_cfe834af-6e28-41eb-9eaa-56b56f1b5707"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_b5259bc5-0d6e-427e-a89c-7337c55efc7e"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f9f91a01-3253-4da5-abde-3de8cc89e4be"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_5f4017c6-d0fd-41d0-8327-f4c53c287684"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_bb13e668-dcb5-417e-9503-9a0491472189"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_87599afa-fa88-4451-8c82-7b6ef7aec9fb"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_4711fc82-9c59-4a45-8fce-a660dd4198ac"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_2a6eed36-43a8-414a-b835-d4be0df7ff88"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_d3529f6e-7730-44f2-aa25-a48ebbf16f0b"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_dc5784f5-1a90-47db-8328-af91b99a00cd"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_eead2558-67c8-401c-88c4-4df538ab99b1"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_c18b2251-fb21-427a-967b-5892c28ad8f0"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_f53af968-300e-4a59-8ad5-dacf934b8ae8"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_16e605d7-d549-4a0a-a244-f2296bb113f3"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_67c2944b-76a7-4d55-b517-6797b2ae5cd6"></gml:surfaceMember>
							<gml:surfaceMember xlink:href="#pol_b5313f96-2470-4e68-89e0-1866bfb1e880"></gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod3MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">3</uro:appearanceSrcDescLod3>
					<uro:lodType codeSpace="../../codelists/Road_lodType.xml">3.3</uro:lodType>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod3>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">011</uro:publicSurveySrcDescLod3>
							<uro:publicSurveySrcDescLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">012</uro:publicSurveySrcDescLod3>
							<uro:publicSurveySrcDescLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod3>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">9</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
	<app:appearanceMember>
		<app:Appearance>
			<app:theme>rgbTexture</app:theme>
			<app:surfaceDataMember>
				<app:X3DMaterial>
					<app:diffuseColor>0.9804 0.9804 0.9804</app:diffuseColor>
					<app:shininess>0</app:shininess>
					<app:target>#pol_f0169b43-be98-4a04-bbdc-e2ddaf220c77</app:target>
					<app:target>#pol_953e3ca7-e5ac-4545-a643-966eb7c04b86</app:target>
					<app:target>#pol_f955c045-bac9-4ef0-a284-e9ac8eaeed4d</app:target>
					<app:target>#pol_fbd79cba-2ac5-4ade-8aac-8698d252d710</app:target>
					<app:target>#pol_549dc592-c27a-4444-989b-da32dc1d96de</app:target>
					<app:target>#pol_756da669-74ed-4d0d-b704-d73fd7b678aa</app:target>
					<app:target>#pol_e033426e-368e-4777-86b8-53630fb92889</app:target>
					<app:target>#pol_e77d1b67-a97a-4d5e-82ac-f2d4dc5f50e6</app:target>
					<app:target>#pol_c5ef8a7c-28e0-4533-8311-e921aa4696a3</app:target>
					<app:target>#pol_42367d28-6247-4ebf-8246-361370d6c662</app:target>
					<app:target>#pol_3cbeb25c-eed6-4709-803f-84fef17e73bc</app:target>
					<app:target>#pol_827996a0-7200-4163-8369-11db9518c922</app:target>
					<app:target>#pol_d42013ae-8aa3-4559-9d59-703df386c728</app:target>
					<app:target>#pol_7974cee0-db54-4c81-a36e-06321b04971c</app:target>
					<app:target>#pol_922b59b5-90e1-4e81-82a9-fda2e7b5585d</app:target>
					<app:target>#pol_74a6f2a5-6041-473a-bc85-e4f0c7a8c24a</app:target>
					<app:target>#pol_f69f6d0b-bd1b-4b01-998e-4bcc61ac1217</app:target>
					<app:target>#pol_4e99e9e6-868d-493f-8dfe-6bcced39ecd6</app:target>
					<app:target>#pol_9a82b3e2-46ec-4a63-892d-19aab2911965</app:target>
					<app:target>#pol_123aac70-57eb-4f05-9f72-8ce19c470577</app:target>
					<app:target>#pol_96817b57-2ff0-44a9-b707-ce6f7438c16c</app:target>
					<app:target>#pol_06145d06-fb85-413f-8d8a-a02233d05ccc</app:target>
					<app:target>#pol_de5594d8-d4ed-440d-ad07-59596f3da1f4</app:target>
					<app:target>#pol_780c84b6-ea2a-4d43-8857-8cb248cdb079</app:target>
					<app:target>#pol_9fe9a4fd-c912-4687-91ec-f2d23135381a</app:target>
					<app:target>#pol_a34a6b66-34da-4b13-b611-0816af47e25e</app:target>
					<app:target>#pol_d5548820-027d-4e90-b559-744ad8477ab0</app:target>
					<app:target>#pol_4082bdfa-055a-48d9-ae7d-927cbeddf23d</app:target>
					<app:target>#pol_57d76baf-432e-4e82-be86-ac4085d01a04</app:target>
					<app:target>#pol_b11f0f4a-d226-402a-a591-82d960cfd3f3</app:target>
					<app:target>#pol_5792e886-6b85-4193-89dd-30de9ae5d257</app:target>
					<app:target>#pol_d9f5cea2-1ce2-4af4-9682-20503865375d</app:target>
					<app:target>#pol_ce534719-fbfb-4d50-9334-72c2a33f7569</app:target>
					<app:target>#pol_54c06e5e-8b0c-464d-bc30-da16b311dbda</app:target>
					<app:target>#pol_b48d257c-1d1a-4802-a83e-fc347a6dec08</app:target>
					<app:target>#pol_4a93259c-538d-4431-975b-0e8323293425</app:target>
					<app:target>#pol_ef1ee3ec-bd3b-4969-a5a2-5cf29502dce6</app:target>
					<app:target>#pol_e0f6abc6-c0d3-46f5-ac00-c0df4a55d7da</app:target>
					<app:target>#pol_8720812c-ae1f-49d9-91c5-6b7fc5354111</app:target>
					<app:target>#pol_fafcfe27-57f9-4e3b-adfe-56665cfde0b7</app:target>
					<app:target>#pol_765b3ccf-804a-4164-83ed-6a717cde6780</app:target>
					<app:target>#pol_d67fcdb2-858f-49c7-8f9b-84d70f81511a</app:target>
					<app:target>#pol_88a8affd-d5e2-46b4-86de-904090b7f368</app:target>
					<app:target>#pol_d46bdd0d-710b-438d-a6db-4d5fc7a83bad</app:target>
					<app:target>#pol_3e2766d3-6af0-425d-9c01-e1d608c8b83b</app:target>
					<app:target>#pol_8aa582a7-c01e-4e43-9bbb-0cb8940cbf5f</app:target>
					<app:target>#pol_5e1974bc-560a-4a15-94f5-6489022afd6e</app:target>
					<app:target>#pol_2df9fda1-b8a7-454e-a433-f1eb26e002df</app:target>
					<app:target>#pol_686d2ca8-8039-40f6-861c-ef59b9831d6d</app:target>
					<app:target>#pol_66987187-2683-41a4-8447-6c04658910bb</app:target>
					<app:target>#pol_356d875b-1c62-43fa-ba95-35d7a0afc2ae</app:target>
					<app:target>#pol_072f2c5d-fe75-437e-bb2b-1b3c9b78d9fd</app:target>
					<app:target>#pol_ec915550-684f-4f96-a6d6-2e642ca64fd8</app:target>
					<app:target>#pol_2bdc2c74-b19e-46d2-a8c0-fe7f240d6eb5</app:target>
					<app:target>#pol_de6cd6c5-4554-45c0-81fa-1d80e7861ac8</app:target>
					<app:target>#pol_c57e8586-0c2b-434f-b0e3-7bf318c8b36c</app:target>
					<app:target>#pol_6b50beb8-911d-425b-8c54-eaab2a7f0b7a</app:target>
					<app:target>#pol_3d37b5ef-99f3-4c47-b187-48c839916bee</app:target>
					<app:target>#pol_f9ffd753-d6e5-4539-93e6-d7481618f39e</app:target>
					<app:target>#pol_eb0c3ce2-8de1-4d70-9e04-c5c0a5f863b9</app:target>
					<app:target>#pol_f56dddfa-b33e-41fa-acef-43b08a2c148d</app:target>
					<app:target>#pol_e17dd8ea-fe5d-443b-b7a9-8b7e48451921</app:target>
					<app:target>#pol_ba234ed6-9a10-4648-8fea-24ac5f926dbe</app:target>
					<app:target>#pol_9ccb0753-2ae0-433b-98d0-5648b95dbd1a</app:target>
					<app:target>#pol_0ee5a6db-bd96-4823-891e-d6d43c8027cc</app:target>
					<app:target>#pol_2ae161b6-af24-4d77-9166-dfeaeb3f8bce</app:target>
					<app:target>#pol_5cdf5318-a08a-4923-9325-5ac25fee867f</app:target>
					<app:target>#pol_164aaac2-b726-4f28-a750-cf7fcd392297</app:target>
					<app:target>#pol_f71f3930-e64c-47ab-bf3d-3cd3b66dd517</app:target>
					<app:target>#pol_3b5b4931-8ae0-44dd-94d4-22a8d473f321</app:target>
					<app:target>#pol_7d59592f-c49b-4a9d-b693-fe756777397f</app:target>
					<app:target>#pol_2fa62caf-5581-44bb-8f4a-5864c3b59c00</app:target>
					<app:target>#pol_a07b1b30-b2e0-4f29-bf5f-0fa426b3fe22</app:target>
					<app:target>#pol_f22d9750-840d-456f-83da-42c9e5e641fc</app:target>
					<app:target>#pol_e06b9d99-50e0-4630-8a0a-743a309536ae</app:target>
					<app:target>#pol_7b2383fe-1be5-4af1-b2b6-73ac4e18bfab</app:target>
					<app:target>#pol_5e5c89c2-38d5-4a90-9086-92a0361b5ad0</app:target>
					<app:target>#pol_4c1f2f21-c6e6-48d6-a032-c014473bad5e</app:target>
					<app:target>#pol_d0caaff3-2eb5-45c5-a9bc-ea9fa49fd674</app:target>
					<app:target>#pol_973a2bfe-9695-4a11-bb49-19846832c0dc</app:target>
					<app:target>#pol_7ac7e793-9e74-48d4-975b-4becd3d792e5</app:target>
					<app:target>#pol_7230b197-0b9b-4ebf-8dc9-05a9fb58befc</app:target>
					<app:target>#pol_c7ecae7d-ca42-485e-95c1-9fd2bce87d1a</app:target>
					<app:target>#pol_81d03425-3d36-41cf-82b4-2d733cc48587</app:target>
					<app:target>#pol_fc4b40cc-f329-40b9-ab89-9c12a41379d8</app:target>
					<app:target>#pol_83d3467c-6313-49b8-bb88-b57923a1986f</app:target>
					<app:target>#pol_447c03d9-76d9-42ea-b421-0b85b6b1e1cc</app:target>
					<app:target>#pol_96c26909-a3cf-4b3f-9c04-123be0b0ca63</app:target>
					<app:target>#pol_2b514e2f-0a80-44f5-894b-45df9ebfd941</app:target>
					<app:target>#pol_942995b3-ede7-4d3c-9427-8a0e2d54b855</app:target>
					<app:target>#pol_a2f1116b-168e-4f4e-a55a-71ff75026511</app:target>
					<app:target>#pol_cfe834af-6e28-41eb-9eaa-56b56f1b5707</app:target>
					<app:target>#pol_b5259bc5-0d6e-427e-a89c-7337c55efc7e</app:target>
					<app:target>#pol_f9f91a01-3253-4da5-abde-3de8cc89e4be</app:target>
					<app:target>#pol_5f4017c6-d0fd-41d0-8327-f4c53c287684</app:target>
					<app:target>#pol_bb13e668-dcb5-417e-9503-9a0491472189</app:target>
					<app:target>#pol_87599afa-fa88-4451-8c82-7b6ef7aec9fb</app:target>
					<app:target>#pol_4711fc82-9c59-4a45-8fce-a660dd4198ac</app:target>
					<app:target>#pol_2a6eed36-43a8-414a-b835-d4be0df7ff88</app:target>
					<app:target>#pol_d3529f6e-7730-44f2-aa25-a48ebbf16f0b</app:target>
					<app:target>#pol_dc5784f5-1a90-47db-8328-af91b99a00cd</app:target>
					<app:target>#pol_eead2558-67c8-401c-88c4-4df538ab99b1</app:target>
					<app:target>#pol_c18b2251-fb21-427a-967b-5892c28ad8f0</app:target>
					<app:target>#pol_f53af968-300e-4a59-8ad5-dacf934b8ae8</app:target>
					<app:target>#pol_16e605d7-d549-4a0a-a244-f2296bb113f3</app:target>
					<app:target>#pol_67c2944b-76a7-4d55-b517-6797b2ae5cd6</app:target>
					<app:target>#pol_b5313f96-2470-4e68-89e0-1866bfb1e880</app:target>
				</app:X3DMaterial>
			</app:surfaceDataMember>
			<app:surfaceDataMember>
				<app:X3DMaterial>
					<app:diffuseColor>0.9804 0.9804 0.9804</app:diffuseColor>
					<app:shininess>0</app:shininess>
					<app:target>#pol_08d67e36-31d5-417b-825b-6f8647912a10</app:target>
					<app:target>#pol_77f4881e-9e65-4654-b2a9-2050584be6d2</app:target>
					<app:target>#pol_fc45a2a7-7d66-491c-b7dc-a6e719933aa4</app:target>
					<app:target>#pol_1b10b40c-89f4-440b-819a-4d2559d5d553</app:target>
					<app:target>#pol_f270c6c8-64db-4808-9c6b-fe5c87160af1</app:target>
					<app:target>#pol_3a94a04f-7473-4867-8a0a-b83a300661cc</app:target>
					<app:target>#pol_d750931a-6d61-410b-90ea-edc3bc7f945e</app:target>
					<app:target>#pol_31645971-31a9-4691-b059-cdac16710365</app:target>
					<app:target>#pol_d2dfc46b-e8d5-45b9-ad40-8100cc525f76</app:target>
					<app:target>#pol_cb2497b5-1f1c-410c-a1f4-c0e1dfba52d0</app:target>
					<app:target>#pol_0a703ede-b162-4e3b-a5f4-1df8a1359f37</app:target>
					<app:target>#pol_db19b251-1ade-4825-937e-5a9916f243ab</app:target>
					<app:target>#pol_4c0a8226-7200-4b85-b7b5-800d75659428</app:target>
					<app:target>#pol_019dbdef-3f7c-459e-8de7-338dc5e205cf</app:target>
					<app:target>#pol_84ac201f-0df5-4981-8869-02182d1f246b</app:target>
					<app:target>#pol_c9311f9f-8ade-4f69-865c-8aae3439a4b1</app:target>
					<app:target>#pol_603fabf3-c263-4667-a179-7db8057046b3</app:target>
					<app:target>#pol_717851b1-d5ef-43e0-9f15-273f22de5a48</app:target>
					<app:target>#pol_cd073381-fa72-4799-a529-d5de1ca0b11b</app:target>
					<app:target>#pol_6f0328d5-7413-41b5-80cf-1154ba4f99da</app:target>
					<app:target>#pol_075ffd22-1ed7-4661-8ac2-c28cafdb2caa</app:target>
					<app:target>#pol_feea300a-c118-41b3-a939-8a2259caf9d8</app:target>
					<app:target>#pol_788fb896-faa5-48e7-bec8-12c516b3acfc</app:target>
					<app:target>#pol_6fd7b288-105e-4ca9-a188-4f1442086678</app:target>
					<app:target>#pol_57afa94b-d7ff-4dac-9c2d-90f3e9a0af90</app:target>
					<app:target>#pol_445b7386-f659-46f2-9362-9cb2305e4441</app:target>
					<app:target>#pol_2c721732-2d83-4222-8b7c-7dcac5e93254</app:target>
					<app:target>#pol_8972b1fb-3f12-4386-9dc3-9d96ad085e5a</app:target>
					<app:target>#pol_303ccad5-e3aa-49fe-8bbe-a449925279af</app:target>
					<app:target>#pol_82745f2f-b34b-4cbb-b022-a31376b61a81</app:target>
					<app:target>#pol_0675b7eb-e04e-44e5-b9d1-a301f98670cf</app:target>
					<app:target>#pol_43a762c4-7cfd-4a7c-adaa-2d0fb51d7c8b</app:target>
					<app:target>#pol_2d7d02c5-e1e9-46d6-b185-632a3843911e</app:target>
					<app:target>#pol_a87204ea-b5f2-4140-a1ea-4949cd29e830</app:target>
					<app:target>#pol_e1685670-0047-4cae-9a71-96f67995f415</app:target>
					<app:target>#pol_d10c0210-4156-41f8-8dd2-6306f90cd404</app:target>
					<app:target>#pol_199f10ad-28de-435e-aca7-7a13cca30dd0</app:target>
					<app:target>#pol_3366410e-ccfc-45ba-bf31-802856b1f89d</app:target>
					<app:target>#pol_57484b24-9260-4d5b-92bb-b369acf2c6d4</app:target>
					<app:target>#pol_c80fcea7-2477-4786-9f31-58c022614906</app:target>
					<app:target>#pol_a4f1cde8-416b-4a64-ae72-bee1883a04c2</app:target>
					<app:target>#pol_28cf1d74-4a7e-4067-8b90-fa3d07428d18</app:target>
					<app:target>#pol_3585dcdc-65b7-4804-ae1f-33c41e76ae2d</app:target>
					<app:target>#pol_92ba41db-fd08-42ff-8be3-cb2420d6a7b0</app:target>
					<app:target>#pol_3e3a21d2-f812-4e79-8b40-ac9c10e0ad8a</app:target>
					<app:target>#pol_62cb6bb9-778a-4cc4-a0b0-f290163e2dc7</app:target>
					<app:target>#pol_09bb6567-6f85-4452-963d-a6ff6bc4def8</app:target>
					<app:target>#pol_3b6e4435-13bd-4f86-81ba-4dafc8596cf3</app:target>
					<app:target>#pol_ac0313c5-fdb2-4690-889d-3b0ef28c6620</app:target>
					<app:target>#pol_fa840ab9-961f-45d9-bb70-86086d8c71d8</app:target>
					<app:target>#pol_eecc9e7b-c2d6-446e-8cb1-f2b4264e6b5c</app:target>
					<app:target>#pol_585d1f61-1d4f-43b9-929f-640221fbf2aa</app:target>
					<app:target>#pol_95d09f45-8c45-42c4-944a-e5a5ca86dfe6</app:target>
					<app:target>#pol_319f825f-e372-4297-80b3-c3ee4ab74fc0</app:target>
					<app:target>#pol_dee19ebc-9421-432e-8534-dc08457cf76e</app:target>
					<app:target>#pol_8b2e8068-9274-4582-b73f-1614f5c058d6</app:target>
					<app:target>#pol_d8614724-393a-423b-8158-1f54586453f1</app:target>
					<app:target>#pol_2005af3a-6c0e-4604-a7df-64bee6f994d6</app:target>
					<app:target>#pol_b7859070-6568-4910-a20d-bd4340652bae</app:target>
					<app:target>#pol_9c5a9154-5786-4539-90de-b45a991f6634</app:target>
					<app:target>#pol_5c350300-91ea-449d-87e6-5b30372132e3</app:target>
					<app:target>#pol_0a7ec33c-cfa7-4606-9d3f-37036843f65f</app:target>
					<app:target>#pol_8d7e0065-6ff1-4e1a-a103-ced81cc4bb0a</app:target>
					<app:target>#pol_1ef5e34d-3c67-4621-8b8e-f69d3efe4da1</app:target>
					<app:target>#pol_7f6de06b-a5ce-497e-b344-91ecd361aa1e</app:target>
				</app:X3DMaterial>
			</app:surfaceDataMember>
			<app:surfaceDataMember>
				<app:X3DMaterial>
					<app:diffuseColor>0.9804 0.9804 0.9804</app:diffuseColor>
					<app:shininess>0</app:shininess>
					<app:target>#pol_4b48ef55-2fb6-400b-8053-30841a072954</app:target>
					<app:target>#pol_5f894980-ecef-4f9b-b64d-0982f2b0ff70</app:target>
					<app:target>#pol_e20af9cd-7950-4d13-95c9-9cdba1803173</app:target>
					<app:target>#pol_401376b3-ffff-4dc8-9c52-9e257fb85389</app:target>
					<app:target>#pol_de73375f-5f33-42c9-baa1-ce567d79e4de</app:target>
					<app:target>#pol_f74f49b6-522a-4a27-a573-0746a3108ce7</app:target>
					<app:target>#pol_6f6ef5ad-d410-4e1e-9796-c08f8a45c505</app:target>
					<app:target>#pol_0b4934db-8566-43c8-8a3b-0a841bb11463</app:target>
					<app:target>#pol_02ebd709-d551-472e-bcd1-9f89a064a57c</app:target>
					<app:target>#pol_07d10b09-a1d1-4638-a212-8e2cc4fa8484</app:target>
					<app:target>#pol_b6a7658f-d5f1-4491-a65c-eb111f21a4ef</app:target>
					<app:target>#pol_d66dbf41-987c-4177-b5bb-c79daba09825</app:target>
					<app:target>#pol_156b5b4c-50c3-4bb3-91ae-70fbb5a2aa34</app:target>
					<app:target>#pol_1c506cd7-36d6-46a8-b71b-f52af9261442</app:target>
					<app:target>#pol_e0e29d5c-088d-4c02-bb18-713f4ebc1312</app:target>
					<app:target>#pol_a981ec94-5429-4d90-8acf-90d9df8da92b</app:target>
					<app:target>#pol_be1d38dc-9793-431d-9fe5-f7712cdaffd1</app:target>
					<app:target>#pol_1e9cc91b-79ca-4f61-948d-8cce0911f61c</app:target>
					<app:target>#pol_7ba7a630-a738-4792-804f-5a637116ffc3</app:target>
					<app:target>#pol_d159ceee-ded7-449a-8c27-0c4689267618</app:target>
					<app:target>#pol_85279719-ea8f-4545-93e7-08b3fc9b6976</app:target>
					<app:target>#pol_525bfff6-e3e4-4cce-9a72-cd9b9c094e11</app:target>
					<app:target>#pol_a9d70515-2c1d-4f4d-82f3-01a989dd2b98</app:target>
					<app:target>#pol_27af8fa7-f71c-47ca-a3fa-b465d81ac89b</app:target>
				</app:X3DMaterial>
			</app:surfaceDataMember>
			<app:surfaceDataMember>
				<app:ParameterizedTexture>
					<app:imageURI>54401008_tran_6697_appearance/2000.jpg</app:imageURI>
					<app:mimeType>image/jpg</app:mimeType>
					<app:target uri="#pol_4a93259c-538d-4431-975b-0e8323293425">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_a568ceb9-d30e-45d2-8b13-05f30b5e886c">-7.103731632 9.477017403 -5.915035725 9.445005417 -6.955510616 20.7993412 -7.103731632 9.477017403</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f955c045-bac9-4ef0-a284-e9ac8eaeed4d">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_b80cda33-2930-4ead-916f-911ff3c3e5ec">13.03399658 20.13116455 5.909824848 9.412178993 6.230271816 9.401785851 13.03399658 20.13116455</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_66987187-2683-41a4-8447-6c04658910bb">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_77ce934c-255a-436a-9c96-ab5da7714201">-6.325774193 32.11468124 -6.955510616 20.7993412 -0.2546005547 23.38838959 -6.325774193 32.11468124</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_7d59592f-c49b-4a9d-b693-fe756777397f">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c88da23c-f950-4202-8958-ca2e66a3741c">0.8385107517 91.53663635 7.434973717 83.09054565 7.504308224 91.33927917 0.8385107517 91.53663635</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_7b2383fe-1be5-4af1-b2b6-73ac4e18bfab">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_655f9fec-c025-4ddb-a6db-00cdb0c8c3c7">-6.075265408 62.21034241 -6.10736084 58.45654297 0.1927908063 58.4722023 -6.075265408 62.21034241</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_5e5c89c2-38d5-4a90-9086-92a0361b5ad0">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ec01dfe5-d70e-4c48-831e-dc4ccce270f4">-6.10736084 58.45654297 -6.146503925 53.55454254 0.1927908063 58.4722023 -6.10736084 58.45654297</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d0caaff3-2eb5-45c5-a9bc-ea9fa49fd674">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_50c1d227-1f18-4849-8d25-eb64e49b3a98">0.6960673928 74.96968079 0.6282215118 66.68557739 7.296347618 66.59313965 0.6960673928 74.96968079</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_973a2bfe-9695-4a11-bb49-19846832c0dc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e7cc17d6-b9fc-4d80-bc40-8bc6d6004c45">7.677496433 49.54503632 8.06052494 58.49230957 7.227057457 58.34446335 7.677496433 49.54503632</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_c5ef8a7c-28e0-4533-8311-e921aa4696a3">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_db58a5e3-a2fa-4844-b846-bca9170962db">-5.950413704 76.80327606 0.6960673928 74.96968079 0.7709713578 83.25295258 -5.950413704 76.80327606</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_de5594d8-d4ed-440d-ad07-59596f3da1f4">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_29c3d852-e649-498c-b8a0-f4ac93ce7378">-20.21806526 26.43707848 -13.4780674 32.19799042 -20.13998604 32.27546692 -20.21806526 26.43707848</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_bb13e668-dcb5-417e-9503-9a0491472189">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e03e9021-6020-4e98-9c0c-eba4b78dbdc3">13.32953548 9.141854286 13.67287254 15.23212719 13.03399658 20.13116455 13.32953548 9.141854286</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2a6eed36-43a8-414a-b835-d4be0df7ff88">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_fc74572a-b0a0-455d-a3b3-7758d18735b3">7.504308224 91.33927917 9.168428421 99.53634644 7.567996979 99.59127045 7.504308224 91.33927917</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_7ac7e793-9e74-48d4-975b-4becd3d792e5">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6b42b19b-f015-43f5-939e-e0e1f14a2408">7.296347618 66.59313965 8.396026611 71.95745087 7.365768433 74.84198761 7.296347618 66.59313965</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_83d3467c-6313-49b8-bb88-b57923a1986f">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_3750c996-5bcd-4eca-b7f3-cd9007ac9ca4">7.229434967 58.48735046 8.06052494 58.49230957 7.296347618 66.59313965 7.229434967 58.48735046</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_e06b9d99-50e0-4630-8a0a-743a309536ae">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2f212046-b7d2-4863-957e-7d08edb0292d">-6.075265408 62.21034241 0.1927908063 58.4722023 0.6282215118 66.68557739 -6.075265408 62.21034241</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_0ee5a6db-bd96-4823-891e-d6d43c8027cc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_3f5df3ac-b305-4286-86a6-5077c93329be">-5.950413704 76.80327606 0.6282215118 66.68557739 0.6960673928 74.96968079 -5.950413704 76.80327606</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_356d875b-1c62-43fa-ba95-35d7a0afc2ae">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_4fa23dee-d54a-4a63-b0ab-1fd6452b9a75">6.60911274 38.85272217 7.33855772 40.43152618 6.734460354 48.67208862 6.60911274 38.85272217</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f53af968-300e-4a59-8ad5-dacf934b8ae8">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f2f99570-2ce6-45d4-9404-62b932429100">0.5354883671 115.0310745 -4.219975948 119.9458771 -4.327790737 115.1306915 0.5354883671 115.0310745</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d5548820-027d-4e90-b559-744ad8477ab0">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6ef53ccf-406b-4d6b-ab34-674eb6cb8173">-13.4780674 32.19799042 -13.62199497 20.91995049 -6.808597565 32.12034988 -13.4780674 32.19799042</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_164aaac2-b726-4f28-a750-cf7fcd392297">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ffe7adac-fc7f-4d2f-9329-e5acffd26e6b">0.7709713578 83.25295258 0.6960673928 74.96968079 7.434973717 83.09054565 0.7709713578 83.25295258</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_447c03d9-76d9-42ea-b421-0b85b6b1e1cc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_93997d2c-7b95-4f7b-b717-533a6a73e699">0.5578491688 58.47283554 0.04816922918 46.77676773 6.864403725 58.48675537 0.5578491688 58.47283554</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_4711fc82-9c59-4a45-8fce-a660dd4198ac">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_3d9908a9-fce3-40d0-a852-02eeee6cd93f">13.67287254 15.23212719 15.24074936 15.19859409 13.03399658 20.13116455 13.67287254 15.23212719</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_b48d257c-1d1a-4802-a83e-fc347a6dec08">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_7e2be6bc-fbf4-44ee-9c84-59a48db9dd68">1.15219605 9.573740005 -0.4070570469 11.692729 -0.5582738519 -0.001229284331 1.15219605 9.573740005</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_de6cd6c5-4554-45c0-81fa-1d80e7861ac8">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_5de28bed-0bd8-4c06-808c-87c32807b5bd">12.89135647 9.158597946 13.03399658 20.13116455 6.230271816 9.401785851 12.89135647 9.158597946</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_756da669-74ed-4d0d-b704-d73fd7b678aa">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f2772d42-6261-4935-987e-bda57d966940">0.04816922918 46.77676773 6.734460354 48.67208862 6.864403725 58.48675537 0.04816922918 46.77676773</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_74a6f2a5-6041-473a-bc85-e4f0c7a8c24a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c60fa74f-d430-47ce-acbd-12d2c3c9dd52">-6.808597565 32.12034988 -13.62199497 20.91995049 -6.955510616 20.7993412 -6.808597565 32.12034988</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_5792e886-6b85-4193-89dd-30de9ae5d257">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_76513c44-9b3b-4f1e-b544-1ea88080f786">3.319251776 9.50086689 -0.4070570469 11.692729 1.15219605 9.573740005 3.319251776 9.50086689</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3cbeb25c-eed6-4709-803f-84fef17e73bc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e1960ee7-45c8-4cb2-afc8-2091bc54da1f">-5.760388851 100.0547028 -5.855401039 88.4289856 0.8385107517 91.53663635 -5.760388851 100.0547028</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f69f6d0b-bd1b-4b01-998e-4bcc61ac1217">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_8724539f-3b4a-4a23-a6d8-9d4f72438cc2">-6.955510616 20.7993412 -13.62199497 20.91995049 -13.1534481 9.624710083 -6.955510616 20.7993412</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_a07b1b30-b2e0-4f29-bf5f-0fa426b3fe22">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6b6e0333-cc5d-433b-9f3f-73a253ac301f">7.677496433 49.54503632 7.227057457 58.34446335 6.864403725 58.48675537 7.677496433 49.54503632</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_06145d06-fb85-413f-8d8a-a02233d05ccc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e06179ea-846f-4fbf-971b-87a52f782dec">-6.325774193 32.11468124 -0.2546005547 23.38838959 -0.1040918604 35.08137512 -6.325774193 32.11468124</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_7230b197-0b9b-4ebf-8dc9-05a9fb58befc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_31be3b87-c265-4db5-9e6b-df687e874ec7">8.753495216 85.06756592 7.434973717 83.09054565 8.396026611 71.95745087 8.753495216 85.06756592</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3e2766d3-6af0-425d-9c01-e1d608c8b83b">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_08b0914c-8e5a-418a-9a50-679178361a00">6.484182835 29.0339241 6.60911274 38.85272217 -0.1040918604 35.08137512 6.484182835 29.0339241</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_9a82b3e2-46ec-4a63-892d-19aab2911965">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_55db1fc0-39a5-47e3-a293-70e225c68b45">-19.98786545 26.07457352 -13.4780674 32.19799042 -20.21806526 26.43707848 -19.98786545 26.07457352</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_b5259bc5-0d6e-427e-a89c-7337c55efc7e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_a3bf22b4-8f51-43ed-b5ac-db00dbd4478f">5.997877598 12.02149391 5.909824848 9.412178993 13.03399658 20.13116455 5.997877598 12.02149391</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_fc4b40cc-f329-40b9-ab89-9c12a41379d8">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_254d89da-1590-4588-8cdc-a40ad029983d">7.229434967 58.48735046 6.864403725 58.48675537 7.227057457 58.34446335 7.229434967 58.48735046</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_4e99e9e6-868d-493f-8dfe-6bcced39ecd6">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2d58d4e2-16f6-490e-b932-8378a89c1799">-13.62199497 20.91995049 -13.4780674 32.19799042 -19.98786545 26.07457352 -13.62199497 20.91995049</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_5cdf5318-a08a-4923-9325-5ac25fee867f">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_aa65413f-95a2-45e4-9983-25df36c33271">7.434973717 83.09054565 0.6960673928 74.96968079 7.365768433 74.84198761 7.434973717 83.09054565</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_123aac70-57eb-4f05-9f72-8ce19c470577">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_fd157cf4-9492-471e-a66c-e7c75e475e70">-19.98786545 26.07457352 -20.27865982 19.61715126 -13.62199497 20.91995049 -19.98786545 26.07457352</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3d37b5ef-99f3-4c47-b187-48c839916bee">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_657cff35-a8ab-442a-aeb0-ef5b3ec66d67">-0.2546005547 23.38838959 6.354765415 19.21998215 6.484182835 29.0339241 -0.2546005547 23.38838959</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2b514e2f-0a80-44f5-894b-45df9ebfd941">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e67b8a6b-55c8-43ec-a1a2-87dc5b37bab8">7.296347618 66.59313965 6.864403725 58.48675537 7.229434967 58.48735046 7.296347618 66.59313965</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f9f91a01-3253-4da5-abde-3de8cc89e4be">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f869ab60-f32d-4344-9cc0-d68a458b9579">5.997877598 12.02149391 13.03399658 20.13116455 6.354765415 19.21998215 5.997877598 12.02149391</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_5e1974bc-560a-4a15-94f5-6489022afd6e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6f98af6d-ed0e-4983-bf47-38a8bed6c4c9">6.999619484 31.31801224 6.60911274 38.85272217 6.484182835 29.0339241 6.999619484 31.31801224</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_eb0c3ce2-8de1-4d70-9e04-c5c0a5f863b9">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_1a54aeac-2fdd-426b-b12b-c08776e9a52b">13.03399658 20.13116455 15.24074936 15.19859409 15.18871975 31.03872681 13.03399658 20.13116455</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f71f3930-e64c-47ab-bf3d-3cd3b66dd517">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_219ad0db-4f0b-4147-b1f1-9383060c3b09">8.396026611 71.95745087 7.434973717 83.09054565 7.365768433 74.84198761 8.396026611 71.95745087</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d3529f6e-7730-44f2-aa25-a48ebbf16f0b">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e88fc9d5-3537-4dca-b482-5d3a60468874">0.8385107517 91.53663635 4.345601559 99.70315552 0.9073059559 99.82253265 0.8385107517 91.53663635</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2df9fda1-b8a7-454e-a433-f1eb26e002df">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_8aab9d4f-f528-4c90-9969-b4822b8beb96">-6.211039543 45.80609894 -6.325774193 32.11468124 -0.1040918604 35.08137512 -6.211039543 45.80609894</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_686d2ca8-8039-40f6-861c-ef59b9831d6d">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9621adc6-b6de-49ae-93ec-a26d8134e1e3">13.03399658 20.13116455 6.484182835 29.0339241 6.354765415 19.21998215 13.03399658 20.13116455</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_b5313f96-2470-4e68-89e0-1866bfb1e880">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_381422af-eccc-4f9c-89bd-9b1e641febd6">0.5354883671 115.0310745 4.866275787 114.9555969 0.616872251 119.8655319 0.5354883671 115.0310745</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_ba234ed6-9a10-4648-8fea-24ac5f926dbe">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ac5d958d-2362-4f8c-92a0-b8b846f9132d">-5.950413704 76.80327606 -6.075265408 62.21034241 0.6282215118 66.68557739 -5.950413704 76.80327606</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_cfe834af-6e28-41eb-9eaa-56b56f1b5707">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_d56b8209-a771-46e4-850c-88b0b1109983">3.319251776 9.50086689 3.407304525 12.11018085 -0.4070570469 11.692729 3.319251776 9.50086689</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_dc5784f5-1a90-47db-8328-af91b99a00cd">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ef2632bf-aa88-49b7-afd6-cacad4167f0c">4.943680286 119.7960892 4.866275787 114.9555969 9.422894478 115.6472321 4.943680286 119.7960892</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_88a8affd-d5e2-46b4-86de-904090b7f368">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_0b625c25-6bd5-4033-9641-c32a8a9d6ba3">-0.2546005547 23.38838959 -0.4070570469 11.692729 6.354765415 19.21998215 -0.2546005547 23.38838959</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_8720812c-ae1f-49d9-91c5-6b7fc5354111">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_a90abcd1-58b7-4d9c-8d63-6fb229fd1931">-6.040359974 4.48818922 -0.5582738519 -0.001229284331 -0.4070570469 11.692729 -6.040359974 4.48818922</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_953e3ca7-e5ac-4545-a643-966eb7c04b86">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_cf2ff595-0230-40d7-b0c6-af2f7a5365d7">5.909824848 9.412178993 5.997877598 12.02149391 3.407304525 12.11018085 5.909824848 9.412178993</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_e77d1b67-a97a-4d5e-82ac-f2d4dc5f50e6">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c00b4fdc-1a0f-43e7-84e6-85f04f81a483">0.01315317117 99.85167694 -5.760388851 100.0547028 0.8385107517 91.53663635 0.01315317117 99.85167694</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_54c06e5e-8b0c-464d-bc30-da16b311dbda">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c73a53ff-1da9-41b1-a757-26edf870c4b8">-20.41677284 10.24318695 -20.43227577 9.800605774 -13.76851654 9.637864113 -20.41677284 10.24318695</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_e17dd8ea-fe5d-443b-b7a9-8b7e48451921">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_7f377955-14cd-45b6-b75a-08e1b908fdca">0.8385107517 91.53663635 -5.855401039 88.4289856 0.7709713578 83.25295258 0.8385107517 91.53663635</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_81d03425-3d36-41cf-82b4-2d733cc48587">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_20914adf-c14d-4f18-9a93-a5283624a153">7.229434967 58.48735046 7.227057457 58.34446335 8.06052494 58.49230957 7.229434967 58.48735046</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_549dc592-c27a-4444-989b-da32dc1d96de">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9d2d1e36-1358-4feb-8bc5-b713c0483ef8">0.5578491688 58.47283554 0.6282215118 66.68557739 0.1927908063 58.4722023 0.5578491688 58.47283554</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_072f2c5d-fe75-437e-bb2b-1b3c9b78d9fd">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_d17ac897-e24f-45ad-8cde-3a8184ed3647">6.999619484 31.31801224 7.33855772 40.43152618 6.60911274 38.85272217 6.999619484 31.31801224</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_ce534719-fbfb-4d50-9334-72c2a33f7569">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9fbbf041-eeb3-40ef-b32b-b289e99565d3">-20.27865982 19.61715126 -13.76851654 9.637864113 -13.62199497 20.91995049 -20.27865982 19.61715126</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_e0f6abc6-c0d3-46f5-ac00-c0df4a55d7da">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_0b4ab25c-f9fa-44c2-a754-c53c1caee095">-6.955510616 20.7993412 -0.4070570469 11.692729 -0.2546005547 23.38838959 -6.955510616 20.7993412</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2fa62caf-5581-44bb-8f4a-5864c3b59c00">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c71543bc-7a4a-40bd-8cd7-4d1a7541d53e">9.168428421 99.53634644 7.504308224 91.33927917 13.94107437 93.89388275 9.168428421 99.53634644</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_b11f0f4a-d226-402a-a591-82d960cfd3f3">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_5d3d7608-ec10-42d5-b599-a4c94b58bb77">-20.41677284 10.24318695 -13.76851654 9.637864113 -20.27865982 19.61715126 -20.41677284 10.24318695</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f22d9750-840d-456f-83da-42c9e5e641fc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_3b1676a5-6fc0-4254-80fc-61fb077c0843">8.396026611 71.95745087 7.296347618 66.59313965 8.06052494 58.49230957 8.396026611 71.95745087</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_6b50beb8-911d-425b-8c54-eaab2a7f0b7a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_30210073-fd8f-4f18-a521-e74cf6cdad5e">13.03399658 20.13116455 15.18871975 31.03872681 13.17205334 31.10893822 13.03399658 20.13116455</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_57d76baf-432e-4e82-be86-ac4085d01a04">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_1be647fc-7279-4122-bb7d-6976acf5f9c2">0.04816922918 46.77676773 -0.1040918604 35.08137512 6.60911274 38.85272217 0.04816922918 46.77676773</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_fbd79cba-2ac5-4ade-8aac-8698d252d710">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_32abfb60-805b-4a7f-92f6-f4c0619f3ea0">0.6282215118 66.68557739 0.5578491688 58.47283554 6.864403725 58.48675537 0.6282215118 66.68557739</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f56dddfa-b33e-41fa-acef-43b08a2c148d">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_5646b322-e284-4a54-ab6f-ca66ea0cd089">-5.855401039 88.4289856 -5.950413704 76.80327606 0.7709713578 83.25295258 -5.855401039 88.4289856</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_942995b3-ede7-4d3c-9427-8a0e2d54b855">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_90e2133c-cf90-4c7e-9f0d-58362894d30d">0.04816922918 46.77676773 0.5578491688 58.47283554 0.1927908063 58.4722023 0.04816922918 46.77676773</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2bdc2c74-b19e-46d2-a8c0-fe7f240d6eb5">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f44e088e-85ce-47ce-8a97-4569b277fe6f">0.04816922918 46.77676773 6.60911274 38.85272217 6.734460354 48.67208862 0.04816922918 46.77676773</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d9f5cea2-1ce2-4af4-9682-20503865375d">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_b432fd11-1c42-4f80-9d85-1aa6f1e51e28">-13.1534481 9.624710083 -7.103731632 9.477017403 -6.955510616 20.7993412 -13.1534481 9.624710083</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_e033426e-368e-4777-86b8-53630fb92889">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_31146f7b-689e-4db4-b523-d52859f06280">0.9073059559 99.82253265 0.01315317117 99.85167694 0.8385107517 91.53663635 0.9073059559 99.82253265</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_4082bdfa-055a-48d9-ae7d-927cbeddf23d">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6e18467c-5591-4cb6-a5a7-f4796da544f0">-6.211039543 45.80609894 -0.1040918604 35.08137512 0.04816922918 46.77676773 -6.211039543 45.80609894</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f9ffd753-d6e5-4539-93e6-d7481618f39e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_5eb3fc8e-8cee-4ef6-8fb0-6136787c77c0">13.17205334 31.10893822 6.999619484 31.31801224 6.484182835 29.0339241 13.17205334 31.10893822</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_42367d28-6247-4ebf-8246-361370d6c662">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2d358add-ed7d-4bfc-b170-efc2f7a93f29">4.345601559 99.70315552 0.8385107517 91.53663635 7.504308224 91.33927917 4.345601559 99.70315552</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_eead2558-67c8-401c-88c4-4df538ab99b1">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_0e84dbec-5eaf-4225-9795-4d5588214be2">4.866275787 114.9555969 4.943680286 119.7960892 0.616872251 119.8655319 4.866275787 114.9555969</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_96817b57-2ff0-44a9-b707-ce6f7438c16c">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c58d371c-1425-4d4b-8438-0c7153319e2e">-13.76851654 9.637864113 -13.1534481 9.624710083 -13.62199497 20.91995049 -13.76851654 9.637864113</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d42013ae-8aa3-4559-9d59-703df386c728">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_bf722f32-b4e4-4cb8-b2ba-67d351466d2a">7.504308224 91.33927917 13.8967247 93.10021973 13.94107437 93.89388275 7.504308224 91.33927917</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_67c2944b-76a7-4d55-b517-6797b2ae5cd6">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_0187fcac-b9b1-4128-a50c-b8d69861c4c3">9.422894478 115.6472321 4.866275787 114.9555969 9.412326813 114.8717575 9.422894478 115.6472321</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_5f4017c6-d0fd-41d0-8327-f4c53c287684">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_b7fd6055-b471-4d9d-a808-61dc2007b541">3.407304525 12.11018085 5.997877598 12.02149391 6.354765415 19.21998215 3.407304525 12.11018085</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_a34a6b66-34da-4b13-b611-0816af47e25e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e0594b5a-377e-4c32-8811-7aed279ae93b">-23.9268856 32.32041168 -20.21806526 26.43707848 -20.13998604 32.27546692 -23.9268856 32.32041168</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d67fcdb2-858f-49c7-8f9b-84d70f81511a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_0bdfcfd2-c893-4652-aecd-db6c8dcd9460">1.15219605 9.573740005 -0.5582738519 -0.001229284331 1 0 1.15219605 9.573740005</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_c18b2251-fb21-427a-967b-5892c28ad8f0">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9dd9fa61-9c47-4a62-93ae-ec93b78d7c70">0.5354883671 115.0310745 0.616872251 119.8655319 -4.219975948 119.9458771 0.5354883671 115.0310745</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_ef1ee3ec-bd3b-4969-a5a2-5cf29502dce6">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_0264b94f-0806-4bfb-95a1-4d5a96a8d7f6">-6.955510616 20.7993412 -5.915035725 9.445005417 -0.4070570469 11.692729 -6.955510616 20.7993412</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2ae161b6-af24-4d77-9166-dfeaeb3f8bce">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_44fee059-5452-403a-9eab-f3e127ae5e2c">7.677496433 49.54503632 6.864403725 58.48675537 6.734460354 48.67208862 7.677496433 49.54503632</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3b5b4931-8ae0-44dd-94d4-22a8d473f321">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_a89f7ca7-c765-4484-a364-41b733ce325f">13.8967247 93.10021973 7.504308224 91.33927917 13.44472599 85.03579712 13.8967247 93.10021973</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_16e605d7-d549-4a0a-a244-f2296bb113f3">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e0ee6484-89d8-4c1a-869f-ce0a85e3746a">9.422894478 115.6472321 9.483909607 119.7277451 4.943680286 119.7960892 9.422894478 115.6472321</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_c57e8586-0c2b-434f-b0e3-7bf318c8b36c">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_355d845f-ab4f-4d60-afcc-1529fb05e113">-0.1040918604 35.08137512 -0.2546005547 23.38838959 6.484182835 29.0339241 -0.1040918604 35.08137512</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_7974cee0-db54-4c81-a36e-06321b04971c">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_a3be5f20-0bde-4509-9c5f-d430c3e4b769">9.168428421 99.53634644 13.94107437 93.89388275 14.17861843 99.21434021 9.168428421 99.53634644</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_96c26909-a3cf-4b3f-9c04-123be0b0ca63">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_84338ce3-8938-48c2-acc5-2983e10b2a19">6.864403725 58.48675537 7.296347618 66.59313965 0.6282215118 66.68557739 6.864403725 58.48675537</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_780c84b6-ea2a-4d43-8857-8cb248cdb079">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ec5585df-0729-409b-a516-bd46c2e562a0">-6.955510616 20.7993412 -6.325774193 32.11468124 -6.808597565 32.12034988 -6.955510616 20.7993412</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d46bdd0d-710b-438d-a6db-4d5fc7a83bad">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_46912905-7144-4248-8c59-2a0d496371a7">0.04816922918 46.77676773 0.1927908063 58.4722023 -6.146503925 53.55454254 0.04816922918 46.77676773</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_827996a0-7200-4163-8369-11db9518c922">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_1d13e0d5-e9cd-45c3-9a1f-583c73bcffd0">8.753495216 85.06756592 7.504308224 91.33927917 7.434973717 83.09054565 8.753495216 85.06756592</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_9ccb0753-2ae0-433b-98d0-5648b95dbd1a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f816e64b-42b5-43ba-a941-820697dc9364">0.8385107517 91.53663635 0.7709713578 83.25295258 7.434973717 83.09054565 0.8385107517 91.53663635</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_765b3ccf-804a-4164-83ed-6a717cde6780">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_390cda28-afe5-41de-a7e3-e6fd1eb42ee4">-5.915035725 9.445005417 -6.040359974 4.48818922 -0.4070570469 11.692729 -5.915035725 9.445005417</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_4c1f2f21-c6e6-48d6-a032-c014473bad5e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_53add22d-86cb-464f-88b4-6a02dd6ec33e">7.365768433 74.84198761 0.6960673928 74.96968079 7.296347618 66.59313965 7.365768433 74.84198761</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_8aa582a7-c01e-4e43-9bbb-0cb8940cbf5f">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_18b409e9-d073-4f01-b265-3d2c206ee7e4">13.17205334 31.10893822 6.484182835 29.0339241 13.03399658 20.13116455 13.17205334 31.10893822</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_a2f1116b-168e-4f4e-a55a-71ff75026511">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_bdf73c86-cc3c-4545-8d4b-8066b51947c4">-0.4070570469 11.692729 3.407304525 12.11018085 6.354765415 19.21998215 -0.4070570469 11.692729</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_87599afa-fa88-4451-8c82-7b6ef7aec9fb">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_38b09aa3-279b-4e02-a9ba-f32594588d55">12.89135647 9.158597946 13.32953548 9.141854286 13.03399658 20.13116455 12.89135647 9.158597946</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_9fe9a4fd-c912-4687-91ec-f2d23135381a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_80a2e138-e7cb-44be-b82b-59f8ac1b86d8">0.04816922918 46.77676773 -6.146503925 53.55454254 -6.211039543 45.80609894 0.04816922918 46.77676773</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_c7ecae7d-ca42-485e-95c1-9fd2bce87d1a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_7f90f39a-2a5c-43df-9ced-5c7dc3cca122">7.504308224 91.33927917 8.753495216 85.06756592 13.44472599 85.03579712 7.504308224 91.33927917</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_ec915550-684f-4f96-a6d6-2e642ca64fd8">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_063121ca-1e2e-4a37-b692-d9d68a4dcbfa">7.33855772 40.43152618 7.677496433 49.54503632 6.734460354 48.67208862 7.33855772 40.43152618</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_fafcfe27-57f9-4e3b-adfe-56665cfde0b7">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_3738c7b4-b4f9-4b24-9eac-1df746d4cea6">-6.040359974 4.48818922 -6.157049656 0 -0.5582738519 -0.001229284331 -6.040359974 4.48818922</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f0169b43-be98-4a04-bbdc-e2ddaf220c77">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_999b2c64-c1f0-46a8-8c2f-35df701638a7">5.909824848 9.412178993 3.407304525 12.11018085 3.319251776 9.50086689 5.909824848 9.412178993</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_922b59b5-90e1-4e81-82a9-fda2e7b5585d">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_566fddeb-6517-4f5a-879a-0a69e204c840">7.504308224 91.33927917 7.567996979 99.59127045 4.345601559 99.70315552 7.504308224 91.33927917</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
				</app:ParameterizedTexture>
			</app:surfaceDataMember>
			<app:surfaceDataMember>
				<app:ParameterizedTexture>
					<app:imageURI>54401008_tran_6697_appearance/1100.jpg</app:imageURI>
					<app:mimeType>image/jpg</app:mimeType>
					<app:target uri="#pol_cb2497b5-1f1c-410c-a1f4-c0e1dfba52d0">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2a4150a2-c9c2-4f76-a881-d006e561d566">4831.858887 -2028.045288 -2028.047485 5.035961628 -2028.533813 5.035004139 4831.858887 -2028.045288</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d8614724-393a-423b-8158-1f54586453f1">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_bca82883-052a-4ded-892f-ecbfd1d3e424">4831.106934 -2030.96875 4830.989258 -2031.160767 4831.104004 -2030.966797 4831.106934 -2030.96875</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_a87204ea-b5f2-4140-a1ea-4949cd29e830">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c044936f-5ca8-45ca-af7d-57b6c0403984">4829.229492 -2034.14917 4829.300781 -2034.183594 4829.302246 -2034.189087 4829.229492 -2034.14917</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_dee19ebc-9421-432e-8534-dc08457cf76e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_1e28b8e3-f817-4367-a35d-f93e00e749ca">4829.834961 -2033.115967 4830.166992 -2032.502197 4830.189453 -2032.515381 4829.834961 -2033.115967</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3a94a04f-7473-4867-8a0a-b83a300661cc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_81dd46f8-bad9-4699-96d5-b694a9c9316b">4831.576172 -2028.53186 -2028.533813 5.035004139 4831.391602 -2028.872803 4831.576172 -2028.53186</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_6fd7b288-105e-4ca9-a188-4f1442086678">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9afc8464-1e5d-41cd-82b8-eebf95af793c">4831.085449 -2032.786133 4831.326172 -2032.928711 4831.293945 -2032.938843 4831.085449 -2032.786133</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_445b7386-f659-46f2-9362-9cb2305e4441">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f0e4a0ca-2cfb-43d4-bcaf-b1a6e4aa4208">4831.438477 -2032.113525 2033.111572 4.98653841 2032.408936 5.000228405 4831.438477 -2032.113525</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_788fb896-faa5-48e7-bec8-12c516b3acfc">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_b45815f4-a102-4f1f-9088-52ba4c02f0d6">4831.053711 -2032.794922 4831.293945 -2032.938843 2033.940063 4.97509861 4831.053711 -2032.794922</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f270c6c8-64db-4808-9c6b-fe5c87160af1">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_423a5e5b-dd50-4413-8a26-77b980bbf434">-2028.533813 5.035004139 4831.576172 -2028.53186 4831.858887 -2028.045288 -2028.533813 5.035004139</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2d7d02c5-e1e9-46d6-b185-632a3843911e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_92075740-c22c-4a4c-9639-3a50dda9657f">4829.302246 -2034.189087 4829.300781 -2034.183594 4829.44043 -2033.939453 4829.302246 -2034.189087</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_1b10b40c-89f4-440b-819a-4d2559d5d553">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2b06bff0-2aef-4c63-93a7-2d175c159b96">4831.368164 -2028.860107 4831.553711 -2028.519287 4831.576172 -2028.53186 4831.368164 -2028.860107</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_1ef5e34d-3c67-4621-8b8e-f69d3efe4da1">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_8d58b2b2-a1f5-434f-bbc9-2322e2e50b79">4830.192871 -2032.517334 4830.189453 -2032.515381 4830.543945 -2031.914795 4830.192871 -2032.517334</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d2dfc46b-e8d5-45b9-ad40-8100cc525f76">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_b96c7f66-e533-467b-ba1c-bd1c2cad5837">4831.391602 -2028.872803 -2028.533813 5.035004139 4831.39502 5.032034397 4831.391602 -2028.872803</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_43a762c4-7cfd-4a7c-adaa-2d0fb51d7c8b">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_d3ca6c9f-6701-4682-b4b3-5e9104149a98">4831.085449 -2032.786133 4831.293945 -2032.938843 4831.053711 -2032.794922 4831.085449 -2032.786133</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_cd073381-fa72-4799-a529-d5de1ca0b11b">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_0313cb0c-3a65-4ad3-9345-5956064d4c82">2034.368774 4.974917889 2033.940063 4.97509861 2034.370483 4.984917641 2034.368774 4.974917889</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_8d7e0065-6ff1-4e1a-a103-ced81cc4bb0a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_fcc43074-6943-44f7-aa18-36252bf6bb64">4830.543945 -2031.914795 4830.989258 -2031.160767 4830.547852 -2031.916748 4830.543945 -2031.914795</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2c721732-2d83-4222-8b7c-7dcac5e93254">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_5e5ba6c4-732d-44b3-9e71-32bf233b7dc3">2032.408936 5.000228405 4831.837891 -2031.410889 4831.438477 -2032.113525 2032.408936 5.000228405</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_08d67e36-31d5-417b-825b-6f8647912a10">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f86dccf7-347e-454b-b188-4f3209f94491">4831.576172 -2028.53186 4831.391602 -2028.872803 4831.368164 -2028.860107 4831.576172 -2028.53186</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_6f0328d5-7413-41b5-80cf-1154ba4f99da">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_73624be1-4bfe-4832-8720-a3af8a8fbfd6">2033.796509 4.972847939 4831.053711 -2032.794922 2033.940063 4.97509861 2033.796509 4.972847939</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3366410e-ccfc-45ba-bf31-802856b1f89d">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_72274603-32f3-4d29-b097-fbb84e1bacb1">4829.250977 -2034.126709 4829.352539 -2033.941162 4829.291016 -2034.148682 4829.250977 -2034.126709</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d750931a-6d61-410b-90ea-edc3bc7f945e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f3c6e597-d5f9-4717-856a-9f379c8f5bea">4831.368164 -2028.860107 4831.391602 -2028.872803 4830.995605 -2029.072021 4831.368164 -2028.860107</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_4c0a8226-7200-4b85-b7b5-800d75659428">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9768232e-8584-482b-a4a6-052f66ef95ae">-2028.02417 5.035881996 -2028.047485 5.035961628 4831.858887 -2028.045288 -2028.02417 5.035881996</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_db19b251-1ade-4825-937e-5a9916f243ab">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_d1249afc-3bd0-4b8f-a041-843a49bc21c5">4831.87207 -2028.022217 -2028.02417 5.035881996 4831.858887 -2028.045288 4831.87207 -2028.022217</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_075ffd22-1ed7-4661-8ac2-c28cafdb2caa">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9aa8d24e-8c7a-444d-bd68-657eeed3297a">4831.293945 -2032.938843 2034.370483 4.984917641 2033.940063 4.97509861 4831.293945 -2032.938843</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_585d1f61-1d4f-43b9-929f-640221fbf2aa">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c2352ece-8b7d-44cb-ab40-308fec4bbca6">4829.834961 -2033.115967 4829.8125 -2033.102783 4830.166992 -2032.502197 4829.834961 -2033.115967</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_7f6de06b-a5ce-497e-b344-91ecd361aa1e">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6efc678d-b91f-4c5e-a117-760bf30bef0a">4830.547852 -2031.916748 4830.192871 -2032.517334 4830.543945 -2031.914795 4830.547852 -2031.916748</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_b7859070-6568-4910-a20d-bd4340652bae">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2acc4ff5-cf2b-4cb3-9eb5-4765a78e6901">4831.081055 -2030.953491 4830.989258 -2031.160767 4830.966797 -2031.147461 4831.081055 -2030.953491</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_c80fcea7-2477-4786-9f31-58c022614906">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_12ad05c6-3e27-4cfd-867e-0535a292b973">4829.362305 -2033.924194 4829.375977 -2033.901733 4829.405273 -2033.948853 4829.362305 -2033.924194</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_95d09f45-8c45-42c4-944a-e5a5ca86dfe6">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6e76b51a-1359-416f-bda2-a350b57b23ca">4830.166992 -2032.502197 4830.521973 -2031.901611 4830.189453 -2032.515381 4830.166992 -2032.502197</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_92ba41db-fd08-42ff-8be3-cb2420d6a7b0">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6766538a-229b-48f9-af23-a6d7a844ea30">4829.37793 -2033.898682 4829.44043 -2033.939453 4829.375977 -2033.901733 4829.37793 -2033.898682</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3e3a21d2-f812-4e79-8b40-ac9c10e0ad8a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_595318b5-8a9e-4373-90fb-312daf77e96a">4829.291016 -2034.148682 4829.352539 -2033.941162 4829.405273 -2033.948853 4829.291016 -2034.148682</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_28cf1d74-4a7e-4067-8b90-fa3d07428d18">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_10d0e453-ebe9-40ef-91c3-08244b918041">4829.37793 -2033.898682 4829.445801 -2033.937866 4829.44043 -2033.939453 4829.37793 -2033.898682</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_2005af3a-6c0e-4604-a7df-64bee6f994d6">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_eee2aaa8-4f92-4421-b269-8493d2c06e28">4831.081055 -2030.953491 4831.104004 -2030.966797 4830.989258 -2031.160767 4831.081055 -2030.953491</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_fa840ab9-961f-45d9-bb70-86086d8c71d8">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c83755b9-6261-40df-9bd9-dbe3f3a74d6b">4830.989258 -2031.160767 4831.106934 -2030.96875 4830.992676 -2031.16272 4830.989258 -2031.160767</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_0a703ede-b162-4e3b-a5f4-1df8a1359f37">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_a9e06e47-5a46-4a8e-aa05-2eca0a862ab7">4831.553711 -2028.519287 4831.840332 -2028.004272 4831.858887 -2028.045288 4831.553711 -2028.519287</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_c9311f9f-8ade-4f69-865c-8aae3439a4b1">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_7959e1c7-e4fa-4359-8d82-c29a40e7bead">4831.391602 -2028.872803 4831.37207 5.03215313 4830.995605 -2029.072021 4831.391602 -2028.872803</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_e1685670-0047-4cae-9a71-96f67995f415">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_f56ebc54-8079-461c-a47e-f79970640bff">4829.44043 -2033.939453 4829.445801 -2033.937866 4829.302246 -2034.189087 4829.44043 -2033.939453</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_a4f1cde8-416b-4a64-ae72-bee1883a04c2">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ecfa5d6b-d83f-4c8f-b502-6db55ddd451b">4829.375977 -2033.901733 4829.44043 -2033.939453 4829.405273 -2033.948853 4829.375977 -2033.901733</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_5c350300-91ea-449d-87e6-5b30372132e3">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2452a508-c3f7-4aad-84a8-5de9239d3acc">4830.192871 -2032.517334 4829.838379 -2033.11792 4829.834961 -2033.115967 4830.192871 -2032.517334</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_31645971-31a9-4691-b059-cdac16710365">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2ecb903a-f282-4ef1-a0aa-776f8cb06421">4831.37207 5.03215313 4831.391602 -2028.872803 4831.39502 5.032034397 4831.37207 5.03215313</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_84ac201f-0df5-4981-8869-02182d1f246b">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_5168dca6-84b5-4eb6-be98-c5c0b0b34ee3">4831.37207 5.03215313 4830.999023 5.026725769 4830.995605 -2029.072021 4831.37207 5.03215313</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_319f825f-e372-4297-80b3-c3ee4ab74fc0">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ae0114be-024e-4638-97fa-4e1bdd6bb767">4830.521973 -2031.901611 4830.543945 -2031.914795 4830.189453 -2032.515381 4830.521973 -2031.901611</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_199f10ad-28de-435e-aca7-7a13cca30dd0">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9999adbf-9913-40a8-8141-aaf91252b17c">4829.352539 -2033.941162 4829.362305 -2033.924194 4829.405273 -2033.948853 4829.352539 -2033.941162</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_0675b7eb-e04e-44e5-b9d1-a301f98670cf">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ec6c2126-39eb-444c-bf49-b036d892f037">4831.837891 -2031.410889 4831.860352 -2031.423584 4831.461426 -2032.126221 4831.837891 -2031.410889</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_62cb6bb9-778a-4cc4-a0b0-f290163e2dc7">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6b30c2bb-dc4b-481a-8967-3bd0f1eb7b17">4829.217773 -2034.108398 4829.250977 -2034.126709 4829.228027 -2034.143799 4829.217773 -2034.108398</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_eecc9e7b-c2d6-446e-8cb1-f2b4264e6b5c">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_bd9d74e5-24d7-4187-92fb-e819c4547415">4829.834961 -2033.115967 4830.189453 -2032.515381 4830.192871 -2032.517334 4829.834961 -2033.115967</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_82745f2f-b34b-4cbb-b022-a31376b61a81">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_70dc53ff-13e5-47bc-9ce2-d878ec2c2dfa">2033.111572 4.98653841 4831.438477 -2032.113525 4831.053711 -2032.794922 2033.111572 4.98653841</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_57484b24-9260-4d5b-92bb-b369acf2c6d4">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_d30aaeb3-61a8-45b1-ad8a-67f2cde9d7e7">4829.405273 -2033.948853 4829.44043 -2033.939453 4829.300781 -2034.183594 4829.405273 -2033.948853</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_ac0313c5-fdb2-4690-889d-3b0ef28c6620">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_340750b8-18c6-42c3-9801-9684511d564e">4829.228027 -2034.143799 4829.291016 -2034.148682 4829.300781 -2034.183594 4829.228027 -2034.143799</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_77f4881e-9e65-4654-b2a9-2050584be6d2">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_a08cfd43-e669-4595-a280-09f7bfdae68f">4830.973145 -2029.05957 4831.368164 -2028.860107 4830.995605 -2029.072021 4830.973145 -2029.05957</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_0a7ec33c-cfa7-4606-9d3f-37036843f65f">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ebfc5821-fcdf-4447-8565-3d5a1eb11727">4830.547852 -2031.916748 4830.989258 -2031.160767 4830.992676 -2031.16272 4830.547852 -2031.916748</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_019dbdef-3f7c-459e-8de7-338dc5e205cf">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_2e3f78fd-a29a-4570-89bf-6c0c8c566a49">4831.576172 -2028.53186 4831.553711 -2028.519287 4831.858887 -2028.045288 4831.576172 -2028.53186</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3b6e4435-13bd-4f86-81ba-4dafc8596cf3">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_71a8a466-0a2c-419c-9996-b25151eb2d60">4829.250977 -2034.126709 4829.291016 -2034.148682 4829.228027 -2034.143799 4829.250977 -2034.126709</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_303ccad5-e3aa-49fe-8bbe-a449925279af">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e3d5c185-72f2-47df-8c22-59f9af23e999">4831.09082 -2033.382446 4831.293945 -2032.938843 4831.326172 -2032.928711 4831.09082 -2033.382446</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_57afa94b-d7ff-4dac-9c2d-90f3e9a0af90">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_acdbb446-5d43-48ca-977e-9dfde45d5104">4831.461426 -2032.126221 4831.438477 -2032.113525 4831.837891 -2031.410889 4831.461426 -2032.126221</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_8972b1fb-3f12-4386-9dc3-9d96ad085e5a">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_30704d06-6fe1-4426-b6e2-31b47c0d8510">4831.461426 -2032.126221 4831.085449 -2032.786133 4831.438477 -2032.113525 4831.461426 -2032.126221</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_717851b1-d5ef-43e0-9f15-273f22de5a48">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_3f8e0d28-a30c-4240-9b51-23f4df1fc5b1">2033.111572 4.98653841 4831.053711 -2032.794922 2033.796509 4.972847939 2033.111572 4.98653841</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_603fabf3-c263-4667-a179-7db8057046b3">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_a4298c7a-b3a2-4f79-8de8-864e8ff53065">4831.085449 -2032.786133 4831.053711 -2032.794922 4831.438477 -2032.113525 4831.085449 -2032.786133</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_9c5a9154-5786-4539-90de-b45a991f6634">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_5b27ef17-cbab-4ba9-8c81-ad3ec659b94e">4830.543945 -2031.914795 4830.521973 -2031.901611 4830.966797 -2031.147461 4830.543945 -2031.914795</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_8b2e8068-9274-4582-b73f-1614f5c058d6">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ceee0208-02a9-4d09-a5b1-991013a7be1e">4830.543945 -2031.914795 4830.966797 -2031.147461 4830.989258 -2031.160767 4830.543945 -2031.914795</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d10c0210-4156-41f8-8dd2-6306f90cd404">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_22c2130a-d016-4e1f-8822-b674050a4715">4829.229492 -2034.14917 4829.228027 -2034.143799 4829.300781 -2034.183594 4829.229492 -2034.14917</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_feea300a-c118-41b3-a939-8a2259caf9d8">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c96d9950-5d74-4cdf-b5d9-5edd0fcce565">4831.09082 -2033.382446 2034.370483 4.984917641 4831.293945 -2032.938843 4831.09082 -2033.382446</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_3585dcdc-65b7-4804-ae1f-33c41e76ae2d">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_0c63fdc9-8df0-4903-8e53-ea67bdcd9f8f">4829.291016 -2034.148682 4829.405273 -2033.948853 4829.300781 -2034.183594 4829.291016 -2034.148682</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_09bb6567-6f85-4452-963d-a6ff6bc4def8">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_65a67a46-3609-4c2d-bd28-28f4eff94d98">4829.217773 -2034.108398 4829.228027 -2034.143799 4829.205566 -2034.130615 4829.217773 -2034.108398</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_fc45a2a7-7d66-491c-b7dc-a6e719933aa4">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_124b0437-836f-4fdc-a6b7-fd29266236f1">4831.858887 -2028.045288 4831.840332 -2028.004272 4831.87207 -2028.022217 4831.858887 -2028.045288</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
				</app:ParameterizedTexture>
			</app:surfaceDataMember>
			<app:surfaceDataMember>
				<app:ParameterizedTexture>
					<app:imageURI>54401008_tran_6697_appearance/1010.jpg</app:imageURI>
					<app:mimeType>image/jpg</app:mimeType>
					<app:target uri="#pol_525bfff6-e3e4-4cce-9a72-cd9b9c094e11">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_625893f2-684d-4f9f-bd0f-aed982b8065f">0 1 0.1894795881936367 0 1 0.2001567338274265 0 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_85279719-ea8f-4545-93e7-08b3fc9b6976">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_72637e71-ff20-464f-8968-c97d0369bf62">0 0.7459985341605255 0.9405011545350653 0 1 1 0 0.7459985341605255</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_02ebd709-d551-472e-bcd1-9f89a064a57c">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9d3c7da8-04fc-4692-acd4-d2efd1082e27">0 1 0.6612140564086567 0 1 0.10088805974853071 0 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d159ceee-ded7-449a-8c27-0c4689267618">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_e4e352e2-1ae2-4fbe-a5c1-1b062c9a0f6b">0.603071291412875 0 1 0.21541066975499457 0 1 0.603071291412875 0</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_156b5b4c-50c3-4bb3-91ae-70fbb5a2aa34">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_5bea01ca-b7fe-4c9a-969e-f2f68095427b">0 0.3634456755628861 1 0 0.03588942650348178 1 0 0.3634456755628861</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_b6a7658f-d5f1-4491-a65c-eb111f21a4ef">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_6566277f-c417-474a-9f5f-e0c267112895">1 0.012118533777542632 0.7588136886096295 1 0 0 1 0.012118533777542632</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_07d10b09-a1d1-4638-a212-8e2cc4fa8484">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_1e25d3b2-bca6-4033-a546-549abce48f76">0 1 0.3524824852612037 0 1 0.6164407259538853 0 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_e20af9cd-7950-4d13-95c9-9cdba1803173">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_82918cb0-c955-4eda-a2eb-dbc3665018f6">0.7082383188923368 0 1 1 0 0.8745178526018677 0.7082383188923368 0</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_0b4934db-8566-43c8-8a3b-0a841bb11463">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_c7682871-b64b-429b-88a3-7fd73d3244b1">1 1 0 0.5319889723403196 0.36364531903208874 0 1 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_6f6ef5ad-d410-4e1e-9796-c08f8a45c505">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_20286c77-f5a2-426e-af79-0d768a3df41a">0 0.9950842173642369 0.624586509091302 0 1 1 0 0.9950842173642369</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_1c506cd7-36d6-46a8-b71b-f52af9261442">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_63f3e130-ca40-4a17-b75c-c5371f5d7fac">0 1 1 0 0.7099746428935753 0.6511472573695051 0 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_4b48ef55-2fb6-400b-8053-30841a072954">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_3a0e121b-6167-47f1-8257-42e955fd75ee">0 0 1 0.3773770205183053 0.8315067648819336 1 0 0</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_be1d38dc-9793-431d-9fe5-f7712cdaffd1">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_4e84bd33-c2e3-4fa1-a7b2-01777cebdf36">0.1253621436646298 1 0 0.8447883669436468 1 0 0.1253621436646298 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_27af8fa7-f71c-47ca-a3fa-b465d81ac89b">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_13210c36-77a3-4cb4-898f-14e7fe619a5a">0 1 0.5682964122658086 0 1 0.23463559146337734 0 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_f74f49b6-522a-4a27-a573-0746a3108ce7">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_9392f7b1-e0a1-46c4-bef4-79842fe0f525">0 0.23172095462365241 1 0 0.4177834688281133 1 0 0.23172095462365241</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_7ba7a630-a738-4792-804f-5a637116ffc3">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_497e1962-5120-49f8-a4bd-f9c3d243288f">1 0 0.8427919844258933 1 0 0.744439884279112 1 0</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_a981ec94-5429-4d90-8acf-90d9df8da92b">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_bd2ded83-b267-4dff-ac08-c31d3245c8d8">0 1 0.4943243916313675 0 1 0.32123383133015576 0 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_1e9cc91b-79ca-4f61-948d-8cce0911f61c">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_022ac6df-a07f-4915-ad0e-2fc881e426ef">0 0.6351149885289892 1 0 0.533770088902392 1 0 0.6351149885289892</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_401376b3-ffff-4dc8-9c52-9e257fb85389">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_ce3d144e-d955-4b5b-8890-5fd11618c8b8">0 0 1 0.40776793443687187 0.5132082974266876 1 0 0</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_e0e29d5c-088d-4c02-bb18-713f4ebc1312">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_3b93b51d-1d79-4279-b384-046f4340bcec">0 1 0.8253241856171128 0 1 0.06562227587224989 0 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_a9d70515-2c1d-4f4d-82f3-01a989dd2b98">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_12dcc8b6-c5a9-4411-8179-6f079f69b1b0">1 0 0.5030553324271527 1 0 0.6751246448482363 1 0</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_d66dbf41-987c-4177-b5bb-c79daba09825">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_45c569b4-1e47-499a-b188-f9a396c34643">0 1 0.8181025600256019 0 1 0.05536269105824167 0 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_5f894980-ecef-4f9b-b64d-0982f2b0ff70">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_347dcbc9-77c1-4d9a-a8df-58ecdee58196">0 0 1 0.3422805193027539 0.35321744527436305 1 0 0</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
					<app:target uri="#pol_de73375f-5f33-42c9-baa1-ce567d79e4de">
						<app:TexCoordList>
							<app:textureCoordinates ring="#lin_188c6108-484b-4384-a296-fede71570c08">0.3240119368172478 1 0 0.8996072563983031 1 0 0.3240119368172478 1</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
				</app:ParameterizedTexture>
			</app:surfaceDataMember>
		</app:Appearance>
	</app:appearanceMember>
</core:CityModel>
