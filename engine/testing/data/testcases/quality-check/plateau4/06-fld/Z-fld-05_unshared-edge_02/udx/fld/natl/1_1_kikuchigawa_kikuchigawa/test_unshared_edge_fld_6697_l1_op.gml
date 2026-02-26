<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>32.700 130.700 0.0</gml:lowerCorner>
            <gml:upperCorner>32.720 130.720 1.0</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>

    <!-- Test case: Large rectangular hole surrounded by triangles -->
    <!-- The hole is intentionally large (above threshold) and should NOT be detected as an error -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_large_hole">
            <gml:name>中央に大きな四角形の穴がある三角形メッシュ</gml:name>
            <core:creationDate>2025-01-06</core:creationDate>
            <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1140</wtr:class>
            <wtr:function codeSpace="../../codelists/WaterBody_function.xml">1</wtr:function>
            <wtr:lod1MultiSurface>
                <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">

                    <!-- Central rectangular hole corners:
                         A: (32.708, 130.708)
                         B: (32.708, 130.712)
                         C: (32.712, 130.712)
                         D: (32.712, 130.708)

                         This creates a 400m x 400m hole (approximately) which should be above the threshold
                    -->

                    <!-- Top triangle: uses top edge of rectangle (B-C) as base -->
                    <!-- Vertices: B(708,712), E(710,716), C(712,712) - CCW -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="triangle_top">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.708 130.712 0.0
                                        32.710 130.716 0.0
                                        32.712 130.712 0.0
                                        32.708 130.712 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <!-- Right triangle: uses right edge of rectangle (C-D) as base -->
                    <!-- Vertices: C(712,712), F(716,710), D(712,708) - CCW -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="triangle_right">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.712 130.712 0.0
                                        32.716 130.710 0.0
                                        32.712 130.708 0.0
                                        32.712 130.712 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <!-- Bottom triangle: uses bottom edge of rectangle (D-A) as base -->
                    <!-- Vertices: D(712,708), G(710,704), A(708,708) - CCW -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="triangle_bottom">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.712 130.708 0.0
                                        32.710 130.704 0.0
                                        32.708 130.708 0.0
                                        32.712 130.708 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <!-- Left triangle: uses left edge of rectangle (A-B) as base -->
                    <!-- Vertices: A(708,708), H(704,710), B(708,712) - CCW -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="triangle_left">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.708 130.708 0.0
                                        32.704 130.710 0.0
                                        32.708 130.712 0.0
                                        32.708 130.708 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                </gml:MultiSurface>
            </wtr:lod1MultiSurface>
        </wtr:WaterBody>
    </core:cityObjectMember>

</core:CityModel>
