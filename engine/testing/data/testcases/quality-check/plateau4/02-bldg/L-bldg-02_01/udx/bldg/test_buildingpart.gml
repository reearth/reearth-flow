<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd  http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd  http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd  http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd  http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd  http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd  http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd  http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>34.73450000 135.50200000 0</gml:lowerCorner>
      <gml:upperCorner>34.73470000 135.50220000 12.5</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_test_001">
      <gml:name>テスト建物</gml:name>
      <bldg:consistsOfBuildingPart>
        <bldg:BuildingPart gml:id="bldg_part_test_001">
          <gml:name>テスト建物部分</gml:name>
          <gen:measureAttribute name="計測周辺長">
            <gen:value uom="m">80.0</gen:value>
          </gen:measureAttribute>
          <gen:intAttribute name="ユーザーID">
            <gen:value>999999</gen:value>
          </gen:intAttribute>
          <gen:stringAttribute name="区名">
            <gen:value>淀川区</gen:value>
          </gen:stringAttribute>
          <bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
          <bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
          <bldg:measuredHeight uom="m">12.5</bldg:measuredHeight>
          <bldg:storeysAboveGround>2</bldg:storeysAboveGround>
          <bldg:lod0FootPrint>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>34.73450000 135.50200000 0 34.73450000 135.50220000 0 34.73470000 135.50220000 0 34.73470000 135.50200000 0 34.73450000 135.50200000 0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </bldg:lod0FootPrint>
          <bldg:lod1Solid>
            <gml:Solid>
              <gml:exterior>
                <gml:CompositeSurface>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73450000 135.50200000 0 34.73470000 135.50200000 0 34.73470000 135.50220000 0 34.73450000 135.50220000 0 34.73450000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73450000 135.50200000 0 34.73450000 135.50220000 0 34.73450000 135.50220000 12.5 34.73450000 135.50200000 12.5 34.73450000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73450000 135.50220000 0 34.73470000 135.50220000 0 34.73470000 135.50220000 12.5 34.73450000 135.50220000 12.5 34.73450000 135.50220000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73470000 135.50220000 0 34.73470000 135.50200000 0 34.73470000 135.50200000 12.5 34.73470000 135.50220000 12.5 34.73470000 135.50220000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73470000 135.50200000 0 34.73450000 135.50200000 0 34.73450000 135.50200000 12.5 34.73470000 135.50200000 12.5 34.73470000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73450000 135.50200000 12.5 34.73450000 135.50220000 12.5 34.73470000 135.50220000 12.5 34.73470000 135.50200000 12.5 34.73450000 135.50200000 12.5</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                </gml:CompositeSurface>
              </gml:exterior>
            </gml:Solid>
          </bldg:lod1Solid>
        </bldg:BuildingPart>
      </bldg:consistsOfBuildingPart>
    </bldg:Building>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_test_002">
      <gml:name>テスト建物2</gml:name>
      <bldg:consistsOfBuildingPart>
        <bldg:BuildingPart gml:id="bldg_part_test_002">
          <gml:name>テスト建物部分2 LOD2</gml:name>
          <gen:measureAttribute name="計測周辺長">
            <gen:value uom="m">60.0</gen:value>
          </gen:measureAttribute>
          <gen:intAttribute name="ユーザーID">
            <gen:value>888888</gen:value>
          </gen:intAttribute>
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
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50200000 0 34.73495000 135.50200000 0 34.73495000 135.50215000 0 34.73480000 135.50215000 0 34.73480000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50200000 0 34.73480000 135.50215000 0 34.73480000 135.50215000 10.0 34.73480000 135.50200000 10.0 34.73480000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50215000 0 34.73495000 135.50215000 0 34.73495000 135.50215000 10.0 34.73480000 135.50215000 10.0 34.73480000 135.50215000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73495000 135.50215000 0 34.73495000 135.50200000 0 34.73495000 135.50200000 10.0 34.73495000 135.50215000 10.0 34.73495000 135.50215000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73495000 135.50200000 0 34.73480000 135.50200000 0 34.73480000 135.50200000 10.0 34.73495000 135.50200000 10.0 34.73495000 135.50200000 0</gml:posList>
                        </gml:LinearRing>
                      </gml:exterior>
                    </gml:Polygon>
                  </gml:surfaceMember>
                  <gml:surfaceMember>
                    <gml:Polygon>
                      <gml:exterior>
                        <gml:LinearRing>
                          <gml:posList>34.73480000 135.50200000 10.0 34.73480000 135.50215000 10.0 34.73495000 135.50215000 10.0 34.73495000 135.50200000 10.0 34.73480000 135.50200000 10.0</gml:posList>
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
