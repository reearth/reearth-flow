<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>32.926 130.561 1.8</gml:lowerCorner>
      <gml:upperCorner>32.927 130.562 2.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <!-- LOD2-3非水密立体: LOD2 Solidで底面が欠落している（5面のみ） -->
  <core:cityObjectMember>
    <wtr:WaterBody gml:id="wtr_not_closed_solid_001">
      <core:creationDate>2024-03-22</core:creationDate>
      <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1020</wtr:class>
      <!-- LOD2 Solid: 非水密立体（底面が欠落、5面のみ） -->
      <wtr:lod2Solid>
        <gml:Solid gml:id="solid-not-closed-001">
          <gml:exterior>
            <gml:CompositeSurface gml:id="composite-surface-not-closed-001">
              <!-- 上面 (z=2.0) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly-top-001">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.927 130.561 2.0 32.927 130.562 2.0 32.926 130.562 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- 左面 (lat=32.926) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly-left-001">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.561 2.0 32.926 130.562 2.0 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- 右面 (lat=32.927) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly-right-001">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.561 1.8 32.927 130.562 1.8 32.927 130.562 2.0 32.927 130.561 2.0 32.927 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- 前面 (lon=130.561) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly-front-001">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- 後面 (lon=130.562) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly-back-001">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.562 1.8 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- 底面は意図的に欠落（非水密状態を作成） -->
            </gml:CompositeSurface>
          </gml:exterior>
        </gml:Solid>
      </wtr:lod2Solid>
      <uro:wtrDataQualityAttribute>
        <uro:DataQualityAttribute>
          <uro:lod>2</uro:lod>
        </uro:DataQualityAttribute>
      </uro:wtrDataQualityAttribute>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
