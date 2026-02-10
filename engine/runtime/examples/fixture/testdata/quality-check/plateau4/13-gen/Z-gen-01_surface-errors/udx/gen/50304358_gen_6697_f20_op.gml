<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>33.716 130.480 0</gml:lowerCorner>
      <gml:upperCorner>33.720 130.485 0.5</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <!-- ===================================================================== -->
  <!-- LOD1面のエラー: Non-Planar Surface (非平面性エラー)                    -->
  <!-- PlanarityFilter (threshold: 0.01m) で検出される                       -->
  <!-- 4頂点の面で、4番目の頂点のZ値を大きくずらして非平面にする              -->
  <!-- ===================================================================== -->
  <core:cityObjectMember>
    <gen:GenericCityObject gml:id="gen_lod1_nonplanar_surface_001">
      <gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
      <core:creationDate>2025-03-31</core:creationDate>
      <gen:stringAttribute name="告示番号">
        <gen:value>福岡県告示第316号</gen:value>
      </gen:stringAttribute>
      <gen:dateAttribute name="告示年月日">
        <gen:value>2019-09-27</gen:value>
      </gen:dateAttribute>
      <gen:stringAttribute name="名称">
        <gen:value>テスト指定区域</gen:value>
      </gen:stringAttribute>
      <gen:lod1Geometry>
        <gml:MultiSurface gml:id="gen_lod1_ms_nonplanar_001">
          <gml:surfaceMember>
            <!-- 非平面な四角形面：4つの頂点がすべて同一平面上にない -->
            <!-- 正常な平面: z = 0 だが、4番目の頂点を z = 0.5 にずらす -->
            <!-- ずれ量 0.5m >> threshold 0.01m なので確実に検出される -->
            <gml:Polygon gml:id="gen_lod1_poly_nonplanar_001">
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>
                    33.717 130.481 0
                    33.717 130.484 0
                    33.719 130.484 0
                    33.719 130.481 0.5
                    33.717 130.481 0
                  </gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </gen:lod1Geometry>
    </gen:GenericCityObject>
  </core:cityObjectMember>
</core:CityModel>
