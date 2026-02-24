<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>34.73480000 135.50200000 0</gml:lowerCorner>
      <gml:upperCorner>34.73515000 135.50245000 10.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_test_partial_001">
      <gml:name>テスト建物_部分接続</gml:name>
      <!-- BuildingPart A: Left bottom box (connected to B) -->
      <bldg:consistsOfBuildingPart>
        <bldg:BuildingPart gml:id="bldg_part_partial_A">
          <gml:name>BuildingPart A (connected to B)</gml:name>
          <gen:stringAttribute name="区名">
            <gen:value>淀川区</gen:value>
          </gen:stringAttribute>
          <bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
          <bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
          <bldg:measuredHeight uom="m">10.0</bldg:measuredHeight>
          <bldg:storeysAboveGround>2</bldg:storeysAboveGround>
          <bldg:lod2Solid>
            <gml:Solid>
              <gml:exterior>
                <gml:CompositeSurface>
                  <!-- Bottom face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50200000 0 34.73490000 135.50200000 0 34.73490000 135.50210000 0 34.73480000 135.50210000 0 34.73480000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- West face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50200000 0 34.73480000 135.50210000 0 34.73480000 135.50210000 10.0 34.73480000 135.50200000 10.0 34.73480000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- North face (SHARED WITH B) -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50210000 0 34.73490000 135.50210000 0 34.73490000 135.50210000 10.0 34.73480000 135.50210000 10.0 34.73480000 135.50210000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- East face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73490000 135.50210000 0 34.73490000 135.50200000 0 34.73490000 135.50200000 10.0 34.73490000 135.50210000 10.0 34.73490000 135.50210000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- South face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73490000 135.50200000 0 34.73480000 135.50200000 0 34.73480000 135.50200000 10.0 34.73490000 135.50200000 10.0 34.73490000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- Top face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50200000 10.0 34.73480000 135.50210000 10.0 34.73490000 135.50210000 10.0 34.73490000 135.50200000 10.0 34.73480000 135.50200000 10.0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:exterior>
            </gml:Solid>
          </bldg:lod2Solid>
        </bldg:BuildingPart>
      </bldg:consistsOfBuildingPart>
      <!-- BuildingPart B: Left top box (connected to A, shares north face with A) -->
      <bldg:consistsOfBuildingPart>
        <bldg:BuildingPart gml:id="bldg_part_partial_B">
          <gml:name>BuildingPart B (connected to A)</gml:name>
          <gen:stringAttribute name="区名">
            <gen:value>淀川区</gen:value>
          </gen:stringAttribute>
          <bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
          <bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
          <bldg:measuredHeight uom="m">10.0</bldg:measuredHeight>
          <bldg:storeysAboveGround>2</bldg:storeysAboveGround>
          <bldg:lod2Solid>
            <gml:Solid>
              <gml:exterior>
                <gml:CompositeSurface>
                  <!-- Bottom face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50210000 0 34.73490000 135.50210000 0 34.73490000 135.50220000 0 34.73480000 135.50220000 0 34.73480000 135.50210000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- West face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50210000 0 34.73480000 135.50220000 0 34.73480000 135.50220000 10.0 34.73480000 135.50210000 10.0 34.73480000 135.50210000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- North face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50220000 0 34.73490000 135.50220000 0 34.73490000 135.50220000 10.0 34.73480000 135.50220000 10.0 34.73480000 135.50220000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- East face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73490000 135.50220000 0 34.73490000 135.50210000 0 34.73490000 135.50210000 10.0 34.73490000 135.50220000 10.0 34.73490000 135.50220000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- South face (SHARED WITH A) -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73490000 135.50210000 0 34.73480000 135.50210000 0 34.73480000 135.50210000 10.0 34.73490000 135.50210000 10.0 34.73490000 135.50210000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- Top face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50210000 10.0 34.73480000 135.50220000 10.0 34.73490000 135.50220000 10.0 34.73490000 135.50210000 10.0 34.73480000 135.50210000 10.0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:exterior>
            </gml:Solid>
          </bldg:lod2Solid>
        </bldg:BuildingPart>
      </bldg:consistsOfBuildingPart>
      <!-- BuildingPart C: Isolated box (NOT connected to A or B) -->
      <bldg:consistsOfBuildingPart>
        <bldg:BuildingPart gml:id="bldg_part_partial_C">
          <gml:name>BuildingPart C (isolated)</gml:name>
          <gen:stringAttribute name="区名">
            <gen:value>淀川区</gen:value>
          </gen:stringAttribute>
          <bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
          <bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
          <bldg:measuredHeight uom="m">10.0</bldg:measuredHeight>
          <bldg:storeysAboveGround>2</bldg:storeysAboveGround>
          <bldg:lod2Solid>
            <gml:Solid>
              <gml:exterior>
                <gml:CompositeSurface>
                  <!-- Bottom face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73500000 135.50230000 0 34.73510000 135.50230000 0 34.73510000 135.50240000 0 34.73500000 135.50240000 0 34.73500000 135.50230000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- West face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73500000 135.50230000 0 34.73500000 135.50240000 0 34.73500000 135.50240000 10.0 34.73500000 135.50230000 10.0 34.73500000 135.50230000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- North face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73500000 135.50240000 0 34.73510000 135.50240000 0 34.73510000 135.50240000 10.0 34.73500000 135.50240000 10.0 34.73500000 135.50240000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- East face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73510000 135.50240000 0 34.73510000 135.50230000 0 34.73510000 135.50230000 10.0 34.73510000 135.50240000 10.0 34.73510000 135.50240000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- South face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73510000 135.50230000 0 34.73500000 135.50230000 0 34.73500000 135.50230000 10.0 34.73510000 135.50230000 10.0 34.73510000 135.50230000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <!-- Top face -->
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73500000 135.50230000 10.0 34.73500000 135.50240000 10.0 34.73510000 135.50240000 10.0 34.73510000 135.50230000 10.0 34.73500000 135.50230000 10.0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:exterior>
            </gml:Solid>
          </bldg:lod2Solid>
        </bldg:BuildingPart>
      </bldg:consistsOfBuildingPart>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>
