<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd
https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd
http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd
http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd
http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd
http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd
http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd
http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd
http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd
http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd
http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd
">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>36.6470041354812 137.05268308385453 0</gml:lowerCorner>
            <gml:upperCorner>36.647798243275254 137.0537094956814 105.03314</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>

  <!-- 建物A: 座標(0,0)から(10,10)の正方形 -->
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_4d583b00-4ce9-49b5-9540-9d1bbca48e46">
      <gml:name>Building A</gml:name>
      <core:creationDate>2024-01-01</core:creationDate>
      <bldg:class codeSpace="http://www.sig3d.org/codelists/standard/building/2.0/_AbstractBuilding_class.xml">1000</bldg:class>
      <bldg:measuredHeight uom="m">5.0</bldg:measuredHeight>

      <!-- LOD0 FootPrint（建物フットプリント） -->
      <bldg:lod0FootPrint>
        <gml:MultiSurface>
          <gml:surfaceMember>
            <!-- 建物A のフットプリント（底面） -->
            <gml:Polygon gml:id="building_A_footprint">
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList srsDimension="3">
                    0.0 0.0 0.0
                    0.0 10.0 0.0
                    10.0 10.0 0.0
                    10.0 0.0 0.0
                    0.0 0.0 0.0
                  </gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </bldg:lod0FootPrint>
    </bldg:Building>
  </core:cityObjectMember>

  <!-- 建物B: 座標(5,5)から(15,15)の正方形 - 建物Aと重複 -->
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_8f724a11-6de8-42c1-8f5d-8b2ccda58f47">
      <gml:name>Building B</gml:name>
      <core:creationDate>2024-01-01</core:creationDate>
      <bldg:class codeSpace="http://www.sig3d.org/codelists/standard/building/2.0/_AbstractBuilding_class.xml">1000</bldg:class>
      <bldg:measuredHeight uom="m">5.0</bldg:measuredHeight>

      <!-- LOD0 FootPrint（建物フットプリント） -->
      <bldg:lod0FootPrint>
        <gml:MultiSurface>
          <gml:surfaceMember>
            <!-- 建物B のフットプリント（底面） - 建物Aと重複する -->
            <gml:Polygon gml:id="building_B_footprint">
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList srsDimension="3">
                    5.0 5.0 0.0
                    5.0 15.0 0.0
                    15.0 15.0 0.0
                    15.0 5.0 0.0
                    5.0 5.0 0.0
                  </gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </bldg:lod0FootPrint>
    </bldg:Building>
  </core:cityObjectMember>

</core:CityModel>
