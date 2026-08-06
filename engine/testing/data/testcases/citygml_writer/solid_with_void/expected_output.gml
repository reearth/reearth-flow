<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>35 135 0</gml:lowerCorner>
      <gml:upperCorner>35.002 135.002 20</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <bldg:Building gml:id="test-building-002">
      <bldg:lod0RoofEdge>
        <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
          <gml:surfaceMember>
            <gml:Polygon>
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>35 135 20 35.002 135 20 35.002 135.002 20 35 135.002 20 35 135 20</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </bldg:lod0RoofEdge>
      <bldg:lod1Solid>
        <gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
          <gml:exterior>
            <gml:CompositeSurface>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35 135 0 35.002 135 0 35 135.002 0 35 135 0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35 135 0 35 135 20 35.002 135 0 35 135 0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.002 135 0 35 135 20 35 135.002 0 35.002 135 0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35 135.002 0 35 135 20 35 135 0 35 135.002 0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:CompositeSurface>
          </gml:exterior>
          <gml:interior>
            <gml:CompositeSurface>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.0005 135.0005 1 35.001 135.0005 1 35.0005 135.001 1 35.0005 135.0005 1</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.0005 135.0005 1 35.0005 135.0005 5 35.001 135.0005 1 35.0005 135.0005 1</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.001 135.0005 1 35.0005 135.0005 5 35.0005 135.001 1 35.001 135.0005 1</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>35.0005 135.001 1 35.0005 135.0005 5 35.0005 135.0005 1 35.0005 135.001 1</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:CompositeSurface>
          </gml:interior>
        </gml:Solid>
      </bldg:lod1Solid>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>
