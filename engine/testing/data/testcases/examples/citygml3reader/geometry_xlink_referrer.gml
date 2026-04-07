<?xml version="1.0" encoding="UTF-8"?>
<!--
  CityGML 3.0: cross-file geometry xlink via ImplicitGeometry.relativeGeometry.
  Per OGC 21-006r2: geometry XLinks across top-level features are only allowed
  for ImplicitGeometry.

  furniture2.relativeGeometry -> geometry_xlink_definition.gml#geom_template1
-->
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  gml:id="geometry_xlink_referrer">

  <core:cityObjectMember>
    <frn:CityFurniture gml:id="furniture2">
      <core:lod2ImplicitRepresentation>
        <core:ImplicitGeometry>
          <!-- Translate prototype to (10, 20, 0) -->
          <core:transformationMatrix>1 0 0 10 0 1 0 20 0 0 1 0 0 0 0 1</core:transformationMatrix>
          <core:referencePoint>
            <gml:Point srsDimension="3">
              <gml:pos>10 20 0</gml:pos>
            </gml:Point>
          </core:referencePoint>
          <!-- External xlink to geometry template defined in geometry_xlink_definition.gml -->
          <core:relativeGeometry xlink:href="geometry_xlink_definition.gml#geom_template1"/>
        </core:ImplicitGeometry>
      </core:lod2ImplicitRepresentation>
    </frn:CityFurniture>
  </core:cityObjectMember>

</core:CityModel>
