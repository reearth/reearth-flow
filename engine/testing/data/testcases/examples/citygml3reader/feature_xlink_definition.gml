<?xml version="1.0" encoding="UTF-8"?>
<!-- CityGML 3.0: defines trafficarea1, referenced by feature_xlink_referrer.gml -->
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
            <gml:MultiSurface srsDimension="3">
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>0 0 0 10 0 0 10 5 0 0 5 0 0 0 0</gml:posList>
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
