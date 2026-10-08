<?xml version="1.0" encoding="UTF-8"?>
<!--
  Duplicate gml:id across files.
  The error-free other constructions with every gml:id except that of the first
  con:OtherConstruction changed, so only that gml:id is also used in
  49303415_cons_6697_op.gml.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
  <gml:boundedBy>
    <gml:Envelope srsDimension="3" srsName="http://www.opengis.net/def/crs/EPSG/0/6697">
      <gml:lowerCorner>32.92647721145687 130.56240409979662 2.40611</gml:lowerCorner>
      <gml:upperCorner>32.92967959516049 130.56521919330308 10.97262</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <con:OtherConstruction gml:id="cons_6248259d-2fd7-4b8c-951e-f11973d5b935">
      <core:creationDate>2024-03-22T00:00:00</core:creationDate>
      <core:adeOfAbstractCityObject>
        <urc:DataQualityAttribute>
          <urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod1>
          <urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod3>
          <urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
          <urc:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">5</urc:appearanceSrcDescLod3>
          <urc:lodType codeSpace="../../codelists/OtherConstruction_lodType.xml">3.0</urc:lodType>
          <urc:publicSurveyDataQualityAttribute>
            <urc:PublicSurveyDataQualityAttribute>
              <urc:srcScaleLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">3</urc:srcScaleLod3>
              <urc:publicSurveySrcDescLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">018</urc:publicSurveySrcDescLod3>
            </urc:PublicSurveyDataQualityAttribute>
          </urc:publicSurveyDataQualityAttribute>
        </urc:DataQualityAttribute>
      </core:adeOfAbstractCityObject>
      <core:boundary>
        <con:GroundSurface gml:id="line_adb80d42-3216-44fa-bfa6-3d5741a1802f">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_77396dce-0b9b-40e3-9bca-1a68ddb47e87">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929475562888015 130.56507228215463 2.8725533090642776 32.929478752146466 130.56506892376652 2.8725533090642776 32.929478700529216 130.56507647174683 2.8725533090642776 32.929475562888015 130.56507228215463 2.8725533090642776</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_cf35e763-b811-4ccc-9e59-8742579c0131">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478700529216 130.56507647174683 2.8725533090642776 32.929478752146466 130.56506892376652 2.8725533090642776 32.92948188978779 130.56507311335878 2.8725533090642776 32.929478700529216 130.56507647174683 2.8725533090642776</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:GroundSurface>
      </core:boundary>
      <core:boundary>
        <con:RoofSurface gml:id="line_afe7c712-e377-4180-addd-7af57600a0d6">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_01e9c761-2453-437c-b4fd-f6ebcb8dbf60">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.92948188978779 130.56507311335878 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_afacb853-b455-4bf6-b574-3f1f6bbb77cc">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.929475562888015 130.56507228215463 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:RoofSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_63276a6e-3ad4-4aee-b87a-2318b7552e56">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_ea89ea09-68f6-461d-b511-a8d77ca41897">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92948188978779 130.56507311335878 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867 32.92948188978779 130.56507311335878 2.8725533090642776 32.92948188978779 130.56507311335878 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_85a52063-8ce6-4027-9101-a00a793e9964">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92948188978779 130.56507311335878 2.8725533090642776 32.929478700529216 130.56507647174683 3.3725579834128867 32.929478700529216 130.56507647174683 2.8725533090642776 32.92948188978779 130.56507311335878 2.8725533090642776</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_2997c868-8b1d-482c-8141-f2dc98774648">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_a7500e09-2bdb-4a43-8c99-91faca2cc56d">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.929475562888015 130.56507228215463 3.3725579834128867 32.929478700529216 130.56507647174683 2.8725533090642776 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_b4089fbe-62b9-4292-a077-07c6a57cafeb">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478700529216 130.56507647174683 2.8725533090642776 32.929475562888015 130.56507228215463 3.3725579834128867 32.929475562888015 130.56507228215463 2.8725533090642776 32.929478700529216 130.56507647174683 2.8725533090642776</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_9b78e0ef-0bcf-49e1-b495-5ccb483fd362">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_34b1284d-d954-4dde-a7db-cfc0d7a0dc6d">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478752146466 130.56506892376652 3.3725579834128867 32.92948188978779 130.56507311335878 3.3725579834128867 32.929478752146466 130.56506892376652 2.8725533090642776 32.929478752146466 130.56506892376652 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_525b3028-68b6-4156-b7f6-39c00d98746a">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478752146466 130.56506892376652 2.8725533090642776 32.92948188978779 130.56507311335878 3.3725579834128867 32.92948188978779 130.56507311335878 2.8725533090642776 32.929478752146466 130.56506892376652 2.8725533090642776</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_d1a65721-590f-48bd-8012-a6edd6e09ba6">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_1e181ead-f09b-44b2-9c89-b3c78bbf0c35">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929475562888015 130.56507228215463 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.929475562888015 130.56507228215463 2.8725533090642776 32.929475562888015 130.56507228215463 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_481cd3aa-a722-4d22-b28f-7b1a1f5f6cd4">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929475562888015 130.56507228215463 2.8725533090642776 32.929478752146466 130.56506892376652 3.3725579834128867 32.929478752146466 130.56506892376652 2.8725533090642776 32.929475562888015 130.56507228215463 2.8725533090642776</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:lod3Solid>
        <gml:Solid>
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_77396dce-0b9b-40e3-9bca-1a68ddb47e87"/>
              <gml:surfaceMember xlink:href="#poly_cf35e763-b811-4ccc-9e59-8742579c0131"/>
              <gml:surfaceMember xlink:href="#poly_01e9c761-2453-437c-b4fd-f6ebcb8dbf60"/>
              <gml:surfaceMember xlink:href="#poly_afacb853-b455-4bf6-b574-3f1f6bbb77cc"/>
              <gml:surfaceMember xlink:href="#poly_ea89ea09-68f6-461d-b511-a8d77ca41897"/>
              <gml:surfaceMember xlink:href="#poly_85a52063-8ce6-4027-9101-a00a793e9964"/>
              <gml:surfaceMember xlink:href="#poly_a7500e09-2bdb-4a43-8c99-91faca2cc56d"/>
              <gml:surfaceMember xlink:href="#poly_b4089fbe-62b9-4292-a077-07c6a57cafeb"/>
              <gml:surfaceMember xlink:href="#poly_34b1284d-d954-4dde-a7db-cfc0d7a0dc6d"/>
              <gml:surfaceMember xlink:href="#poly_525b3028-68b6-4156-b7f6-39c00d98746a"/>
              <gml:surfaceMember xlink:href="#poly_1e181ead-f09b-44b2-9c89-b3c78bbf0c35"/>
              <gml:surfaceMember xlink:href="#poly_481cd3aa-a722-4d22-b28f-7b1a1f5f6cd4"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod3Solid>
      <con:class codeSpace="../../codelists/OtherConstruction_class.xml">09</con:class>
      <con:function codeSpace="../../codelists/OtherConstruction_function.xml">0902</con:function>
    </con:OtherConstruction>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <con:OtherConstruction gml:id="cons_0d786f04-5844-4470-afb1-c06fa9e26f74">
      <core:creationDate>2024-03-22T00:00:00</core:creationDate>
      <core:adeOfAbstractCityObject>
        <urc:DataQualityAttribute>
          <urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod1>
          <urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod3>
          <urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
          <urc:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">5</urc:appearanceSrcDescLod3>
          <urc:lodType codeSpace="../../codelists/OtherConstruction_lodType.xml">3.0</urc:lodType>
          <urc:publicSurveyDataQualityAttribute>
            <urc:PublicSurveyDataQualityAttribute>
              <urc:srcScaleLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">3</urc:srcScaleLod3>
              <urc:publicSurveySrcDescLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">018</urc:publicSurveySrcDescLod3>
            </urc:PublicSurveyDataQualityAttribute>
          </urc:publicSurveyDataQualityAttribute>
        </urc:DataQualityAttribute>
      </core:adeOfAbstractCityObject>
      <core:boundary>
        <con:GroundSurface gml:id="line_17199613-00fa-4e0c-a984-9b9ee2802692">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_fceb36da-5642-4f4b-aaea-d292ebbb8ff7">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.4790922920119365 32.927438153715244 130.56288019582135 2.4790922920119365 32.92744336355139 130.56287308878584 2.4790922920119365 32.92744455172919 130.56287452471838 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_93704fc7-cd73-4fd8-8929-58de26304c10">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927438153715244 130.56288019582135 2.4790922920119365 32.92744455172919 130.56287452471838 2.4790922920119365 32.927439497357376 130.56288155418977 2.4790922920119365 32.927438153715244 130.56288019582135 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_d353219f-0814-420c-a081-bc8d712da206">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.4790922920119365 32.92744336355139 130.56287308878584 2.4790922920119365 32.9274460683982 130.5628735083426 2.4790922920119365 32.92744455172919 130.56287452471838 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_12715e69-db9e-44df-99fe-855310e54bb6">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.9274460683982 130.5628735083426 2.4790922920119365 32.92744336355139 130.56287308878584 2.4790922920119365 32.92744397231607 130.56287132863648 2.4790922920119365 32.9274460683982 130.5628735083426 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:GroundSurface>
      </core:boundary>
      <core:boundary>
        <con:RoofSurface gml:id="line_7ce8aa7e-7309-422a-929c-8048e0c738be">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_af432b61-6e53-4436-a34d-47222dba2d72">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.9083897803727594 32.92744336355139 130.56287308878584 2.9083897803727594 32.927438153715244 130.56288019582135 2.9083897803727594 32.92744455172919 130.56287452471838 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_7a2eea79-3372-44c5-b921-22f847c1e90c">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927438153715244 130.56288019582135 2.9083897803727594 32.927439497357376 130.56288155418977 2.9083897803727594 32.92744455172919 130.56287452471838 2.9083897803727594 32.927438153715244 130.56288019582135 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_e5ea684b-b411-452c-b16f-af692010baa1">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.9083897803727594 32.9274460683982 130.5628735083426 2.9083897803727594 32.92744336355139 130.56287308878584 2.9083897803727594 32.92744455172919 130.56287452471838 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_7de99c40-7a2d-4dc8-aee4-ec6e4c3e6607">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.9274460683982 130.5628735083426 2.9083897803727594 32.92744397231607 130.56287132863648 2.9083897803727594 32.92744336355139 130.56287308878584 2.9083897803727594 32.9274460683982 130.5628735083426 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:RoofSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_79fa6025-e542-469b-bdf0-8fa0def8ed70">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_407fb9ea-2995-462e-b346-dd12eddee240">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927438153715244 130.56288019582135 2.9083897803727594 32.927439497357376 130.56288155418977 2.4790922920119365 32.927439497357376 130.56288155418977 2.9083897803727594 32.927438153715244 130.56288019582135 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_ad7eb6bf-dc29-4e17-901f-b235dc0b844f">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439497357376 130.56288155418977 2.4790922920119365 32.927438153715244 130.56288019582135 2.9083897803727594 32.927438153715244 130.56288019582135 2.4790922920119365 32.927439497357376 130.56288155418977 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_05383eff-f04e-4fbd-baf0-6b776650e57b">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_0a9a1f19-83e6-4a91-9159-01bc6de84965">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.9274460683982 130.5628735083426 2.9083897803727594 32.92744397231607 130.56287132863648 2.4790922920119365 32.92744397231607 130.56287132863648 2.9083897803727594 32.9274460683982 130.5628735083426 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_732a452e-9d0d-4f35-bb2d-d4c66e31d884">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744397231607 130.56287132863648 2.4790922920119365 32.9274460683982 130.5628735083426 2.9083897803727594 32.9274460683982 130.5628735083426 2.4790922920119365 32.92744397231607 130.56287132863648 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_94ea58fa-4be7-40ab-9d48-b7048c5e65ff">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_8b592a12-3573-4935-a16e-01b2224bbc15">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744336355139 130.56287308878584 2.4790922920119365 32.92744397231607 130.56287132863648 2.9083897803727594 32.92744397231607 130.56287132863648 2.4790922920119365 32.92744336355139 130.56287308878584 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_73c2b664-72ee-4be4-a528-70ece2b2b66b">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744397231607 130.56287132863648 2.9083897803727594 32.92744336355139 130.56287308878584 2.4790922920119365 32.92744336355139 130.56287308878584 2.9083897803727594 32.92744397231607 130.56287132863648 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_678d7a84-af8b-4351-bc44-ecc53cce34c2">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927438153715244 130.56288019582135 2.4790922920119365 32.92744336355139 130.56287308878584 2.9083897803727594 32.92744336355139 130.56287308878584 2.4790922920119365 32.927438153715244 130.56288019582135 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_2c1ff9d2-e708-4135-91cb-a3a3d3f3901f">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744336355139 130.56287308878584 2.9083897803727594 32.927438153715244 130.56288019582135 2.4790922920119365 32.927438153715244 130.56288019582135 2.9083897803727594 32.92744336355139 130.56287308878584 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_6c92fe09-a2c0-4c6f-b690-54ed1d4d1a6b">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_267fdc45-7321-4c08-aff9-c304e86c2300">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.4790922920119365 32.927439497357376 130.56288155418977 2.9083897803727594 32.927439497357376 130.56288155418977 2.4790922920119365 32.92744455172919 130.56287452471838 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_856dd616-323f-4fd6-95f2-70171dafa595">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439497357376 130.56288155418977 2.9083897803727594 32.92744455172919 130.56287452471838 2.4790922920119365 32.92744455172919 130.56287452471838 2.9083897803727594 32.927439497357376 130.56288155418977 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_38ddd411-e8e8-4909-a113-5a1238222703">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.9083897803727594 32.9274460683982 130.5628735083426 2.4790922920119365 32.9274460683982 130.5628735083426 2.9083897803727594 32.92744455172919 130.56287452471838 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_29d1cbb6-2c3c-4b90-bcef-c23a1f94d18e">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.9274460683982 130.5628735083426 2.4790922920119365 32.92744455172919 130.56287452471838 2.9083897803727594 32.92744455172919 130.56287452471838 2.4790922920119365 32.9274460683982 130.5628735083426 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:lod3Solid>
        <gml:Solid>
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_fceb36da-5642-4f4b-aaea-d292ebbb8ff7"/>
              <gml:surfaceMember xlink:href="#poly_93704fc7-cd73-4fd8-8929-58de26304c10"/>
              <gml:surfaceMember xlink:href="#poly_d353219f-0814-420c-a081-bc8d712da206"/>
              <gml:surfaceMember xlink:href="#poly_12715e69-db9e-44df-99fe-855310e54bb6"/>
              <gml:surfaceMember xlink:href="#poly_af432b61-6e53-4436-a34d-47222dba2d72"/>
              <gml:surfaceMember xlink:href="#poly_7a2eea79-3372-44c5-b921-22f847c1e90c"/>
              <gml:surfaceMember xlink:href="#poly_e5ea684b-b411-452c-b16f-af692010baa1"/>
              <gml:surfaceMember xlink:href="#poly_7de99c40-7a2d-4dc8-aee4-ec6e4c3e6607"/>
              <gml:surfaceMember xlink:href="#poly_407fb9ea-2995-462e-b346-dd12eddee240"/>
              <gml:surfaceMember xlink:href="#poly_ad7eb6bf-dc29-4e17-901f-b235dc0b844f"/>
              <gml:surfaceMember xlink:href="#poly_0a9a1f19-83e6-4a91-9159-01bc6de84965"/>
              <gml:surfaceMember xlink:href="#poly_732a452e-9d0d-4f35-bb2d-d4c66e31d884"/>
              <gml:surfaceMember xlink:href="#poly_8b592a12-3573-4935-a16e-01b2224bbc15"/>
              <gml:surfaceMember xlink:href="#poly_73c2b664-72ee-4be4-a528-70ece2b2b66b"/>
              <gml:surfaceMember xlink:href="#poly_678d7a84-af8b-4351-bc44-ecc53cce34c2"/>
              <gml:surfaceMember xlink:href="#poly_2c1ff9d2-e708-4135-91cb-a3a3d3f3901f"/>
              <gml:surfaceMember xlink:href="#poly_267fdc45-7321-4c08-aff9-c304e86c2300"/>
              <gml:surfaceMember xlink:href="#poly_856dd616-323f-4fd6-95f2-70171dafa595"/>
              <gml:surfaceMember xlink:href="#poly_38ddd411-e8e8-4909-a113-5a1238222703"/>
              <gml:surfaceMember xlink:href="#poly_29d1cbb6-2c3c-4b90-bcef-c23a1f94d18e"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod3Solid>
      <con:class codeSpace="../../codelists/OtherConstruction_class.xml">09</con:class>
      <con:function codeSpace="../../codelists/OtherConstruction_function.xml">0902</con:function>
    </con:OtherConstruction>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <con:OtherConstruction gml:id="cons_872ea88a-3d0f-42da-b598-d23541014a84">
      <core:creationDate>2024-03-22T00:00:00</core:creationDate>
      <core:adeOfAbstractCityObject>
        <urc:DataQualityAttribute>
          <urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</urc:geometrySrcDescLod1>
          <urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod3>
          <urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
          <urc:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">5</urc:appearanceSrcDescLod3>
          <urc:lodType codeSpace="../../codelists/OtherConstruction_lodType.xml">3.0</urc:lodType>
          <urc:publicSurveyDataQualityAttribute>
            <urc:PublicSurveyDataQualityAttribute>
              <urc:srcScaleLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">3</urc:srcScaleLod3>
              <urc:publicSurveySrcDescLod3 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">018</urc:publicSurveySrcDescLod3>
            </urc:PublicSurveyDataQualityAttribute>
          </urc:publicSurveyDataQualityAttribute>
        </urc:DataQualityAttribute>
      </core:adeOfAbstractCityObject>
      <core:boundary>
        <con:GroundSurface gml:id="line_543bc766-ddcf-4d4a-840c-550887a04908">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_c04d6889-1198-4abd-9866-f3b133477362">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743843930368 130.56288135180674 2.4790922920119374 32.927439188083355 130.56288428238688 2.4790922920119374 32.927434158557034 130.56288288964043 2.4790922920119374 32.92743843930368 130.56288135180674 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_c459decc-68d1-4505-84a9-0a80edab16db">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434158557034 130.56288288964043 2.4790922920119374 32.927435924540504 130.56288065543362 2.4790922920119374 32.92743843930368 130.56288135180674 2.4790922920119374 32.927434158557034 130.56288288964043 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_fccacb76-c063-4e47-a82c-2e87db6d810a">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439188083355 130.56288428238688 2.4790922920119374 32.92743742209983 130.56288651659378 2.4790922920119374 32.927434158557034 130.56288288964043 2.4790922920119374 32.927439188083355 130.56288428238688 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_5376d787-4161-46fa-84f5-a6aebc8df7f1">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743742209983 130.56288651659378 2.4790922920119374 32.927434907336675 130.5628858202205 2.4790922920119374 32.927434158557034 130.56288288964043 2.4790922920119374 32.92743742209983 130.56288651659378 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:GroundSurface>
      </core:boundary>
      <core:boundary>
        <con:RoofSurface gml:id="line_9c8cf786-5a15-4e29-bbfb-4fef5677771a">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_23c6a2f9-1d33-4199-ac34-f0f19eac1a36">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743843930368 130.56288135180674 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099 32.927439188083355 130.56288428238688 3.127932581841099 32.92743843930368 130.56288135180674 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_f1d2a174-2281-4b29-897f-b84ffd2f6824">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434158557034 130.56288288964043 3.127932581841099 32.92743843930368 130.56288135180674 3.127932581841099 32.927435924540504 130.56288065543362 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_fd61c103-e57a-4085-80b0-63c7e378c25e">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439188083355 130.56288428238688 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099 32.92743742209983 130.56288651659378 3.127932581841099 32.927439188083355 130.56288428238688 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_d67c9226-7c86-4195-829d-dbbb3091867d">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743742209983 130.56288651659378 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099 32.927434907336675 130.5628858202205 3.127932581841099 32.92743742209983 130.56288651659378 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:RoofSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_2cc6fc13-c57a-4aa4-954d-dd6d4055477c">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_4db5cc2f-ab51-4f0c-aaf4-b7eff4d1a873">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439188083355 130.56288428238688 3.127932581841099 32.92743742209983 130.56288651659378 3.127932581841099 32.92743742209983 130.56288651659378 2.4790922920119374 32.927439188083355 130.56288428238688 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_15bec775-61e5-492c-bd72-cdf1162337b8">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743742209983 130.56288651659378 2.4790922920119374 32.927439188083355 130.56288428238688 2.4790922920119374 32.927439188083355 130.56288428238688 3.127932581841099 32.92743742209983 130.56288651659378 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_87b38b33-3f85-45c0-b6d5-9009a486e7d6">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_3852256f-2bfc-405b-ad0d-c0b3ab2de436">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434907336675 130.5628858202205 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099 32.927434158557034 130.56288288964043 2.4790922920119374 32.927434907336675 130.5628858202205 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_43b4cce1-b0ea-4c42-96b5-4cc0439c6901">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434158557034 130.56288288964043 2.4790922920119374 32.927434907336675 130.5628858202205 2.4790922920119374 32.927434907336675 130.5628858202205 3.127932581841099 32.927434158557034 130.56288288964043 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_fb8d4d2a-6ab0-465f-bbc0-9afd009b82ba">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743742209983 130.56288651659378 3.127932581841099 32.927434907336675 130.5628858202205 3.127932581841099 32.927434907336675 130.5628858202205 2.4790922920119374 32.92743742209983 130.56288651659378 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_395ce117-75ea-4a0c-965e-836f75b62108">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434907336675 130.5628858202205 2.4790922920119374 32.92743742209983 130.56288651659378 2.4790922920119374 32.92743742209983 130.56288651659378 3.127932581841099 32.927434907336675 130.5628858202205 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_b4bc61e6-4e89-4e40-a1f1-4661944fcfda">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_e81e8081-6d40-4db6-b897-f2d79780ee3f">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439188083355 130.56288428238688 2.4790922920119374 32.92743843930368 130.56288135180674 2.4790922920119374 32.92743843930368 130.56288135180674 3.127932581841099 32.927439188083355 130.56288428238688 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_eae45b0e-9943-490d-b2d8-0f1fbcd5af0d">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743843930368 130.56288135180674 3.127932581841099 32.927439188083355 130.56288428238688 3.127932581841099 32.927439188083355 130.56288428238688 2.4790922920119374 32.92743843930368 130.56288135180674 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_3b23767a-475e-4bc8-be49-796aa9c5aaa4">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743843930368 130.56288135180674 2.4790922920119374 32.927435924540504 130.56288065543362 2.4790922920119374 32.927435924540504 130.56288065543362 3.127932581841099 32.92743843930368 130.56288135180674 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_b5d2a7d4-377a-4090-b4ab-596139682ceb">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927435924540504 130.56288065543362 3.127932581841099 32.92743843930368 130.56288135180674 3.127932581841099 32.92743843930368 130.56288135180674 2.4790922920119374 32.927435924540504 130.56288065543362 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_12e934dd-9d69-4f82-b298-00a652276d71">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_ffab0d15-8ecb-40ce-b349-02d516a2872b">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434158557034 130.56288288964043 3.127932581841099 32.927435924540504 130.56288065543362 3.127932581841099 32.927435924540504 130.56288065543362 2.4790922920119374 32.927434158557034 130.56288288964043 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_656caaa1-9329-4f13-a3f3-df6ab7cc299f">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927435924540504 130.56288065543362 2.4790922920119374 32.927434158557034 130.56288288964043 2.4790922920119374 32.927434158557034 130.56288288964043 3.127932581841099 32.927435924540504 130.56288065543362 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:WallSurface>
      </core:boundary>
      <core:lod3Solid>
        <gml:Solid>
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_c04d6889-1198-4abd-9866-f3b133477362"/>
              <gml:surfaceMember xlink:href="#poly_c459decc-68d1-4505-84a9-0a80edab16db"/>
              <gml:surfaceMember xlink:href="#poly_fccacb76-c063-4e47-a82c-2e87db6d810a"/>
              <gml:surfaceMember xlink:href="#poly_5376d787-4161-46fa-84f5-a6aebc8df7f1"/>
              <gml:surfaceMember xlink:href="#poly_23c6a2f9-1d33-4199-ac34-f0f19eac1a36"/>
              <gml:surfaceMember xlink:href="#poly_f1d2a174-2281-4b29-897f-b84ffd2f6824"/>
              <gml:surfaceMember xlink:href="#poly_fd61c103-e57a-4085-80b0-63c7e378c25e"/>
              <gml:surfaceMember xlink:href="#poly_d67c9226-7c86-4195-829d-dbbb3091867d"/>
              <gml:surfaceMember xlink:href="#poly_4db5cc2f-ab51-4f0c-aaf4-b7eff4d1a873"/>
              <gml:surfaceMember xlink:href="#poly_15bec775-61e5-492c-bd72-cdf1162337b8"/>
              <gml:surfaceMember xlink:href="#poly_3852256f-2bfc-405b-ad0d-c0b3ab2de436"/>
              <gml:surfaceMember xlink:href="#poly_43b4cce1-b0ea-4c42-96b5-4cc0439c6901"/>
              <gml:surfaceMember xlink:href="#poly_fb8d4d2a-6ab0-465f-bbc0-9afd009b82ba"/>
              <gml:surfaceMember xlink:href="#poly_395ce117-75ea-4a0c-965e-836f75b62108"/>
              <gml:surfaceMember xlink:href="#poly_e81e8081-6d40-4db6-b897-f2d79780ee3f"/>
              <gml:surfaceMember xlink:href="#poly_eae45b0e-9943-490d-b2d8-0f1fbcd5af0d"/>
              <gml:surfaceMember xlink:href="#poly_3b23767a-475e-4bc8-be49-796aa9c5aaa4"/>
              <gml:surfaceMember xlink:href="#poly_b5d2a7d4-377a-4090-b4ab-596139682ceb"/>
              <gml:surfaceMember xlink:href="#poly_ffab0d15-8ecb-40ce-b349-02d516a2872b"/>
              <gml:surfaceMember xlink:href="#poly_656caaa1-9329-4f13-a3f3-df6ab7cc299f"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod3Solid>
      <con:class codeSpace="../../codelists/OtherConstruction_class.xml">09</con:class>
      <con:function codeSpace="../../codelists/OtherConstruction_function.xml">0902</con:function>
    </con:OtherConstruction>
  </core:cityObjectMember>
</core:CityModel>
