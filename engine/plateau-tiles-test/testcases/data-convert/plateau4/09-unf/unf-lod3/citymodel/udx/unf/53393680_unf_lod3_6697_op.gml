<?xml version='1.0' encoding='utf-8'?>
<core:CityModel xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/cityfurniture/2.0 http://schemas.opengis.net/citygml/cityfurniture/2.0/cityFurniture.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>35.655 139.763 -6.0</gml:lowerCorner>
      <gml:upperCorner>35.657 139.765 0.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <!-- SewerPipe with LOD2 Solid and LOD3 CompositeSurface -->
  <core:cityObjectMember>
    <uro:SewerPipe gml:id="unf_sewer_001">
      <core:creationDate>2024-03-22</core:creationDate>
      <frn:class codeSpace="../../codelists/UtilityNetworkElement_occupierType.xml">2</frn:class>
      <frn:lod2Geometry>
        <gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
          <gml:exterior>
            <gml:CompositeSurface>
              <!-- Bottom face -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="sewer_lod2_bottom">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.6560 139.7635 -5.0 35.6560 139.7640 -5.0 35.6565 139.7640 -5.0 35.6565 139.7635 -5.0 35.6560 139.7635 -5.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- Top face -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="sewer_lod2_top">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.6560 139.7635 -3.0 35.6565 139.7635 -3.0 35.6565 139.7640 -3.0 35.6560 139.7640 -3.0 35.6560 139.7635 -3.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- Front face -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="sewer_lod2_front">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.6560 139.7635 -5.0 35.6565 139.7635 -5.0 35.6565 139.7635 -3.0 35.6560 139.7635 -3.0 35.6560 139.7635 -5.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- Back face -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="sewer_lod2_back">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.6560 139.7640 -5.0 35.6560 139.7640 -3.0 35.6565 139.7640 -3.0 35.6565 139.7640 -5.0 35.6560 139.7640 -5.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- Left face -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="sewer_lod2_left">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.6560 139.7635 -5.0 35.6560 139.7635 -3.0 35.6560 139.7640 -3.0 35.6560 139.7640 -5.0 35.6560 139.7635 -5.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- Right face -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="sewer_lod2_right">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.6565 139.7635 -5.0 35.6565 139.7640 -5.0 35.6565 139.7640 -3.0 35.6565 139.7635 -3.0 35.6565 139.7635 -5.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:CompositeSurface>
          </gml:exterior>
        </gml:Solid>
      </frn:lod2Geometry>
      <frn:lod3Geometry>
        <gml:CompositeSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
          <!-- Bottom panel -->
          <gml:surfaceMember>
            <gml:Polygon gml:id="sewer_lod3_panel_bottom">
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>35.6560 139.7635 -5.5 35.6560 139.7637 -5.5 35.6562 139.7637 -5.5 35.6562 139.7635 -5.5 35.6560 139.7635 -5.5</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
          <!-- Front panel -->
          <gml:surfaceMember>
            <gml:Polygon gml:id="sewer_lod3_panel_front">
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>35.6560 139.7635 -5.5 35.6562 139.7635 -5.5 35.6562 139.7635 -4.0 35.6560 139.7635 -4.0 35.6560 139.7635 -5.5</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
          <!-- Left panel -->
          <gml:surfaceMember>
            <gml:Polygon gml:id="sewer_lod3_panel_left">
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>35.6560 139.7635 -5.5 35.6560 139.7635 -4.0 35.6560 139.7637 -4.0 35.6560 139.7637 -5.5 35.6560 139.7635 -5.5</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:CompositeSurface>
      </frn:lod3Geometry>
      <uro:occupierType codeSpace="../../codelists/UtilityNetworkElement_occupierType.xml">2</uro:occupierType>
      <uro:occupierName codeSpace="../../codelists/UtilityNetworkElement_occupierName.xml">1</uro:occupierName>
      <uro:year>2020</uro:year>
      <uro:yearType codeSpace="../../codelists/UtilityNetworkElement_yearType.xml">1</uro:yearType>
      <uro:administrator codeSpace="../../codelists/UtilityNetworkElement_administrator.xml">1</uro:administrator>
      <uro:depth uom="m">5.0</uro:depth>
      <uro:minDepth uom="m">5.5</uro:minDepth>
      <uro:maxDepth uom="m">4.0</uro:maxDepth>
      <uro:maxWidth uom="m">0.5</uro:maxWidth>
      <uro:material codeSpace="../../codelists/UtilityNetworkElement_material.xml">1</uro:material>
      <uro:frnDataQualityAttribute>
        <uro:DataQualityAttribute>
          <uro:geometrySrcDesc codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">5</uro:geometrySrcDesc>
          <uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">5</uro:thematicSrcDesc>
          <uro:appearanceSrcDesc codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">5</uro:appearanceSrcDesc>
        </uro:DataQualityAttribute>
      </uro:frnDataQualityAttribute>
    </uro:SewerPipe>
  </core:cityObjectMember>
</core:CityModel>
