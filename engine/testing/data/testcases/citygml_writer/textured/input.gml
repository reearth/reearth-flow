<?xml version="1.0" encoding="UTF-8"?>
<!--
  A textured CityGML 2.0 source: one square wall painted by one
  app:ParameterizedTexture under the `rgbTexture` theme, with per-ring texture
  coordinates and an image that sits beside this file.

  The texture coordinates are the point of the case. CityGML's texture origin is
  bottom-left, Flow's canonical one is top-left, so the reader flips `v` on
  ingest and the writer flips it back; the round trip is only a round trip if
  both happen. `v` runs 0, 0, 1, 1, 0 here so a missing flip is visible in the
  expected document rather than plausible.
-->
<core:CityModel xmlns:gml="http://www.opengis.net/gml" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.0 135.0 0</gml:lowerCorner>
			<gml:upperCorner>35.001 135.001 10</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="textured-building-001">
			<bldg:lod2MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon gml:id="wall-poly-1">
							<gml:exterior>
								<gml:LinearRing gml:id="wall-ring-1">
									<gml:posList>35.0 135.0 0 35.0 135.001 0 35.001 135.001 0 35.001 135.0 0 35.0 135.0 0</gml:posList>
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
				<app:ParameterizedTexture>
					<app:imageURI>wall.png</app:imageURI>
					<app:target uri="#wall-poly-1">
						<app:TexCoordList>
							<app:textureCoordinates ring="#wall-ring-1">0 0 1 0 1 1 0 1 0 0</app:textureCoordinates>
						</app:TexCoordList>
					</app:target>
				</app:ParameterizedTexture>
			</app:surfaceDataMember>
		</app:Appearance>
	</app:appearanceMember>
</core:CityModel>
