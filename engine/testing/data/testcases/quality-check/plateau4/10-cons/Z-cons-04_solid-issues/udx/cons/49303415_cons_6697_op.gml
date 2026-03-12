<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>32.92647721145687 130.56240409979662 2.40611</gml:lowerCorner>
      <gml:upperCorner>32.92967959516049 130.56521919330308 10.97262</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <!-- 立体のその他のエラー: Face Wrong Orientation (面の法線が内向き) -->
  <!-- RoofSurfaceの頂点順序を逆にして法線を内向き(下向き)にしている -->
  <!-- SolidBoundaryValidatorのOtherIssuesポートで検出されるエラー -->
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
    <uro:lod3Geometry>
        <gml:Solid>
          <gml:exterior>
            <gml:CompositeSurface>
              <gml:surfaceMember xlink:href="#poly_2b4e38f9-85c0-48c9-a2d0-9a1ed928bead" />
              <gml:surfaceMember xlink:href="#poly_2c68f6dd-12a8-42ea-aff8-93c60448b0a1" />
              <gml:surfaceMember xlink:href="#poly_ecce564d-5a94-4ba4-a6e0-9cf0828d8fe8" />
              <gml:surfaceMember xlink:href="#poly_f60ba630-be7a-45a7-a6cb-cf0aa246b5e7" />
              <gml:surfaceMember xlink:href="#poly_fe4ad5b9-9a7e-49de-91a9-092f743cba7d" />
              <gml:surfaceMember xlink:href="#poly_800b7ccb-2d93-46d6-9acf-aef531caae15" />
            </gml:CompositeSurface>
          </gml:exterior>
        </gml:Solid>
      </uro:lod3Geometry>
      <uro:boundedBy>
        <uro:GroundSurface gml:id="line_6a42ab31-5762-413c-bf96-60fb939b7367">
          <core:creationDate>2024-03-22</core:creationDate>
          <uro:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:CompositeSurface gml:id="poly_2c68f6dd-12a8-42ea-aff8-93c60448b0a1">
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929475562888015 130.56507228215463 2.8725533090642776 32.929478752146466 130.56506892376652 2.8725533090642776 32.929478700529216 130.56507647174683 2.8725533090642776 32.929475562888015 130.56507228215463 2.8725533090642776</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929478700529216 130.56507647174683 2.8725533090642776 32.929478752146466 130.56506892376652 2.8725533090642776 32.92948188978779 130.56507311335878 2.8725533090642776 32.929478700529216 130.56507647174683 2.8725533090642776</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </uro:lod3MultiSurface>
        </uro:GroundSurface>
      </uro:boundedBy>
      <uro:boundedBy>
        <uro:RoofSurface gml:id="line_6f72914c-c21a-45ab-856d-1664949d71c9">
          <core:creationDate>2024-03-22</core:creationDate>
          <uro:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:CompositeSurface gml:id="poly_2b4e38f9-85c0-48c9-a2d0-9a1ed928bead">
                  <gml:surfaceMember>
                    <!-- RoofSurface Triangle 1: 頂点順序を逆にして法線を内向きにする -->
                    <!-- 元の順序: C→D→B→C (法線が上向き=外向き) -->
                    <!-- 変更後:   C→B→D→C (法線が下向き=内向き) -->
                    <gml:Polygon gml:id="fme-gen-796179f6-762b-43b8-b48e-6c7240c72187">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.92948188978779 130.56507311335878 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <!-- RoofSurface Triangle 2: 頂点順序を逆にして法線を内向きにする -->
                    <!-- 元の順序: C→B→A→C (法線が上向き=外向き) -->
                    <!-- 変更後:   C→A→B→C (法線が下向き=内向き) -->
                    <gml:Polygon gml:id="fme-gen-b0f159da-5e5f-423a-8d56-2c71584f4b2f">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.929475562888015 130.56507228215463 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </uro:lod3MultiSurface>
        </uro:RoofSurface>
      </uro:boundedBy>
      <uro:boundedBy>
        <uro:WallSurface gml:id="line_68e0e360-b8ab-4a22-9c17-13554934a178">
          <core:creationDate>2024-03-22</core:creationDate>
          <uro:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:CompositeSurface gml:id="poly_ecce564d-5a94-4ba4-a6e0-9cf0828d8fe8">
                  <gml:surfaceMember>
                    <gml:Polygon gml:id="fme-gen-2c14788a-c702-45d6-b26f-15df399b3f03">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.92948188978779 130.56507311335878 3.3725579834128867 32.929478700529216 130.56507647174683 3.3725579834128867 32.92948188978779 130.56507311335878 2.8725533090642776 32.92948188978779 130.56507311335878 3.3725579834128867</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon gml:id="fme-gen-e100322e-8e74-4fba-be6e-26c30da9eac1">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.92948188978779 130.56507311335878 2.8725533090642776 32.929478700529216 130.56507647174683 3.3725579834128867 32.929478700529216 130.56507647174683 2.8725533090642776 32.92948188978779 130.56507311335878 2.8725533090642776</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </uro:lod3MultiSurface>
        </uro:WallSurface>
      </uro:boundedBy>
      <uro:boundedBy>
        <uro:WallSurface gml:id="line_789346ed-97ce-4bda-9d3e-c91682032648">
          <core:creationDate>2024-03-22</core:creationDate>
          <uro:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:CompositeSurface gml:id="poly_800b7ccb-2d93-46d6-9acf-aef531caae15">
                  <gml:surfaceMember>
                    <gml:Polygon gml:id="fme-gen-34c81548-068e-4767-a610-81ac8969652b">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929478700529216 130.56507647174683 3.3725579834128867 32.929475562888015 130.56507228215463 3.3725579834128867 32.929478700529216 130.56507647174683 2.8725533090642776 32.929478700529216 130.56507647174683 3.3725579834128867</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon gml:id="fme-gen-485f63b8-bb15-479e-a32c-f946caf99bfc">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929478700529216 130.56507647174683 2.8725533090642776 32.929475562888015 130.56507228215463 3.3725579834128867 32.929475562888015 130.56507228215463 2.8725533090642776 32.929478700529216 130.56507647174683 2.8725533090642776</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </uro:lod3MultiSurface>
        </uro:WallSurface>
      </uro:boundedBy>
      <uro:boundedBy>
        <uro:WallSurface gml:id="line_47d1cc4d-f5af-420a-9ef6-6aeb02b1107a">
          <core:creationDate>2024-03-22</core:creationDate>
          <uro:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:CompositeSurface gml:id="poly_f60ba630-be7a-45a7-a6cb-cf0aa246b5e7">
                  <gml:surfaceMember>
                    <gml:Polygon gml:id="fme-gen-10f8323c-291a-401b-a22e-28ebb80419f7">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929478752146466 130.56506892376652 3.3725579834128867 32.92948188978779 130.56507311335878 3.3725579834128867 32.929478752146466 130.56506892376652 2.8725533090642776 32.929478752146466 130.56506892376652 3.3725579834128867</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon gml:id="fme-gen-fde2b48c-93f6-4532-9453-21773044c6a5">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929478752146466 130.56506892376652 2.8725533090642776 32.92948188978779 130.56507311335878 3.3725579834128867 32.92948188978779 130.56507311335878 2.8725533090642776 32.929478752146466 130.56506892376652 2.8725533090642776</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </uro:lod3MultiSurface>
        </uro:WallSurface>
      </uro:boundedBy>
      <uro:boundedBy>
        <uro:WallSurface gml:id="line_25fd5cd3-2cb7-4d42-9a49-376c3052e36f">
          <core:creationDate>2024-03-22</core:creationDate>
          <uro:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:CompositeSurface gml:id="poly_fe4ad5b9-9a7e-49de-91a9-092f743cba7d">
                  <gml:surfaceMember>
                    <gml:Polygon gml:id="fme-gen-0493b0f4-aae1-44ed-a0cd-162fc085f671">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929475562888015 130.56507228215463 3.3725579834128867 32.929478752146466 130.56506892376652 3.3725579834128867 32.929475562888015 130.56507228215463 2.8725533090642776 32.929475562888015 130.56507228215463 3.3725579834128867</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon gml:id="fme-gen-0320c7ea-4812-429f-a7c1-f0de8b3c38d4">
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>32.929475562888015 130.56507228215463 2.8725533090642776 32.929478752146466 130.56506892376652 3.3725579834128867 32.929478752146466 130.56506892376652 2.8725533090642776 32.929475562888015 130.56507228215463 2.8725533090642776</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </uro:lod3MultiSurface>
        </uro:WallSurface>
      </uro:boundedBy>
      <uro:class codeSpace="../../codelists/OtherConstruction_class.xml">09</uro:class>
      <uro:function codeSpace="../../codelists/OtherConstruction_function.xml">0902</uro:function>
      </uro:OtherConstruction>
  </core:cityObjectMember>
</core:CityModel>
