<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsDimension="3" srsName="http://www.opengis.net/def/crs/EPSG/0/6697">
			<gml:lowerCorner>36.0821455602610 140.0997260017590 0.0000000000000</gml:lowerCorner>
			<gml:upperCorner>36.0921280621680 140.1129533346330 25.2788066900000</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
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
								<gml:Polygon gml:id="pol_4b48ef55-2fb6-400b-8053-30841a072955">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_3a0e121b-6167-47f1-8257-42e955fd75ef">
											<gml:posList>36.09144568715436 140.10149242939048 24.9856472 36.09144791380706 140.10150827002158 24.95467567 36.09145165170068 140.10150561500234 24.96961975 36.09144568715436 140.10149242939048 24.9856472</gml:posList>
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
								<gml:Polygon gml:id="pol_08d67e36-31d5-417b-825b-6f8647912a11">
									<gml:exterior>
										<gml:LinearRing gml:id="lin_f86dccf7-347e-454b-b188-4f3209f94492">
											<gml:posList>36.09116230360817 140.10160251482736 25.22502136 36.0911776944642 140.10159230817487 25.21017265 36.09117712518321 140.10159102663337 25.21017265 36.09116230360817 140.10160251482736 25.22502136</gml:posList>
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
							<!-- tran:AuxiliaryTrafficArea -->
							<gml:surfaceMember xlink:href="#pol_08d67e36-31d5-417b-825b-6f8647912a10"></gml:surfaceMember>
							<!-- The next line is commented out, which should result in a reference error -->
							<!-- <gml:surfaceMember xlink:href="#pol_08d67e36-31d5-417b-825b-6f8647912a11"></gml:surfaceMember> -->

							<!-- tran:TrafficArea -->
							<gml:surfaceMember xlink:href="#pol_4b48ef55-2fb6-400b-8053-30841a072954"></gml:surfaceMember>
							<!-- The next line is commented out, which should result in a reference error -->
							<!-- <gml:surfaceMember xlink:href="#pol_4b48ef55-2fb6-400b-8053-30841a072955"></gml:surfaceMember> -->
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
</core:CityModel>
