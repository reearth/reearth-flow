<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd  https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
			<gml:lowerCorner>157776.92629971774 21935.09520000091 0</gml:lowerCorner>
			<gml:upperCorner>158500.30049971672 22401.42169999933 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:Pipe gml:id="unf_9b61abf9-88c7-499f-b4a5-0865dd1ca6d6">
			<gml:name>高圧管路</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E2</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:MultiCurve>
							<gml:curveMembers>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>158185.42009971655 22195.222100003168 0 158183.64409971528 22194.465300001022 0 158182.60809971803 22194.363100000577 0 158181.55139971644 22194.26060000075 0 158178.89399971606 22187.022400001093 0 158176.94899971492 22179.560999998896 0 158175.73489971686 22171.94650000244 0 158175.26289971764 22164.250300000007 0 158175.53749971668 22156.544500000156 0 158176.80589971776 22147.901399999075 0 158175.34999971752 22132.34870000003 0 158176.1138997172 22131.70330000087 0 158180.29539971534 22125.61039999934 0 158185.57069971727 22121.801199999332 0 158187.79379971608 22119.253000000175 0 158189.52439971748 22116.628299999564 0 158190.89279971604 22113.744999999388 0 158192.06079971595 22109.747499998935 0 158193.00159971722 22105.557399999678 0 158193.22189971583 22101.30550000064 0 158189.42109971747 22087.818799999615 0 158186.0714997165 22076.749500000045 0 158182.9125997163 22067.37150000003 0 158180.41269971634 22063.38490000042 0 158176.56409971692 22061.408100000383 0 158172.3639997171 22060.56030000053 0 158171.36399971755 22060.560299999386 0 158170.36399971752 22060.560299999317 0 158169.14249971675 22059.781999998835 0 158167.87769971666 22059.07609999952 0 158166.57399971643 22058.4448999989 0 158165.23589971667 22057.890599999566 0 158163.8676997166 22057.415000000954 0 158162.47419971487 22057.01980000314 0 158161.06009971758 22056.706400000057 0 158159.63019971745 22056.47570000156 0 158158.18919971742 22056.328499998956 0 158139.14789971724 22059.978200002017 0 158137.42149971635 22060.547299999682 0 158135.66839971862 22061.028000000384 0 158133.89319971713 22061.4190000002 0 158132.1003997176 22061.719299999862 0 158130.2946997178 22061.928100003406 0 158128.4806997159 22062.044900000757 0 158126.66309971857 22062.069400003318 0 158124.84659971728 22062.00140000112 0 158114.41289971716 22058.874100002 0 158113.446999718 22058.615100001014 0 158112.48119971715 22058.356100001238 0 158107.3561997187 22056.173700000396 0 158106.4302997172 22055.60200000105 0 158105.5493997171 22054.963100003133 0 158104.71829971718 22054.260700002204 0 158103.94149971782 22053.498599999064 0 158103.22339971663 22052.68099999962 0 158102.56789971644 22051.81250000016 0 158101.9785997161 22050.897700002428 0 158101.45869971628 22049.941699999337 0 158101.01109971682 22048.949799999395 0 158100.63829971725 22047.927500002235 0 158100.34239971734 22046.880300000827 0 158100.12489971734 22045.814099999257 0 158099.98699971687 22044.734699999724 0 158099.9295997169 22043.64799999906 0 158099.9527997171 22042.560100000184 0 158100.05669971745 22041.47689999989 0 158104.79569971733 22021.712300001258 0 158104.90839971637 22020.880299999757 0 158104.94239971603 22020.04130000078 0 158104.897499718 22019.20289999963 0 158104.77389971845 22018.37230000237 0 158104.5729997192 22017.55709999886 0 158104.29629971684 22016.764300000254 0 158103.94629971666 22016.00110000257 0 158103.52619971763 22015.27410000004 0 158103.03969971673 22014.58970000115 0 158102.4909997171 22013.95409999985 0 158101.9469997176 22013.115099999257 0 158101.4326997183 22012.34240000064 0 158101.0068997167 22011.55740000007 0 158100.51279971824 22010.81359999915 0 158099.95429971628 22010.116799999898 0 158099.33599971558 22009.47250000122 0 158098.66269971675 22008.885900000696 0 158097.93979971748 22008.36160000081 0 158097.1729997179 22007.903900000092 0 158096.36849971596 22007.516400002143 0 158095.5325997169 22007.20210000014 0 158094.672099718 22006.963499999827 0 158093.79369971767 22006.80260000005 0 158092.90449971656 22006.72069999999 0 158092.01149971553 22006.718299999582 0 158091.12179971693 22006.795499999644 0 158090.2425997169 22006.951799999035 0 158089.3807997154 22007.185799999446 0 158088.54329971544 22007.495600000893 0 158087.73669971785 22007.87889999937 0 158086.96759971557 22008.332599999198 0 158063.51259971596 22018.811399999704 0 158061.13759971585 22020.046199999248 0 158058.72429971778 22021.204500002506 0 158056.27539971686 22022.28530000019 0 158053.79319971605 22023.287500000144 0 158051.28039971748 22024.210099999957 0 158048.73939971701 22025.052000003147 0 158046.1727997171 22025.812500000895 0 158043.58339971647 22026.490699999562 0 158040.9735997184 22027.086000000963 0 158033.6511997171 22027.153799999724 0 158032.62159971756 22027.087299998682 0 158029.62109971646 22027.08690000145 0 158027.84899971672 22026.60140000051 0 158026.10489971752 22026.022999999204 0 158024.393899717 22025.353199999383 0 158022.72069971717 22024.59389999896 0 158021.08989971774 22023.747299999544 0 158019.50609971827 22022.815700000687 0 158017.9737997162 22021.801700000004 0 158016.4970997166 22020.708199999375 0 158015.08039971595 22019.538200000927 0 157993.30109971718 21991.589799999405 0 157991.7088997185 21990.340000000266 0 157990.06609971751 21989.157600001425 0 157988.3754997168 21988.04450000276 0 157986.64009971792 21987.002699999506 0 157984.86289971776 21986.033900000242 0 157983.04689971887 21985.13989999954 0 157981.1952997169 21984.32229999919 0 157979.31129971726 21983.582300000835 0 157977.39809971693 21982.92140000045 0 157975.45909971712 21982.34060000099 0 157973.4975997178 21981.840899999806 0 157971.5170997174 21981.42329999981 0 157970.65139971726 21980.922699999373 0 157967.47319971715 21979.056499999428 0 157965.33249971602 21977.43010000291 0 157961.26609971712 21973.872099999775 0 157957.19959971728 21970.338799999518 0 157953.03419971696 21966.94460000201 0 157948.7185997167 21963.73810000031 0 157944.30579971778 21960.651499999054 0 157935.50379971665 21954.450799999664 0 157926.81469971724 21948.107400000834 0 157922.3575997177 21945.0672999992 0 157917.73379971736 21942.266099999717 0 157915.34039971628 21941.026999998892 0 157912.8829997168 21939.93790000055 0 157910.35529971647 21939.027899999754 0 157907.76659971717 21938.288399999605 0 157902.4871997162 21937.179799999278 0 157897.17079971803 21936.342299999167 0 157896.1803997184 21936.19709999984 0 157893.63749971628 21935.889400000582 0 157892.10739971843 21935.551699998996 0 157890.5642997176 21935.33700000123 0 157889.00109971742 21935.454000000245 0 157887.48809971655 21935.867000000417 0 157886.05799971736 21936.492000000213 0 157883.30119971762 21937.988299999666 0 157877.70829971827 21940.77349999908 0 157874.91709971626 21942.185600000284 0 157872.1637997166 21943.674299999835 0 157869.46829971703 21945.26209999969 0 157866.85049971737 21946.971699999798 0 157864.32929971584 21948.823200002596 0 157861.9155997172 21950.81609999952 0 157859.6164997167 21952.93959999959 0 157857.4383997177 21955.18249999994 0 157853.3573997168 21959.916100001865 0 157852.72169971693 21960.688499999764 0 157849.5024997166 21964.534299999345 0 157847.0136997167 21967.69390000062 0 157842.2585997179 21974.191499998895 0 157837.6701997164 21980.766000000483 0 157833.21129971728 21987.402800000167 0 157830.99159971662 21990.736000000143 0 157828.68089971738 21994.01579999906 0 157826.19459971716 21997.186300000627 0 157824.85859971744 21998.710599998867 0 157823.43989971682 22000.156700000825 0 157821.91759971788 22001.47439999936 0 157820.27089971697 22002.61339999954 0 157818.4836997162 22003.529299998532 0 157816.58279971828 22004.232900001585 0 157814.61809971876 22004.76439999951 0 157812.63479971825 22005.164000000423 0 157808.65489971667 22005.717500000028 0 157807.6548997165 22005.831900000136 0 157806.65489971763 22005.673699999617 0 157804.49349971686 22005.68150000104 0 157803.4013997172 22005.57199999885 0 157802.3316997184 22005.33930000016 0 157801.33559971655 22004.939899998888 0 157800.45199971725 22004.345900001434 0 157798.8759997177 22002.842100000707 0 157795.5992997174 22000.026499998672 0 157794.01379971762 21998.553899999584 0 157792.56729971743 21996.931900000152 0 157791.31409971614 21995.16320000121 0 157790.30449971792 21993.25960000301 0 157789.5326997176 21991.246099999236 0 157788.9252997178 21989.162800000253 0 157787.9014997173 21984.942900000722 0 157786.72639971762 21980.79700000059 0 157785.96029971694 21978.782899999525 0 157785.05569971845 21976.81680000123 0 157784.00669971743 21974.909500000624 0 157783.39039971572 21974.019999999196 0 157782.66609971662 21973.22430000073 0 157780.94719971763 21971.889199999732 0 157780.0891997161 21971.236399999885 0 157779.32719971726 21970.514000000658 0 157778.69359971813 21969.69600000069 0 157778.1785997175 21968.792299999328 0 157777.77059971727 21967.814899998917 0 157777.45789971767 21966.775399999126 0 157777.0715997169 21964.55769999914 0 157776.92629971774 21962.23370000178 0 157776.99729971885 21957.626900000218 0 157777.25379971776 21953.277700001396 0 157777.60479971772 21949.081699999133 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
							</gml:curveMembers>
						</gml:MultiCurve>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>1995</uro:year>
		</uro:Pipe>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Pipe gml:id="unf_76aa5e7b-ebfd-4d39-b75e-6ff7d3adb0fc">
			<gml:name>高圧管路</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E2</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:MultiCurve>
							<gml:curveMembers>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>158500.30049971672 22401.42169999933 0 158492.23109971665 22387.34750000009 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>158472.79829971696 22376.734500001174 0 158473.54629971736 22378.14369999885 0 158474.0000997163 22379.03489999921 0 158474.3798997168 22379.963700000586 0 158475.23269971693 22380.193700001582 0 158476.11009971623 22380.296700001538 0 158476.99289971814 22380.270099999547 0 158477.8624997181 22380.114500001735 0 158478.6998997168 22379.833499999495 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>158478.70109971522 22379.834099999476 0 158483.9312997166 22377.3322999992 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>158492.23109971665 22387.34750000009 0 158488.42069971847 22380.672499999506 0 158487.18709971808 22378.40490000203 0 158486.9554997166 22378.052500001173 0 158486.6680997174 22377.744100000597 0 158486.33269971688 22377.488499999283 0 158485.95929971721 22377.292700000915 0 158485.55829971645 22377.162700001765 0 158485.1410997174 22377.10170000309 0 158484.71949971633 22377.111900001215 0 158484.3056997167 22377.192499998997 0 158483.9114997166 22377.34170000011 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
							</gml:curveMembers>
						</gml:MultiCurve>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>1995</uro:year>
		</uro:Pipe>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Pipe gml:id="unf_b8777d61-0cc2-41b1-b4df-3a4476ae8a42">
			<gml:name>高圧管路</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E2</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:MultiCurve>
							<gml:curveMembers>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>158465.4241997182 22075.424700001135 0 158466.22119971752 22074.820600001294 0 158477.15209971744 22056.885000000602 0 158476.54679971692 22056.08899999892 0 158468.87959971678 22046.22380000013 0 158461.81799971664 22035.9165 0 158455.38729971644 22025.204200000426 0 158449.61079971702 22014.125399999717 0 158444.13439971718 22001.793000001966 0 158443.75949971782 22000.865899999368 0 158440.355099717 21990.39909999896 0 158437.56089971794 21979.753200002644 0 158435.38639971733 21968.963600003404 0 158433.83889971706 21958.066499999888 0 158432.92339971632 21947.098099999712 0 158432.64319971748 21935.09520000091 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
							</gml:curveMembers>
						</gml:MultiCurve>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>0001</uro:year>
		</uro:Pipe>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Pipe gml:id="unf_695cc71c-d8b3-4a5a-811f-888ad61e283c">
			<gml:name>高圧管路</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E2</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:MultiCurve>
							<gml:curveMembers>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>158202.54149971702 22189.444899998805 0 158203.42029971676 22188.967499999922 0 158203.8238997165 22188.49989999908 0 158204.18529971672 22188.069699998767 0 158204.50709971646 22187.674300000854 0 158204.79229971766 22187.31130000099 0 158205.04369971715 22186.978099999873 0 158205.26429971645 22186.671900000092 0 158205.45649971676 22186.39030000292 0 158205.6236997174 22186.130499999283 0 158205.89369971663 22185.666100001104 0 158206.09669971667 22185.257899999862 0 158206.39349971706 22184.52730000323 0 158206.679299717 22183.78850000046 0 158207.0036997171 22183.00470000001 0 158207.370099718 22182.184100001203 0 158207.7814997185 22181.33550000257 0 158208.24089971744 22180.467499999253 0 158208.75129971738 22179.58870000227 0 158209.31589971634 22178.70789999874 0 158209.93749971705 22177.83349999928 0 158210.61929971777 22176.97429999985 0 158211.36429971605 22176.1388999997 0 158212.17549971593 22175.335899999744 0 158213.05589971755 22174.57390000172 0 158213.94709971792 22174.120099999498 0 158248.6364997165 22157.124900000985 0 158249.52709971738 22156.670100000512 0 158288.09869971697 22138.55689999928 0 158288.9896997183 22138.10310000071 0 158327.15209971796 22120.385099999017 0 158371.57709971775 22099.83010000077 0 158408.2350997157 22082.62830000079 0 158409.50389971762 22082.1025000008 0 158410.78949971744 22081.600499998814 0 158412.107699716 22081.147100000533 0 158412.78109971716 22080.960300000424 0 158413.46069971687 22080.83110000332 0 158413.80189971632 22080.79610000064 0 158414.14349971773 22080.785099999 0 158414.82709971562 22080.835099998734 0 158416.19729971743 22081.048100000837 0 158416.88489971694 22081.11910000006 0 158417.5740997168 22081.15729999904 0 158418.9546997176 22081.17970000344 0 158419.8470997181 22081.15510000017 0 158420.84649971704 22081.120100002227 0 158430.29289971752 22078.89090000326 0 158439.6074997169 22076.162700003522 0 158448.7640997186 22072.943099999004 0 158457.73629971742 22069.241299999947 0 158466.49929971606 22065.067899999445 0 158467.15229971654 22064.885500000957 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
								<gml:Curve>
									<gml:segments>
										<gml:LineStringSegment>
											<gml:posList>158471.92809971582 22374.93210000046 0 158468.41669971627 22367.795899999364 0 158455.75209971797 22340.085100002892 0 158455.28209971567 22339.202500000792 0 158448.95209971615 22315.785100000063 0 158437.54309971604 22266.899300000132 0 158437.3170997173 22265.925099999135 0 158437.09109971675 22264.95090000273 0 158434.98689971707 22251.338300001553 0 158433.45269971783 22245.183299998964 0 158431.2468997166 22239.23549999967 0 158428.39649971755 22233.568499999274 0 158424.93649971677 22228.251899999374 0 158420.90949971628 22223.350499998844 0 158416.36489971794 22218.924899999864 0 158402.95209971684 22208.690100000724 0 158364.28209971706 22179.84929999881 0 158362.86809971658 22178.759500000622 0 158361.42389971737 22177.702700000234 0 158360.6808997174 22177.196900000497 0 158359.91889971783 22176.711699999774 0 158359.13429971624 22176.251300000284 0 158358.323099718 22175.819500000824 0 158356.64549971797 22175.017300000556 0 158355.81689971776 22174.607899999388 0 158355.0214997166 22174.165300000637 0 158354.27949971805 22173.668499999938 0 158353.93469971596 22173.39329999915 0 158353.61069971765 22173.09690000163 0 158353.31009971697 22172.77649999873 0 158353.0350997164 22172.42970000074 0 158352.5454997167 22171.67449999948 0 158352.00209971666 22170.8351000023 0 158351.45869971518 22169.995700002128 0 158339.74029971653 22145.533900000915 0 158327.57509971655 22121.291100002985 0 158327.15209971796 22120.385099999017 0</gml:posList>
										</gml:LineStringSegment>
									</gml:segments>
								</gml:Curve>
							</gml:curveMembers>
						</gml:MultiCurve>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>1995</uro:year>
		</uro:Pipe>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_5105c159-1c0e-459a-96ce-371bacc7ac75">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158444.13439971718 22001.793000001966 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_4414dd8d-f2f2-4f77-a342-433e6e4ea534">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158477.15209971744 22056.885000000602 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_ceb59c59-8c88-4d87-8a18-f8abf4e46583">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158467.15229971654 22064.885500000957 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2022</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_387aa027-e577-4981-ac44-872eeaad7061">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158419.8470997181 22081.15510000017 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_c28a54eb-7210-485c-9525-b8f87b2a48fb">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158371.57709971775 22099.83010000077 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2022</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_1e23d00f-2e67-4a65-8e11-73b4f6b88278">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158327.15209971796 22120.385099999017 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_5082cfb6-856a-4cb1-811a-f3f1944ea1a7">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158352.00209971666 22170.8351000023 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_b8c70a8e-c10e-425f-96db-e7897c3a1bcf">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158402.95209971684 22208.690100000724 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_8ee09a91-4b42-4041-b67e-d1e84e84c359">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158437.3170997173 22265.925099999135 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_bd053127-73c5-41be-81a2-5bbfce42f2eb">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158448.95209971615 22315.785100000063 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_9226ca83-56da-441e-a681-46a276b4301d">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158455.75209971797 22340.085100002892 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2022</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_f0a76dc3-63c8-498d-9def-2e7a8392bb43">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158468.41669971627 22367.795899999364 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_cb55c581-e394-4f9f-a8db-410ce4900232">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158474.0000997163 22379.03489999921 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_a6ab7ad5-3101-47b3-b82f-0a1181f6474e">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158488.42069971847 22380.672499999506 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_68a211ef-dbc2-49f5-b302-ce4a564cf611">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158288.9896997183 22138.10310000071 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2013</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_f49e0a1b-bddc-417d-aeac-06174f931b98">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158249.52709971738 22156.670100000512 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_212d53b8-6de4-42a2-bb48-643ac471df0f">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158213.94709971792 22174.120099999498 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_ef10d72f-a32f-4bbd-a149-ff40d6173773">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158185.42009971655 22195.222100003168 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_5998fd8a-9b9f-49d0-a14c-9f53bf046f0b">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158176.80589971776 22147.901399999075 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_bfe46591-f3b2-4040-a613-84ae690473c3">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158171.36399971755 22060.560299999386 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_f65a7d73-afb4-4e95-9600-86a2e7560ee7">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158113.446999718 22058.615100001014 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_a3c788f7-6573-4a16-91fd-50f0873bed4a">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158101.9469997176 22013.115099999257 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_cf282915-91df-451a-ac68-9f03026a3f56">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158032.62159971756 22027.087299998682 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_e216fb01-6d17-494a-abc2-46b33a3fb5fa">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158202.54149971702 22189.444899998805 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>1995</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_b34d36f2-4b38-4dda-83f5-20d62ba948ff">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>157807.6548997165 22005.831900000136 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_4d9308bc-8d12-428f-911e-58495325078f">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158500.30049971672 22401.42169999933 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2022</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_1020c876-46ff-4e80-86aa-098e467b1777">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158182.60809971803 22194.363100000577 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_8e0f7a16-3a57-4a7d-aab0-508ca82c8c19">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158176.1138997172 22131.70330000087 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_5463aa4a-5547-423a-a3ac-97ac54f2f958">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158465.4241997182 22075.424700001135 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2013</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_c246c5b4-57fd-4a7e-af3b-bc247ce2c08c">
			<gml:name>高圧その他</gml:name>
			<core:creationDate>2025-03-21</core:creationDate>
			<uro:frnDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">500</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">500</uro:thematicSrcDesc>
				</uro:DataQualityAttribute>
			</uro:frnDataQualityAttribute>
			<uro:frnDmAttribute>
				<uro:DmGeometricAttribute>
					<uro:dmCode codeSpace="../../codelists/Common_dmCode.xml">0001</uro:dmCode>
					<uro:meshCode>08EE752</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158492.2310997179 22387.347400000475 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
</core:CityModel>
