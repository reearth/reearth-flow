<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/cityfurniture/2.0 http://schemas.opengis.net/citygml/cityfurniture/2.0/cityFurniture.xsd http://www.opengis.net/citygml/vegetation/2.0 http://schemas.opengis.net/citygml/vegetation/2.0/vegetation.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.66624522038004 139.66219368287665 0</gml:lowerCorner>
			<gml:upperCorner>35.675322449690725 139.67587782135018 0</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<tran:Road gml:id="tran_947e130c-7a22-4ed5-855b-c91047ea0feb">
			<core:creationDate>2024-03-15</core:creationDate>
			<tran:class codeSpace="../../codelists/TransportationComplex_class.xml">1040</tran:class>
			<tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
			<tran:usage codeSpace="../../codelists/Road_usage.xml">9</tran:usage>
			<tran:trafficArea>
				<tran:TrafficArea gml:id="traf_e5ea50ce-7198-4879-a380-9d99f1c6491f">
					<tran:function codeSpace="../../codelists/TrafficArea_function.xml">1020</tran:function>
					<tran:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ae864f59-b985-4d27-8d66-202cc8675b43">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67288605327986 139.6658240611919 0 35.67288995984246 139.6659106587737 0 35.672915467615795 139.6659101631988 0 35.67288605327986 139.6658240611919 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_e8f029c7-a919-40ce-86a6-89d50eda09b6">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672815516463245 139.6659803729285 0 35.67276991113465 139.66601465318192 0 35.6728191856738 139.66599262788623 0 35.672815516463245 139.6659803729285 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_f3758bbc-db46-494f-8c24-bd30fd322960">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67288995984246 139.6659106587737 0 35.67288605327986 139.6658240611919 0 35.6728730136403 139.66591036276546 0 35.67288995984246 139.6659106587737 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_f672e67c-38cd-4c2a-9735-63f12441e91e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67283723559489 139.66585023319954 0 35.67288605327986 139.6658240611919 0 35.67281603694162 139.66577383506035 0 35.67283723559489 139.66585023319954 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_32e00a91-92a1-4efa-a83d-2c489e50c52f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67270208930679 139.6658282014633 0 35.67267862916285 139.665810796846 0 35.67265103699013 139.66585153241735 0 35.67270208930679 139.6658282014633 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_7fa664e2-6846-4421-b469-2fa0acde0d78">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67270208930679 139.6658282014633 0 35.67269361600829 139.66580148619653 0 35.67267862916285 139.665810796846 0 35.67270208930679 139.6658282014633 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_0a88d571-35bf-4064-bea4-caffca266e54">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67275738250737 139.66579152176772 0 35.67279004937512 139.66575367413256 0 35.67278210733258 139.6657465104569 0 35.67275738250737 139.66579152176772 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_483b2448-eb20-4acb-a267-539acb6916f5">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6727598208154 139.6657948306451 0 35.67281603694162 139.66577383506035 0 35.67279004937512 139.66575367413256 0 35.6727598208154 139.6657948306451 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_0a237387-9a86-473b-a081-429651d2e00a">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672756112451616 139.66598031034974 0 35.67275863444312 139.66597897949597 0 35.672745697262755 139.6659447621201 0 35.672756112451616 139.66598031034974 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_4a01bffb-dfdc-48fb-91c1-7a136e959d33">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672815516463245 139.6659803729285 0 35.67275863444312 139.66597897949597 0 35.67276991113465 139.66601465318192 0 35.672815516463245 139.6659803729285 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_927ea744-0757-4fd8-95ba-ed94f87f7ada">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672715026059365 139.66586208740546 0 35.67265103699013 139.66585153241735 0 35.672745697262755 139.6659447621201 0 35.672715026059365 139.66586208740546 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_76849277-2045-4466-bf90-cc5a987d5cd4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67275576004275 139.6657915251584 0 35.67278210733258 139.6657465104569 0 35.67271079693104 139.66579081246115 0 35.67275576004275 139.6657915251584 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_7629f566-b7c6-446c-91bc-0b5e2b6d5c55">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67275863444312 139.66597897949597 0 35.672815516463245 139.6659803729285 0 35.67281189098665 139.66596826404484 0 35.67275863444312 139.66597897949597 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ec06c138-e25c-4c25-9ca4-65c2ca1caf89">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67281636712214 139.6659461615312 0 35.672745697262755 139.6659447621201 0 35.67281189098665 139.66596826404484 0 35.67281636712214 139.6659461615312 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_24a32920-4920-48da-8f7b-fa1ec53435d5">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6728364509428 139.6658693454348 0 35.672855258048955 139.6659113940384 0 35.6728730136403 139.66591036276546 0 35.6728364509428 139.6658693454348 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_c09ce6f2-a87f-432e-a3c0-6743f5ea2f1b">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67281636712214 139.6659461615312 0 35.67282184581751 139.66593201046305 0 35.67279332144226 139.66590257527028 0 35.67281636712214 139.6659461615312 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_fd5a564f-c4a1-48cd-805b-fa86bc286e1d">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672751175509404 139.66580048246544 0 35.67271079693104 139.66579081246115 0 35.67272933985371 139.66584913301492 0 35.672751175509404 139.66580048246544 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ea1d29df-1a77-4c1c-9b07-91541097ea56">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672828980946875 139.66587753551244 0 35.672828233252645 139.66592315985665 0 35.672855258048955 139.6659113940384 0 35.672828980946875 139.66587753551244 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9e4f304d-60d2-4319-b7db-ff85cac869fd">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672828233252645 139.66592315985665 0 35.67280305208267 139.66589957237107 0 35.67282184581751 139.66593201046305 0 35.672828233252645 139.66592315985665 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_92be9263-1f30-4775-9290-78dbf8055603">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67270208930679 139.6658282014633 0 35.67265103699013 139.66585153241735 0 35.6727105132211 139.6658577886702 0 35.67270208930679 139.6658282014633 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_de6c710e-f1a9-4ad7-a332-a12ccac46881">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67278475121503 139.665897401276 0 35.67272340556685 139.66585975012111 0 35.67271989253145 139.6658614144454 0 35.67278475121503 139.665897401276 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_1a3e1437-73a4-4133-bad5-6cfd2c73432e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67278475121503 139.665897401276 0 35.67272880686831 139.66585476788032 0 35.67272340556685 139.66585975012111 0 35.67278475121503 139.665897401276 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_a7b99b2b-d70b-41f8-ae64-98d06189f1b3">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67278475121503 139.665897401276 0 35.67272933985371 139.66584913301492 0 35.67272880686831 139.66585476788032 0 35.67278475121503 139.665897401276 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3d7ccdbd-269c-40d1-b6bc-f95435dd5f71">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6727105132211 139.6658577886702 0 35.67265103699013 139.66585153241735 0 35.67271313087932 139.66586043437883 0 35.6727105132211 139.6658577886702 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_840c24ca-aa9c-4d36-887b-c0adb62930c8">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67271313087932 139.66586043437883 0 35.67265103699013 139.66585153241735 0 35.672715026059365 139.66586208740546 0 35.67271313087932 139.66586043437883 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_fbb5dee8-3afb-4c01-bc72-50e7d6606ed0">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672745697262755 139.6659447621201 0 35.67271989253145 139.6658614144454 0 35.672715026059365 139.66586208740546 0 35.672745697262755 139.6659447621201 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_88decb30-bbf7-4c3f-a500-5808f8ba4a87">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67288605327986 139.6658240611919 0 35.672838601476975 139.6658601722749 0 35.6728730136403 139.66591036276546 0 35.67288605327986 139.6658240611919 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_01f96699-6c09-4c43-ab0d-4a47934b4d69">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67283723559489 139.66585023319954 0 35.67281603694162 139.66577383506035 0 35.67280159417006 139.66582346446233 0 35.67283723559489 139.66585023319954 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8a121acb-987a-4403-8111-4b102562f715">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67283859548484 139.66585586411824 0 35.67288605327986 139.6658240611919 0 35.67283723559489 139.66585023319954 0 35.67283859548484 139.66585586411824 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_bd8aa6f5-12f0-47c7-9d6e-8d79740cab94">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67278210733258 139.6657465104569 0 35.67275576004275 139.6657915251584 0 35.67275738250737 139.66579152176772 0 35.67278210733258 139.6657465104569 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_15b04944-6cea-4019-9e95-478bbc32fb5a">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672758735944555 139.66579251313118 0 35.67279004937512 139.66575367413256 0 35.67275738250737 139.66579152176772 0 35.672758735944555 139.66579251313118 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6dd272b6-cce8-4f0b-9a57-5bb4f680a61d">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672758735944555 139.66579251313118 0 35.6727598208154 139.6657948306451 0 35.67279004937512 139.66575367413256 0 35.672758735944555 139.66579251313118 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9c78737e-54a7-476c-90b1-55647e60ae15">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6727598208154 139.6657948306451 0 35.672790498568816 139.66581719109465 0 35.67281603694162 139.66577383506035 0 35.6727598208154 139.6657948306451 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ce996f99-c580-4eb3-bc3a-7bddcbdee12d">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67281189098665 139.66596826404484 0 35.672745697262755 139.6659447621201 0 35.67275863444312 139.66597897949597 0 35.67281189098665 139.66596826404484 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_d33c7861-6cbc-411f-8f94-ab44f0262f29">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67275576004275 139.6657915251584 0 35.67271079693104 139.66579081246115 0 35.67275332957357 139.66579385001856 0 35.67275576004275 139.6657915251584 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_4e7dc0eb-edc3-44f8-8c7f-5d2548784276">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67279332144226 139.66590257527028 0 35.672745697262755 139.6659447621201 0 35.67281636712214 139.6659461615312 0 35.67279332144226 139.66590257527028 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_72e18eca-e5d6-4cfd-a046-540c73df8931">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672838601476975 139.6658601722749 0 35.6728364509428 139.6658693454348 0 35.6728730136403 139.66591036276546 0 35.672838601476975 139.6658601722749 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_b9acc917-0f73-4f86-9387-c3b9982f2ae7">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672834293648314 139.66587365811 0 35.672855258048955 139.6659113940384 0 35.6728364509428 139.6658693454348 0 35.672834293648314 139.66587365811 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_fe2a0257-9fc3-4310-83b4-1fe86a4420a6">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67282184581751 139.66593201046305 0 35.67279710627142 139.665901904571 0 35.67279332144226 139.66590257527028 0 35.67282184581751 139.66593201046305 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9b9b6dab-ad8b-49a3-bc5c-9b89334108a7">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672751175509404 139.66580048246544 0 35.67272933985371 139.66584913301492 0 35.67278475121503 139.665897401276 0 35.672751175509404 139.66580048246544 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_a2b2d649-9b0c-4e8b-ae57-0ff6184fcb22">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672751175509404 139.66580048246544 0 35.67275332957357 139.66579385001856 0 35.67271079693104 139.66579081246115 0 35.672751175509404 139.66580048246544 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_14b3a11d-ac34-4bc2-890d-2140742421a3">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672828980946875 139.66587753551244 0 35.672855258048955 139.6659113940384 0 35.672834293648314 139.66587365811 0 35.672828980946875 139.66587753551244 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6480e7dd-6ef4-409f-a6c0-7900c6bac194">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672828980946875 139.66587753551244 0 35.67282177245103 139.66587931802286 0 35.672828233252645 139.66592315985665 0 35.672828980946875 139.66587753551244 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_036171b6-0a33-415e-8a4c-35483013b3a4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67282184581751 139.66593201046305 0 35.67280305208267 139.66589957237107 0 35.67279710627142 139.665901904571 0 35.67282184581751 139.66593201046305 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_36618579-a876-4ff6-ab93-8fbb6b95862e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672804042667174 139.66589890750728 0 35.67280305208267 139.66589957237107 0 35.672828233252645 139.66592315985665 0 35.672804042667174 139.66589890750728 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ae5c2e2b-fa14-4f49-85b9-dcb2aee69878">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67278475121503 139.665897401276 0 35.67271989253145 139.6658614144454 0 35.672745697262755 139.6659447621201 0 35.67278475121503 139.665897401276 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_a9aca213-f496-43fe-a71d-a0d56a7768c0">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67283859548484 139.66585586411824 0 35.672838601476975 139.6658601722749 0 35.67288605327986 139.6658240611919 0 35.67283859548484 139.66585586411824 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6329c057-d8de-42d2-8649-996885c15dc8">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67280159417006 139.66582346446233 0 35.67281603694162 139.66577383506035 0 35.67279212103344 139.6658171877045 0 35.67280159417006 139.66582346446233 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_86609acc-07eb-4052-9890-67659c64c843">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67281603694162 139.66577383506035 0 35.672790498568816 139.66581719109465 0 35.67279212103344 139.6658171877045 0 35.67281603694162 139.66577383506035 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ff6f385d-1da0-4f8a-a41e-3cbab0cd27c4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6727598208154 139.6657948306451 0 35.67278914697603 139.6658175253162 0 35.672790498568816 139.66581719109465 0 35.6727598208154 139.6657948306451 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_273c1853-7621-40b9-b620-645ce59ac8c1">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672787910308344 139.6659004877208 0 35.672745697262755 139.6659447621201 0 35.67279332144226 139.66590257527028 0 35.672787910308344 139.6659004877208 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_2e49bba6-7e94-4f5a-98f8-3179f959126e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672828233252645 139.66592315985665 0 35.67282177245103 139.66587931802286 0 35.67280422140497 139.6658978024757 0 35.672828233252645 139.66592315985665 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ca13509b-b6b4-4bcf-8642-fc449075053e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672804042667174 139.66589890750728 0 35.672828233252645 139.66592315985665 0 35.67280422140497 139.6658978024757 0 35.672804042667174 139.66589890750728 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6da4217d-ffda-48e2-9a00-de01cc943dec">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672787910308344 139.6659004877208 0 35.67278475121503 139.665897401276 0 35.672745697262755 139.6659447621201 0 35.672787910308344 139.6659004877208 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ab707aa8-5232-489a-9b2d-d86cb5e66822">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6727598208154 139.6657948306451 0 35.67278914882019 139.66581885090204 0 35.67278914697603 139.6658175253162 0 35.6727598208154 139.6657948306451 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_022be09f-5bbe-48e4-9b25-1859731d23dd">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67282177245103 139.66587931802286 0 35.6728187074884 139.66587910349304 0 35.67280422140497 139.6658978024757 0 35.67282177245103 139.66587931802286 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_a08f0832-9440-4141-a509-36db3c9ff057">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6727598208154 139.6657948306451 0 35.67278443671457 139.66586569824779 0 35.67278914882019 139.66581885090204 0 35.6727598208154 139.6657948306451 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_ade3a634-d21e-42f2-b86d-c65c27ccc9f4">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67280422140497 139.6658978024757 0 35.6728187074884 139.66587910349304 0 35.672803588449575 139.66589636774185 0 35.67280422140497 139.6658978024757 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_3f47d393-49bd-4cd0-a3ab-2c0ad92e5978">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67278443671457 139.66586569824779 0 35.67280056154447 139.66585870521772 0 35.67278914882019 139.66581885090204 0 35.67278443671457 139.66586569824779 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_30cb4f09-b07c-45bb-8750-eeaadaf37afe">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6728187074884 139.66587910349304 0 35.67281302363895 139.6658753595265 0 35.672803588449575 139.66589636774185 0 35.6728187074884 139.66587910349304 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_84150a73-8187-4811-826d-390a8ab04ffe">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67279220999821 139.66588114722387 0 35.67280056154447 139.66585870521772 0 35.67278443671457 139.66586569824779 0 35.67279220999821 139.66588114722387 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_82e7c049-f079-4721-991c-aed4f728da4e">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672803588449575 139.66589636774185 0 35.67281302363895 139.6658753595265 0 35.67279789968497 139.66588908887883 0 35.672803588449575 139.66589636774185 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_6dfdec69-d593-48cf-b575-be835dfb4fc6">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67280056154447 139.66585870521772 0 35.67279220999821 139.66588114722387 0 35.67281302363895 139.6658753595265 0 35.67280056154447 139.66585870521772 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_9cc730fa-e944-4b87-8ab1-0280c255e1be">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67281302363895 139.6658753595265 0 35.67279220999821 139.66588114722387 0 35.67279789968497 139.66588908887883 0 35.67281302363895 139.6658753595265 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod2MultiSurface>
				</tran:TrafficArea>
			</tran:trafficArea>
			<tran:trafficArea>
				<tran:TrafficArea gml:id="traf_aad39605-4836-42da-9c5a-d6c4817a7e57">
					<tran:function codeSpace="../../codelists/TrafficArea_function.xml">2000</tran:function>
					<tran:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_5e021f45-0d8e-482e-b094-a1fece9873a7">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67283003532266 139.66598777818248 0 35.67283226924722 139.6659737443542 0 35.67284621390668 139.66595460464552 0 35.67286502779971 139.66593678035397 0 35.672915467615795 139.6659101631988 0 35.67288995984246 139.6659106587737 0 35.6728730136403 139.66591036276546 0 35.672855258048955 139.6659113940384 0 35.672828233252645 139.66592315985665 0 35.67282184581751 139.66593201046305 0 35.67281636712214 139.6659461615312 0 35.67281189098665 139.66596826404484 0 35.672815516463245 139.6659803729285 0 35.6728191856738 139.66599262788623 0 35.67283003532266 139.66598777818248 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod2MultiSurface>
				</tran:TrafficArea>
			</tran:trafficArea>
			<tran:auxiliaryTrafficArea>
				<tran:AuxiliaryTrafficArea gml:id="traf_bed931ec-cbff-4d64-ba25-5e8e760e7519">
					<tran:function codeSpace="../../codelists/AuxiliaryTrafficArea_function.xml">3000</tran:function>
					<tran:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_8a63b778-be6d-4d1b-bef5-3730825c094f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672789148820186 139.66581885090204 0 35.67280056154447 139.66585870521772 0 35.672813023638945 139.6658753595265 0 35.672818707488396 139.665879103493 0 35.67282177245102 139.66587931802286 0 35.67282898094687 139.66587753551244 0 35.672834293648314 139.66587365811 0 35.672836450942796 139.6658693454348 0 35.67283860147697 139.6658601722749 0 35.67283859548483 139.66585586411824 0 35.67283723559488 139.6658502331995 0 35.67280159417005 139.66582346446233 0 35.672792121033424 139.66581718770448 0 35.672790498568816 139.66581719109465 0 35.67278914697602 139.6658175253162 0 35.672789148820186 139.66581885090204 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod2MultiSurface>
				</tran:AuxiliaryTrafficArea>
			</tran:auxiliaryTrafficArea>
			<tran:auxiliaryTrafficArea>
				<tran:AuxiliaryTrafficArea gml:id="traf_97b992eb-54b1-426f-8705-67bbf89c6a24">
					<tran:function codeSpace="../../codelists/AuxiliaryTrafficArea_function.xml">3000</tran:function>
					<tran:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_be332343-d8f0-4856-85cf-18f1ec0831ed">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.672784751215026 139.66589740127597 0 35.67278791030834 139.6659004877208 0 35.67279332144225 139.66590257527028 0 35.67279710627141 139.665901904571 0 35.672803052082664 139.66589957237107 0 35.67280404266717 139.66589890750728 0 35.67280422140496 139.6658978024757 0 35.67280358844957 139.66589636774185 0 35.67279789968496 139.6658890888788 0 35.67279220999819 139.66588114722387 0 35.67278443671456 139.66586569824776 0 35.67275982081539 139.6657948306451 0 35.67275873594454 139.66579251313118 0 35.67275738250736 139.66579152176772 0 35.67275576004274 139.66579152515837 0 35.67275332957357 139.66579385001856 0 35.67275117550939 139.66580048246544 0 35.672784751215026 139.66589740127597 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod2MultiSurface>
				</tran:AuxiliaryTrafficArea>
			</tran:auxiliaryTrafficArea>
			<tran:auxiliaryTrafficArea>
				<tran:AuxiliaryTrafficArea gml:id="traf_f8f3ceeb-a7ec-447d-8a20-620b0b7bc413">
					<tran:function codeSpace="../../codelists/AuxiliaryTrafficArea_function.xml">3000</tran:function>
					<tran:lod2MultiSurface>
						<gml:MultiSurface>
							<gml:surfaceMember>
								<gml:Polygon gml:id="poly_daabf7b7-3c9c-408d-85b0-4cc91ec4d271">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.67269361600829 139.66580148619653 0 35.67270208930679 139.6658282014633 0 35.67271051322109 139.6658577886702 0 35.672713130879316 139.66586043437883 0 35.67271502605936 139.66586208740546 0 35.67271989253144 139.6658614144454 0 35.67272340556684 139.66585975012111 0 35.6727288068683 139.66585476788032 0 35.67272933985371 139.66584913301492 0 35.67271079693104 139.66579081246115 0 35.67269361600829 139.66580148619653 0</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</tran:lod2MultiSurface>
				</tran:AuxiliaryTrafficArea>
			</tran:auxiliaryTrafficArea>
			<tran:lod1MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.672745697623895 139.66594476256125 0 35.67275611281274 139.66598031079084 0 35.672758634804254 139.66597897993708 0 35.67276991096716 139.66601465341563 0 35.672819185398836 139.6659926275252 0 35.672830035683845 139.6659877786236 0 35.67283226960838 139.6659737447953 0 35.67284621426781 139.66595460508663 0 35.672865028160864 139.66593678079508 0 35.67291546797697 139.6659101636399 0 35.672886053641015 139.665824061633 0 35.672816037302766 139.66577383550148 0 35.67279004973626 139.66575367457366 0 35.67278210769373 139.665746510898 0 35.672710796917656 139.66579081200044 0 35.67269361629797 139.66580148574957 0 35.672678629523965 139.6658107972871 0 35.67265103660906 139.66585153280363 0 35.672745697623895 139.66594476256125 0</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod1MultiSurface>
			<tran:lod2MultiSurface>
				<gml:MultiSurface>
					<gml:surfaceMember>
						<gml:CompositeSurface>
							<gml:surfaceMember xlink:href="#poly_ae864f59-b985-4d27-8d66-202cc8675b43"/>
							<gml:surfaceMember xlink:href="#poly_e8f029c7-a919-40ce-86a6-89d50eda09b6"/>
							<gml:surfaceMember xlink:href="#poly_f3758bbc-db46-494f-8c24-bd30fd322960"/>
							<gml:surfaceMember xlink:href="#poly_f672e67c-38cd-4c2a-9735-63f12441e91e"/>
							<gml:surfaceMember xlink:href="#poly_32e00a91-92a1-4efa-a83d-2c489e50c52f"/>
							<gml:surfaceMember xlink:href="#poly_7fa664e2-6846-4421-b469-2fa0acde0d78"/>
							<gml:surfaceMember xlink:href="#poly_0a88d571-35bf-4064-bea4-caffca266e54"/>
							<gml:surfaceMember xlink:href="#poly_483b2448-eb20-4acb-a267-539acb6916f5"/>
							<gml:surfaceMember xlink:href="#poly_0a237387-9a86-473b-a081-429651d2e00a"/>
							<gml:surfaceMember xlink:href="#poly_4a01bffb-dfdc-48fb-91c1-7a136e959d33"/>
							<gml:surfaceMember xlink:href="#poly_927ea744-0757-4fd8-95ba-ed94f87f7ada"/>
							<gml:surfaceMember xlink:href="#poly_76849277-2045-4466-bf90-cc5a987d5cd4"/>
							<gml:surfaceMember xlink:href="#poly_7629f566-b7c6-446c-91bc-0b5e2b6d5c55"/>
							<gml:surfaceMember xlink:href="#poly_ec06c138-e25c-4c25-9ca4-65c2ca1caf89"/>
							<gml:surfaceMember xlink:href="#poly_24a32920-4920-48da-8f7b-fa1ec53435d5"/>
							<gml:surfaceMember xlink:href="#poly_c09ce6f2-a87f-432e-a3c0-6743f5ea2f1b"/>
							<gml:surfaceMember xlink:href="#poly_fd5a564f-c4a1-48cd-805b-fa86bc286e1d"/>
							<gml:surfaceMember xlink:href="#poly_ea1d29df-1a77-4c1c-9b07-91541097ea56"/>
							<gml:surfaceMember xlink:href="#poly_9e4f304d-60d2-4319-b7db-ff85cac869fd"/>
							<gml:surfaceMember xlink:href="#poly_92be9263-1f30-4775-9290-78dbf8055603"/>
							<gml:surfaceMember xlink:href="#poly_de6c710e-f1a9-4ad7-a332-a12ccac46881"/>
							<gml:surfaceMember xlink:href="#poly_1a3e1437-73a4-4133-bad5-6cfd2c73432e"/>
							<gml:surfaceMember xlink:href="#poly_a7b99b2b-d70b-41f8-ae64-98d06189f1b3"/>
							<gml:surfaceMember xlink:href="#poly_3d7ccdbd-269c-40d1-b6bc-f95435dd5f71"/>
							<gml:surfaceMember xlink:href="#poly_840c24ca-aa9c-4d36-887b-c0adb62930c8"/>
							<gml:surfaceMember xlink:href="#poly_fbb5dee8-3afb-4c01-bc72-50e7d6606ed0"/>
							<gml:surfaceMember xlink:href="#poly_88decb30-bbf7-4c3f-a500-5808f8ba4a87"/>
							<gml:surfaceMember xlink:href="#poly_01f96699-6c09-4c43-ab0d-4a47934b4d69"/>
							<gml:surfaceMember xlink:href="#poly_8a121acb-987a-4403-8111-4b102562f715"/>
							<gml:surfaceMember xlink:href="#poly_bd8aa6f5-12f0-47c7-9d6e-8d79740cab94"/>
							<gml:surfaceMember xlink:href="#poly_15b04944-6cea-4019-9e95-478bbc32fb5a"/>
							<gml:surfaceMember xlink:href="#poly_6dd272b6-cce8-4f0b-9a57-5bb4f680a61d"/>
							<gml:surfaceMember xlink:href="#poly_9c78737e-54a7-476c-90b1-55647e60ae15"/>
							<gml:surfaceMember xlink:href="#poly_ce996f99-c580-4eb3-bc3a-7bddcbdee12d"/>
							<gml:surfaceMember xlink:href="#poly_d33c7861-6cbc-411f-8f94-ab44f0262f29"/>
							<gml:surfaceMember xlink:href="#poly_4e7dc0eb-edc3-44f8-8c7f-5d2548784276"/>
							<gml:surfaceMember xlink:href="#poly_72e18eca-e5d6-4cfd-a046-540c73df8931"/>
							<gml:surfaceMember xlink:href="#poly_b9acc917-0f73-4f86-9387-c3b9982f2ae7"/>
							<gml:surfaceMember xlink:href="#poly_fe2a0257-9fc3-4310-83b4-1fe86a4420a6"/>
							<gml:surfaceMember xlink:href="#poly_9b9b6dab-ad8b-49a3-bc5c-9b89334108a7"/>
							<gml:surfaceMember xlink:href="#poly_a2b2d649-9b0c-4e8b-ae57-0ff6184fcb22"/>
							<gml:surfaceMember xlink:href="#poly_14b3a11d-ac34-4bc2-890d-2140742421a3"/>
							<gml:surfaceMember xlink:href="#poly_6480e7dd-6ef4-409f-a6c0-7900c6bac194"/>
							<gml:surfaceMember xlink:href="#poly_036171b6-0a33-415e-8a4c-35483013b3a4"/>
							<gml:surfaceMember xlink:href="#poly_36618579-a876-4ff6-ab93-8fbb6b95862e"/>
							<gml:surfaceMember xlink:href="#poly_ae5c2e2b-fa14-4f49-85b9-dcb2aee69878"/>
							<gml:surfaceMember xlink:href="#poly_a9aca213-f496-43fe-a71d-a0d56a7768c0"/>
							<gml:surfaceMember xlink:href="#poly_6329c057-d8de-42d2-8649-996885c15dc8"/>
							<gml:surfaceMember xlink:href="#poly_86609acc-07eb-4052-9890-67659c64c843"/>
							<gml:surfaceMember xlink:href="#poly_ff6f385d-1da0-4f8a-a41e-3cbab0cd27c4"/>
							<gml:surfaceMember xlink:href="#poly_273c1853-7621-40b9-b620-645ce59ac8c1"/>
							<gml:surfaceMember xlink:href="#poly_2e49bba6-7e94-4f5a-98f8-3179f959126e"/>
							<gml:surfaceMember xlink:href="#poly_ca13509b-b6b4-4bcf-8642-fc449075053e"/>
							<gml:surfaceMember xlink:href="#poly_6da4217d-ffda-48e2-9a00-de01cc943dec"/>
							<gml:surfaceMember xlink:href="#poly_ab707aa8-5232-489a-9b2d-d86cb5e66822"/>
							<gml:surfaceMember xlink:href="#poly_022be09f-5bbe-48e4-9b25-1859731d23dd"/>
							<gml:surfaceMember xlink:href="#poly_a08f0832-9440-4141-a509-36db3c9ff057"/>
							<gml:surfaceMember xlink:href="#poly_ade3a634-d21e-42f2-b86d-c65c27ccc9f4"/>
							<gml:surfaceMember xlink:href="#poly_3f47d393-49bd-4cd0-a3ab-2c0ad92e5978"/>
							<gml:surfaceMember xlink:href="#poly_30cb4f09-b07c-45bb-8750-eeaadaf37afe"/>
							<gml:surfaceMember xlink:href="#poly_84150a73-8187-4811-826d-390a8ab04ffe"/>
							<gml:surfaceMember xlink:href="#poly_82e7c049-f079-4721-991c-aed4f728da4e"/>
							<gml:surfaceMember xlink:href="#poly_6dfdec69-d593-48cf-b575-be835dfb4fc6"/>
							<gml:surfaceMember xlink:href="#poly_9cc730fa-e944-4b87-8ab1-0280c255e1be"/>
							<gml:surfaceMember xlink:href="#poly_5e021f45-0d8e-482e-b094-a1fece9873a7"/>
							<gml:surfaceMember xlink:href="#poly_8a63b778-be6d-4d1b-bef5-3730825c094f"/>
							<gml:surfaceMember xlink:href="#poly_be332343-d8f0-4856-85cf-18f1ec0831ed"/>
							<gml:surfaceMember xlink:href="#poly_daabf7b7-3c9c-408d-85b0-4cc91ec4d271"/>
						</gml:CompositeSurface>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</tran:lod2MultiSurface>
			<uro:tranDataQualityAttribute>
				<uro:DataQualityAttribute>
					<uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod0>
					<uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
					<uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod2>
					<uro:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod3>
					<uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">023</uro:thematicSrcDesc>
					<uro:appearanceSrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod3>
					<uro:publicSurveyDataQualityAttribute>
						<uro:PublicSurveyDataQualityAttribute>
							<uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
							<uro:srcScaleLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod2>
							<uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
							<uro:publicSurveySrcDescLod2 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml">023</uro:publicSurveySrcDescLod2>
						</uro:PublicSurveyDataQualityAttribute>
					</uro:publicSurveyDataQualityAttribute>
				</uro:DataQualityAttribute>
			</uro:tranDataQualityAttribute>
			<uro:roadStructureAttribute>
				<uro:RoadStructureAttribute>
					<uro:sectionType codeSpace="../../codelists/RoadStructureAttribute_sectionType.xml">4</uro:sectionType>
				</uro:RoadStructureAttribute>
			</uro:roadStructureAttribute>
		</tran:Road>
	</core:cityObjectMember>
</core:CityModel>
