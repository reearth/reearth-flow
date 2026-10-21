<?xml version="1.0" encoding="UTF-8"?>
<!--
  Water body with a LOD2 solid whose top face intersects itself.
  One wtr:WaterBody in tamana-shi (EPSG:6697) whose LOD2 solid references the
  faces of its boundary surfaces: the top face of the wtr:WaterSurface, the
  bottom face of the wtr:WaterGroundSurface and the four side faces of the
  core:ClosureSurface. The top face visits its corners in the order
  P1 (32.926, 130.561), P3 (32.927, 130.562), P2 (32.926, 130.562),
  P4 (32.927, 130.561), so the edges P1-P3 and P2-P4 cross (a bow tie).

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
    <wtr:WaterBody gml:id="wtr_6812c892-f774-453c-9401-d74a2fae27f0">
      <core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
      <core:adeOfAbstractCityObject>
        <urc:DataQualityAttribute>
          <urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
          <urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
        </urc:DataQualityAttribute>
      </core:adeOfAbstractCityObject>
      <core:boundary>
        <wtr:WaterSurface gml:id="wtrs_432d68ea-cb83-4e52-8e0b-23fd9fb5cb87">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <!-- Top (z 2.0), a self-intersecting bow tie -->
                <gml:Polygon gml:id="poly_6c83bf7a-394e-4a2a-bfd7-8cf662fdaaa6">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.927 130.562 2.0 32.926 130.562 2.0 32.927 130.561 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
        </wtr:WaterSurface>
      </core:boundary>
      <core:boundary>
        <wtr:WaterGroundSurface gml:id="wtrg_6c4d96b6-992f-409a-8aef-d23e77b66e84">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <!-- Bottom (z 1.8) -->
                <gml:Polygon gml:id="poly_49d5a4a0-9219-4784-a9e5-480f75a44af4">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.562 1.8 32.927 130.562 1.8 32.927 130.561 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
        </wtr:WaterGroundSurface>
      </core:boundary>
      <core:boundary>
        <core:ClosureSurface gml:id="clos_d8660f31-45d3-40c3-bc6f-a7c657575062">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <!-- West side (longitude 130.561) -->
                <gml:Polygon gml:id="poly_9080b0ed-ef75-471a-b680-2bc94a1a2116">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- East side (longitude 130.562) -->
                <gml:Polygon gml:id="poly_7b5d6b6d-8ec2-479c-bd11-ca60d127b696">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.562 1.8 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- South side (latitude 32.926) -->
                <gml:Polygon gml:id="poly_cf7c60e6-d0a0-43ec-a4f7-9f017bb4dabd">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.561 2.0 32.926 130.562 2.0 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- North side (latitude 32.927) -->
                <gml:Polygon gml:id="poly_9eab92f2-88f9-4dd7-8e86-608470a74895">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.561 1.8 32.927 130.562 1.8 32.927 130.562 2.0 32.927 130.561 2.0 32.927 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
        </core:ClosureSurface>
      </core:boundary>
      <core:lod2Solid>
        <gml:Solid gml:id="solid_8f487537-7302-4fb6-816c-3309eb9811e7">
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_49d5a4a0-9219-4784-a9e5-480f75a44af4"/>
              <gml:surfaceMember xlink:href="#poly_6c83bf7a-394e-4a2a-bfd7-8cf662fdaaa6"/>
              <gml:surfaceMember xlink:href="#poly_9080b0ed-ef75-471a-b680-2bc94a1a2116"/>
              <gml:surfaceMember xlink:href="#poly_7b5d6b6d-8ec2-479c-bd11-ca60d127b696"/>
              <gml:surfaceMember xlink:href="#poly_cf7c60e6-d0a0-43ec-a4f7-9f017bb4dabd"/>
              <gml:surfaceMember xlink:href="#poly_9eab92f2-88f9-4dd7-8e86-608470a74895"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod2Solid>
      <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1020</wtr:class>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
