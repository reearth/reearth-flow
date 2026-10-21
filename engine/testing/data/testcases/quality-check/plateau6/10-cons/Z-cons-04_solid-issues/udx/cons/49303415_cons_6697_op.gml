<?xml version="1.0" encoding="UTF-8"?>
<!--
  Other construction whose LOD3 solid has faces facing inwards.
  Three con:OtherConstruction in tamana-shi (EPSG:6697), each a LOD3 solid that
  references the faces of its six boundary surfaces (one ground, one roof and
  four walls).
  Both roof triangles of cons_6248259d are wound the other way round, so their
  normals point into the solid.

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
        <con:GroundSurface gml:id="line_6a42ab31-5762-413c-bf96-60fb939b7367">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_ecbd91c4-4404-4a02-af31-4875cfd31cd1">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929475562888015 130.56507228215463 2.8725533090642776 32.929478752146466 130.56506892376652 2.8725533090642776 32.929478700529216 130.56507647174683 2.8725533090642776 32.929475562888015 130.56507228215463 2.8725533090642776</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_613a682f-978e-4933-899f-a74c38748ae7">
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
        <con:RoofSurface gml:id="line_6f72914c-c21a-45ab-856d-1664949d71c9">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_796179f6-762b-43b8-b48e-6c7240c72187">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.92948188978779 130.56507311335878 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_b0f159da-5e5f-423a-8d56-2c71584f4b2f">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.929475562888015 130.56507228215463 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </con:RoofSurface>
      </core:boundary>
      <core:boundary>
        <con:WallSurface gml:id="line_68e0e360-b8ab-4a22-9c17-13554934a178">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_2c14788a-c702-45d6-b26f-15df399b3f03">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92948188978779 130.56507311335878 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867 32.92948188978779 130.56507311335878 2.8725533090642776 32.92948188978779 130.56507311335878 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_e100322e-8e74-4fba-be6e-26c30da9eac1">
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
        <con:WallSurface gml:id="line_789346ed-97ce-4bda-9d3e-c91682032648">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_34c81548-068e-4767-a610-81ac8969652b">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.929475562888015 130.56507228215463 3.3725579834128867 32.929478700529216 130.56507647174683 2.8725533090642776 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_485f63b8-bb15-479e-a32c-f946caf99bfc">
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
        <con:WallSurface gml:id="line_47d1cc4d-f5af-420a-9ef6-6aeb02b1107a">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_10f8323c-291a-401b-a22e-28ebb80419f7">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929478752146466 130.56506892376652 3.3725579834128867 32.92948188978779 130.56507311335878 3.3725579834128867 32.929478752146466 130.56506892376652 2.8725533090642776 32.929478752146466 130.56506892376652 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_fde2b48c-93f6-4532-9453-21773044c6a5">
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
        <con:WallSurface gml:id="line_25fd5cd3-2cb7-4d42-9a49-376c3052e36f">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_0493b0f4-aae1-44ed-a0cd-162fc085f671">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.929475562888015 130.56507228215463 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.929475562888015 130.56507228215463 2.8725533090642776 32.929475562888015 130.56507228215463 3.3725579834128867</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_0320c7ea-4812-429f-a7c1-f0de8b3c38d4">
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
              <gml:surfaceMember xlink:href="#poly_ecbd91c4-4404-4a02-af31-4875cfd31cd1"/>
              <gml:surfaceMember xlink:href="#poly_613a682f-978e-4933-899f-a74c38748ae7"/>
              <gml:surfaceMember xlink:href="#poly_796179f6-762b-43b8-b48e-6c7240c72187"/>
              <gml:surfaceMember xlink:href="#poly_b0f159da-5e5f-423a-8d56-2c71584f4b2f"/>
              <gml:surfaceMember xlink:href="#poly_2c14788a-c702-45d6-b26f-15df399b3f03"/>
              <gml:surfaceMember xlink:href="#poly_e100322e-8e74-4fba-be6e-26c30da9eac1"/>
              <gml:surfaceMember xlink:href="#poly_34c81548-068e-4767-a610-81ac8969652b"/>
              <gml:surfaceMember xlink:href="#poly_485f63b8-bb15-479e-a32c-f946caf99bfc"/>
              <gml:surfaceMember xlink:href="#poly_10f8323c-291a-401b-a22e-28ebb80419f7"/>
              <gml:surfaceMember xlink:href="#poly_fde2b48c-93f6-4532-9453-21773044c6a5"/>
              <gml:surfaceMember xlink:href="#poly_0493b0f4-aae1-44ed-a0cd-162fc085f671"/>
              <gml:surfaceMember xlink:href="#poly_0320c7ea-4812-429f-a7c1-f0de8b3c38d4"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod3Solid>
      <con:class codeSpace="../../codelists/OtherConstruction_class.xml">09</con:class>
      <con:function codeSpace="../../codelists/OtherConstruction_function.xml">0902</con:function>
    </con:OtherConstruction>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <con:OtherConstruction gml:id="cons_c1137182-67d1-4aac-ab74-0cfd6dfa8b97">
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
        <con:GroundSurface gml:id="line_7dff6b48-79d9-4c35-aeda-e9f4dac9c7f4">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_b5a3eec1-55da-4dc0-8706-bb0e6cd75618">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.4790922920119365 32.927438153715244 130.56288019582135 2.4790922920119365 32.92744336355139 130.56287308878584 2.4790922920119365 32.92744455172919 130.56287452471838 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_aa70b312-78ca-49ba-b668-d669dc9e5c3d">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927438153715244 130.56288019582135 2.4790922920119365 32.92744455172919 130.56287452471838 2.4790922920119365 32.927439497357376 130.56288155418977 2.4790922920119365 32.927438153715244 130.56288019582135 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_f07106cb-8afd-46fa-a84c-2933f59e1df2">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.4790922920119365 32.92744336355139 130.56287308878584 2.4790922920119365 32.9274460683982 130.5628735083426 2.4790922920119365 32.92744455172919 130.56287452471838 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_dab69589-3437-4f4d-9f25-04b1f04a76c7">
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
        <con:RoofSurface gml:id="line_72c1838d-6e56-4c8e-9d3d-771fa1bf59b7">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_2cc67dea-e3f9-4c6b-81a7-c26f1d39d587">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.9083897803727594 32.92744336355139 130.56287308878584 2.9083897803727594 32.927438153715244 130.56288019582135 2.9083897803727594 32.92744455172919 130.56287452471838 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_717f5a7f-ced3-4857-b7ec-ed21b1a7883a">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927438153715244 130.56288019582135 2.9083897803727594 32.927439497357376 130.56288155418977 2.9083897803727594 32.92744455172919 130.56287452471838 2.9083897803727594 32.927438153715244 130.56288019582135 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_27890584-406b-4b63-9b85-3dbd47706410">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.9083897803727594 32.9274460683982 130.5628735083426 2.9083897803727594 32.92744336355139 130.56287308878584 2.9083897803727594 32.92744455172919 130.56287452471838 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_cff2082b-cd50-4013-8606-6618412a654a">
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
        <con:WallSurface gml:id="line_a71bc560-a33b-42a6-991b-4f0c678b2e77">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_403f14e7-309e-4a2c-aa7f-9ddc556fb396">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927438153715244 130.56288019582135 2.9083897803727594 32.927439497357376 130.56288155418977 2.4790922920119365 32.927439497357376 130.56288155418977 2.9083897803727594 32.927438153715244 130.56288019582135 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_75aedb8c-6a93-4694-8b01-62cdeaa66697">
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
        <con:WallSurface gml:id="line_32cbdc1a-8035-43b1-a8cc-eaf53784fd0f">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_9b5861fe-79d4-41eb-b93e-3d5a8a546267">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.9274460683982 130.5628735083426 2.9083897803727594 32.92744397231607 130.56287132863648 2.4790922920119365 32.92744397231607 130.56287132863648 2.9083897803727594 32.9274460683982 130.5628735083426 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_f06dfc08-cf5a-42a5-8525-389104b5f40b">
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
        <con:WallSurface gml:id="line_905a85d0-07de-438c-be8a-da3ccc7c4e7c">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_9da2e3d2-ee26-40eb-9fea-9ac980272b7e">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744336355139 130.56287308878584 2.4790922920119365 32.92744397231607 130.56287132863648 2.9083897803727594 32.92744397231607 130.56287132863648 2.4790922920119365 32.92744336355139 130.56287308878584 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_dc475a1c-0a92-4ffd-8f57-5b578c0a9f6a">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744397231607 130.56287132863648 2.9083897803727594 32.92744336355139 130.56287308878584 2.4790922920119365 32.92744336355139 130.56287308878584 2.9083897803727594 32.92744397231607 130.56287132863648 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_2c6ead45-f607-4b7e-b296-dacde101b235">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927438153715244 130.56288019582135 2.4790922920119365 32.92744336355139 130.56287308878584 2.9083897803727594 32.92744336355139 130.56287308878584 2.4790922920119365 32.927438153715244 130.56288019582135 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_c7041a1c-dcfa-40b9-9d16-1b8094249d1d">
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
        <con:WallSurface gml:id="line_68f2814f-37c6-448f-9528-7a8415ce69d2">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_cd115bb9-399e-4881-a222-3e3d15c81caf">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.4790922920119365 32.927439497357376 130.56288155418977 2.9083897803727594 32.927439497357376 130.56288155418977 2.4790922920119365 32.92744455172919 130.56287452471838 2.4790922920119365</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_c34c408e-a9f7-4ed5-b311-78bac96f3265">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439497357376 130.56288155418977 2.9083897803727594 32.92744455172919 130.56287452471838 2.4790922920119365 32.92744455172919 130.56287452471838 2.9083897803727594 32.927439497357376 130.56288155418977 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_c8400408-2c65-4f96-947f-05211721f9c3">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92744455172919 130.56287452471838 2.9083897803727594 32.9274460683982 130.5628735083426 2.4790922920119365 32.9274460683982 130.5628735083426 2.9083897803727594 32.92744455172919 130.56287452471838 2.9083897803727594</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_5efd6fc2-9173-4a2a-84e9-22a81699a2d4">
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
              <gml:surfaceMember xlink:href="#poly_b5a3eec1-55da-4dc0-8706-bb0e6cd75618"/>
              <gml:surfaceMember xlink:href="#poly_aa70b312-78ca-49ba-b668-d669dc9e5c3d"/>
              <gml:surfaceMember xlink:href="#poly_f07106cb-8afd-46fa-a84c-2933f59e1df2"/>
              <gml:surfaceMember xlink:href="#poly_dab69589-3437-4f4d-9f25-04b1f04a76c7"/>
              <gml:surfaceMember xlink:href="#poly_2cc67dea-e3f9-4c6b-81a7-c26f1d39d587"/>
              <gml:surfaceMember xlink:href="#poly_717f5a7f-ced3-4857-b7ec-ed21b1a7883a"/>
              <gml:surfaceMember xlink:href="#poly_27890584-406b-4b63-9b85-3dbd47706410"/>
              <gml:surfaceMember xlink:href="#poly_cff2082b-cd50-4013-8606-6618412a654a"/>
              <gml:surfaceMember xlink:href="#poly_403f14e7-309e-4a2c-aa7f-9ddc556fb396"/>
              <gml:surfaceMember xlink:href="#poly_75aedb8c-6a93-4694-8b01-62cdeaa66697"/>
              <gml:surfaceMember xlink:href="#poly_9b5861fe-79d4-41eb-b93e-3d5a8a546267"/>
              <gml:surfaceMember xlink:href="#poly_f06dfc08-cf5a-42a5-8525-389104b5f40b"/>
              <gml:surfaceMember xlink:href="#poly_9da2e3d2-ee26-40eb-9fea-9ac980272b7e"/>
              <gml:surfaceMember xlink:href="#poly_dc475a1c-0a92-4ffd-8f57-5b578c0a9f6a"/>
              <gml:surfaceMember xlink:href="#poly_2c6ead45-f607-4b7e-b296-dacde101b235"/>
              <gml:surfaceMember xlink:href="#poly_c7041a1c-dcfa-40b9-9d16-1b8094249d1d"/>
              <gml:surfaceMember xlink:href="#poly_cd115bb9-399e-4881-a222-3e3d15c81caf"/>
              <gml:surfaceMember xlink:href="#poly_c34c408e-a9f7-4ed5-b311-78bac96f3265"/>
              <gml:surfaceMember xlink:href="#poly_c8400408-2c65-4f96-947f-05211721f9c3"/>
              <gml:surfaceMember xlink:href="#poly_5efd6fc2-9173-4a2a-84e9-22a81699a2d4"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod3Solid>
      <con:class codeSpace="../../codelists/OtherConstruction_class.xml">09</con:class>
      <con:function codeSpace="../../codelists/OtherConstruction_function.xml">0902</con:function>
    </con:OtherConstruction>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <con:OtherConstruction gml:id="cons_fa550f29-9878-47cb-816f-b5ef2c13bbaf">
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
        <con:GroundSurface gml:id="line_95cb7d80-26a6-44de-8730-d8170067be05">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_1037bc50-d298-4b15-b83a-773920f22cff">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743843930368 130.56288135180674 2.4790922920119374 32.927439188083355 130.56288428238688 2.4790922920119374 32.927434158557034 130.56288288964043 2.4790922920119374 32.92743843930368 130.56288135180674 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_17e4f3fd-c4bf-4244-acf5-8bd668c8b4a3">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434158557034 130.56288288964043 2.4790922920119374 32.927435924540504 130.56288065543362 2.4790922920119374 32.92743843930368 130.56288135180674 2.4790922920119374 32.927434158557034 130.56288288964043 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_19d1fddf-0c5f-435d-971a-b908dfe5643e">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439188083355 130.56288428238688 2.4790922920119374 32.92743742209983 130.56288651659378 2.4790922920119374 32.927434158557034 130.56288288964043 2.4790922920119374 32.927439188083355 130.56288428238688 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_65ab2bec-f7f0-494d-9a9f-9d096df71aa1">
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
        <con:RoofSurface gml:id="line_a5eed834-7f13-49a0-ad30-eaf0fcf14059">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_5f431eed-3316-4c28-aa5f-65f306329971">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743843930368 130.56288135180674 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099 32.927439188083355 130.56288428238688 3.127932581841099 32.92743843930368 130.56288135180674 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_247d0935-c5dd-4ca2-9029-af0de5bb3b8b">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434158557034 130.56288288964043 3.127932581841099 32.92743843930368 130.56288135180674 3.127932581841099 32.927435924540504 130.56288065543362 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_b121e777-6f5f-4957-8272-10cdb0378d33">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439188083355 130.56288428238688 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099 32.92743742209983 130.56288651659378 3.127932581841099 32.927439188083355 130.56288428238688 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_85b5f7b6-c143-48ae-b4cd-8383787ffa2d">
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
        <con:WallSurface gml:id="line_330c6a8f-de95-496a-bebf-07f9b4834861">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_68beaa04-2b30-499d-9cc4-1e48b1c2fe18">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439188083355 130.56288428238688 3.127932581841099 32.92743742209983 130.56288651659378 3.127932581841099 32.92743742209983 130.56288651659378 2.4790922920119374 32.927439188083355 130.56288428238688 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_ff2817a4-dff6-49d8-a1af-038f412c73f9">
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
        <con:WallSurface gml:id="line_9a5e27f0-af47-4dc8-bc81-b1eea8749a8d">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_57617f7c-46a7-4f95-9f1a-b654add44b8a">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434907336675 130.5628858202205 3.127932581841099 32.927434158557034 130.56288288964043 3.127932581841099 32.927434158557034 130.56288288964043 2.4790922920119374 32.927434907336675 130.5628858202205 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_3bbee998-8714-4693-8cb2-3839d8aa97cf">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434158557034 130.56288288964043 2.4790922920119374 32.927434907336675 130.5628858202205 2.4790922920119374 32.927434907336675 130.5628858202205 3.127932581841099 32.927434158557034 130.56288288964043 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_f1567201-198f-47b6-a8c4-1b19f79af620">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743742209983 130.56288651659378 3.127932581841099 32.927434907336675 130.5628858202205 3.127932581841099 32.927434907336675 130.5628858202205 2.4790922920119374 32.92743742209983 130.56288651659378 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_215147a5-2c85-48ee-97bd-6020246c3995">
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
        <con:WallSurface gml:id="line_c3266497-3930-4ceb-b1e8-308f911d495d">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_5ee46dce-fb5a-4e7d-a300-a064f5ebee28">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927439188083355 130.56288428238688 2.4790922920119374 32.92743843930368 130.56288135180674 2.4790922920119374 32.92743843930368 130.56288135180674 3.127932581841099 32.927439188083355 130.56288428238688 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_e1c5b714-dbf4-46e0-985f-fe5f8aaa379f">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743843930368 130.56288135180674 3.127932581841099 32.927439188083355 130.56288428238688 3.127932581841099 32.927439188083355 130.56288428238688 2.4790922920119374 32.92743843930368 130.56288135180674 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_b9dbec3f-1edf-459a-9334-0fe4cb88966e">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.92743843930368 130.56288135180674 2.4790922920119374 32.927435924540504 130.56288065543362 2.4790922920119374 32.927435924540504 130.56288065543362 3.127932581841099 32.92743843930368 130.56288135180674 2.4790922920119374</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_37b7e6d7-e1d5-4f2b-bbf8-9443a1e3b781">
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
        <con:WallSurface gml:id="line_41f5a384-10a9-4c0a-8ef6-8b3449f9211f">
          <core:creationDate>2024-03-22T00:00:00</core:creationDate>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_71bedf29-db63-4be2-8aed-7e9eee4fb42f">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927434158557034 130.56288288964043 3.127932581841099 32.927435924540504 130.56288065543362 3.127932581841099 32.927435924540504 130.56288065543362 2.4790922920119374 32.927434158557034 130.56288288964043 3.127932581841099</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_5d01624d-c6d6-401a-809e-3335ac51e7f1">
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
              <gml:surfaceMember xlink:href="#poly_1037bc50-d298-4b15-b83a-773920f22cff"/>
              <gml:surfaceMember xlink:href="#poly_17e4f3fd-c4bf-4244-acf5-8bd668c8b4a3"/>
              <gml:surfaceMember xlink:href="#poly_19d1fddf-0c5f-435d-971a-b908dfe5643e"/>
              <gml:surfaceMember xlink:href="#poly_65ab2bec-f7f0-494d-9a9f-9d096df71aa1"/>
              <gml:surfaceMember xlink:href="#poly_5f431eed-3316-4c28-aa5f-65f306329971"/>
              <gml:surfaceMember xlink:href="#poly_247d0935-c5dd-4ca2-9029-af0de5bb3b8b"/>
              <gml:surfaceMember xlink:href="#poly_b121e777-6f5f-4957-8272-10cdb0378d33"/>
              <gml:surfaceMember xlink:href="#poly_85b5f7b6-c143-48ae-b4cd-8383787ffa2d"/>
              <gml:surfaceMember xlink:href="#poly_68beaa04-2b30-499d-9cc4-1e48b1c2fe18"/>
              <gml:surfaceMember xlink:href="#poly_ff2817a4-dff6-49d8-a1af-038f412c73f9"/>
              <gml:surfaceMember xlink:href="#poly_57617f7c-46a7-4f95-9f1a-b654add44b8a"/>
              <gml:surfaceMember xlink:href="#poly_3bbee998-8714-4693-8cb2-3839d8aa97cf"/>
              <gml:surfaceMember xlink:href="#poly_f1567201-198f-47b6-a8c4-1b19f79af620"/>
              <gml:surfaceMember xlink:href="#poly_215147a5-2c85-48ee-97bd-6020246c3995"/>
              <gml:surfaceMember xlink:href="#poly_5ee46dce-fb5a-4e7d-a300-a064f5ebee28"/>
              <gml:surfaceMember xlink:href="#poly_e1c5b714-dbf4-46e0-985f-fe5f8aaa379f"/>
              <gml:surfaceMember xlink:href="#poly_b9dbec3f-1edf-459a-9334-0fe4cb88966e"/>
              <gml:surfaceMember xlink:href="#poly_37b7e6d7-e1d5-4f2b-bbf8-9443a1e3b781"/>
              <gml:surfaceMember xlink:href="#poly_71bedf29-db63-4be2-8aed-7e9eee4fb42f"/>
              <gml:surfaceMember xlink:href="#poly_5d01624d-c6d6-401a-809e-3335ac51e7f1"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod3Solid>
      <con:class codeSpace="../../codelists/OtherConstruction_class.xml">09</con:class>
      <con:function codeSpace="../../codelists/OtherConstruction_function.xml">0902</con:function>
    </con:OtherConstruction>
  </core:cityObjectMember>
</core:CityModel>
