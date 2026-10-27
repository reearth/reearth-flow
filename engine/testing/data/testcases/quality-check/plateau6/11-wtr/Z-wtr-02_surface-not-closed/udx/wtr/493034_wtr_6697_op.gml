<?xml version="1.0" encoding="UTF-8"?>
<!--
  Water body with a LOD2 solid that does not close.
  One wtr:WaterBody in tamana-shi (EPSG:6697) whose LOD2 solid references the
  faces of its boundary surfaces: the top face of the wtr:WaterSurface and the
  four side faces of the core:ClosureSurface. The bottom face is missing, which
  leaves the four lower edges without a neighbouring face.

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
    <wtr:WaterBody gml:id="wtr_43cf68d5-79f5-4497-9c5c-1db8e0a4dc03">
      <core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
      <core:adeOfAbstractCityObject>
        <urc:DataQualityAttribute>
          <urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
          <urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
        </urc:DataQualityAttribute>
      </core:adeOfAbstractCityObject>
      <core:boundary>
        <wtr:WaterSurface gml:id="wtrs_0ab933f1-5b3d-443c-8b25-452c08086f83">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <!-- Top (z 2.0) -->
                <gml:Polygon gml:id="poly_24d5f55b-1043-45ae-aa73-c8a2503e25e0">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.927 130.561 2.0 32.927 130.562 2.0 32.926 130.562 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
        </wtr:WaterSurface>
      </core:boundary>
      <core:boundary>
        <core:ClosureSurface gml:id="clos_81c1baba-3e64-485f-aaff-c98a97f02b2f">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <!-- South side (latitude 32.926) -->
                <gml:Polygon gml:id="poly_2eaef1f8-b682-4b04-adf7-8eb74648cf02">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.561 2.0 32.926 130.562 2.0 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- North side (latitude 32.927) -->
                <gml:Polygon gml:id="poly_6a55de08-35bb-4348-b8f0-3c498510e775">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.561 1.8 32.927 130.562 1.8 32.927 130.562 2.0 32.927 130.561 2.0 32.927 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- West side (longitude 130.561) -->
                <gml:Polygon gml:id="poly_dd3dd85c-040c-45b7-9b61-1f01d7eec9a9">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- East side (longitude 130.562) -->
                <gml:Polygon gml:id="poly_161f4f39-636c-403c-9027-cc537089b465">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.562 1.8 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
        </core:ClosureSurface>
      </core:boundary>
      <core:lod2Solid>
        <gml:Solid gml:id="solid_e5de222d-0b87-49d5-bcf0-f1c8161d451a">
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_24d5f55b-1043-45ae-aa73-c8a2503e25e0"/>
              <gml:surfaceMember xlink:href="#poly_2eaef1f8-b682-4b04-adf7-8eb74648cf02"/>
              <gml:surfaceMember xlink:href="#poly_6a55de08-35bb-4348-b8f0-3c498510e775"/>
              <gml:surfaceMember xlink:href="#poly_dd3dd85c-040c-45b7-9b61-1f01d7eec9a9"/>
              <gml:surfaceMember xlink:href="#poly_161f4f39-636c-403c-9027-cc537089b465"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod2Solid>
      <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1020</wtr:class>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
