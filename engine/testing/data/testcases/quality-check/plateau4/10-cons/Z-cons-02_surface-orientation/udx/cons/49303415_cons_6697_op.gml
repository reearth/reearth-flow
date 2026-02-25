<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>32.92647721145687 130.56240409979662 0</gml:lowerCorner>
      <gml:upperCorner>32.92967959516049 130.56521919330308 0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <!--
    面の向き不正テストデータ (LOD0)

    座標の順序:
    - CW (時計回り/不正): A→B→D→C→A
    - CCW (反時計回り/正常): A→C→D→B→A

    頂点:
      A = (32.929475562888015, 130.56507228215463)
      B = (32.929478752146466, 130.56506892376652)
      C = (32.929478700529216, 130.56507647174683)
      D = (32.92948188978779, 130.56507311335878)

    FMEワークフローではLOD0のみ面の向き検査(SurfaceValidator_2D)の対象
  -->
  <core:cityObjectMember>
    <uro:OtherConstruction gml:id="cons_6248259d-2fd7-4b8c-951e-f11973d5b935">
      <core:creationDate>2024-03-22</core:creationDate>
      <uro:consDataQualityAttribute>
        <uro:DataQualityAttribute>
          <uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod1>
        <uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod3>
  <uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
  <uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">5</uro:appearanceSrcDescLod3>
  <uro:lodType codeSpace="../../codelists/OtherConstruction_lodType.xml">3.0</uro:lodType>
  <uro:publicSurveyDataQualityAttribute>
    <uro:PublicSurveyDataQualityAttribute>
      <uro:srcScaleLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">3</uro:srcScaleLod3>
      <uro:publicSurveySrcDescLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">018</uro:publicSurveySrcDescLod3>
      </uro:PublicSurveyDataQualityAttribute>
  </uro:publicSurveyDataQualityAttribute>
</uro:DataQualityAttribute>
      </uro:consDataQualityAttribute>
      <uro:lod0Geometry>
        <gml:MultiSurface>
          <!--
            ポリゴン1: 時計回り(CW) - 不正な向き
            座標順: A→B→D→C→A
          -->
          <gml:surfaceMember>
            <gml:Polygon gml:id="poly_2c68f6dd-12a8-42ea-aff8-93c60448b0a1">
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>32.929475562888015 130.56507228215463 0 32.929478752146466 130.56506892376652 0 32.92948188978779 130.56507311335878 0 32.929478700529216 130.56507647174683 0 32.929475562888015 130.56507228215463 0</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </uro:lod0Geometry>
      <uro:class codeSpace="../../codelists/OtherConstruction_class.xml">09</uro:class>
      <uro:function codeSpace="../../codelists/OtherConstruction_function.xml">0902</uro:function>
      </uro:OtherConstruction>
  </core:cityObjectMember>
</core:CityModel>
