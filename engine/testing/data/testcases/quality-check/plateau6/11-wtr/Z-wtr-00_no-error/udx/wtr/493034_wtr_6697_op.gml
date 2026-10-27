<?xml version="1.0" encoding="UTF-8"?>
<!--
  Valid water body with geometry at LOD0 to LOD3.
  One wtr:WaterBody in tamana-shi (EPSG:6697):
  LOD0: a centre line (core:lod0MultiCurve).
  LOD1: the water surface (wtr:WaterSurface core:lod1MultiSurface).
  LOD2, LOD3: a closed box solid whose shell references the faces of its
  boundary surfaces: the top face of the wtr:WaterSurface, the bottom face of
  the wtr:WaterGroundSurface and the four side faces of the core:ClosureSurface.
  Every face is wound counter-clockwise seen from outside, read as
  (longitude, latitude).

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
    <wtr:WaterBody gml:id="wtr_7fe93243-d2c1-4137-921f-09e6f206d5e1">
      <core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
      <core:adeOfAbstractCityObject>
        <urc:DataQualityAttribute>
          <urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
          <urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
        </urc:DataQualityAttribute>
      </core:adeOfAbstractCityObject>
      <core:boundary>
        <wtr:WaterSurface gml:id="wtrs_845c1fb6-3e37-41cf-8f5f-06062c86365a">
          <core:lod1MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_302c58b8-9b6d-4707-be6e-e2f317e5ecdd">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.561 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod1MultiSurface>
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_4dd52b80-b325-448d-b4b5-44d820222d64">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.561 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_84d72971-b343-42c8-863c-4cef55621518">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.561 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </wtr:WaterSurface>
      </core:boundary>
      <core:boundary>
        <wtr:WaterGroundSurface gml:id="wtrg_ee4e3975-c0a0-420b-84ea-80c9fd99bfe2">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_a8c31ce1-63a1-46e5-95ef-e4e55d91ae29">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.562 1.8 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_3d0bce84-8fe6-4519-9dde-fe42179897dd">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.562 1.8 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </wtr:WaterGroundSurface>
      </core:boundary>
      <core:boundary>
        <core:ClosureSurface gml:id="clos_469ab2c0-561d-462f-acaf-8521db055261">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <!-- South side (latitude 32.926) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_442fc0fc-70b2-49d8-a5e8-b5bb9696618a">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.562 1.8 32.926 130.562 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- North side (latitude 32.927) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_3a2598a0-2a2c-4028-a027-953f37120eba">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.562 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.927 130.562 2.0 32.927 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- East side (longitude 130.562) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_aa57dd90-1c2f-4e02-bbb9-8aa226545626">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.927 130.562 1.8 32.927 130.562 2.0 32.926 130.562 2.0 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <!-- West side (longitude 130.561) -->
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_4f1c5894-3bb6-42af-a1c6-11863d144080">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.561 1.8 32.926 130.561 1.8 32.926 130.561 2.0 32.927 130.561 2.0 32.927 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
          <core:lod3MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_686a522f-4529-4010-bb59-3ecc36f18dbb">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.562 1.8 32.926 130.562 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_64ee8c5e-42c5-4482-8ad6-408dfafbe591">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.562 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.927 130.562 2.0 32.927 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_6e39d3e2-b188-45e4-8a53-e11aa19749b3">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.927 130.562 1.8 32.927 130.562 2.0 32.926 130.562 2.0 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_2801195c-0abf-4010-85a6-6498906afaa2">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.927 130.561 1.8 32.926 130.561 1.8 32.926 130.561 2.0 32.927 130.561 2.0 32.927 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod3MultiSurface>
        </core:ClosureSurface>
      </core:boundary>
      <core:lod0MultiCurve>
        <gml:MultiCurve>
          <gml:curveMember>
            <gml:LineString>
              <gml:posList>32.9265 130.561 2.0 32.9265 130.562 2.0</gml:posList>
            </gml:LineString>
          </gml:curveMember>
        </gml:MultiCurve>
      </core:lod0MultiCurve>
      <core:lod2Solid>
        <gml:Solid gml:id="solid_da614543-cf06-4228-aa5f-e3d96c63ab59">
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_4dd52b80-b325-448d-b4b5-44d820222d64"/>
              <gml:surfaceMember xlink:href="#poly_a8c31ce1-63a1-46e5-95ef-e4e55d91ae29"/>
              <gml:surfaceMember xlink:href="#poly_442fc0fc-70b2-49d8-a5e8-b5bb9696618a"/>
              <gml:surfaceMember xlink:href="#poly_3a2598a0-2a2c-4028-a027-953f37120eba"/>
              <gml:surfaceMember xlink:href="#poly_aa57dd90-1c2f-4e02-bbb9-8aa226545626"/>
              <gml:surfaceMember xlink:href="#poly_4f1c5894-3bb6-42af-a1c6-11863d144080"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod2Solid>
      <core:lod3Solid>
        <gml:Solid gml:id="solid_e0702c22-f984-4645-8fc6-fe92d92ef152">
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_84d72971-b343-42c8-863c-4cef55621518"/>
              <gml:surfaceMember xlink:href="#poly_3d0bce84-8fe6-4519-9dde-fe42179897dd"/>
              <gml:surfaceMember xlink:href="#poly_686a522f-4529-4010-bb59-3ecc36f18dbb"/>
              <gml:surfaceMember xlink:href="#poly_64ee8c5e-42c5-4482-8ad6-408dfafbe591"/>
              <gml:surfaceMember xlink:href="#poly_6e39d3e2-b188-45e4-8a53-e11aa19749b3"/>
              <gml:surfaceMember xlink:href="#poly_2801195c-0abf-4010-85a6-6498906afaa2"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod3Solid>
      <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1020</wtr:class>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
