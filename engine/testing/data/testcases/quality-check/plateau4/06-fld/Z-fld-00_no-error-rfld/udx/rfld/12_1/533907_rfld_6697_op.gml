<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd https://www.geospatial.jp/iur/uro/3.1 ../../../schemas/iur/uro/3.1/urbanObject.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>35.37 139.98 14.0</gml:lowerCorner>
      <gml:upperCorner>35.38 139.99 36.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <wtr:WaterBody gml:id="rfld_test_single_polygon_12_1">
      <gml:name>河川浸水想定区域テスト</gml:name>
      <core:creationDate>2025-03-21</core:creationDate>
      <wtr:class codeSpace="../../../codelists/WaterBody_class.xml">1140</wtr:class>
      <wtr:function codeSpace="../../../codelists/WaterBody_function.xml">5</wtr:function>
      <wtr:lod1MultiSurface>
        <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
          <gml:surfaceMember>
            <gml:Polygon>
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>35.371 139.981 18.0 35.371 139.983 18.0 35.372 139.982 18.0 35.371 139.981 18.0</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </wtr:lod1MultiSurface>
      <uro:floodingRiskAttribute>
        <uro:RiverFloodingRiskAttribute>
          <uro:description codeSpace="../../../codelists/RiverFloodingRiskAttribute_description.xml">1</uro:description>
          <uro:rankOrg codeSpace="../../../codelists/RiverFloodingRiskAttribute_rankOrg.xml">1</uro:rankOrg>
        </uro:RiverFloodingRiskAttribute>
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
