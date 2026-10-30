<?xml version="1.0" encoding="UTF-8"?>
<!-- Cut from the sample: ranks 1 to 3, several bodies each. -->
<core:CityModel xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:tran="http://www.opengis.net/citygml/transportation/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:app="http://www.opengis.net/citygml/appearance/3.0" xmlns:dem="http://www.opengis.net/citygml/relief/3.0" xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:brid="http://www.opengis.net/citygml/bridge/3.0" xmlns:dyn="http://www.opengis.net/citygml/dynamizer/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:vers="http://www.opengis.net/citygml/versioning/3.0" xmlns:pcl="http://www.opengis.net/citygml/pointcloud/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0" xmlns:ct="urn:oasis:names:tc:ciq:ct:3" xmlns:luse="http://www.opengis.net/citygml/landuse/3.0" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:xAL="urn:oasis:names:tc:ciq:xal:3" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:urf="https://www.geospatial.jp/iur/urf/4.0" xsi:schemaLocation="http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd http://www.opengis.net/citygml/bridge/3.0 http://schemas.opengis.net/citygml/bridge/3.0/bridge.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd http://www.opengis.net/citygml/cityobjectgroup/3.0 http://schemas.opengis.net/citygml/cityobjectgroup/3.0/cityObjectGroup.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd http://www.opengis.net/citygml/dynamizer/3.0 http://schemas.opengis.net/citygml/dynamizer/3.0/dynamizer.xsd http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd http://www.opengis.net/citygml/pointcloud/3.0 http://schemas.opengis.net/citygml/pointcloud/3.0/pointCloud.xsd http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd http://www.opengis.net/citygml/versioning/3.0 http://schemas.opengis.net/citygml/versioning/3.0/versioning.xsd http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd https://www.geospatial.jp/iur/urc/4.0 ../../../schemas/iur/urc/4.0/urbanCore.xsd https://www.geospatial.jp/iur/urf/4.0 ../../../schemas/iur/urf/4.0/urbanFunction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../../schemas/iur/uro/4.0/urbanObject.xsd  http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.69774285 139.72546838 2.24</gml:lowerCorner>
			<gml:upperCorner>35.75002878 139.75001587 6.01</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="htd_18bc4ed2-9f52-48c6-bf0a-67a9de54fabd">
			<gml:name>東京都高潮浸水想定区域図（令和6年12月19日）</gml:name>
			<core:creationDate>2026-03-13T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../../codelists/HighTideRiskAttribute_rank.xml">2</urc:rank>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71030777 139.72950197 4.92 35.71035289 139.72955717 4.857 35.71035279 139.72944665 4.877 35.71030777 139.72950197 4.92</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71026275 139.72955729 5.517 35.71035289 139.72955717 4.857 35.71030777 139.72950197 4.92 35.71026275 139.72955729 5.517</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71026266 139.72944677 5.19 35.71026275 139.72955729 5.517 35.71030777 139.72950197 4.92 35.71026266 139.72944677 5.19</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71026266 139.72944677 5.19 35.71030777 139.72950197 4.92 35.71035279 139.72944665 4.877 35.71026266 139.72944677 5.19</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">3</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="htd_02c45520-013a-4f28-9801-d01eb83dd27d">
			<gml:name>東京都高潮浸水想定区域図（令和6年12月19日）</gml:name>
			<core:creationDate>2026-03-13T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../../codelists/HighTideRiskAttribute_rank.xml">2</urc:rank>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70598852 139.7383485 5.94 35.70603363 139.73840371 5.94 35.70603354 139.73829319 5.94 35.70598852 139.7383485 5.94</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70598852 139.7383485 5.94 35.70594349 139.73840381 5.94 35.70603363 139.73840371 5.94 35.70598852 139.7383485 5.94</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70598852 139.7383485 5.94 35.7059434 139.7382933 5.94 35.70594349 139.73840381 5.94 35.70598852 139.7383485 5.94</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70598852 139.7383485 5.94 35.70603354 139.73829319 5.94 35.7059434 139.7382933 5.94 35.70598852 139.7383485 5.94</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">3</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="htd_36008e9c-1f53-488c-8183-7a11495c9be4">
			<gml:name>東京都高潮浸水想定区域図（令和6年12月19日）</gml:name>
			<core:creationDate>2026-03-13T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../../codelists/HighTideRiskAttribute_rank.xml">3</urc:rank>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70873703 139.73751638 5.953 35.70882716 139.73751627 5.94 35.70878205 139.73746107 5.94 35.70873703 139.73751638 5.953</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70873703 139.73751638 5.953 35.70878205 139.73746107 5.94 35.70873694 139.73740586 5.955 35.70873703 139.73751638 5.953</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70882708 139.73740575 5.94 35.70873694 139.73740586 5.955 35.70878205 139.73746107 5.94 35.70882708 139.73740575 5.94</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70882716 139.73751627 5.94 35.70882708 139.73740575 5.94 35.70878205 139.73746107 5.94 35.70882716 139.73751627 5.94</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">3</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="htd_33dea9ad-217b-4aff-882a-e309cc044a1a">
			<gml:name>東京都高潮浸水想定区域図（令和6年12月19日）</gml:name>
			<core:creationDate>2026-03-13T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../../codelists/HighTideRiskAttribute_rank.xml">3</urc:rank>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70396512 139.74448415 5.51 35.70396504 139.74437364 5.527 35.70392001 139.74442894 5.51 35.70396512 139.74448415 5.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70387499 139.74448425 5.52 35.70396512 139.74448415 5.51 35.70392001 139.74442894 5.51 35.70387499 139.74448425 5.52</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70387499 139.74448425 5.52 35.70392001 139.74442894 5.51 35.7038749 139.74437374 5.52 35.70387499 139.74448425 5.52</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7038749 139.74437374 5.52 35.70392001 139.74442894 5.51 35.70396504 139.74437364 5.527 35.7038749 139.74437374 5.52</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">3</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="htd_0355c911-696a-41de-b6f1-6b12412e8f03">
			<gml:name>東京都高潮浸水想定区域図（令和6年12月19日）</gml:name>
			<core:creationDate>2026-03-13T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../../codelists/HighTideRiskAttribute_rank.xml">1</urc:rank>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7009479 139.74774739 4.85 35.70099293 139.74769209 4.85 35.7009028 139.74769218 4.85 35.7009479 139.74774739 4.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7009479 139.74774739 4.85 35.7009028 139.74769218 4.85 35.70090287 139.74780269 4.85 35.7009479 139.74774739 4.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7009479 139.74774739 4.85 35.70090287 139.74780269 4.85 35.70099301 139.74780259 4.85 35.7009479 139.74774739 4.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7009479 139.74774739 4.85 35.70099301 139.74780259 4.85 35.70099293 139.74769209 4.85 35.7009479 139.74774739 4.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">3</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="htd_9e915b02-1b7d-4597-848a-1c4402f30d17">
			<gml:name>東京都高潮浸水想定区域図（令和6年12月19日）</gml:name>
			<core:creationDate>2026-03-13T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../../codelists/HighTideRiskAttribute_rank.xml">1</urc:rank>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999561 139.73342573 5.78 35.71004073 139.73348094 5.78 35.71008575 139.73342562 5.76 35.70999561 139.73342573 5.78</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71008566 139.7333151 5.74 35.71008575 139.73342562 5.76 35.71013077 139.73337031 5.74 35.71008566 139.7333151 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71008566 139.7333151 5.74 35.71013077 139.73337031 5.74 35.7101758 139.73331499 5.74 35.71008566 139.7333151 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71008575 139.73342562 5.76 35.71013087 139.73348082 5.76 35.71013077 139.73337031 5.74 35.71008575 139.73342562 5.76</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71008575 139.73342562 5.76 35.71004073 139.73348094 5.78 35.71013087 139.73348082 5.76 35.71008575 139.73342562 5.76</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031086 139.73314905 5.74 35.71035589 139.73309373 5.74 35.71031077 139.73303853 5.74 35.71031086 139.73314905 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031086 139.73314905 5.74 35.71031077 139.73303853 5.74 35.71026575 139.73309384 5.74 35.71031086 139.73314905 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031086 139.73314905 5.74 35.71026575 139.73309384 5.74 35.71026584 139.73320436 5.74 35.71031086 139.73314905 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031086 139.73314905 5.74 35.71026584 139.73320436 5.74 35.71031096 139.73325956 5.74 35.71031086 139.73314905 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031086 139.73314905 5.74 35.71031096 139.73325956 5.74 35.71040109 139.73325945 5.74 35.71031086 139.73314905 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031086 139.73314905 5.74 35.71040109 139.73325945 5.74 35.710401 139.73314893 5.74 35.71031086 139.73314905 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031086 139.73314905 5.74 35.710401 139.73314893 5.74 35.71035589 139.73309373 5.74 35.71031086 139.73314905 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031096 139.73325956 5.74 35.71026584 139.73320436 5.74 35.71026593 139.73331488 5.74 35.71031096 139.73325956 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031096 139.73325956 5.74 35.71026593 139.73331488 5.74 35.71031105 139.73337008 5.74 35.71031096 139.73325956 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031096 139.73325956 5.74 35.71031105 139.73337008 5.74 35.71035607 139.73331477 5.74 35.71031096 139.73325956 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031096 139.73325956 5.74 35.71035607 139.73331477 5.74 35.71040109 139.73325945 5.74 35.71031096 139.73325956 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031105 139.73337008 5.74 35.71022091 139.73337019 5.74 35.71031114 139.7334806 5.74 35.71031105 139.73337008 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031105 139.73337008 5.74 35.71031114 139.7334806 5.74 35.71035616 139.73342528 5.74 35.71031105 139.73337008 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031105 139.73337008 5.74 35.71035616 139.73342528 5.74 35.71035607 139.73331477 5.74 35.71031105 139.73337008 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031105 139.73337008 5.74 35.71026593 139.73331488 5.74 35.71022091 139.73337019 5.74 35.71031105 139.73337008 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031114 139.7334806 5.74 35.71022091 139.73337019 5.74 35.710221 139.73348071 5.77 35.71031114 139.7334806 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.710401 139.73314893 5.74 35.71040109 139.73325945 5.74 35.71044611 139.73320414 5.74 35.710401 139.73314893 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.710401 139.73314893 5.74 35.71044611 139.73320414 5.74 35.71044602 139.73309362 5.74 35.710401 139.73314893 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.710401 139.73314893 5.74 35.71044602 139.73309362 5.74 35.71035589 139.73309373 5.74 35.710401 139.73314893 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71040109 139.73325945 5.74 35.71044621 139.73331465 5.74 35.71044611 139.73320414 5.74 35.71040109 139.73325945 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71040109 139.73325945 5.74 35.71035607 139.73331477 5.74 35.71044621 139.73331465 5.74 35.71040109 139.73325945 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7101758 139.73331499 5.74 35.71022091 139.73337019 5.74 35.71026593 139.73331488 5.74 35.7101758 139.73331499 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7101758 139.73331499 5.74 35.71013077 139.73337031 5.74 35.71022091 139.73337019 5.74 35.7101758 139.73331499 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022091 139.73337019 5.74 35.71013077 139.73337031 5.74 35.71013087 139.73348082 5.76 35.71022091 139.73337019 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022091 139.73337019 5.74 35.71013087 139.73348082 5.76 35.710221 139.73348071 5.77 35.71022091 139.73337019 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999571 139.73353625 5.85 35.7099958 139.73364677 5.95 35.71004082 139.73359145 5.92 35.70999571 139.73353625 5.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999571 139.73353625 5.85 35.71004082 139.73359145 5.92 35.71004073 139.73348094 5.78 35.70999571 139.73353625 5.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999571 139.73353625 5.85 35.71004073 139.73348094 5.78 35.70999561 139.73342573 5.78 35.70999571 139.73353625 5.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7099958 139.73364677 5.95 35.70999589 139.73375729 5.985 35.71004091 139.73370197 5.98 35.7099958 139.73364677 5.95</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7099958 139.73364677 5.95 35.71004091 139.73370197 5.98 35.71004082 139.73359145 5.92 35.7099958 139.73364677 5.95</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999589 139.73375729 5.985 35.70999598 139.7338678 5.997 35.710041 139.73381249 5.99 35.70999589 139.73375729 5.985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999589 139.73375729 5.985 35.710041 139.73381249 5.99 35.71004091 139.73370197 5.98 35.70999589 139.73375729 5.985</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981571 139.73386803 6 35.7098158 139.73397854 6 35.70986082 139.73392323 6 35.70981571 139.73386803 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981571 139.73386803 6 35.70986082 139.73392323 6 35.70990584 139.73386791 6 35.70981571 139.73386803 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70990584 139.73386791 6 35.70986082 139.73392323 6 35.70995096 139.73392312 6 35.70990584 139.73386791 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70990584 139.73386791 6 35.70995096 139.73392312 6 35.70999598 139.7338678 5.997 35.70990584 139.73386791 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999598 139.7338678 5.997 35.70995096 139.73392312 6 35.71004109 139.733923 6 35.70999598 139.7338678 5.997</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999598 139.7338678 5.997 35.71004109 139.733923 6 35.710041 139.73381249 5.99 35.70999598 139.7338678 5.997</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004082 139.73359145 5.92 35.71004091 139.73370197 5.98 35.71013105 139.73370186 5.98 35.71004082 139.73359145 5.92</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004082 139.73359145 5.92 35.71013105 139.73370186 5.98 35.71013096 139.73359134 5.89 35.71004082 139.73359145 5.92</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004082 139.73359145 5.92 35.71013096 139.73359134 5.89 35.71013087 139.73348082 5.76 35.71004082 139.73359145 5.92</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004082 139.73359145 5.92 35.71013087 139.73348082 5.76 35.71004073 139.73348094 5.78 35.71004082 139.73359145 5.92</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004091 139.73370197 5.98 35.710041 139.73381249 5.99 35.71013114 139.73381238 5.99 35.71004091 139.73370197 5.98</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004091 139.73370197 5.98 35.71013114 139.73381238 5.99 35.71013105 139.73370186 5.98 35.71004091 139.73370197 5.98</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.710041 139.73381249 5.99 35.71004109 139.733923 6 35.71013123 139.73392289 5.99 35.710041 139.73381249 5.99</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.710041 139.73381249 5.99 35.71013123 139.73392289 5.99 35.71013114 139.73381238 5.99 35.710041 139.73381249 5.99</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022119 139.73370175 5.87 35.7102663 139.73375695 5.87 35.71026621 139.73364643 5.865 35.71022119 139.73370175 5.87</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022119 139.73370175 5.87 35.71026621 139.73364643 5.865 35.71022109 139.73359123 5.86 35.71022119 139.73370175 5.87</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022119 139.73370175 5.87 35.71022109 139.73359123 5.86 35.71013096 139.73359134 5.89 35.71022119 139.73370175 5.87</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022119 139.73370175 5.87 35.71013096 139.73359134 5.89 35.71013105 139.73370186 5.98 35.71022119 139.73370175 5.87</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022119 139.73370175 5.87 35.71013105 139.73370186 5.98 35.71017616 139.73375706 5.947 35.71022119 139.73370175 5.87</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022119 139.73370175 5.87 35.71017616 139.73375706 5.947 35.7102663 139.73375695 5.87 35.71022119 139.73370175 5.87</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022109 139.73359123 5.86 35.710221 139.73348071 5.77 35.71013087 139.73348082 5.76 35.71022109 139.73359123 5.86</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022109 139.73359123 5.86 35.71013087 139.73348082 5.76 35.71013096 139.73359134 5.89 35.71022109 139.73359123 5.86</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022109 139.73359123 5.86 35.71026621 139.73364643 5.865 35.71026612 139.73353591 5.79 35.71022109 139.73359123 5.86</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71022109 139.73359123 5.86 35.71026612 139.73353591 5.79 35.710221 139.73348071 5.77 35.71022109 139.73359123 5.86</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71017616 139.73375706 5.947 35.71013114 139.73381238 5.99 35.71017625 139.73386758 5.99 35.71017616 139.73375706 5.947</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71017616 139.73375706 5.947 35.71013105 139.73370186 5.98 35.71013114 139.73381238 5.99 35.71017616 139.73375706 5.947</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71026612 139.73353591 5.79 35.71035625 139.7335358 5.74 35.71031114 139.7334806 5.74 35.71026612 139.73353591 5.79</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71026612 139.73353591 5.79 35.71031114 139.7334806 5.74 35.710221 139.73348071 5.77 35.71026612 139.73353591 5.79</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71035625 139.7335358 5.74 35.71035616 139.73342528 5.74 35.71031114 139.7334806 5.74 35.71035625 139.7335358 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71017625 139.73386758 5.99 35.71013114 139.73381238 5.99 35.71013123 139.73392289 5.99 35.71017625 139.73386758 5.99</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71017625 139.73386758 5.99 35.71013123 139.73392289 5.99 35.71017635 139.7339781 5.99 35.71017625 139.73386758 5.99</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70972584 139.73419969 6 35.70972594 139.73431021 6 35.70977096 139.73425489 6 35.70972584 139.73419969 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004109 139.733923 6 35.71008621 139.73397821 5.997 35.71013123 139.73392289 5.99 35.71004109 139.733923 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004119 139.73403352 6 35.71008621 139.73397821 5.997 35.71004109 139.733923 6 35.71004119 139.73403352 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004119 139.73403352 6 35.7100863 139.73408872 6 35.71008621 139.73397821 5.997 35.71004119 139.73403352 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004119 139.73403352 6 35.70995105 139.73403363 6 35.70995114 139.73414415 6 35.71004119 139.73403352 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004119 139.73403352 6 35.71004109 139.733923 6 35.70995096 139.73392312 6 35.71004119 139.73403352 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004128 139.73414404 6 35.7100863 139.73408872 6 35.71004119 139.73403352 6 35.71004128 139.73414404 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004128 139.73414404 6 35.71008639 139.73419924 6 35.7100863 139.73408872 6 35.71004128 139.73414404 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004128 139.73414404 6 35.70995114 139.73414415 6 35.70995123 139.73425467 6 35.71004128 139.73414404 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004128 139.73414404 6 35.71004119 139.73403352 6 35.70995114 139.73414415 6 35.71004128 139.73414404 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004137 139.73425456 6 35.71008639 139.73419924 6 35.71004128 139.73414404 6 35.71004137 139.73425456 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70990657 139.73475205 6 35.70986146 139.73469685 6 35.70986155 139.73480736 6 35.70990657 139.73475205 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004137 139.73425456 6 35.71008648 139.73430976 6 35.71008639 139.73419924 6 35.71004137 139.73425456 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004137 139.73425456 6 35.71004146 139.73436507 6 35.71008648 139.73430976 6 35.71004137 139.73425456 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004137 139.73425456 6 35.70995132 139.73436519 6 35.71004146 139.73436507 6 35.71004137 139.73425456 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004137 139.73425456 6 35.70995123 139.73425467 6 35.70995132 139.73436519 6 35.71004137 139.73425456 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70972594 139.73431021 6 35.70972603 139.73442072 6 35.70977105 139.73436541 6 35.70972594 139.73431021 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70972603 139.73442072 6 35.70981616 139.73442061 6 35.70977105 139.73436541 6 35.70972603 139.73442072 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981616 139.73442061 6 35.70981625 139.73453113 6 35.70986128 139.73447581 6 35.70981616 139.73442061 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981616 139.73442061 6 35.70986128 139.73447581 6 35.70986119 139.7343653 6 35.70981616 139.73442061 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981616 139.73442061 6 35.70986119 139.7343653 6 35.70977105 139.73436541 6 35.70981616 139.73442061 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7098158 139.73397854 6 35.70981589 139.73408906 6 35.70986091 139.73403375 6 35.7098158 139.73397854 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70972575 139.73408917 6 35.70977087 139.73414437 6 35.70981589 139.73408906 6 35.70972575 139.73408917 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981625 139.73453113 6 35.70981634 139.73464164 6 35.70986137 139.73458633 6 35.70981625 139.73453113 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981644 139.73475216 6 35.70981653 139.73486268 6 35.70986155 139.73480736 6 35.70981644 139.73475216 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981644 139.73475216 6 35.70986155 139.73480736 6 35.70986146 139.73469685 6 35.70981644 139.73475216 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70990657 139.73475205 6 35.70986155 139.73480736 6 35.70990666 139.73486257 6 35.70990657 139.73475205 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031068 139.73292801 5.74 35.71026556 139.73287281 5.74 35.71026566 139.73298333 5.74 35.71031068 139.73292801 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981653 139.73486268 6 35.70986164 139.73491788 6 35.70990666 139.73486257 6 35.70981653 139.73486268 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981653 139.73486268 6 35.70990666 139.73486257 6 35.70986155 139.73480736 6 35.70981653 139.73486268 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981653 139.73486268 6 35.70981662 139.7349732 6 35.70986164 139.73491788 6 35.70981653 139.73486268 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70990666 139.73486257 6 35.70986164 139.73491788 6 35.70990675 139.73497308 6 35.70990666 139.73486257 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981662 139.7349732 6 35.70990675 139.73497308 6 35.70986164 139.73491788 6 35.70981662 139.7349732 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031059 139.73281749 5.74 35.71026547 139.73276229 5.74 35.71026556 139.73287281 5.74 35.71031059 139.73281749 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981625 139.73453113 6 35.70986137 139.73458633 6 35.70986128 139.73447581 6 35.70981625 139.73453113 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981634 139.73464164 6 35.70981644 139.73475216 6 35.70986146 139.73469685 6 35.70981634 139.73464164 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981634 139.73464164 6 35.70986146 139.73469685 6 35.70990648 139.73464153 6 35.70981634 139.73464164 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981634 139.73464164 6 35.70990648 139.73464153 6 35.70986137 139.73458633 6 35.70981634 139.73464164 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999653 139.73453091 6 35.70999644 139.73442039 6 35.70995141 139.7344757 6 35.70999653 139.73453091 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999653 139.73453091 6 35.70995141 139.7344757 6 35.7099515 139.73458622 6 35.70999653 139.73453091 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999653 139.73453091 6 35.7099515 139.73458622 6 35.70999662 139.73464142 6 35.70999653 139.73453091 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70990648 139.73464153 6 35.70986146 139.73469685 6 35.70990657 139.73475205 6 35.70990648 139.73464153 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70990648 139.73464153 6 35.70999662 139.73464142 6 35.7099515 139.73458622 6 35.70990648 139.73464153 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70990648 139.73464153 6 35.7099515 139.73458622 6 35.70986137 139.73458633 6 35.70990648 139.73464153 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70999644 139.73442039 6 35.70995132 139.73436519 6 35.70995141 139.7344757 6 35.70999644 139.73442039 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7099515 139.73458622 6 35.70986128 139.73447581 6 35.70986137 139.73458633 6 35.7099515 139.73458622 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7099515 139.73458622 6 35.70995141 139.7344757 6 35.70986128 139.73447581 6 35.7099515 139.73458622 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995141 139.7344757 6 35.70986119 139.7343653 6 35.70986128 139.73447581 6 35.70995141 139.7344757 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995141 139.7344757 6 35.70995132 139.73436519 6 35.70986119 139.7343653 6 35.70995141 139.7344757 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004119 139.73403352 6 35.70995096 139.73392312 6 35.70995105 139.73403363 6 35.71004119 139.73403352 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995105 139.73403363 6 35.70995096 139.73392312 6 35.70986091 139.73403375 6 35.70995105 139.73403363 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995114 139.73414415 6 35.70995105 139.73403363 6 35.709861 139.73414426 6 35.70995114 139.73414415 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995123 139.73425467 6 35.70995114 139.73414415 6 35.709861 139.73414426 6 35.70995123 139.73425467 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995123 139.73425467 6 35.70986119 139.7343653 6 35.70995132 139.73436519 6 35.70995123 139.73425467 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995123 139.73425467 6 35.70986109 139.73425478 6 35.70986119 139.7343653 6 35.70995123 139.73425467 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995123 139.73425467 6 35.709861 139.73414426 6 35.70986109 139.73425478 6 35.70995123 139.73425467 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71008621 139.73397821 5.997 35.71017635 139.7339781 5.99 35.71013123 139.73392289 5.99 35.71008621 139.73397821 5.997</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981589 139.73408906 6 35.709861 139.73414426 6 35.70986091 139.73403375 6 35.70981589 139.73408906 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70981589 139.73408906 6 35.70977087 139.73414437 6 35.709861 139.73414426 6 35.70981589 139.73408906 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.7098158 139.73397854 6 35.70986091 139.73403375 6 35.70986082 139.73392323 6 35.7098158 139.73397854 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004137 139.73425456 6 35.71004128 139.73414404 6 35.70995123 139.73425467 6 35.71004137 139.73425456 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70977096 139.73425489 6 35.70972594 139.73431021 6 35.70977105 139.73436541 6 35.70977096 139.73425489 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.709861 139.73414426 6 35.70977087 139.73414437 6 35.70977096 139.73425489 6 35.709861 139.73414426 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70986109 139.73425478 6 35.709861 139.73414426 6 35.70977096 139.73425489 6 35.70986109 139.73425478 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70986109 139.73425478 6 35.70977105 139.73436541 6 35.70986119 139.7343653 6 35.70986109 139.73425478 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70986109 139.73425478 6 35.70977096 139.73425489 6 35.70977105 139.73436541 6 35.70986109 139.73425478 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995096 139.73392312 6 35.70986082 139.73392323 6 35.70986091 139.73403375 6 35.70995096 139.73392312 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70995105 139.73403363 6 35.70986091 139.73403375 6 35.709861 139.73414426 6 35.70995105 139.73403363 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004146 139.73436507 6 35.70995132 139.73436519 6 35.70999644 139.73442039 6 35.71004146 139.73436507 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004146 139.73436507 6 35.70999644 139.73442039 6 35.71008657 139.73442028 6 35.71004146 139.73436507 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71004146 139.73436507 6 35.71008657 139.73442028 6 35.71008648 139.73430976 6 35.71004146 139.73436507 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031059 139.73281749 5.74 35.71026556 139.73287281 5.74 35.7103557 139.7328727 5.74 35.71031059 139.73281749 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031059 139.73281749 5.74 35.7103557 139.7328727 5.74 35.71035561 139.73276218 5.74 35.71031059 139.73281749 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031059 139.73281749 5.74 35.71035561 139.73276218 5.74 35.71026547 139.73276229 5.74 35.71031059 139.73281749 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031068 139.73292801 5.74 35.71035579 139.73298321 5.74 35.7103557 139.7328727 5.74 35.71031068 139.73292801 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031068 139.73292801 5.74 35.7103557 139.7328727 5.74 35.71026556 139.73287281 5.74 35.71031068 139.73292801 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031068 139.73292801 5.74 35.71026566 139.73298333 5.74 35.71035579 139.73298321 5.74 35.71031068 139.73292801 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031077 139.73303853 5.74 35.71026566 139.73298333 5.74 35.71026575 139.73309384 5.74 35.71031077 139.73303853 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031077 139.73303853 5.74 35.71035589 139.73309373 5.74 35.71035579 139.73298321 5.74 35.71031077 139.73303853 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.71031077 139.73303853 5.74 35.71035579 139.73298321 5.74 35.71026566 139.73298333 5.74 35.71031077 139.73303853 5.74</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70972584 139.73419969 6 35.70977087 139.73414437 6 35.70972575 139.73408917 6 35.70972584 139.73419969 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70972584 139.73419969 6 35.70977096 139.73425489 6 35.70977087 139.73414437 6 35.70972584 139.73419969 6</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">3</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="htd_04990f51-d654-40ea-af8f-c91b675f1210">
			<gml:name>東京都高潮浸水想定区域図（令和6年12月19日）</gml:name>
			<core:creationDate>2026-03-13T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:HighTideRiskAttribute>
					<urc:description codeSpace="../../../codelists/HighTideRiskAttribute_description.xml">2</urc:description>
					<urc:rank codeSpace="../../../codelists/HighTideRiskAttribute_rank.xml">1</urc:rank>
				</urc:HighTideRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70130523 139.74332679 5.44 35.70135025 139.74327149 5.44 35.70126012 139.74327159 5.44 35.70130523 139.74332679 5.44</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70130523 139.74332679 5.44 35.70135034 139.743382 5.44 35.70135025 139.74327149 5.44 35.70130523 139.74332679 5.44</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70130523 139.74332679 5.44 35.7012602 139.7433821 5.44 35.70135034 139.743382 5.44 35.70130523 139.74332679 5.44</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.70130523 139.74332679 5.44 35.70126012 139.74327159 5.44 35.7012602 139.7433821 5.44 35.70130523 139.74332679 5.44</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../codelists/WaterBody_function.xml">3</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
</core:CityModel>
