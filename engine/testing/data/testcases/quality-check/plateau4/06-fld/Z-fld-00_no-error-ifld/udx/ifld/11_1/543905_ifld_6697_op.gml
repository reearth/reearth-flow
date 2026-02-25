<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../../schemas/iur/uro/3.1/urbanObject.xsd https://www.geospatial.jp/iur/urf/3.1 ../../../schemas/iur/urf/3.1/urbanFunction.xsd">
  <gml:boundedBy>
    <gml:Envelope srsDimension="3" srsName="http://www.opengis.net/def/crs/EPSG/0/6697">
      <gml:lowerCorner>36.0 139.69 10.0</gml:lowerCorner>
      <gml:upperCorner>36.01 139.70 12.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <wtr:WaterBody gml:id="ifld_test_single_polygon">
      <gml:name>さいたま市内水浸水想定区域</gml:name>
      <core:creationDate>2024-02-29</core:creationDate>
      <wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
      <wtr:function codeSpace="../../../codelists/WaterBody_function.xml">4</wtr:function>
      <wtr:lod1MultiSurface>
        <gml:MultiSurface srsDimension="3" srsName="http://www.opengis.net/def/crs/EPSG/0/6697">
          <gml:surfaceMember>
            <gml:Polygon>
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>36.001 139.691 11.0 36.001 139.693 11.0 36.002 139.692 11.0 36.001 139.691 11.0</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </wtr:lod1MultiSurface>
      <uro:floodingRiskAttribute>
        <uro:InlandFloodingRiskAttribute>
          <uro:description codeSpace="../../../codelists/InlandFloodingRiskAttribute_description.xml">1</uro:description>
          <uro:rankOrg codeSpace="../../../codelists/InlandFloodingRiskAttribute_rankOrg.xml">1</uro:rankOrg>
        </uro:InlandFloodingRiskAttribute>
      </uro:floodingRiskAttribute>
      <uro:wtrDataQualityAttribute>
        <uro:DataQualityAttribute>
          <uro:geometrySrcDescLod1 codeSpace="../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</uro:geometrySrcDescLod1>
          <uro:thematicSrcDesc codeSpace="../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
        </uro:DataQualityAttribute>
      </uro:wtrDataQualityAttribute>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
