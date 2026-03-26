<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>32.730 130.730 0.0</gml:lowerCorner>
            <gml:upperCorner>32.746 130.738 1.0</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>

    <!-- Test case: 4 triangles with 2 misaligned junctions → 4 unshared edges, 0 orientation errors -->
    <!-- All triangles are CCW (correct orientation) -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_misaligned_edge_02">
            <gml:name>2箇所のずれを持つ4つの三角形（ファイル02）</gml:name>
            <core:creationDate>2025-01-06</core:creationDate>
            <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1140</wtr:class>
            <wtr:function codeSpace="../../codelists/WaterBody_function.xml">1</wtr:function>
            <wtr:lod1MultiSurface>
                <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">

                    <!-- === Misaligned pair 1 (lower area) === -->

                    <!-- Triangle 1: CCW orientation -->
                    <!-- Vertices: A(32.730,130.730), B(32.735,130.7325), C(32.730,130.735) -->
                    <!-- Winding: A→B→C is CCW (cross product > 0) -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="tri_02_1">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.730 130.730 0.0
                                        32.735 130.7325 0.0
                                        32.730 130.735 0.0
                                        32.730 130.730 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <!-- Triangle 2: CCW orientation, MISALIGNED shared edge with T1 -->
                    <!-- Shares edge B-C with T1, but slightly offset as B'-C' -->
                    <!-- B'=(32.735,130.732501), C'=(32.730,130.735001) -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="tri_02_2">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.730 130.735001 0.0
                                        32.735 130.732501 0.0
                                        32.735 130.7375 0.0
                                        32.730 130.735001 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <!-- === Misaligned pair 2 (upper area) === -->

                    <!-- Triangle 3: CCW orientation -->
                    <!-- Vertices: E(32.740,130.730), F(32.745,130.7325), G(32.740,130.735) -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="tri_02_3">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.740 130.730 0.0
                                        32.745 130.7325 0.0
                                        32.740 130.735 0.0
                                        32.740 130.730 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <!-- Triangle 4: CCW orientation, MISALIGNED shared edge with T3 -->
                    <!-- Shares edge F-G with T3, but slightly offset as F'-G' -->
                    <!-- F'=(32.745,130.732501), G'=(32.740,130.735001) -->
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="tri_02_4">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.740 130.735001 0.0
                                        32.745 130.732501 0.0
                                        32.745 130.7375 0.0
                                        32.740 130.735001 0.0
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
