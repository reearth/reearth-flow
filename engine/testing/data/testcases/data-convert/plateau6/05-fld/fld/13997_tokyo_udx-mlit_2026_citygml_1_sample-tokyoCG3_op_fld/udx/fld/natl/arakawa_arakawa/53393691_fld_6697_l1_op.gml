<?xml version="1.0" encoding="UTF-8"?>
<!-- Cut from the sample: ranks 1 to 4, two bodies each of ranks 1 and 2; MultiSurface has no srsName. -->
<core:CityModel xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:tran="http://www.opengis.net/citygml/transportation/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:app="http://www.opengis.net/citygml/appearance/3.0" xmlns:dem="http://www.opengis.net/citygml/relief/3.0" xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:brid="http://www.opengis.net/citygml/bridge/3.0" xmlns:dyn="http://www.opengis.net/citygml/dynamizer/3.0" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:vers="http://www.opengis.net/citygml/versioning/3.0" xmlns:pcl="http://www.opengis.net/citygml/pointcloud/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/3.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/3.0" xmlns:ct="urn:oasis:names:tc:ciq:ct:3" xmlns:luse="http://www.opengis.net/citygml/landuse/3.0" xmlns:uro="https://www.geospatial.jp/iur/uro/4.0" xmlns:xAL="urn:oasis:names:tc:ciq:xal:3" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:urf="https://www.geospatial.jp/iur/urf/4.0" xsi:schemaLocation="http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd http://www.opengis.net/citygml/appearance/3.0 http://schemas.opengis.net/citygml/appearance/3.0/appearance.xsd http://www.opengis.net/citygml/bridge/3.0 http://schemas.opengis.net/citygml/bridge/3.0/bridge.xsd http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd http://www.opengis.net/citygml/cityfurniture/3.0 http://schemas.opengis.net/citygml/cityfurniture/3.0/cityFurniture.xsd http://www.opengis.net/citygml/cityobjectgroup/3.0 http://schemas.opengis.net/citygml/cityobjectgroup/3.0/cityObjectGroup.xsd http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd http://www.opengis.net/citygml/dynamizer/3.0 http://schemas.opengis.net/citygml/dynamizer/3.0/dynamizer.xsd http://www.opengis.net/citygml/generics/3.0 http://schemas.opengis.net/citygml/generics/3.0/generics.xsd http://www.opengis.net/citygml/landuse/3.0 http://schemas.opengis.net/citygml/landuse/3.0/landUse.xsd http://www.opengis.net/citygml/pointcloud/3.0 http://schemas.opengis.net/citygml/pointcloud/3.0/pointCloud.xsd http://www.opengis.net/citygml/relief/3.0 http://schemas.opengis.net/citygml/relief/3.0/relief.xsd http://www.opengis.net/citygml/transportation/3.0 http://schemas.opengis.net/citygml/transportation/3.0/transportation.xsd http://www.opengis.net/citygml/tunnel/3.0 http://schemas.opengis.net/citygml/tunnel/3.0/tunnel.xsd http://www.opengis.net/citygml/vegetation/3.0 http://schemas.opengis.net/citygml/vegetation/3.0/vegetation.xsd http://www.opengis.net/citygml/versioning/3.0 http://schemas.opengis.net/citygml/versioning/3.0/versioning.xsd http://www.opengis.net/citygml/waterbody/3.0 http://schemas.opengis.net/citygml/waterbody/3.0/waterBody.xsd https://www.geospatial.jp/iur/urc/4.0 ../../../../schemas/iur/urc/4.0/urbanCore.xsd https://www.geospatial.jp/iur/urf/4.0 ../../../../schemas/iur/urf/4.0/urbanFunction.xsd https://www.geospatial.jp/iur/uro/4.0 ../../../../schemas/iur/uro/4.0/urbanObject.xsd  http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd http://www.opengis.net/gml/3.2 http://schemas.opengis.net/gml/3.2.1/gml.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.66358293999841 139.76437856 1.63</gml:lowerCorner>
			<gml:upperCorner>35.66666694999842 139.77500361 2.95</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_f54abaed-d3a3-4d32-96ec-36c91329c9f5">
			<gml:name>荒川水系荒川洪水浸水想定区域図（計画規模）</gml:name>
			<core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">6</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">2</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">1</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666541949998425 139.76700295 2.32 35.666541949998425 139.76697208000002 2.32 35.666520949998414 139.76697208000002 2.32 35.666541949998425 139.76700295 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66649994999842 139.76700358 2.32 35.66652094999842 139.76703458 2.32 35.666541949998425 139.76700358 2.32 35.66649994999842 139.76700358 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66646097999841 139.76700358 2.32 35.66647894999842 139.76697208000002 2.32 35.66645794999842 139.76696228 2.32 35.66646097999841 139.76700358 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76684708 2.38 35.66649994999842 139.76684708 2.38 35.66649994999842 139.76681583000004 2.38 35.666478949998414 139.76684708 2.38</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76684708 2.38 35.66649994999842 139.76681583000004 2.38 35.66647894999842 139.76681558 2.38 35.666478949998414 139.76684708 2.38</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666520949998414 139.76697208000002 2.32 35.666541949998425 139.76694058 2.32 35.66652094999842 139.76694058 2.32 35.666520949998414 139.76697208000002 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645794999842 139.76694058 2.32 35.66645794999842 139.76696228 2.32 35.66647894999842 139.76697208000002 2.32 35.66645794999842 139.76694058 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66656244999842 139.76700358 2.32 35.66656244999842 139.76703458 2.32 35.66658294999841 139.76700358 2.32 35.66656244999842 139.76700358 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666499949998425 139.76694058 2.32 35.666520949998414 139.76697208000002 2.32 35.66652094999842 139.76694058 2.32 35.666499949998425 139.76694058 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66647894999842 139.76690958 2.32 35.666499949998425 139.76690958 2.32 35.66649994999842 139.76687858 2.35 35.66647894999842 139.76690958 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666520949998414 139.76697208000002 2.32 35.666499949998425 139.76694058 2.32 35.66647894999842 139.76697208000002 2.32 35.666520949998414 139.76697208000002 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666499949998425 139.76690958 2.32 35.66647894999842 139.76690958 2.32 35.666499949998425 139.76694058 2.32 35.666499949998425 139.76690958 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645794999842 139.76696228 2.32 35.66645794999842 139.76700358 2.32 35.66646097999841 139.76700358 2.32 35.66645794999842 139.76696228 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66656244999842 139.76703458 2.32 35.66656153999841 139.76706558 2.32 35.666582949998414 139.76706558 2.32 35.66656244999842 139.76703458 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66649994999842 139.76706558 2.32 35.66652094999842 139.76703458 2.32 35.66649994999842 139.76700358 2.32 35.66649994999842 139.76706558 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66649994999842 139.76687833 2.35 35.666478949998414 139.76684708 2.38 35.66645794999842 139.76687858 2.34 35.66649994999842 139.76687833 2.35</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66652094999842 139.76703458 2.32 35.666541949998425 139.76706558 2.32 35.66656244999842 139.76703458 2.32 35.66652094999842 139.76703458 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66656244999842 139.76703458 2.32 35.66656244999842 139.76700358 2.32 35.666541949998425 139.76700358 2.32 35.66656244999842 139.76703458 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645794999842 139.76681558 2.38 35.66645794999842 139.76685966 2.35 35.666478949998414 139.76684708 2.38 35.66645794999842 139.76681558 2.38</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645794999842 139.76685966 2.35 35.66645794999842 139.76687858 2.34 35.666478949998414 139.76684708 2.38 35.66645794999842 139.76685966 2.35</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666520949998414 139.76697208000002 2.32 35.66649994999842 139.76700358 2.32 35.666541949998425 139.76700295 2.32 35.666520949998414 139.76697208000002 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666520949998414 139.76697208000002 2.32 35.666541949998425 139.76697208000002 2.32 35.666541949998425 139.76694058 2.32 35.666520949998414 139.76697208000002 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645794999842 139.76687858 2.34 35.66649994999842 139.76687858 2.35 35.66649994999842 139.76687833 2.35 35.66645794999842 139.76687858 2.34</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66649994999842 139.76700358 2.32 35.666541949998425 139.76700358 2.32 35.666541949998425 139.76700295 2.32 35.66649994999842 139.76700358 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76684708 2.38 35.66647894999842 139.76681558 2.38 35.66645794999842 139.76681558 2.38 35.666478949998414 139.76684708 2.38</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666541949998425 139.76700358 2.32 35.66652094999842 139.76703458 2.32 35.66656244999842 139.76703458 2.32 35.666541949998425 139.76700358 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666520949998414 139.76697208000002 2.32 35.66647894999842 139.76697208000002 2.32 35.66649994999842 139.76700358 2.32 35.666520949998414 139.76697208000002 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66649994999842 139.76706558 2.32 35.666527429998425 139.76706558 2.32 35.66652094999842 139.76703458 2.32 35.66649994999842 139.76706558 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666499949998425 139.76694058 2.32 35.66645794999842 139.76694058 2.32 35.66647894999842 139.76697208000002 2.32 35.666499949998425 139.76694058 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66656244999842 139.76703458 2.32 35.666582949998414 139.76706558 2.32 35.66658294999841 139.76700358 2.32 35.66656244999842 139.76703458 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66647894999842 139.76681558 2.38 35.66649994999842 139.76681583000004 2.38 35.66649994999842 139.76681558 2.38 35.66647894999842 139.76681558 2.38</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666541949998425 139.76706558 2.32 35.66656153999841 139.76706558 2.32 35.66656244999842 139.76703458 2.32 35.666541949998425 139.76706558 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66647894999842 139.76690958 2.32 35.66645794999842 139.76687858 2.34 35.66645794999842 139.76694058 2.32 35.66647894999842 139.76690958 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645794999842 139.76694058 2.32 35.666499949998425 139.76694058 2.32 35.66647894999842 139.76690958 2.32 35.66645794999842 139.76694058 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645794999842 139.76687858 2.34 35.66647894999842 139.76690958 2.32 35.66649994999842 139.76687858 2.35 35.66645794999842 139.76687858 2.34</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66652094999842 139.76703458 2.32 35.666527429998425 139.76706558 2.32 35.666541949998425 139.76706558 2.32 35.66652094999842 139.76703458 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76684708 2.38 35.66649994999842 139.76687833 2.35 35.66649994999842 139.76684708 2.38 35.666478949998414 139.76684708 2.38</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66649994999842 139.76700358 2.32 35.66647894999842 139.76697208000002 2.32 35.66646097999841 139.76700358 2.32 35.66649994999842 139.76700358 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_1adbc7be-9b5f-41ed-a1fe-418fe506153d">
			<gml:name>荒川水系荒川洪水浸水想定区域図（計画規模）</gml:name>
			<core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">6</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">3</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">1</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76503457 2.5 35.66581244999842 139.76503457 2.5 35.665791699998415 139.76500356 2.51 35.66577094999842 139.76503457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76503457 2.5 35.66579194999842 139.76500356 2.51 35.665791699998415 139.76500356 2.51 35.66581244999842 139.76503457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665791949998415 139.76525357 2.5 35.66577094999842 139.76522207 2.5 35.66574994999842 139.76525357 2.5 35.665791949998415 139.76525357 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665853949998414 139.76515957 2.5 35.665853949998414 139.76519057 2.5 35.66587494999842 139.76519057 2.5 35.665853949998414 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76494156 2.5 35.66570836999841 139.76494156 2.5 35.66572894999842 139.76497256 2.5 35.66572894999842 139.76494156 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665791949998415 139.76525357 2.5 35.66581244999842 139.76522207 2.5 35.66577094999842 139.76522207 2.5 35.665791949998415 139.76525357 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66547894999841 139.76484706 2.51 35.66552094999842 139.76484706 2.51 35.66549994999841 139.76481556 2.51 35.66547894999841 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76528457 2.5 35.665791949998415 139.76525357 2.5 35.66574994999842 139.76525357 2.5 35.66577094999842 139.76528457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665666949998425 139.76484706 2.51 35.665645949998414 139.76484706 2.51 35.665666949998425 139.76487818 2.51 35.665666949998425 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76484706 2.51 35.66552094999842 139.76481556 2.51 35.66549994999841 139.76481556 2.51 35.66552094999842 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66585394999842 139.76506557 2.51 35.665853949998414 139.76509707 2.5 35.66587494999842 139.76506582 2.51 35.66585394999842 139.76506557 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76528457 2.5 35.66574994999842 139.76525357 2.5 35.665749949998414 139.76528457 2.5 35.66577094999842 139.76528457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66549994999841 139.76494156 2.51 35.66549994999841 139.76497256 2.5 35.66552094999842 139.76497256 2.51 35.66549994999841 139.76494156 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66566694999842 139.76487856 2.51 35.665666949998425 139.76487818 2.51 35.665645949998414 139.76484706 2.51 35.66566694999842 139.76487856 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665728949998424 139.76515957 2.5 35.665687449998416 139.76515956 2.5 35.665687449998416 139.76522207 2.5 35.665728949998424 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665687449998416 139.76522207 2.5 35.66566686999842 139.76519056 2.5 35.66566694999842 139.76522207 2.5 35.665687449998416 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76509706 2.5 35.665645949998414 139.76509706 2.5 35.665603949998406 139.76503456 2.5 35.66560394999841 139.76509706 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665853949998414 139.76509707 2.5 35.66581244999842 139.76509707 2.5 35.665853949998414 139.76515957 2.5 35.665853949998414 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665687449998416 139.76503456 2.5 35.66564594999842 139.76503456 2.5 35.66568744999841 139.76509706 2.5 35.665687449998416 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76522207 2.5 35.66583294999842 139.76519119 2.5 35.66583294999842 139.76519057 2.5 35.66581244999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76491006 2.5 35.665478949998416 139.76491006 2.51 35.66549994999841 139.76494156 2.51 35.66552094999842 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665478949998416 139.76491006 2.51 35.66545794999841 139.76491006 2.51 35.66547894999841 139.76494156 2.5 35.665478949998416 139.76491006 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76497256 2.5 35.66570836999841 139.76494156 2.5 35.665687449998416 139.76497256 2.5 35.66572894999842 139.76497256 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665687449998416 139.76497256 2.5 35.66564594999842 139.76503456 2.5 35.665687449998416 139.76503456 2.5 35.665687449998416 139.76497256 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76503456 2.5 35.66556244999843 139.76497256 2.51 35.66554194999842 139.76500356 2.51 35.665562449998426 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76478456 2.51 35.66562494999842 139.76478456 2.51 35.665624949998424 139.76475356 2.51 35.66560394999841 139.76478456 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76515957 2.5 35.66581244999842 139.76522207 2.5 35.66583294999842 139.76519057 2.5 35.66581244999842 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665832949998425 139.76506544 2.5 35.66581244999842 139.76509707 2.5 35.665853949998414 139.76509707 2.5 35.665832949998425 139.76506544 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665687449998416 139.76515956 2.5 35.66568744999841 139.76509706 2.5 35.665645949998414 139.76515956 2.5 35.665687449998416 139.76515956 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76522207 2.5 35.66572894999842 139.76525357 2.5 35.66574994999842 139.76525357 2.5 35.66572894999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76525357 2.5 35.66581244999842 139.76522207 2.5 35.665791949998415 139.76525357 2.5 35.66581244999842 139.76525357 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66547894999841 139.76484706 2.51 35.66545794999842 139.76487856 2.51 35.665478949998416 139.76491006 2.51 35.66547894999841 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66549994999841 139.76500356 2.51 35.665520949998424 139.76500356 2.51 35.66552094999842 139.76497256 2.51 35.66549994999841 139.76500356 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76497256 2.51 35.66560394999841 139.76491006 2.5 35.66560394999841 139.76497256 2.51 35.665645949998414 139.76497256 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76509706 2.5 35.66558294999842 139.76506619 2.5 35.66558294999842 139.76509706 2.5 35.66560394999841 139.76509706 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76491006 2.5 35.66552094999842 139.76497256 2.51 35.66556244999843 139.76497256 2.51 35.66552094999842 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665728949998424 139.76509707 2.5 35.66577094999842 139.76503457 2.5 35.66572894999841 139.76503456 2.5 35.665728949998424 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999841 139.76503456 2.5 35.66577094999842 139.76503457 2.5 35.665749949998414 139.76500356 2.51 35.66572894999841 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66566694999842 139.76487856 2.51 35.66568744999842 139.76491006 2.51 35.66568744999842 139.76487856 2.51 35.66566694999842 139.76487856 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76528457 2.5 35.665749949998414 139.76528457 2.5 35.665749949998414 139.76531557 2.5 35.66577094999842 139.76528457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76522207 2.5 35.665728949998424 139.76515957 2.5 35.665687449998416 139.76522207 2.5 35.66572894999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66568744999842 139.76491006 2.51 35.665687449998416 139.76497256 2.5 35.66570794999841 139.76494156 2.5 35.66568744999842 139.76491006 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76515956 2.5 35.66568744999841 139.76509706 2.5 35.665645949998414 139.76509706 2.5 35.665645949998414 139.76515956 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665687449998416 139.76522207 2.5 35.66566694999842 139.76525357 2.5 35.665687449998416 139.76525357 2.5 35.665687449998416 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76481556 2.51 35.66552094999842 139.76484706 2.51 35.66554194999841 139.76481544 2.51 35.66552094999842 139.76481556 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999841 139.76503456 2.5 35.665749949998414 139.76500356 2.51 35.66572894999842 139.76497256 2.5 35.66572894999841 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76503457 2.5 35.665832949998425 139.76503457 2.5 35.66583294999842 139.76500357 2.51 35.66581244999842 139.76503457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665603949998406 139.76503456 2.5 35.66558294999841 139.76506556 2.5 35.66560394999841 139.76509706 2.5 35.665603949998406 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76491006 2.5 35.665562449998426 139.76491006 2.51 35.66560394999841 139.76497256 2.51 35.66560394999841 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66583294999842 139.76500357 2.51 35.66581244999842 139.76500357 2.51 35.66581244999842 139.76503457 2.5 35.66583294999842 139.76500357 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665624949998424 139.76481556 2.51 35.66560394999841 139.76478456 2.51 35.66560394999841 139.76484706 2.51 35.665624949998424 139.76481556 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76491006 2.5 35.66566694999842 139.76487856 2.51 35.665645949998414 139.76484706 2.51 35.665645949998414 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76503456 2.5 35.66554194999843 139.76503456 2.5 35.66554194999842 139.76506556 2.5 35.665562449998426 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76497256 2.5 35.665687449998416 139.76497256 2.5 35.665687449998416 139.76503456 2.5 35.66572894999842 139.76497256 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66547894999841 139.76484706 2.51 35.66545794999842 139.76484706 2.51 35.66545794999842 139.76487856 2.51 35.66547894999841 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76491006 2.5 35.66560394999841 139.76491006 2.5 35.665645949998414 139.76497256 2.51 35.665645949998414 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76497256 2.51 35.665687449998416 139.76497256 2.5 35.66568744999842 139.76491006 2.51 35.665645949998414 139.76497256 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76503457 2.5 35.66577094999842 139.76500356 2.51 35.665749949998414 139.76500356 2.51 35.66577094999842 139.76503457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76478456 2.51 35.66556244999843 139.76475356 2.51 35.665541869998414 139.76475356 2.51 35.665562449998426 139.76478456 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66564594999842 139.76503456 2.5 35.665645949998414 139.76497256 2.51 35.66560394999841 139.76497256 2.51 35.66564594999842 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76522207 2.5 35.66581244999842 139.76522207 2.5 35.66581244999842 139.76515957 2.5 35.66577094999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66547894999841 139.76484706 2.51 35.665478949998416 139.76481556 2.51 35.66545794999841 139.76481544 2.5 35.66547894999841 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76491006 2.5 35.66568744999842 139.76491006 2.51 35.66566694999842 139.76487856 2.51 35.665645949998414 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76497256 2.51 35.66568744999842 139.76491006 2.51 35.665645949998414 139.76491006 2.5 35.665645949998414 139.76497256 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76484706 2.51 35.665478949998416 139.76491006 2.51 35.66552094999842 139.76491006 2.5 35.66552094999842 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76503456 2.5 35.66554194999842 139.76500356 2.51 35.66554194999843 139.76503456 2.5 35.665562449998426 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66583294999842 139.76522207 2.5 35.66581244999842 139.76525357 2.5 35.66583302999841 139.76525357 2.5 35.66583294999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665728949998424 139.76509707 2.5 35.665770949998425 139.76515956 2.5 35.665770949998425 139.76509707 2.5 35.665728949998424 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66545794999841 139.76481544 2.5 35.66545794999842 139.76484706 2.51 35.66547894999841 139.76484706 2.51 35.66545794999841 139.76481544 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76478456 2.51 35.66560394999841 139.76478456 2.51 35.66558336999841 139.76475356 2.51 35.665562449998426 139.76478456 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665853949998414 139.76509707 2.5 35.665874949998425 139.76509707 2.51 35.66587494999842 139.76506582 2.51 35.665853949998414 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76512856 2.5 35.66560394999841 139.76509706 2.5 35.66558294999842 139.76512856 2.5 35.66560394999841 139.76512856 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76503456 2.5 35.66554194999842 139.76506556 2.5 35.66556244999843 139.76506556 2.5 35.665562449998426 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665770949998425 139.76515956 2.5 35.66581244999842 139.76515957 2.5 35.665770949998425 139.76509707 2.5 35.665770949998425 139.76515956 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76500357 2.51 35.66579194999842 139.76500356 2.51 35.66581244999842 139.76503457 2.5 35.66581244999842 139.76500357 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66556244999843 139.76475356 2.51 35.665562449998426 139.76478456 2.51 35.66558294999842 139.76475356 2.51 35.66556244999843 139.76475356 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66574994999842 139.76497256 2.51 35.66572894999842 139.76497256 2.5 35.665749949998414 139.76500356 2.51 35.66574994999842 139.76497256 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76522207 2.5 35.66577094999842 139.76522207 2.5 35.665728949998424 139.76515957 2.5 35.66572894999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76522207 2.5 35.66581244999842 139.76515957 2.5 35.665770949998425 139.76515956 2.5 35.66577094999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76515957 2.5 35.66583294999842 139.76519057 2.5 35.66583336999842 139.76519057 2.5 35.66581244999842 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66545794999841 139.76494156 2.51 35.66547894999841 139.76494156 2.5 35.66545794999841 139.76491006 2.51 35.66545794999841 139.76494156 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665791949998415 139.76528457 2.5 35.66577094999842 139.76528457 2.5 35.66579194999843 139.76531557 2.5 35.665791949998415 139.76528457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665728949998424 139.76509707 2.5 35.66568744999841 139.76509706 2.5 35.665687449998416 139.76515956 2.5 35.665728949998424 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76519056 2.5 35.66566686999842 139.76519056 2.5 35.665645949998414 139.76515956 2.5 35.665645949998414 139.76519056 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76484706 2.51 35.665666949998425 139.76481556 2.51 35.665645949998414 139.76481556 2.51 35.665645949998414 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76503457 2.5 35.66581244999842 139.76509707 2.5 35.665832949998425 139.76506544 2.5 35.66581244999842 139.76503457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76497256 2.51 35.66549994999841 139.76497256 2.5 35.66549994999841 139.76500356 2.51 35.66552094999842 139.76497256 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66554194999842 139.76478456 2.51 35.665562449998426 139.76478456 2.51 35.665541869998414 139.76475356 2.51 35.66554194999842 139.76478456 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76528457 2.5 35.665791949998415 139.76528457 2.5 35.665791949998415 139.76525357 2.5 35.66577094999842 139.76528457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76503457 2.5 35.665791699998415 139.76500356 2.51 35.66577094999842 139.76500356 2.51 35.66577094999842 139.76503457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999841 139.76503456 2.5 35.665687449998416 139.76503456 2.5 35.665728949998424 139.76509707 2.5 35.66572894999841 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66587494999842 139.76506557 2.51 35.66585394999842 139.76506557 2.51 35.66587494999842 139.76506582 2.51 35.66587494999842 139.76506557 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665728949998424 139.76509707 2.5 35.665728949998424 139.76515957 2.5 35.665770949998425 139.76515956 2.5 35.665728949998424 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665603949998406 139.76503456 2.5 35.66560394999841 139.76497256 2.51 35.665562449998426 139.76503456 2.5 35.665603949998406 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76484706 2.51 35.665562449998426 139.76491006 2.51 35.66560394999841 139.76491006 2.5 35.66560394999841 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66587494999842 139.76512832 2.5 35.665874949998425 139.76509707 2.51 35.665853949998414 139.76509707 2.5 35.66587494999842 139.76512832 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999842 139.76475356 2.51 35.66560394999841 139.76478456 2.51 35.665624949998424 139.76475356 2.51 35.66560394999842 139.76475356 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665687449998416 139.76522207 2.5 35.66566694999842 139.76522207 2.5 35.66566694999842 139.76525357 2.5 35.665687449998416 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665770949998425 139.76509707 2.5 35.66581244999842 139.76503457 2.5 35.66577094999842 139.76503457 2.5 35.665770949998425 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76503456 2.5 35.66556244999843 139.76506556 2.5 35.66558294999841 139.76506556 2.5 35.665562449998426 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76484706 2.51 35.66556244999843 139.76484706 2.51 35.665562449998426 139.76491006 2.51 35.66560394999841 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76478456 2.51 35.66554194999842 139.76478456 2.51 35.66554194999841 139.76481544 2.51 35.665562449998426 139.76478456 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76484706 2.51 35.66560394999841 139.76484706 2.51 35.66560394999841 139.76491006 2.5 35.665645949998414 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76484706 2.51 35.665666949998425 139.76484706 2.51 35.665666949998425 139.76481556 2.51 35.665645949998414 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66587494999842 139.76512832 2.5 35.665853949998414 139.76509707 2.5 35.665853949998414 139.76515957 2.5 35.66587494999842 139.76512832 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76522207 2.5 35.66581244999842 139.76525357 2.5 35.66583294999842 139.76522207 2.5 35.66581244999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76503456 2.5 35.66560394999841 139.76497256 2.51 35.66556244999843 139.76497256 2.51 35.665562449998426 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76528457 2.5 35.665749949998414 139.76531557 2.5 35.66579194999843 139.76531557 2.5 35.66577094999842 139.76528457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76484706 2.51 35.66552094999842 139.76491006 2.5 35.665562449998426 139.76491006 2.51 35.66552094999842 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665853949998414 139.76515957 2.5 35.66583336999842 139.76519057 2.5 35.665853949998414 139.76519057 2.5 35.665853949998414 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665853949998414 139.76515957 2.5 35.66587494999842 139.76515957 2.5 35.66587494999842 139.76512857 2.5 35.665853949998414 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76478456 2.51 35.66560394999841 139.76484706 2.51 35.66560394999841 139.76478456 2.51 35.665562449998426 139.76478456 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76522207 2.5 35.66583294999842 139.76522207 2.5 35.66583294999842 139.76519119 2.5 35.66581244999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665853949998414 139.76509707 2.5 35.66585394999842 139.76506557 2.51 35.665832949998425 139.76506544 2.5 35.665853949998414 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665478949998416 139.76481556 2.51 35.66547894999841 139.76484706 2.51 35.66549994999841 139.76481556 2.51 35.665478949998416 139.76481556 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76522207 2.5 35.66570836999842 139.76525357 2.5 35.66572894999842 139.76525357 2.5 35.66572894999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665603949998406 139.76503456 2.5 35.665562449998426 139.76503456 2.5 35.66558294999841 139.76506556 2.5 35.665603949998406 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66587494999842 139.76515957 2.5 35.665853949998414 139.76515957 2.5 35.66587494999842 139.76519057 2.5 35.66587494999842 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76491006 2.5 35.66556244999843 139.76497256 2.51 35.665562449998426 139.76491006 2.51 35.66552094999842 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66587494999842 139.76512857 2.5 35.66587494999842 139.76512832 2.5 35.665853949998414 139.76515957 2.5 35.66587494999842 139.76512857 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66562494999842 139.76519057 2.5 35.665645949998414 139.76519056 2.5 35.665624949998424 139.76515956 2.5 35.66562494999842 139.76519057 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66570794999841 139.76494156 2.5 35.665687449998416 139.76497256 2.5 35.66570836999841 139.76494156 2.5 35.66570794999841 139.76494156 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66577094999842 139.76522207 2.5 35.665770949998425 139.76515956 2.5 35.665728949998424 139.76515957 2.5 35.66577094999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665624949998424 139.76515956 2.5 35.665645949998414 139.76515956 2.5 35.665624779998424 139.76512856 2.5 35.665624949998424 139.76515956 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76491006 2.5 35.66549994999841 139.76494156 2.51 35.66552094999842 139.76497256 2.51 35.66552094999842 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76515956 2.5 35.665624949998424 139.76515956 2.5 35.665645949998414 139.76519056 2.5 35.665645949998414 139.76515956 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76481556 2.51 35.665624949998424 139.76481556 2.51 35.665645949998414 139.76484706 2.51 35.665645949998414 139.76481556 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76509706 2.5 35.66564594999842 139.76503456 2.5 35.665603949998406 139.76503456 2.5 35.665645949998414 139.76509706 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76515957 2.5 35.66581244999842 139.76509707 2.5 35.665770949998425 139.76509707 2.5 35.66581244999842 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76509706 2.5 35.66558294999842 139.76509706 2.5 35.66558294999842 139.76512856 2.5 35.66560394999841 139.76509706 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665687449998416 139.76522207 2.5 35.665687449998416 139.76515956 2.5 35.66566686999842 139.76519056 2.5 35.665687449998416 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665728949998424 139.76509707 2.5 35.665770949998425 139.76509707 2.5 35.66577094999842 139.76503457 2.5 35.665728949998424 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76509706 2.5 35.66568744999841 139.76509706 2.5 35.66564594999842 139.76503456 2.5 35.665645949998414 139.76509706 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66570794999842 139.76491006 2.51 35.66568744999842 139.76491006 2.51 35.66570794999842 139.76494094 2.51 35.66570794999842 139.76491006 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66568744999842 139.76491006 2.51 35.66570794999842 139.76491006 2.51 35.66568744999842 139.76487856 2.51 35.66568744999842 139.76491006 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76512856 2.5 35.665624779998424 139.76512856 2.5 35.66560394999841 139.76509706 2.5 35.66560394999841 139.76512856 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66564594999842 139.76503456 2.5 35.66560394999841 139.76497256 2.51 35.665603949998406 139.76503456 2.5 35.66564594999842 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999841 139.76503456 2.5 35.66572894999842 139.76497256 2.5 35.665687449998416 139.76503456 2.5 35.66572894999841 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66568744999842 139.76487856 2.51 35.66570794999842 139.76491006 2.51 35.66570794999841 139.76487856 2.51 35.66568744999842 139.76487856 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999841 139.76484706 2.51 35.665562449998426 139.76478456 2.51 35.66556244999843 139.76484706 2.51 35.66560394999841 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665853949998414 139.76515957 2.5 35.66581244999842 139.76509707 2.5 35.66581244999842 139.76515957 2.5 35.665853949998414 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66562494999842 139.76478456 2.51 35.66560394999841 139.76478456 2.51 35.665624949998424 139.76481556 2.51 35.66562494999842 139.76478456 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66574994999842 139.76525357 2.5 35.66577094999842 139.76522207 2.5 35.66572894999842 139.76522207 2.5 35.66574994999842 139.76525357 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66552094999842 139.76484706 2.51 35.66547894999841 139.76484706 2.51 35.665478949998416 139.76491006 2.51 35.66552094999842 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665687449998416 139.76525357 2.5 35.665707949998406 139.76525357 2.5 35.665687449998416 139.76522207 2.5 35.665687449998416 139.76525357 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665562449998426 139.76491006 2.51 35.66556244999843 139.76497256 2.51 35.66560394999841 139.76497256 2.51 35.665562449998426 139.76491006 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66574994999842 139.76494156 2.51 35.66572894999842 139.76494156 2.5 35.66572894999842 139.76497256 2.5 35.66574994999842 139.76494156 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66558336999841 139.76475356 2.51 35.66558294999842 139.76475356 2.51 35.665562449998426 139.76478456 2.51 35.66558336999841 139.76475356 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66556244999843 139.76497256 2.51 35.66552094999842 139.76497256 2.51 35.66554169999842 139.76500356 2.51 35.66556244999843 139.76497256 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66556244999843 139.76484706 2.51 35.66552094999842 139.76484706 2.51 35.665562449998426 139.76491006 2.51 35.66556244999843 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76497256 2.5 35.66574994999842 139.76497256 2.51 35.66574994999842 139.76494156 2.51 35.66572894999842 139.76497256 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665520949998424 139.76500356 2.51 35.66554169999842 139.76500356 2.51 35.66552094999842 139.76497256 2.51 35.665520949998424 139.76500356 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66560394999842 139.76475356 2.51 35.66558336999841 139.76475356 2.51 35.66560394999841 139.76478456 2.51 35.66560394999842 139.76475356 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665728949998424 139.76509707 2.5 35.665687449998416 139.76515956 2.5 35.665728949998424 139.76515957 2.5 35.665728949998424 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665707949998406 139.76525357 2.5 35.66570836999842 139.76525357 2.5 35.665687449998416 139.76522207 2.5 35.665707949998406 139.76525357 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66554194999842 139.76500356 2.51 35.66556244999843 139.76497256 2.51 35.66554169999842 139.76500356 2.51 35.66554194999842 139.76500356 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665478949998416 139.76491006 2.51 35.66545794999842 139.76487894 2.51 35.66545794999841 139.76491006 2.51 35.665478949998416 139.76491006 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66564594999842 139.76503456 2.5 35.665687449998416 139.76497256 2.5 35.665645949998414 139.76497256 2.51 35.66564594999842 139.76503456 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76503457 2.5 35.665832949998425 139.76506544 2.5 35.665832949998425 139.76503457 2.5 35.66581244999842 139.76503457 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76509706 2.5 35.66560394999841 139.76509706 2.5 35.665624779998424 139.76512856 2.5 35.665645949998414 139.76509706 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66568744999842 139.76491006 2.51 35.66570794999841 139.76494156 2.5 35.66570794999842 139.76494094 2.51 35.66568744999842 139.76491006 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66558294999841 139.76506556 2.5 35.66558294999842 139.76506619 2.5 35.66560394999841 139.76509706 2.5 35.66558294999841 139.76506556 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66545794999842 139.76487856 2.51 35.66545794999842 139.76487894 2.51 35.665478949998416 139.76491006 2.51 35.66545794999842 139.76487856 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66556244999843 139.76484706 2.51 35.66554194999841 139.76481544 2.51 35.66552094999842 139.76484706 2.51 35.66556244999843 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665770949998425 139.76509707 2.5 35.66581244999842 139.76509707 2.5 35.66581244999842 139.76503457 2.5 35.665770949998425 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66549994999841 139.76494156 2.51 35.665478949998416 139.76491006 2.51 35.66547894999841 139.76494156 2.5 35.66549994999841 139.76494156 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66556244999843 139.76484706 2.51 35.665562449998426 139.76478456 2.51 35.66554194999841 139.76481544 2.51 35.66556244999843 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665728949998424 139.76509707 2.5 35.665687449998416 139.76503456 2.5 35.66568744999841 139.76509706 2.5 35.665728949998424 139.76509707 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66581244999842 139.76515957 2.5 35.66583336999842 139.76519057 2.5 35.665853949998414 139.76515957 2.5 35.66581244999842 139.76515957 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76509706 2.5 35.665624779998424 139.76512856 2.5 35.665645949998414 139.76515956 2.5 35.665645949998414 139.76509706 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66572894999842 139.76522207 2.5 35.665687449998416 139.76522207 2.5 35.66570836999842 139.76525357 2.5 35.66572894999842 139.76522207 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76484706 2.51 35.665624949998424 139.76481556 2.51 35.66560394999841 139.76484706 2.51 35.665645949998414 139.76484706 2.51</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66566686999842 139.76519056 2.5 35.665687449998416 139.76515956 2.5 35.665645949998414 139.76515956 2.5 35.66566686999842 139.76519056 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665645949998414 139.76491006 2.5 35.665645949998414 139.76484706 2.51 35.66560394999841 139.76491006 2.5 35.665645949998414 139.76491006 2.5</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_2c0bd0b3-388d-4751-ba44-93bbaddce911">
			<gml:name>荒川水系荒川洪水浸水想定区域図（計画規模）</gml:name>
			<core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">6</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">1</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77059708 2.82 35.664124939998416 139.77059708 2.82 35.66412493999841 139.77056583 2.82 35.66410393999842 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66377093999842 139.77100357 2.81 35.663791939998426 139.7710032 2.81 35.66377093999842 139.77097258 2.81 35.66377093999842 139.77100357 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77053457 2.82 35.664020939998416 139.77053457 2.82 35.66399993999841 139.77050357 2.82 35.66397893999842 139.77053457 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77059708 2.82 35.664020939998416 139.77065957 2.81 35.66404193999842 139.77062858 2.81 35.664020939998416 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664145939998406 139.77047208 2.81 35.664187439998415 139.77053458 2.81 35.66418743999842 139.77047208 2.82 35.664145939998406 139.77047208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66416693999843 139.77031557 2.82 35.664187439998415 139.77034707 2.82 35.66420793999841 139.77032524 2.82 35.66416693999843 139.77031557 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664187439998415 139.77053458 2.81 35.66414593999842 139.77053458 2.82 35.66416693999842 139.77056558 2.81 35.664187439998415 139.77053458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77059707 2.82 35.66397893999842 139.77065958 2.82 35.664020939998416 139.77065957 2.81 35.66397893999842 139.77059707 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663749939998425 139.77094157 2.81 35.66377093999842 139.77091007 2.81 35.663749939998425 139.77090843 2.81 35.663749939998425 139.77094157 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66406243999842 139.77047208 2.82 35.66406243999842 139.77053457 2.81 35.66410393999842 139.77047207 2.82 35.66406243999842 139.77047208 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663874939998415 139.77062857 2.81 35.663895939998426 139.77065957 2.81 35.66391693999842 139.77062857 2.82 35.663874939998415 139.77062857 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66416693999842 139.77044058 2.82 35.664124939998416 139.77044058 2.81 35.664145939998406 139.77047208 2.81 35.66416693999842 139.77044058 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77078457 2.81 35.663895939998426 139.77072207 2.81 35.663853939998425 139.77072207 2.81 35.663853939998425 139.77078457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66399993999841 139.77046265 2.81 35.66399993999841 139.77044057 2.81 35.66397893999842 139.77047207 2.81 35.66399993999841 139.77046265 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66406243999842 139.77062858 2.81 35.66406243999841 139.77059708 2.81 35.66404193999842 139.77062858 2.81 35.66406243999842 139.77062858 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66406243999842 139.77053457 2.81 35.66410393999842 139.77053458 2.81 35.66410393999842 139.77047207 2.82 35.66406243999842 139.77053457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77065957 2.81 35.664020939998416 139.77059708 2.82 35.66397893999842 139.77059707 2.82 35.664020939998416 139.77065957 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66383293999842 139.77069057 2.81 35.663832939998414 139.77072955 2.81 35.663853939998425 139.77072207 2.81 35.66383293999842 139.77069057 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664082939998416 139.77044058 2.82 35.66410393999842 139.77047207 2.82 35.664104829998415 139.77044057 2.82 35.664082939998416 139.77044058 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77072208 2.81 35.66397893999842 139.77072207 2.81 35.66399993999841 139.77075358 2.81 35.664020939998416 139.77072208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663895939998426 139.77084707 2.82 35.663895939998426 139.77087858 2.82 35.66391693999842 139.7708782 2.82 35.663895939998426 139.77084707 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663895939998426 139.77065957 2.81 35.663937439998406 139.77065957 2.82 35.66391693999842 139.77062857 2.82 35.663895939998426 139.77065957 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77072207 2.81 35.663937439998406 139.77072207 2.81 35.66395835999842 139.77075358 2.81 35.66397893999842 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66391693999842 139.77084708 2.82 35.663895939998426 139.77084707 2.82 35.66391693999842 139.7708782 2.82 35.66391693999842 139.77084708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663895939998426 139.77065957 2.81 35.663937439998406 139.77072207 2.81 35.663937439998406 139.77065957 2.82 35.663895939998426 139.77065957 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66420793999841 139.77050358 2.82 35.664187439998415 139.77053458 2.81 35.66420793999841 139.77056558 2.82 35.66420793999841 139.77050358 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77078457 2.81 35.663812439998416 139.77078457 2.81 35.663812439998416 139.77084707 2.81 35.663853939998425 139.77078457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663957939998426 139.77050357 2.82 35.66395793999841 139.77056557 2.82 35.66397893999842 139.77053457 2.82 35.663957939998426 139.77050357 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77078457 2.81 35.663895939998426 139.77078458 2.81 35.663895939998426 139.77072207 2.81 35.663853939998425 139.77078457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66404193999841 139.77050357 2.82 35.66406243999842 139.77047208 2.82 35.66404193999841 139.77048445 2.82 35.66404193999841 139.77050357 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66420793999841 139.77044058 2.82 35.664187439998415 139.77040958 2.82 35.66416693999842 139.77044058 2.82 35.66420793999841 139.77044058 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663937439998406 139.77065957 2.82 35.663957939998426 139.77062857 2.82 35.66391693999842 139.77062857 2.82 35.663937439998406 139.77065957 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66414593999842 139.77053458 2.82 35.66412510999842 139.77056558 2.82 35.66414593999841 139.77056558 2.82 35.66414593999842 139.77053458 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664187439998415 139.77040958 2.82 35.664166939998424 139.77037858 2.82 35.66416693999842 139.77044058 2.82 35.664187439998415 139.77040958 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77078457 2.81 35.663832939998414 139.77075357 2.81 35.663812439998416 139.77078457 2.81 35.663853939998425 139.77078457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66414593999842 139.77053458 2.82 35.66414593999841 139.77056558 2.82 35.66416693999842 139.77056558 2.81 35.66414593999842 139.77053458 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663895939998426 139.77084707 2.82 35.66391693999842 139.77084708 2.82 35.66391693999842 139.77081558 2.81 35.663895939998426 139.77084707 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66414593999841 139.77034708 2.82 35.664187439998415 139.77034707 2.82 35.66416693999843 139.77031557 2.82 35.66414593999841 139.77034708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663812439998416 139.77091007 2.82 35.66377093999842 139.77084707 2.81 35.66377093999842 139.77091007 2.81 35.663812439998416 139.77091007 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66420793999841 139.77037858 2.82 35.66420793999841 139.77032524 2.82 35.664187439998415 139.77034707 2.82 35.66420793999841 139.77037858 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77059708 2.82 35.66410393999841 139.77062858 2.82 35.664124939998416 139.77062833 2.82 35.66410393999842 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77053458 2.81 35.664145939998406 139.77047208 2.81 35.66410393999842 139.77047207 2.82 35.66410393999842 139.77053458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77059708 2.82 35.664083359998415 139.77062858 2.82 35.66410393999841 139.77062858 2.82 35.66410393999842 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66406243999841 139.77059708 2.81 35.664020939998416 139.77053457 2.82 35.664020939998416 139.77059708 2.82 35.66406243999841 139.77059708 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66404193999841 139.77050357 2.82 35.664020939998416 139.77053457 2.82 35.66406243999842 139.77053457 2.81 35.66404193999841 139.77050357 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664124939998416 139.77036775 2.82 35.66412493999842 139.77037857 2.82 35.66414593999841 139.77034708 2.82 35.664124939998416 139.77036775 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77072208 2.81 35.66399993999841 139.77075358 2.81 35.664020939998416 139.77075358 2.82 35.664020939998416 139.77072208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664165309998424 139.77031557 2.82 35.664124939998416 139.77031557 2.82 35.66414593999841 139.77034708 2.82 35.664165309998424 139.77031557 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77072208 2.81 35.66397893999842 139.77065958 2.82 35.66397893999842 139.77072207 2.81 35.664020939998416 139.77072208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663874939998415 139.77087858 2.82 35.663895939998426 139.77084707 2.82 35.663853939998425 139.77084708 2.82 35.663874939998415 139.77087858 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66377093999842 139.77084707 2.81 35.66377093999842 139.77078457 2.81 35.663749939998425 139.77081557 2.81 35.66377093999842 139.77084707 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77065957 2.81 35.66404193999841 139.77065958 2.81 35.66404193999842 139.77062858 2.81 35.664020939998416 139.77065957 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663749939998425 139.77087857000004 2.81 35.663749939998425 139.77090843 2.81 35.66377093999842 139.77091007 2.81 35.663749939998425 139.77087857000004 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66395793999842 139.77081545 2.81 35.66395793999842 139.77078458 2.81 35.66393743999841 139.77078458 2.81 35.66395793999842 139.77081545 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77059708 2.82 35.66404193999842 139.77062858 2.81 35.66406243999841 139.77059708 2.81 35.664020939998416 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66412510999842 139.77056558 2.82 35.66414593999842 139.77053458 2.82 35.66410393999842 139.77053458 2.81 35.66412510999842 139.77056558 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77053458 2.81 35.66406243999842 139.77053457 2.81 35.66406243999841 139.77059708 2.81 35.66410393999842 139.77053458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77091007 2.81 35.663874939998415 139.77087858 2.82 35.663853939998425 139.77084708 2.82 35.663853939998425 139.77091007 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66377093999842 139.77097258 2.81 35.66379193999843 139.77097258 2.81 35.663791939998426 139.77094157 2.81 35.66377093999842 139.77097258 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663749939998425 139.77094157 2.81 35.66377093999842 139.77097258 2.81 35.663791939998426 139.77094157 2.81 35.663749939998425 139.77094157 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663895939998426 139.77087858 2.82 35.663895939998426 139.77084707 2.82 35.663874939998415 139.77087858 2.82 35.663895939998426 139.77087858 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66412493999841 139.77056583 2.82 35.66410393999842 139.77053458 2.81 35.66410393999842 139.77059708 2.82 35.66412493999841 139.77056583 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663874939998415 139.77091008 2.82 35.663853939998425 139.77091007 2.81 35.66387493999842 139.77094133 2.82 35.663874939998415 139.77091008 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66406243999841 139.77059708 2.81 35.66406243999842 139.77062858 2.81 35.66408293999842 139.77062858 2.81 35.66406243999841 139.77059708 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663749939998425 139.77081557 2.81 35.66377093999842 139.77078457 2.81 35.66374993999843 139.77075357 2.81 35.663749939998425 139.77081557 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66420793999841 139.77050358 2.82 35.66420793999842 139.77049184 2.82 35.66418743999842 139.77047208 2.82 35.66420793999841 139.77050358 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663812439998416 139.77091007 2.82 35.663812439998416 139.77094157 2.81 35.663832939998414 139.77094158 2.81 35.663812439998416 139.77091007 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663832939998414 139.77065904 2.81 35.663853939998425 139.77065957 2.81 35.66383293999842 139.77062857 2.81 35.663832939998414 139.77065904 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664187439998415 139.77040958 2.82 35.66420793999841 139.77037858 2.82 35.664166939998424 139.77037858 2.82 35.664187439998415 139.77040958 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66393743999841 139.77078458 2.81 35.663895939998426 139.77078458 2.81 35.66391693999842 139.77081558 2.81 35.66393743999841 139.77078458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66391693999842 139.77081558 2.81 35.663895939998426 139.77078458 2.81 35.663895939998426 139.77084707 2.82 35.66391693999842 139.77081558 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66391693999842 139.77087858 2.82 35.66391693999842 139.7708782 2.82 35.663916689998416 139.77087858 2.82 35.66391693999842 139.77087858 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66406243999842 139.77047208 2.82 35.66404193999842 139.77044057 2.82 35.66404193999841 139.77048445 2.82 35.66406243999842 139.77047208 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77065958 2.82 35.663957939998426 139.77062857 2.82 35.663937439998406 139.77065957 2.82 35.66397893999842 139.77065958 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77053457 2.82 35.66399993999841 139.77050357 2.82 35.663957939998426 139.77050357 2.82 35.66397893999842 139.77053457 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663874769998415 139.77094158 2.82 35.66387493999842 139.77094133 2.82 35.663853939998425 139.77094158 2.81 35.663874769998415 139.77094158 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77091007 2.81 35.663833359998414 139.77094158 2.81 35.663853939998425 139.77094158 2.81 35.663853939998425 139.77091007 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664187439998415 139.77053458 2.81 35.66418743999842 139.77056558 2.81 35.66420793999841 139.77056558 2.82 35.664187439998415 139.77053458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77053458 2.81 35.66414593999842 139.77053458 2.82 35.664145939998406 139.77047208 2.81 35.66410393999842 139.77053458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66393743999841 139.77078458 2.81 35.663895939998426 139.77072207 2.81 35.663895939998426 139.77078458 2.81 35.66393743999841 139.77078458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66418743999842 139.77047208 2.82 35.66416693999842 139.77044058 2.82 35.664145939998406 139.77047208 2.81 35.66418743999842 139.77047208 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66377093999842 139.77097258 2.81 35.663791939998426 139.7710032 2.81 35.66379193999843 139.77097258 2.81 35.66377093999842 139.77097258 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66418743999842 139.77047208 2.82 35.66420793999842 139.77049184 2.82 35.66420793999841 139.77044058 2.82 35.66418743999842 139.77047208 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663895939998426 139.77072207 2.81 35.663937439998406 139.77072207 2.81 35.663895939998426 139.77065957 2.81 35.663895939998426 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66399993999841 139.77050357 2.82 35.66397893999842 139.77047207 2.81 35.663957939998426 139.77050357 2.82 35.66399993999841 139.77050357 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663957939998426 139.77050357 2.82 35.66397893999842 139.77047207 2.81 35.66395793999842 139.77047672 2.81 35.663957939998426 139.77050357 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77091007 2.81 35.663853939998425 139.77094158 2.81 35.66387493999842 139.77094133 2.82 35.663853939998425 139.77091007 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663749939998425 139.77094157 2.81 35.663749939998425 139.77098183 2.81 35.66377093999842 139.77097258 2.81 35.663749939998425 139.77094157 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66399993999841 139.77044057 2.81 35.663957939998426 139.77044057 2.81 35.66397893999842 139.77047207 2.81 35.66399993999841 139.77044057 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663957939998426 139.77062857 2.82 35.66397893999842 139.77059707 2.82 35.66395793999841 139.77056557 2.82 35.663957939998426 139.77062857 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66418743999842 139.77047208 2.82 35.66420793999841 139.77044058 2.82 35.66416693999842 139.77044058 2.82 35.66418743999842 139.77047208 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66383293999842 139.77062857 2.81 35.663853939998425 139.77065957 2.81 35.663874939998415 139.77062857 2.81 35.66383293999842 139.77062857 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66420793999841 139.77037858 2.82 35.664187439998415 139.77034707 2.82 35.664166939998424 139.77037858 2.82 35.66420793999841 139.77037858 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663791939998426 139.7710032 2.81 35.66379168999841 139.77100357 2.81 35.66379193999843 139.77100357 2.81 35.663791939998426 139.7710032 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66412493999841 139.77056583 2.82 35.66412510999842 139.77056558 2.82 35.66410393999842 139.77053458 2.81 35.66412493999841 139.77056583 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77065957 2.81 35.663832939998414 139.77065904 2.81 35.66383293999842 139.77069057 2.81 35.663853939998425 139.77065957 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66391693999842 139.77081558 2.81 35.66393743999841 139.77081558 2.81 35.66393743999841 139.77078458 2.81 35.66391693999842 139.77081558 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663812439998416 139.77084707 2.81 35.66377093999842 139.77084707 2.81 35.663812439998416 139.77091007 2.82 35.663812439998416 139.77084707 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77078457 2.81 35.663812439998416 139.77084707 2.81 35.663853939998425 139.77084708 2.82 35.663853939998425 139.77078457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66418743999842 139.77047208 2.82 35.664187439998415 139.77053458 2.81 35.66420793999841 139.77050358 2.82 35.66418743999842 139.77047208 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77084708 2.82 35.663812439998416 139.77091007 2.82 35.663853939998425 139.77091007 2.81 35.663853939998425 139.77084708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77078457 2.81 35.663895939998426 139.77084707 2.82 35.663895939998426 139.77078458 2.81 35.663853939998425 139.77078457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66406243999842 139.77047208 2.82 35.66404193999841 139.77050357 2.82 35.66406243999842 139.77053457 2.81 35.66406243999842 139.77047208 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77065957 2.81 35.663895939998426 139.77065957 2.81 35.663874939998415 139.77062857 2.81 35.663853939998425 139.77065957 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77047207 2.81 35.663957939998426 139.77044057 2.81 35.66395793999842 139.77047672 2.81 35.66397893999842 139.77047207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66379168999841 139.77100357 2.81 35.663791939998426 139.7710032 2.81 35.66377093999842 139.77100357 2.81 35.66379168999841 139.77100357 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66395793999842 139.77075358 2.81 35.663937439998406 139.77072207 2.81 35.66393743999841 139.77078458 2.81 35.66395793999842 139.77075358 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77059708 2.82 35.66406243999841 139.77059708 2.81 35.664083359998415 139.77062858 2.82 35.66410393999842 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77072208 2.81 35.66404193999841 139.77069058 2.81 35.664020939998416 139.77065957 2.81 35.664020939998416 139.77072208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664124939998416 139.77031557 2.82 35.664124939998416 139.77036775 2.82 35.66414593999841 139.77034708 2.82 35.664124939998416 139.77031557 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77091007 2.81 35.663874939998415 139.77091008 2.82 35.663874939998415 139.77087858 2.82 35.663853939998425 139.77091007 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66416693999842 139.77056558 2.81 35.66418743999842 139.77056558 2.81 35.664187439998415 139.77053458 2.81 35.66416693999842 139.77056558 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77072207 2.81 35.663832939998414 139.77075357 2.81 35.663853939998425 139.77078457 2.81 35.663853939998425 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663916689998416 139.77087858 2.82 35.66391693999842 139.7708782 2.82 35.663895939998426 139.77087858 2.82 35.663916689998416 139.77087858 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66420793999842 139.77040958 2.82 35.664187439998415 139.77040958 2.82 35.66420793999841 139.77044058 2.82 35.66420793999842 139.77040958 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66393743999841 139.77078458 2.81 35.66395793999842 139.77078458 2.81 35.66395793999842 139.77075358 2.81 35.66393743999841 139.77078458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66404185999841 139.77075358 2.82 35.66404193999842 139.77072208 2.82 35.664020939998416 139.77072208 2.81 35.66404185999841 139.77075358 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77072208 2.81 35.664020939998416 139.77075358 2.82 35.66404185999841 139.77075358 2.82 35.664020939998416 139.77072208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77059708 2.82 35.66397893999842 139.77053457 2.82 35.66397893999842 139.77059707 2.82 35.664020939998416 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77059708 2.82 35.664124939998416 139.77062833 2.82 35.664124939998416 139.77059708 2.82 35.66410393999842 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664145939998406 139.77047208 2.81 35.664124939998416 139.77044058 2.81 35.66410393999842 139.77047207 2.82 35.664145939998406 139.77047208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66404193999841 139.77050357 2.82 35.66400864999842 139.77050357 2.82 35.664020939998416 139.77053457 2.82 35.66404193999841 139.77050357 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663812439998416 139.77091007 2.82 35.66377093999842 139.77091007 2.81 35.663791939998426 139.77094157 2.81 35.663812439998416 139.77091007 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66377093999842 139.77091007 2.81 35.66377093999842 139.77084707 2.81 35.663749939998425 139.77087857000004 2.81 35.66377093999842 139.77091007 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77072207 2.81 35.663895939998426 139.77072207 2.81 35.663895939998426 139.77065957 2.81 35.663853939998425 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66377093999842 139.77100357 2.81 35.66377093999842 139.77097258 2.81 35.663749939998425 139.77098183 2.81 35.66377093999842 139.77100357 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66377093999842 139.77084707 2.81 35.663812439998416 139.77084707 2.81 35.663812439998416 139.77078457 2.81 35.66377093999842 139.77084707 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77065957 2.81 35.66397893999842 139.77065958 2.82 35.664020939998416 139.77072208 2.81 35.664020939998416 139.77065957 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663895939998426 139.77072207 2.81 35.66393743999841 139.77078458 2.81 35.663937439998406 139.77072207 2.81 35.663895939998426 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66420793999841 139.77031558 2.82 35.66416693999843 139.77031557 2.82 35.66420793999841 139.77032524 2.82 35.66420793999841 139.77031558 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77053457 2.82 35.66406243999841 139.77059708 2.81 35.66406243999842 139.77053457 2.81 35.664020939998416 139.77053457 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663957939998426 139.77062857 2.82 35.66397893999842 139.77065958 2.82 35.66397893999842 139.77059707 2.82 35.663957939998426 139.77062857 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663791939998426 139.77075357 2.81 35.66377093999842 139.77078457 2.81 35.663812439998416 139.77078457 2.81 35.663791939998426 139.77075357 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663749939998425 139.77087857000004 2.81 35.66377093999842 139.77084707 2.81 35.663749939998425 139.77081557 2.81 35.663749939998425 139.77087857000004 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663812439998416 139.77078457 2.81 35.66377093999842 139.77078457 2.81 35.66377093999842 139.77084707 2.81 35.663812439998416 139.77078457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663832939998414 139.77072955 2.81 35.663832939998414 139.77075357 2.81 35.663853939998425 139.77072207 2.81 35.663832939998414 139.77072955 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663832939998414 139.77075357 2.81 35.663791939998426 139.77075357 2.81 35.663812439998416 139.77078457 2.81 35.663832939998414 139.77075357 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663812439998416 139.77084707 2.81 35.663812439998416 139.77091007 2.82 35.663853939998425 139.77084708 2.82 35.663812439998416 139.77084707 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66395835999842 139.77075358 2.81 35.663937439998406 139.77072207 2.81 35.66395793999842 139.77075358 2.81 35.66395835999842 139.77075358 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77072207 2.81 35.66397893999842 139.77075358 2.81 35.66399993999841 139.77075358 2.81 35.66397893999842 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663937439998406 139.77072207 2.81 35.66397893999842 139.77065958 2.82 35.663937439998406 139.77065957 2.82 35.663937439998406 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66399993999841 139.77050357 2.82 35.66399993999841 139.77046265 2.81 35.66397893999842 139.77047207 2.81 35.66399993999841 139.77050357 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77053457 2.82 35.664020939998416 139.77059708 2.82 35.664020939998416 139.77053457 2.82 35.66397893999842 139.77053457 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664187439998415 139.77040958 2.82 35.66420793999842 139.77040958 2.82 35.66420793999841 139.77037858 2.82 35.664187439998415 139.77040958 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66393743999841 139.77078458 2.81 35.66393743999841 139.77081558 2.81 35.66395793999842 139.77081545 2.81 35.66393743999841 139.77078458 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663791939998426 139.77094157 2.81 35.663812439998416 139.77094157 2.81 35.663812439998416 139.77091007 2.82 35.663791939998426 139.77094157 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999842 139.77059708 2.82 35.66410393999842 139.77053458 2.81 35.66406243999841 139.77059708 2.81 35.66410393999842 139.77059708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77059707 2.82 35.66397893999842 139.77053457 2.82 35.66395793999841 139.77056557 2.82 35.66397893999842 139.77059707 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77072207 2.81 35.66397893999842 139.77065958 2.82 35.663937439998406 139.77072207 2.81 35.66397893999842 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664145939998406 139.77047208 2.81 35.66414593999842 139.77053458 2.82 35.664187439998415 139.77053458 2.81 35.664145939998406 139.77047208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77091007 2.81 35.663812439998416 139.77091007 2.82 35.663833359998414 139.77094158 2.81 35.663853939998425 139.77091007 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664082939998416 139.77044058 2.82 35.66406236999842 139.77044057 2.82 35.66406243999842 139.77047208 2.82 35.664082939998416 139.77044058 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664124939998416 139.77044058 2.81 35.664104829998415 139.77044057 2.82 35.66410393999842 139.77047207 2.82 35.664124939998416 139.77044058 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664020939998416 139.77072208 2.81 35.66404193999842 139.77072208 2.82 35.66404193999841 139.77069058 2.81 35.664020939998416 139.77072208 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66397893999842 139.77075358 2.81 35.66397893999842 139.77072207 2.81 35.66395835999842 139.77075358 2.81 35.66397893999842 139.77075358 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664082939998416 139.77044058 2.82 35.66406243999842 139.77047208 2.82 35.66410393999842 139.77047207 2.82 35.664082939998416 139.77044058 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663853939998425 139.77072207 2.81 35.663895939998426 139.77065957 2.81 35.663853939998425 139.77065957 2.81 35.663853939998425 139.77072207 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66377093999842 139.77078457 2.81 35.663791939998426 139.77075357 2.81 35.66374993999843 139.77075357 2.81 35.66377093999842 139.77078457 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663749939998425 139.77098183 2.81 35.663749939998425 139.77100357 2.82 35.66377093999842 139.77100357 2.81 35.663749939998425 139.77098183 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66414593999841 139.77034708 2.82 35.664166939998424 139.77037858 2.82 35.664187439998415 139.77034707 2.82 35.66414593999841 139.77034708 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66416693999843 139.77031557 2.82 35.664165309998424 139.77031557 2.82 35.66414593999841 139.77034708 2.82 35.66416693999843 139.77031557 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.664083359998415 139.77062858 2.82 35.66406243999841 139.77059708 2.81 35.66408293999842 139.77062858 2.81 35.664083359998415 139.77062858 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66399993999841 139.77050357 2.82 35.664020939998416 139.77053457 2.82 35.66400864999842 139.77050357 2.82 35.66399993999841 139.77050357 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66406243999842 139.77047208 2.82 35.66406236999842 139.77044057 2.82 35.66404193999842 139.77044057 2.82 35.66406243999842 139.77047208 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66410393999841 139.77062858 2.82 35.664124769998416 139.77062858 2.82 35.664124939998416 139.77062833 2.82 35.66410393999841 139.77062858 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66404193999841 139.77065958 2.81 35.664020939998416 139.77065957 2.81 35.66404193999841 139.77069058 2.81 35.66404193999841 139.77065958 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663895939998426 139.77084707 2.82 35.663853939998425 139.77078457 2.81 35.663853939998425 139.77084708 2.82 35.663895939998426 139.77084707 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66383293999842 139.77069057 2.81 35.663853939998425 139.77072207 2.81 35.663853939998425 139.77065957 2.81 35.66383293999842 139.77069057 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66412493999842 139.77037857 2.82 35.664166939998424 139.77037858 2.82 35.66414593999841 139.77034708 2.82 35.66412493999842 139.77037857 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663833359998414 139.77094158 2.81 35.663812439998416 139.77091007 2.82 35.663832939998414 139.77094158 2.81 35.663833359998414 139.77094158 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.663749939998425 139.77094157 2.81 35.663791939998426 139.77094157 2.81 35.66377093999842 139.77091007 2.81 35.663749939998425 139.77094157 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_cb2ff35e-5f43-4628-8f2c-f2752f6084e3">
			<gml:name>荒川水系荒川洪水浸水想定区域図（計画規模）</gml:name>
			<core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">6</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">2</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">1</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66587494999842 139.76669057 1.67 35.665874949998425 139.7666958 1.67 35.665895949998415 139.76669057 1.67 35.66587494999842 139.76669057 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665958029998414 139.76675357 1.67 35.66595794999842 139.76672207 1.67 35.665937449998424 139.76672207 1.67 35.665958029998414 139.76675357 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66602094999841 139.76684707 1.66 35.666020949998405 139.76681557 1.67 35.66599994999842 139.76681557 1.67 35.66602094999841 139.76684707 1.66</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665978949998426 139.76684707 1.67 35.665957949998415 139.76682415 1.67 35.66595794999842 139.76687857 1.67 35.665978949998426 139.76684707 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665958029998414 139.76675357 1.67 35.665937449998424 139.76672207 1.67 35.66591694999842 139.76675357 1.67 35.665958029998414 139.76675357 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66595794999842 139.76681557 1.67 35.66593744999842 139.76678457 1.67 35.66592073999842 139.76681557 1.67 35.66595794999842 139.76681557 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665978949998426 139.76678457 1.67 35.665978949998426 139.76675357 1.67 35.665958029998414 139.76675357 1.67 35.665978949998426 139.76678457 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66591652999842 139.76669057 1.67 35.665895949998415 139.76672207 1.67 35.665937449998424 139.76672207 1.67 35.66591652999842 139.76669057 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665978949998426 139.76678457 1.67 35.66599977999841 139.76675357 1.67 35.665978949998426 139.76675357 1.67 35.665978949998426 139.76678457 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66595794999842 139.76681557 1.67 35.665957949998415 139.76682415 1.67 35.66599994999842 139.76681557 1.67 35.66595794999842 139.76681557 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66599994999841 139.76687857 1.66 35.666029049998414 139.76687857 1.65 35.66602094999841 139.76684707 1.66 35.66599994999841 139.76687857 1.66</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66599994999841 139.76687857 1.66 35.66602094999841 139.76684707 1.66 35.665978949998426 139.76684707 1.67 35.66599994999841 139.76687857 1.66</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66599994999842 139.76681557 1.67 35.665999949998415 139.76678457 1.67 35.665978949998426 139.76678457 1.67 35.66599994999842 139.76681557 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66595794999842 139.76681557 1.67 35.66599994999842 139.76681557 1.67 35.665978949998426 139.76678457 1.67 35.66595794999842 139.76681557 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66602094999841 139.76684707 1.66 35.666029049998414 139.76687857 1.65 35.666041949998416 139.76684707 1.65 35.66602094999841 139.76684707 1.66</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665958029998414 139.76675357 1.67 35.66591694999842 139.76675357 1.67 35.66593744999842 139.76678457 1.67 35.665958029998414 139.76675357 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665978949998426 139.76684707 1.67 35.66599994999842 139.76681557 1.67 35.665957949998415 139.76682415 1.67 35.665978949998426 139.76684707 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66592073999842 139.76681557 1.67 35.66593744999842 139.76678457 1.67 35.66591694999841 139.76681557 1.67 35.66592073999842 139.76681557 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665874949998425 139.7666958 1.67 35.665874949998425 139.76675357 1.67 35.665895949998415 139.76672207 1.67 35.665874949998425 139.7666958 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66595794999842 139.76687857 1.67 35.66597226999841 139.76687857 1.67 35.665978949998426 139.76684707 1.67 35.66595794999842 139.76687857 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665978949998426 139.76678457 1.67 35.66593744999842 139.76678457 1.67 35.66595794999842 139.76681557 1.67 35.665978949998426 139.76678457 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665937449998424 139.76672207 1.67 35.66593744999842 139.76669057 1.67 35.66591694999842 139.76669057 1.67 35.665937449998424 139.76672207 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665937449998424 139.76672207 1.67 35.66595794999842 139.76672207 1.67 35.665957949998415 139.7666912 1.67 35.665937449998424 139.76672207 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665957949998415 139.7666912 1.67 35.66595794999842 139.76669057 1.67 35.66593744999842 139.76669057 1.67 35.665957949998415 139.7666912 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66591694999842 139.76669057 1.67 35.66591652999842 139.76669057 1.67 35.665937449998424 139.76672207 1.67 35.66591694999842 139.76669057 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66591694999842 139.76675357 1.67 35.66591694999841 139.76681557 1.67 35.66593744999842 139.76678457 1.67 35.66591694999842 139.76675357 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665895949998415 139.76672207 1.67 35.665874949998425 139.76675357 1.67 35.665894219998414 139.76675357 1.67 35.665895949998415 139.76672207 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66591652999842 139.76669057 1.67 35.665895949998415 139.76669057 1.67 35.665895949998415 139.76672207 1.67 35.66591652999842 139.76669057 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66591694999842 139.76675357 1.67 35.665937449998424 139.76672207 1.67 35.665895949998415 139.76672207 1.67 35.66591694999842 139.76675357 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66593744999842 139.76669057 1.67 35.665937449998424 139.76672207 1.67 35.665957949998415 139.7666912 1.67 35.66593744999842 139.76669057 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666029049998414 139.76687857 1.65 35.666041949998416 139.76687857 1.65 35.666041949998416 139.76684707 1.65 35.666029049998414 139.76687857 1.65</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665978949998426 139.76684707 1.67 35.66602094999841 139.76684707 1.66 35.66599994999842 139.76681557 1.67 35.665978949998426 139.76684707 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66602094999841 139.76684707 1.66 35.666041949998416 139.76681557 1.66 35.666020949998405 139.76681557 1.67 35.66602094999841 139.76684707 1.66</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665978949998426 139.76678457 1.67 35.665999949998415 139.76678457 1.67 35.66599977999841 139.76675357 1.67 35.665978949998426 139.76678457 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665895949998415 139.76672207 1.67 35.665895949998415 139.76669057 1.67 35.665874949998425 139.7666958 1.67 35.665895949998415 139.76672207 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66597226999841 139.76687857 1.67 35.66599994999841 139.76687857 1.66 35.665978949998426 139.76684707 1.67 35.66597226999841 139.76687857 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665895949998415 139.76672207 1.67 35.665894219998414 139.76675357 1.67 35.66591694999842 139.76675357 1.67 35.665895949998415 139.76672207 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66602094999841 139.76684707 1.66 35.666041949998416 139.76684707 1.65 35.666041949998416 139.76681557 1.66 35.66602094999841 139.76684707 1.66</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.665958029998414 139.76675357 1.67 35.66593744999842 139.76678457 1.67 35.665978949998426 139.76678457 1.67 35.665958029998414 139.76675357 1.67</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_e3c16239-92e6-4c1b-8f51-e28b238eede0">
			<gml:name>荒川水系荒川洪水浸水想定区域図（計画規模）</gml:name>
			<core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">6</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">4</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">1</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66649994999842 139.76712858 2.32 35.666478949998414 139.76712858 2.32 35.66649994999842 139.76715958 2.32 35.66649994999842 139.76712858 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66649994999842 139.76719058 1.98 35.66649994999842 139.76715958 2.32 35.666478949998414 139.76715958 2.32 35.66649994999842 139.76719058 1.98</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645835999842 139.76719058 2 35.66645794999842 139.76715958 1.99 35.66645794999842 139.76719058 1.99 35.66645835999842 139.76719058 2</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76715958 2.32 35.66647894999842 139.76719058 1.98 35.66649994999842 139.76719058 1.98 35.666478949998414 139.76715958 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76715958 2.32 35.66645794999842 139.76715958 1.99 35.66645835999842 139.76719058 2 35.666478949998414 139.76715958 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76715958 2.32 35.66645835999842 139.76719058 2 35.66647894999842 139.76719058 1.98 35.666478949998414 139.76715958 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645794999842 139.76712858 1.99 35.66645794999842 139.76715958 1.99 35.66645835999842 139.76712858 1.99 35.66645794999842 139.76712858 1.99</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76712858 2.32 35.66645835999842 139.76712858 1.99 35.666478949998414 139.76715958 2.32 35.666478949998414 139.76712858 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66645835999842 139.76712858 1.99 35.66645794999842 139.76715958 1.99 35.666478949998414 139.76715958 2.32 35.66645835999842 139.76712858 1.99</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666478949998414 139.76715958 2.32 35.66649994999842 139.76715958 2.32 35.666478949998414 139.76712858 2.32 35.666478949998414 139.76715958 2.32</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
	<core:cityObjectMember>
		<wtr:WaterBody gml:id="fld_cd442505-7231-4e52-bb67-9b33ec62dc9e">
			<gml:name>荒川水系荒川洪水浸水想定区域図（計画規模）</gml:name>
			<core:creationDate>2024-03-22T00:00:00Z</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:RiverFloodingRiskAttribute>
					<urc:description codeSpace="../../../../codelists/RiverFloodingRiskAttribute_description.xml">6</urc:description>
					<urc:rank codeSpace="../../../../codelists/RiverFloodingRiskAttribute_rank.xml">1</urc:rank>
					<urc:adminType codeSpace="../../../../codelists/RiverFloodingRiskAttribute_adminType.xml">1</urc:adminType>
					<urc:scale codeSpace="../../../../codelists/RiverFloodingRiskAttribute_scale.xml">1</urc:scale>
				</urc:RiverFloodingRiskAttribute>
			</core:adeOfAbstractCityObject>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../../../codelists/DataQualityAttribute_geometrySrcDesc.xml">400</urc:geometrySrcDescLod1>
					<urc:thematicSrcDesc codeSpace="../../../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</urc:thematicSrcDesc>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<wtr:WaterSurface>
					<core:lod1MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66658293999842 139.77375361 2.82 35.666603929998416 139.77378461 2.82 35.666604529998416 139.77375361 2.82 35.66658293999842 139.77375361 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66656243999842 139.77378461 2.82 35.666541939998424 139.77378461 2.82 35.66656243999842 139.77381561 2.82 35.66656243999842 139.77378461 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66662493999842 139.77381561 2.82 35.666603939998424 139.77384711 2.82 35.666645939998425 139.77384711 2.81 35.66662493999842 139.77381561 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66664592999841 139.77390961 2.81 35.66664592999842 139.77394061 2.81 35.66666692999842 139.77394061 2.87 35.66664592999841 139.77390961 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666666939998414 139.77384711 2.86 35.666645939998425 139.77384711 2.81 35.66666693999841 139.77387861 2.86 35.666666939998414 139.77384711 2.86</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666541939998424 139.77378461 2.82 35.66655581999842 139.77375361 2.82 35.66654193999842 139.77375361 2.82 35.666541939998424 139.77378461 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66658293999842 139.77381561 2.82 35.666603939998424 139.77384711 2.82 35.66662493999842 139.77381561 2.82 35.66658293999842 139.77381561 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66662493999842 139.77381561 2.82 35.666603929998416 139.77378461 2.82 35.66658293999842 139.77381561 2.82 35.66662493999842 139.77381561 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66664592999841 139.77390961 2.81 35.66666693999842 139.77390961 2.86 35.66666693999841 139.77387861 2.86 35.66664592999841 139.77390961 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66664592999841 139.77390961 2.81 35.66662493999842 139.77394061 2.81 35.66664592999842 139.77394061 2.81 35.66664592999841 139.77390961 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66658334999842 139.77387861 2.82 35.66658293999842 139.77387798 2.82 35.66658293999841 139.77387861 2.82 35.66658334999842 139.77387861 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666603939998424 139.77384711 2.82 35.66658293999842 139.77384711 2.82 35.66658293999842 139.77387798 2.82 35.666603939998424 139.77384711 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66666693999841 139.77387861 2.86 35.666624939998414 139.77387861 2.82 35.66664592999841 139.77390961 2.81 35.66666693999841 139.77387861 2.86</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666603939998424 139.77384711 2.82 35.666624939998414 139.77387861 2.82 35.666645939998425 139.77384711 2.81 35.666603939998424 139.77384711 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66662493999842 139.77381561 2.82 35.666645939998425 139.77384711 2.81 35.66665328999842 139.77381561 2.85 35.66662493999842 139.77381561 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666541939998424 139.77381561 2.82 35.66656243999842 139.77381561 2.82 35.666541939998424 139.77378461 2.82 35.666541939998424 139.77381561 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666624939998414 139.77387861 2.82 35.66666693999841 139.77387861 2.86 35.666645939998425 139.77384711 2.81 35.666624939998414 139.77387861 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66658293999842 139.77375361 2.82 35.66656243999842 139.77378461 2.82 35.666603929998416 139.77378461 2.82 35.66658293999842 139.77375361 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66658293999842 139.77381561 2.82 35.66658293999842 139.77384711 2.82 35.666603939998424 139.77384711 2.82 35.66658293999842 139.77381561 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666624939998414 139.77387861 2.82 35.66662493999842 139.77390961 2.81 35.66664592999841 139.77390961 2.81 35.666624939998414 139.77387861 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666603939998424 139.77384711 2.82 35.66660393999842 139.77387861 2.82 35.666624939998414 139.77387861 2.82 35.666603939998424 139.77384711 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66665328999842 139.77381561 2.85 35.666666939998414 139.77384711 2.86 35.666666939998414 139.77381561 2.87 35.66665328999842 139.77381561 2.85</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66655581999842 139.77375361 2.82 35.66656243999842 139.77378461 2.82 35.66658293999842 139.77375361 2.82 35.66655581999842 139.77375361 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66658293999842 139.77381561 2.82 35.666603929998416 139.77378461 2.82 35.66656243999842 139.77378461 2.82 35.66658293999842 139.77381561 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666541939998424 139.77378461 2.82 35.66656243999842 139.77378461 2.82 35.66655581999842 139.77375361 2.82 35.666541939998424 139.77378461 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66656243999842 139.77378461 2.82 35.66656243999842 139.77381561 2.82 35.66658293999842 139.77381561 2.82 35.66656243999842 139.77378461 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666645939998425 139.77384711 2.81 35.666666939998414 139.77384711 2.86 35.66665328999842 139.77381561 2.85 35.666645939998425 139.77384711 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66658293999842 139.77387798 2.82 35.66658334999842 139.77387861 2.82 35.66660393999842 139.77387861 2.82 35.66658293999842 139.77387798 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66662493999843 139.77375361 2.82 35.666603929998416 139.77378461 2.82 35.66662493999842 139.77381561 2.82 35.66662493999843 139.77375361 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.666603939998424 139.77384711 2.82 35.66658293999842 139.77387798 2.82 35.66660393999842 139.77387861 2.82 35.666603939998424 139.77384711 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66666693999842 139.77390961 2.86 35.66664592999841 139.77390961 2.81 35.66666692999842 139.77394061 2.87 35.66666693999842 139.77390961 2.86</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66662493999843 139.77375361 2.82 35.666604529998416 139.77375361 2.82 35.666603929998416 139.77378461 2.82 35.66662493999843 139.77375361 2.82</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon>
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.66664592999841 139.77390961 2.81 35.66662493999842 139.77390961 2.81 35.66662493999842 139.77394061 2.81 35.66664592999841 139.77390961 2.81</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod1MultiSurface>
				</wtr:WaterSurface>
			</core:boundary>
			<wtr:class codeSpace="../../../../codelists/WaterBody_class.xml">1140</wtr:class>
			<wtr:function codeSpace="../../../../codelists/WaterBody_function.xml">1</wtr:function>
		</wtr:WaterBody>
	</core:cityObjectMember>
</core:CityModel>
