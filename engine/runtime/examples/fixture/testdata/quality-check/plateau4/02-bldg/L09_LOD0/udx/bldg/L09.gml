<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/urf/3.1 ../../schemas/iur/urf/3.1/urbanFunction.xsd
https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd
http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd
http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd
http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd
http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd
http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd
http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd
http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd
http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd
http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd
">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>36.6470041354812 137.05268308385453 0</gml:lowerCorner>
			<gml:upperCorner>36.647798243275254 137.0537094956814 105.03314</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef114-02e4-11f0-a3af-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">8.6</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
                    <!-- Passes -->
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        36.64701 137.05270 0.0
                                        36.64701 137.05272 0.0
                                        36.64702 137.05272 0.0
                                        36.64702 137.05270 0.0
                                        36.64701 137.05270 0.0
                                </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                    <!-- Fails due to self-intersection -->
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        36.64701 137.05270 0.0
                                        36.64701 137.05272 0.0
                                        36.64702 137.05272 0.0
                                        36.64701 137.05271 0.0
                                        36.64703 137.05272 0.0
                                        36.64703 137.05270 0.0
                                        36.64701 137.05270 0.0
                                </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                    <!-- Fails due to self-crossing -->
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>
                                        36.64701 137.05271 0.0
                                        36.64703 137.05271 0.0
                                        36.64703 137.05272 0.0
                                        36.64702 137.05272 0.0
                                        36.64702 137.05270 0.0
                                        36.64701 137.05270 0.0
                                        36.64701 137.05271 0.0
 	                               </gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
			</bldg:lod0RoofEdge>
<!--
            <bldg:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64773967207627 137.0537094956814 101.19183 36.64772975430324 137.053609970643 101.19183 36.64778832538864 137.053600937653 101.19183 36.647798243275254 137.05370057460766 101.19183 36.64773967207627 137.0537094956814 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64773967207627 137.0537094956814 101.19183 36.647798243275254 137.05370057460766 101.19183 36.647798243275254 137.05370057460766 105.03314 36.64773967207627 137.0537094956814 105.03314 36.64773967207627 137.0537094956814 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.647798243275254 137.05370057460766 101.19183 36.64778832538864 137.053600937653 101.19183 36.64778832538864 137.053600937653 105.03314 36.647798243275254 137.05370057460766 105.03314 36.647798243275254 137.05370057460766 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64778832538864 137.053600937653 101.19183 36.64772975430324 137.053609970643 101.19183 36.64772975430324 137.053609970643 105.03314 36.64778832538864 137.053600937653 105.03314 36.64778832538864 137.053600937653 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64772975430324 137.053609970643 101.19183 36.64773967207627 137.0537094956814 101.19183 36.64773967207627 137.0537094956814 105.03314 36.64772975430324 137.053609970643 105.03314 36.64772975430324 137.053609970643 101.19183</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64773967207627 137.0537094956814 105.03314 36.647798243275254 137.05370057460766 105.03314 36.64778832538864 137.053600937653 105.03314 36.64772975430324 137.053609970643 105.03314 36.64773967207627 137.0537094956814 105.03314</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
-->
		</bldg:Building>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<bldg:Building gml:id="bldg_b3eef115-02e4-11f0-810e-18ece7a5508c">
			<core:creationDate>2025-03-21</core:creationDate>
			<bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
			<bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
			<bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
			<bldg:measuredHeight uom="m">8</bldg:measuredHeight>
			<bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
			<bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
			<bldg:lod0RoofEdge>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>36.64705388584696 137.0527173474674 0 36.64705502737237 137.05268591841337 0 36.64700527700596 137.05268308385453 0 36.6470041354812 137.05271451288834 0 36.64705388584696 137.0527173474674 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</bldg:lod0RoofEdge>
			<bldg:lod1Solid>
				<gml:Solid>
					<gml:exterior>
						<gml:CompositeSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64705388584696 137.0527173474674 100.85896 36.6470041354812 137.05271451288834 100.85896 36.64700527700596 137.05268308385453 100.85896 36.64705502737237 137.05268591841337 100.85896 36.64705388584696 137.0527173474674 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64705388584696 137.0527173474674 100.85896 36.64705502737237 137.05268591841337 100.85896 36.64705502737237 137.05268591841337 104.29314 36.64705388584696 137.0527173474674 104.29314 36.64705388584696 137.0527173474674 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64705502737237 137.05268591841337 100.85896 36.64700527700596 137.05268308385453 100.85896 36.64700527700596 137.05268308385453 104.29314 36.64705502737237 137.05268591841337 104.29314 36.64705502737237 137.05268591841337 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64700527700596 137.05268308385453 100.85896 36.6470041354812 137.05271451288834 100.85896 36.6470041354812 137.05271451288834 104.29314 36.64700527700596 137.05268308385453 104.29314 36.64700527700596 137.05268308385453 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.6470041354812 137.05271451288834 100.85896 36.64705388584696 137.0527173474674 100.85896 36.64705388584696 137.0527173474674 104.29314 36.6470041354812 137.05271451288834 104.29314 36.6470041354812 137.05271451288834 100.85896</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>36.64705388584696 137.0527173474674 104.29314 36.64705502737237 137.05268591841337 104.29314 36.64700527700596 137.05268308385453 104.29314 36.6470041354812 137.05271451288834 104.29314 36.64705388584696 137.0527173474674 104.29314</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:CompositeSurface>
					</gml:exterior>
				</gml:Solid>
			</bldg:lod1Solid>
		</bldg:Building>
	</core:cityObjectMember>
</core:CityModel>
