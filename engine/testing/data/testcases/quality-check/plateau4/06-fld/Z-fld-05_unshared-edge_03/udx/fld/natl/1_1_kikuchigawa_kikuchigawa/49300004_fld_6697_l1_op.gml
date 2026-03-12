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

    <!-- FMEで隙間と判定されない -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_49300004_inside">
            <gml:name>国管理河川_菊池川_菊池川_浸水想定区域（計画規模）_閾値内側</gml:name>
            <core:creationDate>2025-01-12</core:creationDate>
            <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1140</wtr:class>
            <wtr:function codeSpace="../../codelists/WaterBody_function.xml">1</wtr:function>
            <wtr:lod1MultiSurface>
                <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="inside_tri1">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.710 130.710 0.0
                                        32.710 130.715 0.0
                                        32.715 130.7125 0.0
                                        32.710 130.710 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <gml:surfaceMember>
                        <gml:Polygon gml:id="inside_tri2">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.715 130.71250009 0.0
                                        32.710 130.71500009 0.0
                                        32.715 130.7175 0.0
                                        32.715 130.71250009 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                </gml:MultiSurface>
            </wtr:lod1MultiSurface>
        </wtr:WaterBody>
    </core:cityObjectMember>

    <!-- FMEで隙間と判定される (共有先のない辺=2) -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_49300004_outside">
            <gml:name>国管理河川_菊池川_菊池川_浸水想定区域（計画規模）_閾値外側</gml:name>
            <core:creationDate>2025-01-12</core:creationDate>
            <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1140</wtr:class>
            <wtr:function codeSpace="../../codelists/WaterBody_function.xml">1</wtr:function>
            <wtr:lod1MultiSurface>
                <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">

                    <gml:surfaceMember>
                        <gml:Polygon gml:id="outside_tri1">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.700 130.710 0.0
                                        32.700 130.715 0.0
                                        32.705 130.7125 0.0
                                        32.700 130.710 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>

                    <gml:surfaceMember>
                        <gml:Polygon gml:id="outside_tri2">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        32.705 130.712512 0.0
                                        32.700 130.715012 0.0
                                        32.705 130.7175 0.0
                                        32.705 130.712512 0.0
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
