<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <gml:boundedBy>
    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
      <gml:lowerCorner>35 135 0</gml:lowerCorner>
      <gml:upperCorner>35.001 135.001 0</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
  <core:cityObjectMember>
    <bldg:Building gml:id="textured-building-001">
      <bldg:lod2MultiSurface>
        <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
          <gml:surfaceMember>
            <gml:Polygon gml:id="poly_1">
              <gml:exterior>
                <gml:LinearRing gml:id="poly_1_e">
                  <gml:posList>35 135 0 35 135.001 0 35.001 135.001 0 35.001 135 0 35 135 0</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:MultiSurface>
      </bldg:lod2MultiSurface>
    </bldg:Building>
  </core:cityObjectMember>
  <app:appearanceMember>
    <app:Appearance>
      <app:theme>rgbTexture</app:theme>
      <app:surfaceDataMember>
        <app:X3DMaterial>
          <app:diffuseColor>1 1 1</app:diffuseColor>
          <app:specularColor>0 0 0</app:specularColor>
          <app:ambientIntensity>0</app:ambientIntensity>
          <app:target>#poly_1</app:target>
        </app:X3DMaterial>
      </app:surfaceDataMember>
      <app:surfaceDataMember>
        <app:ParameterizedTexture>
          <app:imageURI>expected_output_appearance/wall.png</app:imageURI>
          <app:mimeType>image/png</app:mimeType>
          <app:target uri="#poly_1">
            <app:TexCoordList>
              <app:textureCoordinates ring="#poly_1_e">0 0 1 0 1 1 0 1 0 0</app:textureCoordinates>
            </app:TexCoordList>
          </app:target>
        </app:ParameterizedTexture>
      </app:surfaceDataMember>
    </app:Appearance>
  </app:appearanceMember>
</core:CityModel>
