<?xml version="1.0" encoding="UTF-8"?>
<!--
  Second file of the instance count fixture for plateau6 05-luse-urf-area.

  Two luse:LandUse within imizu-shi mesh 54377075 (EPSG:6697), each a planar
  rectangle wound counter-clockwise in the inspection frame, so this file
  contributes a row of its own with a different count from the first file.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:luse="http://www.opengis.net/citygml/landuse/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.647500 137.052900 0</gml:lowerCorner>
			<gml:upperCorner>36.647630 137.052930 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<luse:LandUse gml:id="luse_1dee98bc-5951-476d-bf29-a5641a91cec6">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_94455e57-c4fa-4112-99ea-4ed8b64a0e49">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647500 137.052900 0 36.647500 137.052930 0 36.647530 137.052930 0 36.647530 137.052900 0 36.647500 137.052900 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<luse:class codeSpace="../../codelists/Common_landUseType.xml">202</luse:class>
		</luse:LandUse>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<luse:LandUse gml:id="luse_7fa7bbfa-b3bb-4027-bbd6-855ef7ef8591">
			<core:creationDate>2024-03-19T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon gml:id="poly_1d6b7325-560b-43a9-a2c0-34d884553076">
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.647600 137.052900 0 36.647600 137.052930 0 36.647630 137.052930 0 36.647630 137.052900 0 36.647600 137.052900 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
			<luse:class codeSpace="../../codelists/Common_landUseType.xml">204</luse:class>
		</luse:LandUse>
	</core:cityObjectMember>
</core:CityModel>
