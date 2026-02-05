<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd  https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/10169" srsDimension="3">
			<gml:lowerCorner>157777.93049971716 21512.86490000118 0</gml:lowerCorner>
			<gml:upperCorner>158840.21719971678 21980.922699999373 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:Pipe gml:id="unf_c108f7a7-e80a-4d61-b31b-868874dabec4">
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
											<gml:posList>157811.28989971615 21918.21509999917 0 157809.41149971692 21919.276299999696 0 157809.1222997179 21919.564099999392 0 157808.87469971803 21919.888500000914 0 157808.67349971647 21920.243700000374 0 157807.29209971667 21923.121700001968 0 157807.06829971678 21923.510499999913 0 157806.7890997189 21923.8615000005 0 157806.46069971687 21924.166900000124 0 157806.09029971584 21924.419900000375 0 157805.686499717 21924.614899999717 0 157785.43469971645 21932.588499998623 0 157785.03529971675 21932.78109999988 0 157784.66849971842 21933.03030000078 0 157784.34249971702 21933.33090000055 0 157784.06449971764 21933.6762999996 0 157780.09849971812 21939.492700000195 0 157779.00869971688 21941.00470000035 0 157778.69089971678 21941.64710000008 0 157778.4552997167 21942.324100000405 0 157778.30609971704 21943.025100003233 0 157777.93049971716 21947.294500000266 0</gml:posList>
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
		<uro:Pipe gml:id="unf_0806f423-e098-4368-bf6d-ee21c70e696b">
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
											<gml:posList>158407.42729971715 21830.825099999198 0 158386.07929971616 21887.28090000009 0 158385.9639997163 21887.674499998775 0 158385.91469971693 21888.08159999945 0 158385.93259971737 21888.491400000497 0 158386.01729971694 21888.892699998745 0 158386.1663997168 21889.274700001824 0 158386.37599971727 21889.62720000067 0 158386.64039971586 21889.940699999443 0 158386.95249971843 21890.20679999917 0 158387.30389971624 21890.41829999996 0 158387.68519971703 21890.5694000012 0 158392.36559971664 21892.026300002915 0</gml:posList>
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
		<uro:Pipe gml:id="unf_6712a50f-5ed5-47b6-a734-2717913b6b9a">
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
											<gml:posList>158407.42729971715 21830.825099999198 0 158407.6141997174 21831.24900000073 0 158386.2489997163 21887.47309999941 0 158386.14979971838 21888.630600000924 0 158386.57979971773 21889.490499999258 0 158387.27429971626 21890.085799999644 0 158392.4336997177 21891.640200001195 0 158393.3482997191 21892.227700000636 0 158395.1619997167 21926.150099999548 0 158395.15199971787 21932.59509999876 0 158395.18999971676 21933.02919999884 0 158395.30279971685 21933.450099999154 0 158395.48699971664 21933.845099999526 0 158395.73689971588 21934.202000001784 0 158396.0450997165 21934.51019999978 0 158396.40199971735 21934.760100002088 0 158396.79699971757 21934.94430000254 0 158397.21789971666 21935.057100001828 0 158397.65199971752 21935.095099999005 0 158432.64319971748 21935.09520000091 0</gml:posList>
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
		<uro:Pipe gml:id="unf_5960e68a-1c16-4f40-8c06-ba119bf6c1b1">
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
											<gml:posList>158432.64319971748 21935.09520000091 0 158432.64319971768 21934.09520000246 0 158433.26509971742 21923.268400001063 0 158434.55649971755 21912.50099999916 0 158436.51229971743 21901.834200000954 0 158439.1251997169 21891.30910000045 0 158442.3849997159 21880.966000001004 0 158446.27929971696 21870.844700001267 0 158446.67689971585 21869.92720000087 0 158447.07449971684 21869.009600000794 0 158452.03149971765 21858.47580000106 0 158457.6415997159 21848.274699999707 0 158463.88249971787 21838.446899999864 0 158470.72949971553 21829.031300000417 0 158478.15529971736 21820.065300000897 0 158486.13069971616 21811.5842 0 158494.62399971616 21803.62190000078 0 158504.40539971783 21795.61499999972 0 158505.209299718 21795.020300002743 0 158514.89739971608 21789.026900002496 0 158524.8924997175 21783.56080000133 0 158535.16579971666 21778.63749999921 0 158536.12199971575 21778.34499999969 0 158537.07829971536 21778.052500001126 0 158545.55639971583 21775.6078000014 0 158554.18779971692 21773.77680000012 0 158562.9281997189 21772.56890000052 0 158571.73279971632 21771.99030000028 0 158580.55609971672 21772.043899999117 0 158589.352899717 21772.729600000563 0 158598.0779997167 21774.043699999547 0 158599.05209971764 21774.270100001446 0 158600.0260997164 21774.496400000502 0 158609.8252997173 21776.300300001556 0 158619.4549997165 21778.85889999931 0 158628.8572997174 21782.156600000602 0 158637.9753997175 21786.17360000242 0 158638.97539971728 21786.1736000011 0 158639.04139971785 21785.184100000137 0 158644.9943997169 21775.508100000643 0 158644.97969971685 21774.5185999998 0 158644.89139971646 21773.539599999833 0 158644.40359971617 21770.692499999885 0 158644.0007997178 21767.83209999997 0 158643.6834997149 21764.96090000129 0 158643.45189971663 21762.081600001067 0 158643.30619971626 21759.196599999963 0 158643.2465997163 21756.30859999934 0 158643.2729997168 21753.41999999974 0 158643.38559971735 21750.533600000756 0 158643.5840997168 21747.65179999936 0 158643.8683997167 21744.777199999433 0 158644.23819971606 21741.91230000048 0 158644.69329971587 21739.05970000115 0 158645.23319971675 21736.222000000977 0 158645.85749971686 21733.401600000274 0 158646.565499717 21730.601099999214 0 158647.35659971624 21727.822900000345 0 158648.23029971647 21725.06949999942 0 158649.01239971587 21721.9442999992 0 158649.87579971744 21718.840599998686 0 158650.8197997174 21715.76040000061 0 158651.84379971802 21712.705800000716 0 158652.94709971733 21709.67899999921 0 158654.12889971794 21706.682000000834 0 158655.38849971615 21703.716900000483 0 158656.72489971723 21700.785499999783 0 158658.13729971662 21697.88999999962 0 158658.64709971746 21696.889999999235 0 158658.64699971635 21695.890000002317 0 158660.65089971898 21689.19330000017 0 158661.99379971536 21682.333300000024 0 158662.66299971731 21675.37520000038 0 158662.65209971686 21668.385100000432 0 158662.65209971694 21667.385100002015 0 158675.0793997156 21666.328500000785 0 158686.41989971654 21664.569899999922 0 158697.6304997168 21662.116099999337 0 158708.6686997162 21658.976299999493 0 158719.49249971705 21655.16239999872 0 158720.43199971752 21654.82009999985 0 158721.3715997164 21654.477700001 0 158731.74189971678 21651.675300000767 0 158742.28079971645 21649.594900001506 0 158752.93809971766 21648.246599999613 0 158763.66309971694 21647.636699998788 0 158774.40459971677 21647.768199999977 0 158786.11139971737 21648.868200000186 0</gml:posList>
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
		<uro:Pipe gml:id="unf_dd4f982f-2f03-41f5-a8c3-d5b9ad249c40">
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
											<gml:posList>158840.18849971666 21602.921500000914 0 158840.21719971678 21598.28889999901 0</gml:posList>
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
		<uro:Pipe gml:id="unf_500c3529-dd70-4e75-988e-3a158c58f5f1">
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
											<gml:posList>158829.68729971672 21602.928800000034 0 158840.18849971666 21602.921500000914 0</gml:posList>
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
		<uro:Pipe gml:id="unf_0c789770-4341-4223-8ec1-3d0e89a24a13">
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
											<gml:posList>158791.73889971676 21651.3579000019 0 158821.47169971664 21651.357900000683 0 158821.64429971736 21651.37290000121 0 158821.8114997172 21651.417699999096 0 158821.9684997155 21651.490899999204 0 158822.1102997172 21651.59030000069 0 158822.23269971658 21651.712700000215 0 158822.3320997174 21651.854500000376 0 158822.40529971765 21652.011499999528 0 158822.45009971687 21652.17869999922 0 158822.46509971606 21652.3512999992 0 158822.4650997163 21653.049700000563 0 158822.86689971728 21654.04969999976 0 158823.8668997179 21653.74189999947 0 158823.9236997166 21653.741900000816 0 158823.9872997159 21653.747100001754 0 158824.04929971672 21653.762299999387 0 158824.10809971738 21653.787300003143 0 158824.16209971753 21653.82129999969 0 158824.20989971532 21653.863499999305 0 158824.25029971593 21653.912899999763 0 158824.28229971536 21653.968099999795 0 158825.9564997151 21657.476099998843 0 158825.9764997171 21657.526499999403 0 158825.9894997156 21657.579300001173 0 158825.99509971807 21657.633499999833 0 158826.7288997182 21678.644900000407 0 158826.28969971652 21679.64489999969 0 158826.43869971656 21694.86770000102 0 158826.4234997168 21695.040300001045 0 158826.37869971595 21695.207499998716 0 158826.30549971692 21695.364299999033 0 158826.2062997161 21695.50630000017 0 158826.08369971658 21695.628699999543 0 158825.94189971613 21695.728099999047 0 158825.7850997157 21695.801099999804 0 158825.61769971694 21695.845900000604 0 158825.44529971588 21695.861100000093 0 158820.86569971716 21695.861100003487 0 158819.86569971734 21696.12970000144 0 158819.88229971766 21699.733900002946 0 158819.99869971702 21700.733900001407 0</gml:posList>
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
		<uro:Pipe gml:id="unf_675f27fe-f91f-451d-bee6-085116c139b2">
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
											<gml:posList>158406.40139971685 21830.692500000012 0 158380.4813997168 21820.244500001307 0 158380.35149971693 21820.219400000857 0 158380.2191997174 21820.21729999889 0 158380.08859971765 21820.238300000827 0 158379.96359971722 21820.281900000573 0 158379.84819971805 21820.346499999763 0 158379.74579971627 21820.43040000288 0 158379.65969971716 21820.5308 0 158379.59239971603 21820.644700001667 0 158379.5459997168 21820.768599998966 0 158379.5218997167 21820.898700000147 0</gml:posList>
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
		<uro:Pipe gml:id="unf_f1e7e13b-b453-4704-be0f-2f7696d40917">
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
											<gml:posList>158787.59759971607 21651.534099999073 0 158787.55799971684 21653.78450000088 0 158820.35849971735 21653.78449999977 0 158821.866899717 21653.84130000036 0 158822.86689971728 21654.04969999976 0 158826.48949971487 21654.027300000293 0 158826.66309971767 21654.042499999334 0 158826.83149971633 21654.087700000982 0 158826.9894997159 21654.161299999123 0 158827.13229971714 21654.261300001395 0 158827.2556997165 21654.384700003273 0 158827.35549971717 21654.5273000008 0 158827.4292997171 21654.685299999997 0 158827.4742997163 21654.85370000087 0 158827.4894997168 21655.02729999948 0 158827.48949971597 21660.80949999951 0 158827.48449971713 21705.92089999915 0 158824.18049971765 21705.920900003068 0 158823.18049971678 21705.92069999895 0</gml:posList>
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
		<uro:Pipe gml:id="unf_b2ab5331-1f4a-446b-a159-adb74191343a">
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
											<gml:posList>158789.35269971704 21648.542700001275 0 158788.3850997163 21648.461300000065 0 158787.11149971696 21649.04129999886 0 158786.11149971755 21648.868300000144 0 158786.0082997175 21649.867700002917 0 158786.0082997171 21653.53310000037 0 158785.6108997159 21655.03329999931 0 158786.61089971618 21655.03350000003 0 158789.261699717 21649.544699999937 0 158789.38429971598 21648.54470000149 0 158790.38429971592 21648.27849999957 0 158819.48509971576 21648.27850000112 0 158819.65749971877 21648.29350000054 0 158819.82469971603 21648.33829999898 0 158819.98169971656 21648.411500000275 0 158820.12349971704 21648.51070000324 0 158820.24609971585 21648.63329999948 0 158820.34529971727 21648.775100001112 0 158820.41849971775 21648.932099998972 0 158820.4632997177 21649.099300000493 0 158820.47829971637 21649.271700001314 0 158820.35849971735 21653.78449999977 0</gml:posList>
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
		<uro:Pipe gml:id="unf_b2ab5331-1f4a-446b-a159-adb74191343b">
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
											<gml:posList>158812.75909971577 21512.86490000118 0 158812.759099717 21517.35750000316 0 158809.71489971684 21517.353299999584 0 158808.7148997169 21517.353299999813 0 158808.81849971542 21551.348100000974 0 158808.52509971656 21598.94970000021 0 158781.68509971676 21598.969900000506 0 158781.5968997158 21603.287300000888 0 158781.57649971658 21604.287300000986 0 158781.5764997171 21605.287300000295 0 158785.92589971633 21647.86830000014 0 158786.11149971755 21648.868300000144 0</gml:posList>
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
		<uro:Manhole gml:id="unf_bcf511b1-ca3a-4074-becb-0a019c44c576">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>157811.28989971615 21918.21509999917 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_9d583ace-d51b-4b6c-babd-29d56929d196">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158808.7148997169 21517.353299999813 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_56e4f5df-89c3-4e8d-8985-53394a79e788">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158812.759099717 21517.35750000316 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2022</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_100a76b1-1e73-49b3-9be2-b91dfc2cc312">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158808.81849971542 21551.348100000974 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_d803db61-f3b4-479a-8cc3-2212dbfd30d3">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158808.52509971656 21598.94970000021 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_f854a76b-1801-4efb-a772-9cf11a55278c">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158781.68509971676 21598.969900000506 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_1273c6fd-8eca-4ce3-8baf-e47e8290a6d4">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158781.57649971658 21604.287300000986 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_06a80a46-8fa8-4c89-b4c7-3d5ab6e727eb">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158785.6108997159 21655.03329999931 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_726f4a1e-f4d2-4d4a-a1d9-af1af93edc0c">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158819.99869971702 21700.733900001407 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_e8f813d1-2cb1-41be-8343-569ad21cb3f3">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158644.97969971685 21774.5185999998 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_4b9c13cb-df63-404c-abde-8dfbe857cf68">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158638.97539971728 21786.1736000011 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_fa8f6161-59d5-4197-a65e-3938d27dd9da">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158599.05209971764 21774.270100001446 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_c918c18a-cc32-45e7-aa8b-fe0c632368ae">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158536.12199971575 21778.34499999969 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_0a4f9b13-ee4d-4dea-a89d-c89a6418f245">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158432.64319971748 21935.09520000091 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_7f06f1ec-c469-4398-9d36-3ade8166c234">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158395.1619997167 21926.150099999548 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2013</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_92ccd4c8-80a1-4992-bab5-d15e6b9520a4">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158504.40539971783 21795.61499999972 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_cf42bb50-95aa-4f2c-8784-45b423b88ae4">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158446.67689971585 21869.92720000087 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_f382d83e-f34c-4e89-ad92-d68ffcf06f33">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158720.43199971752 21654.82009999985 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_3004c2d9-72fe-4e79-81be-0f9d4b2d3a1c">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158658.64709971746 21696.889999999235 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_9b5607d0-95d6-43ad-8d35-290d017f5eac">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158827.48949971597 21660.80949999951 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_5eddeabe-6894-413d-af61-8ad7f11e317e">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158662.65209971694 21667.385100002015 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_95acd93d-fde0-4372-ad1e-d6ab088401ce">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158827.48449971713 21705.92089999915 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_51ad2dc4-a169-4c25-a671-c4b753cb183a">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>157970.65139971726 21980.922699999373 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2022</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_bc224750-5248-4896-ab7e-6095b2f898ce">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>157896.1803997184 21936.19709999984 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_4b1101d3-25a0-4314-86e7-da6d6b6bec78">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>157852.72169971693 21960.688499999764 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2007</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_e06366d3-0732-4729-9f27-048189eaa2c8">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158823.1803997181 21705.920700000184 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2013</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_50d51835-8c93-48c2-875d-3b6ec0437789">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158840.21719971678 21598.28889999901 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2004</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_30153cf2-7c3e-4bd0-8ed7-d1e359890cc0">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158819.86569971734 21696.12970000144 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_6c08a1c0-a79c-413b-b99b-457885fe29e0">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158826.28959971643 21679.644800000584 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_90b55148-d98b-44fa-a52f-70ce4fe6c580">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158822.86689971844 21654.04959999969 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2022</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_dd5730f6-2736-4f4a-a53f-a7beb3523b97">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158820.35849971735 21653.78449999977 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_cf4b435e-fb6e-44de-9893-4825c25b1ee4">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158789.38429971598 21648.54470000149 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_baf78447-02c3-4ab5-adb6-3e4f05f7a347">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158786.11139971737 21648.868200000186 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2021</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<uro:Manhole gml:id="unf_fa70a2b9-e958-4a40-9e67-0603ecd5c912">
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
					<uro:meshCode>08EE751</uro:meshCode>
					<uro:geometryType codeSpace="../../codelists/Common_geometryType.xml">E5</uro:geometryType>
					<uro:mapLevel codeSpace="../../codelists/Common_MapLevel.xml">2500</uro:mapLevel>
					<uro:shapeType codeSpace="../../codelists/Common_shapeType.xml">0</uro:shapeType>
					<uro:lod0Geometry>
						<gml:Point>
							<gml:pos>158829.68729971672 21602.928800000034 0</gml:pos>
						</gml:Point>
					</uro:lod0Geometry>
				</uro:DmGeometricAttribute>
			</uro:frnDmAttribute>
			<uro:year>2006</uro:year>
		</uro:Manhole>
	</core:cityObjectMember>
</core:CityModel>
