<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.0" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.0 ../../schemas/iur/uro/3.0/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>34.6833952578084 137.31255089625097 -0.1</gml:lowerCorner>
            <gml:upperCorner>34.69172867904967 137.32508280776253 31.708999999999996</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>
    <!-- TEST: 自己交差 -->
    <core:cityObjectMember>
        <bldg:Building gml:id="bldg_b3eef116-02e4-11f0-a3af-18ece7a5508c">
            <core:creationDate>2025-03-21</core:creationDate>
            <bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
            <bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
            <bldg:measuredHeight uom="m">8.0</bldg:measuredHeight>
            <bldg:lod1Solid>
                <gml:Solid>
                    <gml:exterior>
                        <gml:CompositeSurface>
                            <!-- Ground surface - self-intersecting -->
                            <gml:surfaceMember>
                                <gml:Polygon>
                                    <gml:exterior>
                                        <gml:LinearRing>
                                            <gml:posList>
                                                0.0 0.0 0.0
                                                1.0 0.0 0.0
                                                0.0 1.0 0.0
                                                1.0 1.0 0.0
                                                0.0 0.0 0.0
                                            </gml:posList>
                                        </gml:LinearRing>
                                    </gml:exterior>
                                </gml:Polygon>
                            </gml:surfaceMember>
                            <!-- Roof surface - self-intersecting -->
                            <gml:surfaceMember>
                                <gml:Polygon>
                                    <gml:exterior>
                                        <gml:LinearRing>
                                            <gml:posList>
                                                0.0 0.0 8.0
                                                1.0 0.0 8.0
                                                0.0 1.0 8.0
                                                1.0 1.0 8.0
                                                0.0 0.0 8.0
                                            </gml:posList>
                                        </gml:LinearRing>
                                    </gml:exterior>
                                </gml:Polygon>
                            </gml:surfaceMember>
                            <!-- Wall surface 1 -->
                            <gml:surfaceMember>
                                <gml:Polygon>
                                    <gml:exterior>
                                        <gml:LinearRing>
                                            <gml:posList>
                                                0.0 0.0 0.0
                                                1.0 0.0 0.0
                                                1.0 0.0 8.0
                                                0.0 0.0 8.0
                                                0.0 0.0 0.0
                                            </gml:posList>
                                        </gml:LinearRing>
                                    </gml:exterior>
                                </gml:Polygon>
                            </gml:surfaceMember>
                            <!-- Wall surface 2
                            <gml:surfaceMember>
                                <gml:Polygon>
                                    <gml:exterior>
                                        <gml:LinearRing>
                                            <gml:posList>
                                                1.0 0.0 0.0
                                                1.0 1.0 0.0
                                                1.0 1.0 8.0
                                                1.0 0.0 8.0
                                                1.0 0.0 0.0
                                            </gml:posList>
                                        </gml:LinearRing>
                                    </gml:exterior>
                                </gml:Polygon>
                            </gml:surfaceMember>
                            -->
                            <!-- Wall surface 3
                            <gml:surfaceMember>
                                <gml:Polygon>
                                    <gml:exterior>
                                        <gml:LinearRing>
                                            <gml:posList>
                                                1.0 1.0 0.0
                                                0.0 1.0 0.0
                                                0.0 1.0 8.0
                                                1.0 1.0 8.0
                                                1.0 1.0 0.0
                                            </gml:posList>
                                        </gml:LinearRing>
                                    </gml:exterior>
                                </gml:Polygon>
                            </gml:surfaceMember>
                            -->
                            <!-- Wall surface 4
                            <gml:surfaceMember>
                                <gml:Polygon>
                                    <gml:exterior>
                                        <gml:LinearRing>
                                            <gml:posList>
                                                0.0 1.0 0.0
                                                0.0 0.0 0.0
                                                0.0 0.0 8.0
                                                0.0 1.0 8.0
                                                0.0 1.0 0.0
                                            </gml:posList>
                                        </gml:LinearRing>
                                    </gml:exterior>
                                </gml:Polygon>
                            </gml:surfaceMember>
                            -->
                        </gml:CompositeSurface>
                    </gml:exterior>
                </gml:Solid>
            </bldg:lod1Solid>
        </bldg:Building>
    </core:cityObjectMember>
</core:CityModel>
