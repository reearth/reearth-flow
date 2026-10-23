<?xml version="1.0" encoding="UTF-8"?>
<!--
  Water body with a closed LOD2 solid whose top face is wound the wrong way.
  One wtr:WaterBody in tamana-shi (EPSG:6697) whose LOD2 solid references the
  faces of its boundary surfaces: the top face of the wtr:WaterSurface, the
  bottom face of the wtr:WaterGroundSurface and the four side faces of the
  core:ClosureSurface. The top face is wound the same way as the bottom face,
  so it faces the opposite way to the other five faces.

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
    <wtr:WaterBody gml:id="wtr_dc960de7-8c95-4bb5-b4bd-57619fef0d0b">
      <core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
      <core:adeOfAbstractCityObject>
        <urc:DataQualityAttribute>
          <urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
          <urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
        </urc:DataQualityAttribute>
      </core:adeOfAbstractCityObject>
      <core:boundary>
        <wtr:WaterSurface gml:id="wtrs_310fcfea-34d4-4d89-916b-5901f395a178">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <!-- Top (z 2.0), wound the same way as the bottom face -->
                <gml:Polygon gml:id="poly_724bce99-5b67-42a4-9661-4f044d46a8b9">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 2.0 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.561 2.0 32.926 130.561 2.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
        </wtr:WaterSurface>
      </core:boundary>
      <core:boundary>
        <wtr:WaterGroundSurface gml:id="wtrg_b1e876ad-b85c-4e8e-baf2-6857f17a1791">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <!-- Bottom (z 1.8) -->
                <gml:Polygon gml:id="poly_0c14247c-783c-4093-8c3e-46dca840e362">
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
        <core:ClosureSurface gml:id="clos_003bccdc-9e76-4b52-bbfe-d0e87347ad7d">
          <core:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <!-- West side (longitude 130.561) -->
                <gml:Polygon gml:id="poly_781ab072-7cb6-4171-8cda-1b86aaba3102">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.927 130.561 1.8 32.927 130.561 2.0 32.926 130.561 2.0 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- East side (longitude 130.562) -->
                <gml:Polygon gml:id="poly_89ed06a8-1a9f-47f8-a58a-fb5ecc08c17e">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.562 1.8 32.926 130.562 2.0 32.927 130.562 2.0 32.927 130.562 1.8 32.926 130.562 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- South side (latitude 32.926) -->
                <gml:Polygon gml:id="poly_788cfbef-3761-4d3f-9a4c-8c401d108747">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>32.926 130.561 1.8 32.926 130.561 2.0 32.926 130.562 2.0 32.926 130.562 1.8 32.926 130.561 1.8</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <!-- North side (latitude 32.927) -->
                <gml:Polygon gml:id="poly_3c02febc-a7ff-47e9-b21d-db2a03831f95">
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
        <gml:Solid gml:id="solid_dd50abe4-ee70-47a8-933f-328a4337ad88">
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly_0c14247c-783c-4093-8c3e-46dca840e362"/>
              <gml:surfaceMember xlink:href="#poly_724bce99-5b67-42a4-9661-4f044d46a8b9"/>
              <gml:surfaceMember xlink:href="#poly_781ab072-7cb6-4171-8cda-1b86aaba3102"/>
              <gml:surfaceMember xlink:href="#poly_89ed06a8-1a9f-47f8-a58a-fb5ecc08c17e"/>
              <gml:surfaceMember xlink:href="#poly_788cfbef-3761-4d3f-9a4c-8c401d108747"/>
              <gml:surfaceMember xlink:href="#poly_3c02febc-a7ff-47e9-b21d-db2a03831f95"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </core:lod2Solid>
      <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1020</wtr:class>
    </wtr:WaterBody>
  </core:cityObjectMember>
</core:CityModel>
