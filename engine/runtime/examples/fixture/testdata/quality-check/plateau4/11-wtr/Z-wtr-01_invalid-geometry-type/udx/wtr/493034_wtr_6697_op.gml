<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>32.926 130.561 1.8</gml:lowerCorner>
      <gml:upperCorner>32.927 130.562 2.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <!-- 無効な空間属性の型: LOD1として認識されるがSolidジオメトリを持つ -->
  <core:cityObjectMember>
    <wtr:WaterBody gml:id="wtr_e5ccc228-245c-4ba1-bf11-094e272f26b7">
      <core:creationDate>2024-03-22</core:creationDate>
      <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1020</wtr:class>
      <!-- 非標準要素: lod1Solidを使用（LOD1だがSolid型） -->
      <wtr:lod1Solid>
        <gml:Solid gml:id="wtr_508ae261-0aff-4d58-a114-52a539ba45f4">
          <gml:exterior>
            <gml:CompositeSurface>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.562 1.8 32.927 130.562 1.8 32.927 130.561 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.927 130.561 2.0 32.927 130.562 2.0 32.926 130.562 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.561 2.0 32.926 130.562 2.0 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.561 1.8 32.927 130.562 1.8 32.927 130.562 2.0 32.927 130.561 2.0 32.927 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.562 1.8 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:CompositeSurface>
          </gml:exterior>
        </gml:Solid>
      </wtr:lod1Solid>
      <uro:wtrDataQualityAttribute>
        <uro:DataQualityAttribute>
          <uro:lod>1</uro:lod>
        </uro:DataQualityAttribute>
      </uro:wtrDataQualityAttribute>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
