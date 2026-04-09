<?xml version="1.0" encoding="UTF-8"?>
<!--
  CityGML 3.0: defines geom_template1, referenced by geometry_xlink_referrer.gml.
  ImplicitGeometry reuse: the geometry template is stored once here; other features
  may reference it via xlink:href on core:relativeGeometry (the one exception to the
  rule that geometry XLinks across top-level features are forbidden).

  Coordinates are JGD2011 geographic (lon lat height) — Tokyo area, EPSG:6697.
-->
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  gml:id="geometry_xlink_definition">

  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>139.7457 35.6587 0.0</gml:lowerCorner>
      <gml:upperCorner>139.7458 35.6588 5.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>

  <core:cityObjectMember>
    <frn:CityFurniture gml:id="furniture1">
      <core:lod2ImplicitRepresentation>
        <core:ImplicitGeometry>
          <!-- Identity matrix: no rotation/scaling/translation -->
          <core:transformationMatrix>1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</core:transformationMatrix>
          <core:referencePoint>
            <gml:Point>
              <gml:pos>139.7457 35.6587 5.0</gml:pos>
            </gml:Point>
          </core:referencePoint>
          <!-- geom_template1: referenced from geometry_xlink_referrer.gml -->
          <core:relativeGeometry>
            <gml:MultiSurface gml:id="geom_template1">
              <gml:surfaceMember>
                <gml:Polygon>
                  <gml:exterior>
                    <gml:LinearRing>
                      <gml:posList>139.7457 35.6587 5.0 139.7458 35.6587 5.0 139.7458 35.6588 5.0 139.7457 35.6588 5.0 139.7457 35.6587 5.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </core:relativeGeometry>
        </core:ImplicitGeometry>
      </core:lod2ImplicitRepresentation>
    </frn:CityFurniture>
  </core:cityObjectMember>

</core:CityModel>
