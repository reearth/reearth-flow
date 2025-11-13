<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>32.700 130.700 0.0</gml:lowerCorner>
            <gml:upperCorner>32.710 130.710 1.0</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>

    <!-- Test case: Two adjacent triangles with slightly misaligned shared edge -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_misaligned_edge">
            <gml:name>共有辺の座標がずれている2つの三角形</gml:name>
            <core:creationDate>2025-01-06</core:creationDate>
            <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1140</wtr:class>
            <wtr:function codeSpace="../../codelists/WaterBody_function.xml">1</wtr:function>
            <wtr:lod1MultiSurface>
                <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">

                    <!-- Triangle 1: Left triangle -->
                    <!-- Vertices: A(700,700), B(705,700), C(702.5,705) -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="triangle_left">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.700 130.700 0.0
                                        32.700 130.705 0.0
                                        32.705 130.7025 0.0
                                        32.700 130.700 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <!-- Triangle 2: Right triangle with MISALIGNED shared edge -->
                    <!-- Should share edge B-C with Triangle 1, but coordinates are slightly off -->
                    <!-- Vertices: B'(705,700.0001), C'(702.5,705.0001), D(707.5,705) -->
                    <!-- This creates 4 unshared edges: B-C (from tri1), B'-C' (from tri2) -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="triangle_right">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.700 130.705001 0.0
                                        32.705 130.702501 0.0
                                        32.705 130.7075 0.0
                                        32.700 130.705001 0.0
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
