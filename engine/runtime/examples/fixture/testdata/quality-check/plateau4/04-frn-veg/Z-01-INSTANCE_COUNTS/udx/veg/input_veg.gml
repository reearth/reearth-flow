<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
<gml:boundedBy>
<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
<gml:lowerCorner>36.083279283543504 140.0771528965646 23.3</gml:lowerCorner>
<gml:upperCorner>36.08495965372956 140.08253189866358 33.458</gml:upperCorner>
</gml:Envelope>
</gml:boundedBy>
<core:cityObjectMember>
<veg:PlantCover gml:id="veg_763cbf45-83f3-4f0d-9b3c-f752323bccce">
			<core:creationDate>2024-03-19</core:creationDate>
			<uro:vegDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">100</uro:thematicSrcDesc>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod2>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">5</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod3>
							<uro:publicSurveySrcDescLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod3>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:vegDataQualityAttribute>
<veg:class codeSpace="../../codelists/PlantCover_class.xml">3</veg:class>
<veg:averageHeight uom="m">0.3</veg:averageHeight>
<veg:lod3MultiSurface>
<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
<gml:surfaceMember>
<gml:Polygon gml:id="fme-gen-5077c118-5770-449e-8ce7-5bf895bc408d">
<gml:exterior>
<gml:LinearRing gml:id="fme-gen-5077c118-5770-449e-8ce7-5bf895bc408d_0">
<gml:posList>36.08384660825188 140.0796782969165 24.05 36.08384835148443 140.07968087843255 24.05 36.08384835148443 140.07968087843255 23.668 36.08384660825188 140.0796782969165 24.05</gml:posList>
</gml:LinearRing>
</gml:exterior>
</gml:Polygon>
</gml:surfaceMember>
</gml:MultiSurface>
</veg:lod3MultiSurface>
</veg:PlantCover>
</core:cityObjectMember>
<app:appearanceMember>
<app:Appearance>
<app:theme>rgbTexture</app:theme>
<app:surfaceDataMember>
<app:X3DMaterial>
<app:diffuseColor>1 1 1</app:diffuseColor>
<app:specularColor>0.698 0.698 0.698</app:specularColor>
<app:shininess>0.08159999999999999</app:shininess>
<app:target>#fme-gen-5077c118-5770-449e-8ce7-5bf895bc408d</app:target>
</app:X3DMaterial>
</app:surfaceDataMember>
<app:surfaceDataMember>
<app:ParameterizedTexture>
<app:imageURI>54401006_veg_6697_appearance/col0_1_000000_Leaf_jpg.jpg</app:imageURI>
<app:mimeType>image/jpg</app:mimeType>
<app:target uri="#fme-gen-5077c118-5770-449e-8ce7-5bf895bc408d">
<app:TexCoordList>
<app:textureCoordinates ring="#fme-gen-5077c118-5770-449e-8ce7-5bf895bc408d_0">0 0 0.09325999767 0.06268399954 0.01420199964 0.1803040057 0 0</app:textureCoordinates>
</app:TexCoordList>
</app:target>
</app:ParameterizedTexture>
</app:surfaceDataMember>
</app:Appearance>
</app:appearanceMember>
</core:CityModel>