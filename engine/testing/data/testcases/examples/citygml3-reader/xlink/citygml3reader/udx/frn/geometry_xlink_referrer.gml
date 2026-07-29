<?xml version="1.0" encoding="UTF-8"?>
<!--
  CityGML 3.0: cross-file geometry xlink via ImplicitGeometry.relativeGeometry.
  Per OGC 21-006r2: geometry XLinks across top-level features are only allowed
  for ImplicitGeometry.

  Coordinates are JGD2011 geographic (lon lat height) — Tokyo area, EPSG:6697.
  furniture2.relativeGeometry -> geometry_xlink_definition.gml#geom_template1
-->
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  gml:id="geometry_xlink_referrer">

  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>139.7457 35.6587 0.0</gml:lowerCorner>
      <gml:upperCorner>139.7458 35.6588 5.0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>

  <core:cityObjectMember>
    <frn:CityFurniture gml:id="furniture2">
      <core:lod2ImplicitRepresentation>
        <core:ImplicitGeometry>
          <!-- Identity matrix: template coordinates are already in geographic range -->
          <core:transformationMatrix>1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1</core:transformationMatrix>
          <core:referencePoint>
            <gml:Point>
              <gml:pos>139.7457 35.6587 5.0</gml:pos>
            </gml:Point>
          </core:referencePoint>
          <!-- External xlink to geometry template defined in geometry_xlink_definition.gml -->
          <core:relativeGeometry xlink:href="geometry_xlink_definition.gml#geom_template1"/>
        </core:ImplicitGeometry>
      </core:lod2ImplicitRepresentation>
    </frn:CityFurniture>
  </core:cityObjectMember>

</core:CityModel>
