<?xml version="1.0" encoding="UTF-8"?>
<!--
  Real PLATEAU data, rewritten as CityGML 3.0 + i-UR 4.0: ajigasawa-machi mesh
  61400178, pref/nakamuragawa, planning scale. Copied from the plateau4 test
  Z-fld-05_unshared-edge_04, whose fixture is the CityGML 2.0 original;
  geometry, gml:id values and coded values are unchanged, and only the encoding
  differs.

  Five wtr:WaterBody features, ranks 5, 3, 4, 1 and 2, hold 40 triangles between
  them in eight patches scattered over 750 m by 570 m. 50 of the 85 edges are
  used once, but every one of them runs along the outline of what the file
  covers, so the outline test removes them all and nothing is reported.

  The l2 file beside this one covers the same mesh at the largest assumed scale
  with its own triangles. The two scales are separate groups: matched against
  each other they would leave four edges over, and the run would report them.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>40.7265997410125 140.2283803202968 23.198199999998906</gml:lowerCorner>
			<gml:upperCorner>40.733328752105706 140.23516129642363 35.78759999999602</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_e60d7948-de61-429d-aa0f-92d76c514374">
			<gml:name>中村川水系中村川洪水浸水想定区域（計画規模）</gml:name>
			<core:creationDate>2026-03-31T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">1</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">5</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_569f07b0-44d5-4635-a06d-ac7a902c38fd">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5d1caae7-232e-4aee-be06-1b1299439153">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.728558248207236 140.22841132025545 35.78759999999602 40.72853774864779 140.2283803202968 34.07850000000326 40.72851674908753 140.22841132078352 35.518800000005285 40.728558248207236 140.22841132025545 35.78759999999602</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_ccbeadbf-ccdd-4760-96ad-f4afc8c450f4">
			<gml:name>中村川水系中村川洪水浸水想定区域（計画規模）</gml:name>
			<core:creationDate>2026-03-31T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">1</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">3</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_630d6912-c3cc-4988-8d92-5e4be523bf69">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_2bc0a111-3326-4dd7-8e10-b7d6315f259a">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574947592 140.2293803162423 26.57510000000184 40.72949575032972 140.2293173155251 26.038000000000466 40.729474749534184 140.22934881576023 26.11040000000503 40.72949574947592 140.2293803162423 26.57510000000184</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5de6a296-ab12-405a-a379-3482363937bd">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.729453749630395 140.2293803159674 26.165299999993294 40.72949574947592 140.2293803162423 26.57510000000184 40.729474749534184 140.22934881576023 26.11040000000503 40.729453749630395 140.2293803159674 26.165299999993294</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_61d6d363-e1fb-401a-be29-420003e01303">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72945374963393 140.22906731568455 27.8237999999983 40.72949575037862 140.22906731575446 28.737999999997555 40.72947475027208 140.2290363161391 29.30909999999858 40.72945374963393 140.22906731568455 27.8237999999983</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_cc4cfb8d-659a-4f9a-8a3e-3b7aa182c689">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949575037862 140.22906731575446 28.737999999997555 40.72949574999498 140.22900531539847 29.709799999996903 40.72947475027208 140.2290363161391 29.30909999999858 40.72949575037862 140.22906731575446 28.737999999997555</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_0a7c2b2d-3826-45d7-a7b9-f8be1d292402">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72945375015731 140.22900531654318 29.965700000000652 40.72947475027208 140.2290363161391 29.30909999999858 40.72949574999498 140.22900531539847 29.709799999996903 40.72945375015731 140.22900531654318 29.965700000000652</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_dd00fbb6-16c2-4def-9f83-123d81070cc5">
			<gml:name>中村川水系中村川洪水浸水想定区域（計画規模）</gml:name>
			<core:creationDate>2026-03-31T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">1</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">4</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_7392f856-1233-43d2-a0f6-ea2b30975236">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_bd679c2d-c9ab-4404-b1c4-5948c35aa6c4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.726641740879586 140.23234932562988 33.43039999999746 40.72662074074568 140.23231832629355 33.218800000002375 40.7265997410125 140.23234932583387 33.01119999999355 40.726641740879586 140.23234932562988 33.43039999999746</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_4484ba69-2dda-4c56-b8cd-689ce94de4fc">
			<gml:name>中村川水系中村川洪水浸水想定区域（計画規模）</gml:name>
			<core:creationDate>2026-03-31T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">1</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_1c80c300-fa58-4aae-a381-029e7ffe9260">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c0abe560-cc15-49cd-ac5a-04ef9d49e766">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949575037862 140.22906731575446 28.737999999997555 40.72953774932191 140.22906731584038 31.29189999999653 40.72951675010976 140.2290363150137 31.29189999999653 40.72949575037862 140.22906731575446 28.737999999997555</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_7a1bcde2-73ef-4864-883a-20d4e607ffee">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574999498 140.22900531539847 29.709799999996903 40.72951675010976 140.2290363150137 31.29189999999653 40.729537749838535 140.22900531543726 31.29189999999653 40.72949574999498 140.22900531539847 29.709799999996903</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3d22d693-6c0a-4eaf-bc2d-a3b9995fbe30">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574999498 140.22900531539847 29.709799999996903 40.72949575037862 140.22906731575446 28.737999999997555 40.72951675010976 140.2290363150137 31.29189999999653 40.72949574999498 140.22900531539847 29.709799999996903</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ca37270c-e4de-48d0-bfc0-a6601d4cd701">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72953774947626 140.22950531648004 26.180500000002212 40.729495749630466 140.22950531612688 26.17500000000291 40.72951674961205 140.2295363159009 26.140400000003865 40.72953774947626 140.22950531648004 26.180500000002212</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_7778afa5-83a2-4380-b731-9f443e7d8566">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574974552 140.22956731648617 26.172600000005332 40.72951674961205 140.2295363159009 26.140400000003865 40.729495749630466 140.22950531612688 26.17500000000291 40.72949574974552 140.22956731648617 26.172600000005332</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3b9bcaea-e286-4ee1-8eef-ed673a8e13fa">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72953774868485 140.22956731570264 26.068299999999 40.72951674961205 140.2295363159009 26.140400000003865 40.72949574974552 140.22956731648617 26.172600000005332 40.72953774868485 140.22956731570264 26.068299999999</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_d8b507b0-1298-44e5-b6dd-e3bfbc01f6a1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72955824853198 140.23072381493253 25.26919999999518 40.72951674851491 140.2307238152234 25.451900000000023 40.7295582478991 140.23078631494198 25.286800000001676 40.72955824853198 140.23072381493253 25.26919999999518</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_39e99629-aeb9-4517-b46a-314b28eecd0a">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.7295582478991 140.23078631494198 25.286800000001676 40.72951674851491 140.2307238152234 25.451900000000023 40.72951674787558 140.23078631401023 25.407099999996717 40.7295582478991 140.23078631494198 25.286800000001676</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5bb6d25f-4d7e-443a-a734-93e65dffc8eb">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674851491 140.2307238152234 25.451900000000023 40.72949574823308 140.2307553150229 25.42949999999837 40.72951674787558 140.23078631401023 25.407099999996717 40.72951674851491 140.2307238152234 25.451900000000023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_af051f1c-8618-459c-b5af-4f14606dbf83">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674821369 140.23066131526133 25.64100000000326 40.72949574894303 140.23069231548845 25.58969999999681 40.72951674851491 140.2307238152234 25.451900000000023 40.72951674821369 140.23066131526133 25.64100000000326</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_79a70041-4e06-4939-b5bd-5fa84087eb96">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674851491 140.2307238152234 25.451900000000023 40.72955824823048 140.23066131493164 25.29670000000624 40.72951674821369 140.23066131526133 25.64100000000326 40.72951674851491 140.2307238152234 25.451900000000023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c60f709c-52e4-4c62-ad33-33d84da1d8df">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674851491 140.2307238152234 25.451900000000023 40.72949574894303 140.23069231548845 25.58969999999681 40.72949574823308 140.2307553150229 25.42949999999837 40.72951674851491 140.2307238152234 25.451900000000023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_d68e2eb6-04fe-47f0-9f4a-1c4fa9f4b8e4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72955824853198 140.23072381493253 25.26919999999518 40.72955824823048 140.23066131493164 25.29670000000624 40.72951674851491 140.2307238152234 25.451900000000023 40.72955824853198 140.23072381493253 25.26919999999518</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_81e033de-0002-4abc-8eb5-43d20ab003a0">
			<gml:name>中村川水系中村川洪水浸水想定区域（計画規模）</gml:name>
			<core:creationDate>2026-03-31T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">1</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">2</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_7e126b14-444f-4386-aa01-a749c1395b98">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_db5761f6-67ed-418c-a903-66dd638d421e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.7294747497513 140.22966131537544 26.190400000006775 40.729495749691345 140.2296923151894 26.190400000006775 40.72949574964963 140.2296303160131 26.19210000000021 40.7294747497513 140.22966131537544 26.190400000006775</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_f3b4718e-165f-454e-8791-50d469944124">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.7294747497513 140.22966131537544 26.190400000006775 40.72949574964963 140.2296303160131 26.19210000000021 40.72945374980299 140.229630315581 26.28620000000228 40.7294747497513 140.22966131537544 26.190400000006775</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3e9b8b23-6024-46c7-8acb-15d020b5f5b5">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72945374895007 140.22969231591026 26.190400000006775 40.7294747497513 140.22966131537544 26.190400000006775 40.72945374980299 140.229630315581 26.28620000000228 40.72945374895007 140.22969231591026 26.190400000006775</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_bbe599a2-be57-41c2-8642-177c7ff16c8f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72945374895007 140.22969231591026 26.190400000006775 40.729495749691345 140.2296923151894 26.190400000006775 40.7294747497513 140.22966131537544 26.190400000006775 40.72945374895007 140.22969231591026 26.190400000006775</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8259d70b-ae04-4cd1-b9e6-256a498a5c54">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72947474987195 140.2289743158028 28.5283999999956 40.72949574999498 140.22900531539847 29.709799999996903 40.729495750484766 140.22894331621856 28.94770000000426 40.72947474987195 140.2289743158028 28.5283999999956</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c47c1db2-1358-4cb8-ba49-79e7b47672bc">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72947474987195 140.2289743158028 28.5283999999956 40.729495750484766 140.22894331621856 28.94770000000426 40.72945374974685 140.2289433174104 29.991599999993923 40.72947474987195 140.2289743158028 28.5283999999956</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6afa831f-8cee-401e-b96f-6e7f35ce6061">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.729495750484766 140.22894331621856 28.94770000000426 40.72947475043095 140.2289118170591 29.36689999999362 40.72945374974685 140.2289433174104 29.991599999993923 40.729495750484766 140.22894331621856 28.94770000000426</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_d6fb228e-cc8e-452b-b919-3e8abacf2510">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574999498 140.22900531539847 29.709799999996903 40.72947474987195 140.2289743158028 28.5283999999956 40.72945375015731 140.22900531654318 29.965700000000652 40.72949574999498 140.22900531539847 29.709799999996903</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3cc93b6f-bb41-41e3-9d23-0cb96362968c">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.729495750484766 140.22894331621856 28.94770000000426 40.72949575020569 140.2288803166962 29.36380000000645 40.72947475043095 140.2289118170591 29.36689999999362 40.729495750484766 140.22894331621856 28.94770000000426</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_0b34bd8b-945c-4aff-becd-54f88121f9e2">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72945374963393 140.22906731568455 27.8237999999983 40.72947474965188 140.22909881606736 25.612999999997555 40.72949575037862 140.22906731575446 28.737999999997555 40.72945374963393 140.22906731568455 27.8237999999983</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5731d7ec-7810-4475-839e-e9e0df70554b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674937944 140.22934881601515 27.308199999999488 40.72949575032972 140.2293173155251 26.038000000000466 40.72949574947592 140.2293803162423 26.57510000000184 40.72951674937944 140.22934881601515 27.308199999999488</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_f6b90408-ca2f-475b-8f5c-c8fea03e25f4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72953774931497 140.2293803153331 26.807499999995343 40.72949574947592 140.2293803162423 26.57510000000184 40.72951674949081 140.22941131597665 26.30680000000575 40.72953774931497 140.2293803153331 26.807499999995343</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_b04e956b-025a-4ccd-abd2-dd4eedfd018b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674949081 140.22941131597665 26.30680000000575 40.72949574947592 140.2293803162423 26.57510000000184 40.72949574965193 140.22944231541697 26.26369999999588 40.72951674949081 140.22941131597665 26.30680000000575</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_63315f70-c55c-413e-8aba-91a064b5aac7">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674937944 140.22934881601515 27.308199999999488 40.72949574947592 140.2293803162423 26.57510000000184 40.72953774931497 140.2293803153331 26.807499999995343 40.72951674937944 140.22934881601515 27.308199999999488</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_db2ae02b-bac0-49e0-adac-a48722e3c1d5">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72947474965188 140.22909881606736 25.612999999997555 40.72949574966129 140.22913031647002 25.642699999996694 40.72949575037862 140.22906731575446 28.737999999997555 40.72947474965188 140.22909881606736 25.612999999997555</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c1eade38-3b9f-4ac0-9323-434260314def">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.73326675228715 140.2350987964207 23.25220000000263 40.73330825228012 140.23509879645212 23.277900000000955 40.733308252544354 140.23503629648098 23.2158999999956 40.73326675228715 140.2350987964207 23.25220000000263</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_0fa4e7a0-d116-4ddb-b5ca-18d6e1a264cc">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.733328752105706 140.23506729594897 23.198199999998906 40.733308252544354 140.23503629648098 23.2158999999956 40.73330825228012 140.23509879645212 23.277900000000955 40.733328752105706 140.23506729594897 23.198199999998906</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_dbbc5698-b00e-4656-9ee3-2938ae1def11">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.73332875171642 140.2351302955521 23.201900000000023 40.733328752105706 140.23506729594897 23.198199999998906 40.73330825228012 140.23509879645212 23.277900000000955 40.73332875171642 140.2351302955521 23.201900000000023</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_90d6edef-129e-44e1-95b7-c29fdbf83f9d">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.733308251990124 140.23516129642363 23.246100000003935 40.73332875171642 140.2351302955521 23.201900000000023 40.73330825228012 140.23509879645212 23.277900000000955 40.733308251990124 140.23516129642363 23.246100000003935</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_a92de6b3-4540-4208-a790-8bed60d8d45b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.733308251990124 140.23516129642363 23.246100000003935 40.73330825228012 140.23509879645212 23.277900000000955 40.73326675228715 140.2350987964207 23.25220000000263 40.733308251990124 140.23516129642363 23.246100000003935</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
</core:CityModel>
