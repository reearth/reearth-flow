<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>33.715 130.480 0</gml:lowerCorner>
			<gml:upperCorner>33.720 130.490 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<!--
		面の向き不正テストデータ (LOD0)

		座標の順序:
		- CW (時計回り/不正): SW→NW→NE→SE→SW (north→east→south→west)
		- CCW (反時計回り/正常): SW→SE→NE→NW→SW (east→north→west→south)
	-->
	<core:cityObjectMember>
		<gen:GenericCityObject gml:id="gen_incorrect_orientation_001">
			<gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
			<core:creationDate>2025-03-31</core:creationDate>
			<gen:stringAttribute name="告示番号">
				<gen:value>福岡県告示第316号</gen:value>
			</gen:stringAttribute>
			<gen:dateAttribute name="告示年月日">
				<gen:value>2019-09-27</gen:value>
			</gen:dateAttribute>
			<gen:stringAttribute name="名称">
				<gen:value>テスト指定区域（面の向き不正）</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="区域の所在">
				<gen:value>古賀市テスト町</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="区域の面積">
				<gen:value>50000</gen:value>
			</gen:stringAttribute>
			<gen:stringAttribute name="備考">
				<gen:value>都市計画法第34条第12号 集落活性化タイプ（テストデータ - 面の向き不正）</gen:value>
			</gen:stringAttribute>
			<gen:lod0Geometry>
				<gml:MultiSurface>
					<!--
						ポリゴン1: 時計回り(CW) - 不正な向き
						座標順: SW(33.715,130.480) → NW(33.716,130.480) → NE(33.716,130.481) → SE(33.715,130.481) → SW
					-->
					<gml:surfaceMember>
						<gml:Polygon gml:id="pol_incorrect_cw_001">
							<gml:exterior>
								<gml:LinearRing gml:id="lin_incorrect_cw_001">
									<gml:posList>33.715 130.480 0 33.716 130.480 0 33.716 130.481 0 33.715 130.481 0 33.715 130.480 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<!--
						ポリゴン2: 反時計回り(CCW) - 正常な向き
						座標順: SW(33.715,130.485) → SE(33.715,130.486) → NE(33.716,130.486) → NW(33.716,130.485) → SW
					-->
					<gml:surfaceMember>
						<gml:Polygon gml:id="pol_correct_ccw_001">
							<gml:exterior>
								<gml:LinearRing gml:id="lin_correct_ccw_001">
									<gml:posList>33.715 130.485 0 33.715 130.486 0 33.716 130.486 0 33.716 130.485 0 33.715 130.485 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</gen:lod0Geometry>
		</gen:GenericCityObject>
	</core:cityObjectMember>
</core:CityModel>
