<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:uro="https://www.geospatial.jp/iur/uro/3.1"
    xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/profiles/base/2.0 http://schemas.opengis.net/citygml/profiles/base/2.0/CityGML.xsd https://www.geospatial.jp/iur/uro/3.1 ../../../../schemas/iur/uro/3.1/urbanObject.xsd https://www.geospatial.jp/iur/urf/3.1 ../../../../schemas/iur/urf/3.1/urbanFunction.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>32.700 130.700 0.0</gml:lowerCorner>
            <gml:upperCorner>32.720 130.720 1.0</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>

    <!-- FMEで隙間と判定されない -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_a0000000-0000-0000-0000-000000000001">
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
            <uro:floodingRiskAttribute>
                <uro:RiverFloodingRiskAttribute>
                    <uro:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">1</uro:description>
                    <uro:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">5</uro:rank>
                    <uro:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</uro:adminType>
                    <uro:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">1</uro:scale>
                </uro:RiverFloodingRiskAttribute>
            </uro:floodingRiskAttribute>
        </wtr:WaterBody>
    </core:cityObjectMember>

    <!-- FMEで隙間と判定される (共有先のない辺=2) -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_b0000000-0000-0000-0000-000000000002">
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
            <uro:floodingRiskAttribute>
                <uro:RiverFloodingRiskAttribute>
                    <uro:description codeSpace="../../codelists/RiverFloodingRiskAttribute_description.xml">1</uro:description>
                    <uro:rank codeSpace="../../codelists/RiverFloodingRiskAttribute_rank.xml">5</uro:rank>
                    <uro:adminType codeSpace="../../codelists/RiverFloodingRiskAttribute_adminType.xml">2</uro:adminType>
                    <uro:scale codeSpace="../../codelists/RiverFloodingRiskAttribute_scale.xml">1</uro:scale>
                </uro:RiverFloodingRiskAttribute>
            </uro:floodingRiskAttribute>
        </wtr:WaterBody>
    </core:cityObjectMember>

</core:CityModel>
