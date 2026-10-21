<?xml version="1.0" encoding="UTF-8"?>
<!--
  Duplicate gml:id across files.
  The valid water body with every gml:id except that of the wtr:WaterBody
  changed, so only that gml:id is also used in 493034_wtr_6697_op.gml.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>32.926 130.561 1.8</gml:lowerCorner>
      <gml:upperCorner>32.927 130.562 2.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <wtr:WaterBody gml:id="wtr_7fe93243-d2c1-4137-921f-09e6f206d5e1">
      <core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
      <core:adeOfAbstractCityObject>
        <urc:DataQualityAttribute>
          <urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
          <urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
        </urc:DataQualityAttribute>
      </core:adeOfAbstractCityObject>
      <core:boundary>
        <wtr:WaterSurface gml:id="wtrs_6b28f965-2211-45f0-ab88-68ff2aeee0e8">
          <core:lod1MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_c66c2ff4-2f87-47a1-977a-dbdd624799e5">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.561 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod1MultiSurface>
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_886c133b-24b9-417b-ba57-f72fc68da048">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.561 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_202f0a43-b428-4d45-86e5-6880b719228f">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.561 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </wtr:WaterSurface>
      </core:boundary>
      <core:boundary>
        <wtr:WaterGroundSurface gml:id="wtrg_2790ef2c-8337-4e1b-932f-8721ff96648c">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_d037c352-fa58-44af-a523-3c256b55747c">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.562 1.8 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_de1d7638-0183-419b-80e6-df7343c9bc98">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.562 1.8 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </wtr:WaterGroundSurface>
      </core:boundary>
      <core:boundary>
        <core:ClosureSurface gml:id="clos_e1b733fd-d72e-4546-99ea-18de3a4276b0">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <!-- South side (latitude 32.926) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_73953e59-b651-4607-b7a3-b8ad28bac75d">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.562 1.8 32.926 130.562 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- North side (latitude 32.927) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_6450b526-6f85-4118-b327-f8b6f0a45437">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.562 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.927 130.562 2.0 32.927 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- East side (longitude 130.562) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_5bc746a5-acaa-41be-b6cc-c8811fc84489">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.927 130.562 1.8 32.927 130.562 2.0 32.926 130.562 2.0 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- West side (longitude 130.561) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_0ecf29fc-2f0d-4e94-82a2-430cc246592e">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.561 1.8 32.926 130.561 1.8 32.926 130.561 2.0 32.927 130.561 2.0 32.927 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_684b2f38-9ae7-40c5-98b8-43e7522f89a0">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.562 1.8 32.926 130.562 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_e1ec826d-0088-425f-9289-ee8958e66fa2">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.562 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.927 130.562 2.0 32.927 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_6d6d85af-4333-4d6b-93a1-6fd43840fa2b">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.927 130.562 1.8 32.927 130.562 2.0 32.926 130.562 2.0 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_8443d89d-885d-4d0a-95eb-4f70d80fd534">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.561 1.8 32.926 130.561 1.8 32.926 130.561 2.0 32.927 130.561 2.0 32.927 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </core:ClosureSurface>
      </core:boundary>
      <core:lod0MultiCurve>
        <gml:MultiCurve>
          <gml:curveMember>
            <gml:LineString>
              <gml:posList>32.9265 130.561 2.0 32.9265 130.562 2.0</gml:posList>
            </gml:LineString>
          </gml:curveMember>
        </gml:MultiCurve>
      </core:lod0MultiCurve>
      <core:lod2Solid>
        <gml:Solid gml:id="solid_d14d8c47-5072-4811-b581-dacf66f819e4">
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_886c133b-24b9-417b-ba57-f72fc68da048"/>
              <gml:surfaceMember xlink:href="#poly_d037c352-fa58-44af-a523-3c256b55747c"/>
              <gml:surfaceMember xlink:href="#poly_73953e59-b651-4607-b7a3-b8ad28bac75d"/>
              <gml:surfaceMember xlink:href="#poly_6450b526-6f85-4118-b327-f8b6f0a45437"/>
              <gml:surfaceMember xlink:href="#poly_5bc746a5-acaa-41be-b6cc-c8811fc84489"/>
              <gml:surfaceMember xlink:href="#poly_0ecf29fc-2f0d-4e94-82a2-430cc246592e"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod2Solid>
      <core:lod3Solid>
        <gml:Solid gml:id="solid_f3e1410e-794e-46b3-85ab-8d15caa2da7b">
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_202f0a43-b428-4d45-86e5-6880b719228f"/>
              <gml:surfaceMember xlink:href="#poly_de1d7638-0183-419b-80e6-df7343c9bc98"/>
              <gml:surfaceMember xlink:href="#poly_684b2f38-9ae7-40c5-98b8-43e7522f89a0"/>
              <gml:surfaceMember xlink:href="#poly_e1ec826d-0088-425f-9289-ee8958e66fa2"/>
              <gml:surfaceMember xlink:href="#poly_6d6d85af-4333-4d6b-93a1-6fd43840fa2b"/>
              <gml:surfaceMember xlink:href="#poly_8443d89d-885d-4d0a-95eb-4f70d80fd534"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod3Solid>
      <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1020</wtr:class>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
