<?xml version="1.0" encoding="UTF-8"?>
<!--
  CityGML 3.0: defines trafficarea1, referenced by feature_xlink_referrer.gml.
  Coordinates are JGD2011 geographic (lon lat height) — Tokyo area, EPSG:6697.
-->
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:tran="http://www.opengis.net/citygml/transportation/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  gml:id="feature_xlink_definition">

  <core:cityObjectMember>
    <tran:Road gml:id="road1">
      <core:boundary>
        <tran:TrafficArea gml:id="trafficarea1">
          <core:lod2MultiSurface>
            <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <!-- lon lat height — road surface adjacent to building1 footprint -->
                      <gml:posList>139.7455 35.6586 0.5 139.7456 35.6586 0.5 139.7456 35.6587 0.5 139.7455 35.6587 0.5 139.7455 35.6586 0.5</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:lod2MultiSurface>
        </tran:TrafficArea>
      </core:boundary>
    </tran:Road>
  </core:cityObjectMember>

</core:CityModel>
