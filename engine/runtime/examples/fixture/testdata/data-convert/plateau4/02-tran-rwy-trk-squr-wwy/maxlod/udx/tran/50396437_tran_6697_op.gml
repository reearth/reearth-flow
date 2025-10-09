<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1  ../../schemas/iur/uro/3.1/urbanObject.xsd 
http://www.opengis.net/citygml/2.0  http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd 
http://www.opengis.net/citygml/landuse/2.0  http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd 
http://www.opengis.net/citygml/building/2.0  http://schemas.opengis.net/citygml/building/2.0/building.xsd 
http://www.opengis.net/citygml/transportation/2.0  http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd 
http://www.opengis.net/citygml/generics/2.0  http://schemas.opengis.net/citygml/generics/2.0/generics.xsd 
http://www.opengis.net/citygml/cityobjectgroup/2.0  http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd 
http://www.opengis.net/gml  http://schemas.opengis.net/gml/3.1.1/base/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>33.86582743337961 139.58820432055924 0</gml:lowerCorner>
			<gml:upperCorner>33.866489391450656 139.58998638537653 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_f5f1047f-d4ca-4859-92a8-20a5cc6f1224">
			<core:creationDate>2025-03-14</core:creationDate>
			<tran:class codeSpace="../../codelists/TransportationComplex_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">4</tran:function>
			<tran:usage codeSpace="../../codelists/Road_usage.xml">4</tran:usage>
			<tran:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>33.866489391450656 139.5885297403156 0 33.866482501240775 139.58846534158025 0 33.866428698243666 139.58847781681862 0 33.86641793270414 139.58841429387522 0 33.866380261421504 139.58842294011808 0 33.86634378317179 139.58843393918644 0 33.86635116061642 139.588479443271 0 33.86626982025398 139.58851847774315 0 33.86628851774733 139.58858078902816 0 33.86630978531041 139.5886206114638 0 33.86632058387307 139.58860998838645 0 33.86633715159695 139.5885987003385 0 33.86634300448388 139.5885947925998 0 33.86636426284306 139.588584572005 0 33.8663904825426 139.588575526176 0 33.866438338852205 139.5885651215811 0 33.866472423146696 139.58856632132594 0 33.86648338991386 139.5885677567367 0 33.866489391450656 139.5885297403156 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
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
		<tran:Road gml:id="tran_0ab83ff6-7a58-4a99-8f7f-0892699c063f">
			<core:creationDate>2025-03-14</core:creationDate>
			<tran:class codeSpace="../../codelists/TransportationComplex_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:usage codeSpace="../../codelists/Road_usage.xml">4</tran:usage>
			<tran:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>33.866394132086526 139.58823418505375 0 33.86636792519582 139.58820432055924 0 33.86634796719956 139.58823366847042 0 33.86633296495276 139.58826138094213 0 33.86632704619719 139.58827750242506 0 33.86632418998256 139.58829210197504 0 33.86632386435431 139.588309720657 0 33.86632652678791 139.588333707781 0 33.86633783356325 139.58839733718528 0 33.8663434016075 139.58843158403621 0 33.86634378317179 139.58843393918644 0 33.866380261421504 139.58842294011808 0 33.86637315995471 139.58838837337893 0 33.86636203672407 139.5883263646916 0 33.86636001531613 139.58830734760951 0 33.86636017630392 139.58829761955258 0 33.86636142591172 139.5882912390022 0 33.86636483363576 139.58828193400646 0 33.866377140659985 139.58825909302882 0 33.866390535872284 139.58823938338404 0 33.866394132086526 139.58823418505375 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
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
		<tran:Road gml:id="tran_8f1dccd3-5358-4a71-af0a-edb936cd00a3">
			<core:creationDate>2025-03-14</core:creationDate>
			<tran:class codeSpace="../../codelists/TransportationComplex_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">4</tran:function>
			<tran:usage codeSpace="../../codelists/Road_usage.xml">4</tran:usage>
			<tran:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>33.86582743337961 139.58995347129607 0 33.86589364610459 139.58996992832377 0 33.865959858826784 139.58998638537653 0 33.86596841697088 139.58993696674278 0 33.86597705550116 139.5898369645902 0 33.8659467666609 139.58979446552306 0 33.86596627936635 139.5896764905622 0 33.86597921841375 139.58960836092567 0 33.86599216020264 139.5895416363523 0 33.86600738212592 139.58948809152054 0 33.86601859693715 139.58945995780093 0 33.866033421481724 139.5894334350692 0 33.8660393540785 139.58942423105813 0 33.86604718267042 139.5894163186622 0 33.866055465072904 139.58940991815058 0 33.86606951706986 139.58940306889295 0 33.866091867511535 139.5893979253769 0 33.86613054050277 139.58939424861265 0 33.86619952611302 139.58939945662107 0 33.86621918620158 139.58940167047587 0 33.86624181825593 139.5894021465311 0 33.86625650845416 139.58939886222336 0 33.86627380611432 139.58939189563222 0 33.866287848696864 139.58938029067048 0 33.86630107552989 139.58936652633844 0 33.8663138487635 139.58935135819584 0 33.866326349793596 139.58933532614813 0 33.866335322684236 139.58931335949217 0 33.86634122081688 139.5892867539515 0 33.866343809227885 139.58927345220604 0 33.866347200759556 139.58925593288285 0 33.86635411361883 139.58919538594293 0 33.86636853376164 139.5890558079047 0 33.866370661226185 139.58899192394924 0 33.86637122572636 139.58895830808413 0 33.866368895990426 139.5889201609202 0 33.86636160490422 139.5888808389701 0 33.86634706322116 139.58882262292568 0 33.86633625595043 139.5887834191039 0 33.866314674621385 139.58872176447045 0 33.866300965788234 139.58867435454303 0 33.86629931173861 139.5886586870325 0 33.86630027242211 139.5886430120631 0 33.866303944654256 139.58863067997174 0 33.86630978531041 139.5886206114638 0 33.86628851774733 139.58858078902816 0 33.866271595166545 139.58859499636182 0 33.86626413114416 139.5886048533304 0 33.86625937459461 139.58861599958374 0 33.86625723535423 139.58862843537992 0 33.8662581708909 139.58864551002767 0 33.866265641091324 139.58868418289342 0 33.86628089363578 139.58873710070046 0 33.866302474528304 139.58879853914615 0 33.86631264636313 139.58883558307875 0 33.86632646051741 139.58889066672873 0 33.86633384476428 139.58893150157942 0 33.86633570953686 139.58896251648355 0 33.86633541209706 139.5889944022238 0 33.86633246245082 139.58905288427565 0 33.866318485205845 139.5891884618735 0 33.866314447086566 139.58924370447434 0 33.866309732001106 139.5892758189451 0 33.8663064218555 139.58928890657006 0 33.86630167016593 139.58930253871722 0 33.86629430010183 139.58931434087856 0 33.86628198089878 139.58933112899624 0 33.86626767831508 139.58934803083545 0 33.86625741275181 139.5893546531788 0 33.86624074324149 139.58936010480963 0 33.86622875302212 139.58936089550565 0 33.86620232597955 139.589356323056 0 33.866169408677756 139.5893525256605 0 33.86613018422471 139.58935090789186 0 33.86608709408953 139.58935502957667 0 33.86605924604251 139.58936137766588 0 33.866038798658586 139.58937137955678 0 33.866025744877376 139.58938146849775 0 33.86601323699511 139.5893940418145 0 33.86600406836182 139.5894082268905 0 33.86598717755267 139.58943853843118 0 33.8659738990415 139.58947175795547 0 33.865957423604826 139.58952973777255 0 33.86594412501632 139.5895984088446 0 33.86593100757073 139.58966751171639 0 33.865911405991696 139.58978613538616 0 33.8658470741948 139.58985462699872 0 33.865835991511005 139.58990405273553 0 33.86582743337961 139.58995347129607 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
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
</core:CityModel>