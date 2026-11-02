<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/bridge/2.0 http://schemas.opengis.net/citygml/bridge/2.0/bridge.xsd http://www.opengis.net/citygml/tunnel/2.0 http://schemas.opengis.net/citygml/tunnel/2.0/tunnel.xsd http://www.opengis.net/citygml/waterbody/2.0 http://schemas.opengis.net/citygml/waterbody/2.0/waterBody.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/vegetation/2.0 http://schemas.opengis.net/citygml/vegetation/2.0/vegetation.xsd http://www.opengis.net/citygml/cityfurniture/2.0 http://schemas.opengis.net/citygml/cityfurniture/2.0/cityFurniture.xsd http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>34.7715 137.3634 0</gml:lowerCorner>
      <gml:upperCorner>34.7716 137.3635 5</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_shared_0001">
      <bldg:measuredHeight uom="m">5.0</bldg:measuredHeight>
      <bldg:lod2Solid>
        <gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
          <gml:exterior>
            <gml:CompositeSurface>
              <gml:surfaceMember xlink:href="#poly_1"/>
              <gml:surfaceMember xlink:href="#poly_2"/>
              <gml:surfaceMember xlink:href="#poly_3"/>
              <gml:surfaceMember xlink:href="#poly_4"/>
            </gml:CompositeSurface>
          </gml:exterior>
        </gml:Solid>
      </bldg:lod2Solid>
      <bldg:boundedBy>
        <bldg:GroundSurface gml:id="bldg_ground_0001">
          <bldg:lod2MultiSurface>
            <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_1">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>34.7715 137.3634 0 34.7716 137.3634 0 34.7716 137.3635 0 34.7715 137.3635 0 34.7715 137.3634 0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </bldg:lod2MultiSurface>
        </bldg:GroundSurface>
      </bldg:boundedBy>
      <bldg:boundedBy>
        <bldg:RoofSurface gml:id="bldg_roof_0001">
          <bldg:lod2MultiSurface>
            <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_2">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>34.7715 137.3634 5 34.7715 137.3635 5 34.7716 137.3635 5 34.7716 137.3634 5 34.7715 137.3634 5</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </bldg:lod2MultiSurface>
        </bldg:RoofSurface>
      </bldg:boundedBy>
      <bldg:boundedBy>
        <bldg:WallSurface gml:id="bldg_wall_0001">
          <bldg:lod2MultiSurface>
            <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_3">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>34.7715 137.3634 0 34.7715 137.3634 5 34.7716 137.3634 5 34.7716 137.3634 0 34.7715 137.3634 0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </bldg:lod2MultiSurface>
        </bldg:WallSurface>
      </bldg:boundedBy>
      <bldg:boundedBy>
        <bldg:WallSurface gml:id="bldg_wall_0002">
          <bldg:lod2MultiSurface>
            <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_4">
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>34.7716 137.3635 0 34.7716 137.3635 5 34.7715 137.3635 5 34.7715 137.3635 0 34.7716 137.3635 0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </bldg:lod2MultiSurface>
        </bldg:WallSurface>
      </bldg:boundedBy>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>
