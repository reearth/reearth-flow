<?xml version="1.0" encoding="UTF-8"?>
<!--
  Real PLATEAU data, rewritten as CityGML 3.0 + i-UR 4.0: ajigasawa-machi mesh
  61400178, pref/nakamuragawa, largest assumed scale. Carried over from the
  plateau4 test of the same name, whose copy is the CityGML 2.0 original;
  geometry, gml:id values and coded values are unchanged, and only the encoding
  differs.

  Five wtr:WaterBody features, ranks 4, 2, 5, 1 and 3, hold 39 triangles between
  them in seven patches. 47 of the 82 edges are used once and all of them lie on
  the outline, so nothing is reported here either.
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
			<gml:lowerCorner>40.72807874714095 140.22869332274465 27.59299999999348</gml:lowerCorner>
			<gml:upperCorner>40.7333287521057 140.23516129642363 33.96360000000277</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_66d52d09-961c-44b5-9819-1b4b61daad84">
			<gml:name>中村川水系中村川洪水浸水想定区域（想定最大規模）</gml:name>
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
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_bd45efde-582b-4c69-b7c9-d661ae5bf315">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_4f5e03e7-2003-449d-8930-44c8bec9aa34">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574947591 140.2293803162423 28.72849999999744 40.72949575032972 140.2293173155251 28.015499999994063 40.729474749534184 140.22934881576023 28.090400000000955 40.72949574947591 140.2293803162423 28.72849999999744</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6a770d76-6905-43db-b6a5-2587e76b76be">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72945374963039 140.2293803159674 28.145300000003772 40.72949574947591 140.2293803162423 28.72849999999744 40.729474749534184 140.22934881576023 28.090400000000955 40.72945374963039 140.2293803159674 28.145300000003772</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ab96c658-3ac6-4792-a814-0bbdd83d7e77">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.73330825228416 140.23509879645212 28.55789999999979 40.7333287521057 140.23506729594897 28.498200000001816 40.733308252544354 140.23503629648098 28.495899999994435 40.73330825228416 140.23509879645212 28.55789999999979</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_51f9dde0-410a-4db1-9c62-8acac3b19c5b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.733308252544354 140.23503629648098 28.495899999994435 40.73326675228715 140.2350987964207 28.532200000001467 40.73330825228416 140.23509879645212 28.55789999999979 40.733308252544354 140.23503629648098 28.495899999994435</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_200b1fe6-ec4b-434d-95eb-40e2bccc9eae">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.733308251990124 140.23516129642363 28.536099999997532 40.73330825228416 140.23509879645212 28.55789999999979 40.73326675228715 140.2350987964207 28.532200000001467 40.733308251990124 140.23516129642363 28.536099999997532</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_47212227-442e-4c08-a540-64fe2c955788">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.733308251990124 140.23516129642363 28.536099999997532 40.73332875171642 140.23513029555207 28.504400000005262 40.73330825228416 140.23509879645212 28.55789999999979 40.733308251990124 140.23516129642363 28.536099999997532</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_fcbb6ce8-cef6-4b75-a3dc-b62a560d6c0b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.7333287521057 140.23506729594897 28.498200000001816 40.73330825228416 140.23509879645212 28.55789999999979 40.73332875171642 140.23513029555207 28.504400000005262 40.7333287521057 140.23506729594897 28.498200000001816</gml:posList>
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
		<wtr:WaterBody gml:id="fld_71eb7942-9a0d-4563-bbfc-fbbade236ada">
			<gml:name>中村川水系中村川洪水浸水想定区域（想定最大規模）</gml:name>
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
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_0d5c2374-5341-4402-b238-474c299bf9cb">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_730a66f7-c1e7-4dd6-aace-98124cc2128f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72947474987196 140.2289743158028 28.57839999999851 40.72949574999497 140.2290053153985 29.759799999999814 40.72949575048478 140.22894331621856 29.002600000007078 40.72947474987196 140.2289743158028 28.57839999999851</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_d9da2362-d3ae-4565-a68b-9097eef9ae07">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72947474987196 140.2289743158028 28.57839999999851 40.72949575048478 140.22894331621856 29.002600000007078 40.72945374974685 140.22894331741037 30.044099999999162 40.72947474987196 140.2289743158028 28.57839999999851</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3b3b64b1-2af7-481f-9efe-f638add7c2dc">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949575048478 140.22894331621856 29.002600000007078 40.72947475043095 140.2289118170591 29.426900000005844 40.72945374974685 140.22894331741037 30.044099999999162 40.72949575048478 140.22894331621856 29.002600000007078</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6070bf6f-9d83-40bd-a0aa-b7c6c35a9662">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574999497 140.2290053153985 29.759799999999814 40.72947474987196 140.2289743158028 28.57839999999851 40.729453750157326 140.22900531654318 30.015700000003562 40.72949574999497 140.2290053153985 29.759799999999814</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_16a7ba69-4c42-4556-974f-7058888d7a6f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949575048478 140.22894331621856 29.002600000007078 40.729495750205686 140.22888031669618 29.418799999999464 40.72947475043095 140.2289118170591 29.426900000005844 40.72949575048478 140.22894331621856 29.002600000007078</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9ba56457-1e7a-436e-abdf-fba502ce2508">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674962192 140.22959881586368 28.503800000005867 40.72951674961206 140.22953631590093 28.640400000003865 40.72949574974552 140.22956731648614 28.672600000005332 40.72951674962192 140.22959881586368 28.503800000005867</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_d0742583-63ee-43fc-8505-c3f261d65cbe">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674961206 140.22953631590093 28.640400000003865 40.729495749630466 140.22950531612688 28.67500000000291 40.72949574974552 140.22956731648614 28.672600000005332 40.72951674961206 140.22953631590093 28.640400000003865</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5e783b59-5ec2-4539-a811-64eb8648ddc3">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.729495749630466 140.22950531612688 28.67500000000291 40.72951674961206 140.22953631590093 28.640400000003865 40.729516749568354 140.22947381593858 28.72060000000056 40.729495749630466 140.22950531612688 28.67500000000291</gml:posList>
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
		<wtr:WaterBody gml:id="fld_ae2b9600-5408-4d3c-be44-0d72c6e2a549">
			<gml:name>中村川水系中村川洪水浸水想定区域（想定最大規模）</gml:name>
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
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_4bba0b62-b3b9-407d-a5dc-cb62c4d7962e">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8b48152e-2a77-4c29-82f2-f6c198a50fd3">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.7280787474996 140.22869332274465 33.695999999996275 40.72807874714095 140.22875532179148 33.96360000000277 40.72809974770414 140.22872432279237 33.863400000002 40.7280787474996 140.22869332274465 33.695999999996275</gml:posList>
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
		<wtr:WaterBody gml:id="fld_251458cc-3655-4268-b55d-4c6a59d628a6">
			<gml:name>中村川水系中村川洪水浸水想定区域（想定最大規模）</gml:name>
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
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_0458a4c5-03ee-4e1a-8564-ec70990803eb">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_30d142e5-be4d-4a78-89f7-da5fd1e809c1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949575037861 140.22906731575446 29.431299999996554 40.7295377493219 140.22906731584038 31.34189999999944 40.72951675010976 140.2290363150137 31.34189999999944 40.72949575037861 140.22906731575446 29.431299999996554</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_2fb7a5f5-9c21-4404-ae60-ff46fb14995b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574999497 140.2290053153985 29.759799999999814 40.72951675010976 140.2290363150137 31.34189999999944 40.729537749838535 140.22900531543726 31.34189999999944 40.72949574999497 140.2290053153985 29.759799999999814</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9206976e-9977-4125-a88a-778594b44bd1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574999497 140.2290053153985 29.759799999999814 40.72949575037861 140.22906731575446 29.431299999996554 40.72951675010976 140.2290363150137 31.34189999999944 40.72949574999497 140.2290053153985 29.759799999999814</gml:posList>
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
		<wtr:WaterBody gml:id="fld_ce616d52-b875-45fa-9837-5d9f0ddbf192">
			<gml:name>中村川水系中村川洪水浸水想定区域（想定最大規模）</gml:name>
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
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">2</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface gml:id="wtrs_d302de73-bf44-4b8d-a80b-a8a403c21f9d">
					<core:lod1MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_730fbecc-c5d7-415a-bfae-7d9831a0e7ee">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.729474749751304 140.22966131537544 28.690400000006775 40.729495749691345 140.2296923151894 28.674499999993714 40.72949574964963 140.2296303160131 28.614499999996042 40.729474749751304 140.22966131537544 28.690400000006775</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_dcb0be93-928d-48e4-a2da-0b7a6f5c8451">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.729474749751304 140.22966131537544 28.690400000006775 40.72949574964963 140.2296303160131 28.614499999996042 40.72945374980299 140.229630315581 28.78620000000228 40.729474749751304 140.22966131537544 28.690400000006775</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_a060b51f-68fe-478c-aa21-c412c7ea5e99">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72945374895007 140.22969231591026 28.677700000000186 40.729474749751304 140.22966131537544 28.690400000006775 40.72945374980299 140.229630315581 28.78620000000228 40.72945374895007 140.22969231591026 28.677700000000186</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9a148b04-aa48-4cb8-8551-db48639efd49">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72945374895007 140.22969231591026 28.677700000000186 40.729495749691345 140.2296923151894 28.674499999993714 40.729474749751304 140.22966131537544 28.690400000006775 40.72945374895007 140.22969231591026 28.677700000000186</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_b9c48c7b-3f6f-4c75-898a-be59de0dbe8b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949575037861 140.22906731575446 29.431299999996554 40.72945374963393 140.22906731568455 28.83879999999772 40.72947474965187 140.22909881606736 27.59299999999348 40.72949575037861 140.22906731575446 29.431299999996554</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_176b3146-1131-4727-a416-685a53f79bba">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574966129 140.22913031647002 27.8353000000061 40.72949575037861 140.22906731575446 29.431299999996554 40.72947474965187 140.22909881606736 27.59299999999348 40.72949574966129 140.22913031647002 27.8353000000061</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3beea375-6368-43df-b0e0-c76861437875">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.729453750157326 140.22900531654318 30.015700000003562 40.72947475027208 140.2290363161391 29.35910000000149 40.72949574999497 140.2290053153985 29.759799999999814 40.729453750157326 140.22900531654318 30.015700000003562</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_1d972be4-0a53-4396-9c83-252a7c3c4e37">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949575037861 140.22906731575446 29.431299999996554 40.72949574999497 140.2290053153985 29.759799999999814 40.72947475027208 140.2290363161391 29.35910000000149 40.72949575037861 140.22906731575446 29.431299999996554</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ea9f607c-2668-488b-b939-3bbcf95fa0be">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949575037861 140.22906731575446 29.431299999996554 40.72947475027208 140.2290363161391 29.35910000000149 40.72945374963393 140.22906731568455 28.83879999999772 40.72949575037861 140.22906731575446 29.431299999996554</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_e3eb4753-ce43-430b-a8bb-87352f7c218a">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72953774931497 140.2293803153331 28.48070000000007 40.72951674937945 140.22934881601515 29.288199999995413 40.72949574947591 140.2293803162423 28.72849999999744 40.72953774931497 140.2293803153331 28.48070000000007</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_a4d02c7e-e701-4d94-b9d9-3c2ef95621e1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574947591 140.2293803162423 28.72849999999744 40.72951674937945 140.22934881601515 29.288199999995413 40.72949575032972 140.2293173155251 28.015499999994063 40.72949574947591 140.2293803162423 28.72849999999744</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6fbf2240-7689-4610-bcad-9af3fabc3ae3">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574965193 140.22944231541697 28.76369999999588 40.72951674949082 140.22941131597668 28.80680000000575 40.72949574947591 140.2293803162423 28.72849999999744 40.72949574965193 140.22944231541697 28.76369999999588</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8d8ed700-4d8c-4b6a-be83-4f9035855e06">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574947591 140.2293803162423 28.72849999999744 40.72951674949082 140.22941131597668 28.80680000000575 40.72953774931497 140.2293803153331 28.48070000000007 40.72949574947591 140.2293803162423 28.72849999999744</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c7733ad7-0f5b-4bab-9ae7-18d0208c4eb2">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.7295582478991 140.23078631494198 28.496799999993527 40.7295167485149 140.23072381522343 28.661900000006426 40.72951674787558 140.23078631401023 28.61710000000312 40.7295582478991 140.23078631494198 28.496799999993527</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_bbeea680-50e5-40b3-9c68-ed83314dae7f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72951674787558 140.23078631401023 28.61710000000312 40.7295167485149 140.23072381522343 28.661900000006426 40.72949574823309 140.2307553150229 28.639500000004773 40.72951674787558 140.23078631401023 28.61710000000312</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3af1bb81-ae78-40f4-acf4-689c7a06fc37">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.7295167485149 140.23072381522343 28.661900000006426 40.7295582478991 140.23078631494198 28.496799999993527 40.72955824853199 140.23072381493253 28.479200000001583 40.7295167485149 140.23072381522343 28.661900000006426</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9d3e8378-8879-4f1a-a5c3-4b8be235bd2a">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72955824823048 140.23066131493164 28.50669999999809 40.7295167482137 140.23066131526133 28.85099999999511 40.7295167485149 140.23072381522343 28.661900000006426 40.72955824823048 140.23066131493164 28.50669999999809</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_e673b9a6-3c2a-45f1-98e5-5e5bf28e7f4d">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.7295167482137 140.23066131526133 28.85099999999511 40.72949574894304 140.23069231548845 28.796400000006543 40.7295167485149 140.23072381522343 28.661900000006426 40.7295167482137 140.23066131526133 28.85099999999511</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9b3d8e68-e145-4544-b41e-6869b049cb03">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72955824853199 140.23072381493253 28.479200000001583 40.72955824823048 140.23066131493164 28.50669999999809 40.7295167485149 140.23072381522343 28.661900000006426 40.72955824853199 140.23072381493253 28.479200000001583</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_00206190-3d12-4a3f-952e-25a5cbe2bff9">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>40.72949574894304 140.23069231548845 28.796400000006543 40.72949574823309 140.2307553150229 28.639500000004773 40.7295167485149 140.23072381522343 28.661900000006426 40.72949574894304 140.23069231548845 28.796400000006543</gml:posList>
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
