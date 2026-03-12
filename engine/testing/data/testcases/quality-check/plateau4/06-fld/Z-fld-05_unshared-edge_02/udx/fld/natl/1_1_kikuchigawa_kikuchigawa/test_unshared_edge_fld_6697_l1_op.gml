<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0"
    xmlns:uro="https://www.geospatial.jp/iur/uro/3.1"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd https://www.geospatial.jp/iur/uro/3.1 ../../../../schemas/iur/uro/3.1/urbanObject.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>32.700 130.700 0.0</gml:lowerCorner>
            <gml:upperCorner>32.720 130.720 1.0</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>

    <!-- Test case: Large rectangular hole surrounded by triangles -->
    <!-- The hole is intentionally large (above threshold) and should NOT be detected as an error -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_a1b2c3d4-e5f6-7890-abcd-ef1234567890">
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
                        <gml:Polygon gml:id="fld_a1b2c3d4-e5f6-7890-abcd-ef1234567891">
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
                        <gml:Polygon gml:id="fld_a1b2c3d4-e5f6-7890-abcd-ef1234567892">
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
                        <gml:Polygon gml:id="fld_a1b2c3d4-e5f6-7890-abcd-ef1234567893">
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
                        <gml:Polygon gml:id="fld_a1b2c3d4-e5f6-7890-abcd-ef1234567894">
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
            <uro:floodingRiskAttribute>
                <uro:RiverFloodingRiskAttribute>
                    <uro:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">1</uro:description>
                    <uro:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">1</uro:rank>
                    <uro:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">1</uro:adminType>
                    <uro:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</uro:scale>
                </uro:RiverFloodingRiskAttribute>
            </uro:floodingRiskAttribute>
            <uro:wtrDataQualityAttribute>
                <uro:DataQualityAttribute>
                    <uro:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</uro:geometrySrcDescLod1>
                    <uro:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
                </uro:DataQualityAttribute>
            </uro:wtrDataQualityAttribute>
        </wtr:WaterBody>
    </core:cityObjectMember>

</core:CityModel>
