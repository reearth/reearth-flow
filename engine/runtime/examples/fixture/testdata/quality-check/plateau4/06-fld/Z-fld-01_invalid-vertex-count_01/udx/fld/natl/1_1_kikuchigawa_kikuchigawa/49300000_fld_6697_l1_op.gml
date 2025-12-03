<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>35.0 135.0 0.0</gml:lowerCorner>
            <gml:upperCorner>35.1 135.1 10.0</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>

    <!-- 正常な三角形 -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_valid">
            <gml:name>正常な三角形</gml:name>
            <core:creationDate>2025-01-22</core:creationDate>
            <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1140</wtr:class>
            <wtr:function codeSpace="../../codelists/WaterBody_function.xml">1</wtr:function>
            <wtr:lod1MultiSurface>
                <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        35.00 135.00 0.0
                                        35.00 135.01 0.0
                                        35.02 135.01 0.0
                                        35.00 135.00 0.0
                                    </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </wtr:lod1MultiSurface>
        </wtr:WaterBody>
    </core:cityObjectMember>

    <!-- 四角形（頂点数不正） - 正常な三角形と1辺を共有 -->
    <core:cityObjectMember>
        <wtr:WaterBody gml:id="fld_invalid_vertex_count">
            <gml:name>四角形</gml:name>
            <core:creationDate>2025-01-22</core:creationDate>
            <wtr:class codeSpace="../../codelists/WaterBody_class.xml">1140</wtr:class>
            <wtr:function codeSpace="../../codelists/WaterBody_function.xml">1</wtr:function>
            <wtr:lod1MultiSurface>
                <gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        35.00 135.01 0.0
                                        35.00 135.02 0.0
                                        35.02 135.02 0.0
                                        35.02 135.01 0.0
                                        35.00 135.01 0.0
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
