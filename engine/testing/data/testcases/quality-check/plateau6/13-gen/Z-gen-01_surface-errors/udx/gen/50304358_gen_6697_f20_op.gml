<?xml version="1.0" encoding="UTF-8"?>
<!--
  Non-planar faces of generic city objects (EPSG:6697).
  - GenericThematicSurface: a LOD1 face whose fourth vertex is 0.5 m higher.
  - GenericOccupiedSpace: a LOD3 face whose third vertex is 0.5 m higher.
  - frn:CityFurniture: the same kind of face at LOD2. It is not a generic city
    object, so the gen checks neither check nor count it.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:gen="http://www.opengis.net/citygml/generics/3.0"
	xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd
http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>33.717 130.481 0</gml:lowerCorner>
			<gml:upperCorner>33.719 130.484 2.5</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<gen:GenericThematicSurface gml:id="gen_4c5d6e7f-8091-4a2b-b3c4-d5e6f708192a">
			<gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
			<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>33.717 130.481 0 33.717 130.484 0 33.719 130.484 0 33.719 130.481 0.5 33.717 130.481 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod1MultiSurface>
		</gen:GenericThematicSurface>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<gen:GenericOccupiedSpace gml:id="gen_5d6e7f80-91a2-4b3c-84d5-e6f708192a3b">
			<gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
			<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod3MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>33.7175 130.4815 2 33.7175 130.4816 2 33.7176 130.4816 2.5 33.7176 130.4815 2 33.7175 130.4815 2</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod3MultiSurface>
		</gen:GenericOccupiedSpace>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<frn:CityFurniture gml:id="gen_6e7f8091-a2b3-4c4d-95e6-f708192a3b4c">
			<core:creationDate>2025-03-31T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:lod2MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>33.7178 130.4815 1 33.7178 130.4816 1 33.7179 130.4816 1.5 33.7179 130.4815 1 33.7178 130.4815 1</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod2MultiSurface>
		</frn:CityFurniture>
	</core:cityObjectMember>
</core:CityModel>
