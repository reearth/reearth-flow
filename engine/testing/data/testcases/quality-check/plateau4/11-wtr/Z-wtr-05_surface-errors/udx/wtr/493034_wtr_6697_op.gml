<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>32.926 130.561 1.8</gml:lowerCorner>
      <gml:upperCorner>32.927 130.562 2.5</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <!-- ===================================================================== -->
  <!-- LOD1面のエラー: Non-Planar Surface (非平面性エラー)                    -->
  <!-- PlanarityFilter (threshold: 0.01m) で検出される                       -->
  <!-- 4頂点の面で、4番目の頂点のZ値を大きくずらして非平面にする              -->
  <!-- ===================================================================== -->
  <core:cityObjectMember>
    <wtr:WaterBody gml:id="wtr_lod1_nonplanar_surface_001">
      <core:creationDate>2024-03-22</core:creationDate>
      <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1020</wtr:class>
      <wtr:lod1MultiSurface>
        <gml:MultiSurface gml:id="wtr_lod1_ms_nonplanar_001">
          <gml:surfaceMember>
            <!-- 非平面な四角形面：4つの頂点がすべて同一平面上にない -->
            <!-- 正常な平面: z = 1.8 だが、4番目の頂点を z = 1.8 + 0.5 = 2.3 にずらす -->
            <!-- ずれ量 0.5m >> threshold 0.01m なので確実に検出される -->
            <gml:Polygon gml:id="wtr_lod1_poly_nonplanar_001">
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>
                    32.926 130.561 1.8
                    32.926 130.562 1.8
                    32.927 130.562 1.8
                    32.927 130.561 2.3
                    32.926 130.561 1.8
                  </gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </wtr:lod1MultiSurface>
      <uro:wtrDataQualityAttribute>
        <uro:DataQualityAttribute>
          <uro:lod>1</uro:lod>
        </uro:DataQualityAttribute>
      </uro:wtrDataQualityAttribute>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
