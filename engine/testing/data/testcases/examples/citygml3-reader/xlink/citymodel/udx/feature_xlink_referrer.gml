<?xml version="1.0" encoding="UTF-8"?>
<!--
  CityGML 3.0: cross-file feature xlink via CityObjectRelation.
  Per OGC 21-006r2 §7.2.3: external reference = "filename.gml#gml:id"

  Coordinates are JGD2011 geographic (lon lat height) — Tokyo area, EPSG:6697.
  floorsurface1.relatedTo -> feature_xlink_definition.gml#trafficarea1
-->
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:con="http://www.opengis.net/citygml/construction/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  gml:id="feature_xlink_referrer">

  <core:cityObjectMember>
    <bldg:Building gml:id="building1">
      <core:boundary>
        <con:FloorSurface gml:id="floorsurface1">
          <core:relatedTo>
            <core:CityObjectRelation>
              <core:relationType>equal</core:relationType>
              <core:relatedTo xlink:href="feature_xlink_definition.gml#trafficarea1"/>
            </core:CityObjectRelation>
          </core:relatedTo>
          <core:lod2MultiSurface>
            <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <!-- lon lat height (degrees, degrees, metres) — small footprint in Tokyo -->
                      <gml:posList>139.7454 35.6586 10.0 139.7455 35.6586 10.0 139.7455 35.6587 10.0 139.7454 35.6587 10.0 139.7454 35.6586 10.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
        </con:FloorSurface>
      </core:boundary>
    </bldg:Building>
  </core:cityObjectMember>

</core:CityModel>
