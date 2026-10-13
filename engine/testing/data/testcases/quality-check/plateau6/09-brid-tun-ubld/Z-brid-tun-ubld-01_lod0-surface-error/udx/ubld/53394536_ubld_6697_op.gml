<?xml version="1.0" encoding="UTF-8"?>
<!--
  LOD0 face errors.
  The error-free underground building with a core:lod0MultiSurface of four 5 m
  square faces, side by side from west to east and apart from each other:
    - a face wound counter-clockwise seen from above, which is valid;
    - a face wound clockwise, an incorrect orientation;
    - a bow tie whose two halves cross at its centre, a self-intersection;
    - a face with its north-east corner raised 1 m, a non-planar surface.

  The objectlist referenced by the test is a provisional plateau4-derived Excel; a
  CityGML 3.0 / i-UR 4.0 objectlist is not yet standardized.
-->
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
	xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
	xmlns:con="http://www.opengis.net/citygml/construction/3.0"
	xmlns:gml="http://www.opengis.net/gml/3.2"
	xmlns:uro="https://www.geospatial.jp/iur/uro/4.0"
	xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
	xmlns:xlink="http://www.w3.org/1999/xlink"
	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
	xsi:schemaLocation="http://www.opengis.net/citygml/3.0 http://schemas.opengis.net/citygml/3.0/core.xsd
http://www.opengis.net/citygml/building/3.0 http://schemas.opengis.net/citygml/building/3.0/building.xsd
http://www.opengis.net/citygml/construction/3.0 http://schemas.opengis.net/citygml/construction/3.0/construction.xsd
https://www.geospatial.jp/iur/uro/4.0 ../../schemas/iur/uro/4.0/urbanObject.xsd
https://www.geospatial.jp/iur/urc/4.0 ../../schemas/iur/urc/4.0/urbanCore.xsd">
	<gml:boundedBy>
		<gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
			<gml:lowerCorner>35.69069941674934 139.69803124047718 15.321</gml:lowerCorner>
			<gml:upperCorner>35.69501377379006 139.7036188035924 45.493</gml:upperCorner>
		</gml:Envelope>
	</gml:boundedBy>
	<core:cityObjectMember>
		<uro:UndergroundBuilding gml:id="ubld_f5e8c2f0-eb17-4362-9760-f807cdefe839">
			<gml:name>新宿駅周辺エリア</gml:name>
			<core:creationDate>2024-03-15T00:00:00</core:creationDate>
			<core:adeOfAbstractCityObject>
				<urc:DataQualityAttribute>
					<urc:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod1>
					<urc:geometrySrcDescLod3 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</urc:geometrySrcDescLod3>
					<urc:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</urc:thematicSrcDesc>
					<urc:lodType codeSpace="../../codelists/UndergroundBuilding_lodType.xml">3.0_interior</urc:lodType>
				</urc:DataQualityAttribute>
			</core:adeOfAbstractCityObject>
			<core:boundary>
				<con:CeilingSurface gml:id="bdry_2621036d-5ef8-4e67-b6b6-cd58e1ea0139">
					<core:lod3MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="face_645a0488-786d-4294-aac3-1463d9aeeefe">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69182965066727 139.6981758901787 35.539 35.69177300557311 139.6981812120156 35.539 35.691766489292235 139.69818176441947 35.539 35.69174164105228 139.6981840051287 35.539 35.69170854663349 139.69809915894908 35.539 35.691705850704274 139.6980502818663 35.539 35.6917308881297 139.69804795240358 35.539 35.69173081430672 139.698046427739 35.539 35.691883824069635 139.69803243517578 35.539 35.691891577986354 139.698170747259 35.539 35.69182965066727 139.6981758901787 35.539</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod3MultiSurface>
				</con:CeilingSurface>
			</core:boundary>
			<core:boundary>
				<core:ClosureSurface gml:id="bdry_299b0d5d-af37-4dd5-b983-ba8077b3486e">
					<core:lod3MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="face_779a8ec2-e176-41c2-abef-85c890ed56ea">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69177300557311 139.6981812120156 35.539 35.69177300557311 139.6981812120156 32.584 35.69182965066727 139.6981758901787 32.584 35.69182965066727 139.6981758901787 35.539 35.69177300557311 139.6981812120156 35.539</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod3MultiSurface>
				</core:ClosureSurface>
			</core:boundary>
			<core:boundary>
				<core:ClosureSurface gml:id="bdry_62f5caf9-f37c-4534-afd7-1249d9577eb2">
					<core:lod3MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="face_52d48daa-8b76-4272-8f4a-c97488ff5052">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69204610685156 139.6987402406507 35.539 35.69204610685156 139.6987402406507 32.584 35.692066982850015 139.69869213031873 32.584 35.692066982850015 139.69869213031873 35.539 35.69204610685156 139.6987402406507 35.539</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod3MultiSurface>
				</core:ClosureSurface>
			</core:boundary>
			<core:boundary>
				<core:ClosureSurface gml:id="bdry_95f8d805-52d0-4a96-bbd3-6322f5a6fa36">
					<core:lod3MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="face_39f4c820-00a1-4133-a933-aea07d30ee0f">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.6920341212962 139.6987506802194 34.828 35.69203781556572 139.69874141475051 34.828 35.69203781556572 139.69874141475051 32.584 35.6920341212962 139.6987506802194 32.584 35.6920341212962 139.6987506802194 34.828</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod3MultiSurface>
				</core:ClosureSurface>
			</core:boundary>
			<core:boundary>
				<core:ClosureSurface gml:id="bdry_7f8cf8c6-b65a-4f3f-81c7-005164826a09">
					<core:lod3MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="face_516c03c7-9236-4eb4-b0e5-70699c3defa2">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69200176329444 139.6987114103512 35.539 35.69200176329444 139.6987114103512 32.584 35.69201232924445 139.69865669887704 32.584 35.69201232924445 139.69865669887704 35.539 35.69200176329444 139.6987114103512 35.539</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod3MultiSurface>
				</core:ClosureSurface>
			</core:boundary>
			<core:boundary>
				<core:ClosureSurface gml:id="bdry_02a861cd-d258-4b76-98eb-3e38286e3626">
					<core:lod3MultiSurface>
						<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:surfaceMember>
								<gml:Polygon gml:id="face_ea4adeef-b783-4318-99fd-73fda4215054">
									<gml:exterior>
										<gml:LinearRing>
											<gml:posList>35.69198235946176 139.69877827976205 34.804 35.691999158411974 139.698775975405 34.804 35.691999158411974 139.698775975405 32.584 35.69198235946176 139.69877827976205 32.584 35.69198235946176 139.69877827976205 34.804</gml:posList>
										</gml:LinearRing>
									</gml:exterior>
								</gml:Polygon>
							</gml:surfaceMember>
						</gml:MultiSurface>
					</core:lod3MultiSurface>
				</core:ClosureSurface>
			</core:boundary>
			<core:lod0MultiSurface>
				<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69100 139.69900 20 35.69100 139.69905 20 35.69105 139.69905 20 35.69105 139.69900 20 35.69100 139.69900 20</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69100 139.69910 20 35.69105 139.69910 20 35.69105 139.69915 20 35.69100 139.69915 20 35.69100 139.69910 20</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69100 139.69920 20 35.69105 139.69925 20 35.69100 139.69925 20 35.69105 139.69920 20 35.69100 139.69920 20</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
					<gml:surfaceMember>
						<gml:Polygon>
							<gml:exterior>
								<gml:LinearRing>
									<gml:posList>35.69100 139.69930 20 35.69100 139.69935 20 35.69105 139.69935 21 35.69105 139.69930 20 35.69100 139.69930 20</gml:posList>
								</gml:LinearRing>
							</gml:exterior>
						</gml:Polygon>
					</gml:surfaceMember>
				</gml:MultiSurface>
			</core:lod0MultiSurface>
			<bldg:buildingRoom>
				<bldg:BuildingRoom gml:id="room_a2facd4a-6d69-4c38-8698-556eaf47db76">
					<core:lod3Solid>
						<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:exterior>
								<gml:Shell>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69174164105228 139.6981840051287 32.584 35.691766489292235 139.69818176441947 32.584 35.691766489292235 139.69818176441947 35.539 35.69174164105228 139.6981840051287 35.539 35.69174164105228 139.6981840051287 32.584</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.691891577986354 139.698170747259 35.539 35.691883824069635 139.69803243517578 35.539 35.69173081430672 139.698046427739 35.539 35.6917308881297 139.69804795240358 35.539 35.691705850704274 139.6980502818663 35.539 35.69170854663349 139.69809915894908 35.539 35.69174164105228 139.6981840051287 35.539 35.691766489292235 139.69818176441947 35.539 35.69177300557311 139.6981812120156 35.539 35.69182965066727 139.6981758901787 35.539 35.691891577986354 139.698170747259 35.539</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.691766489292235 139.69818176441947 32.584 35.69177300557311 139.6981812120156 32.584 35.69177300557311 139.6981812120156 35.539 35.691766489292235 139.69818176441947 35.539 35.691766489292235 139.69818176441947 32.584</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.691891577986354 139.698170747259 32.584 35.69182965066727 139.6981758901787 32.584 35.69177300557311 139.6981812120156 32.584 35.691766489292235 139.69818176441947 32.584 35.69174164105228 139.6981840051287 32.584 35.69170854663349 139.69809915894908 32.584 35.691705850704274 139.6980502818663 32.584 35.6917308881297 139.69804795240358 32.584 35.69173081430672 139.698046427739 32.584 35.691883824069635 139.69803243517578 32.584 35.691891577986354 139.698170747259 32.584</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69170854663349 139.69809915894908 35.539 35.69170854663349 139.69809915894908 32.584 35.69174164105228 139.6981840051287 32.584 35.69174164105228 139.6981840051287 35.539 35.69170854663349 139.69809915894908 35.539</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.691705850704274 139.6980502818663 35.539 35.691705850704274 139.6980502818663 32.584 35.69170854663349 139.69809915894908 32.584 35.69170854663349 139.69809915894908 35.539 35.691705850704274 139.6980502818663 35.539</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.6917308881297 139.69804795240358 35.539 35.6917308881297 139.69804795240358 32.584 35.691705850704274 139.6980502818663 32.584 35.691705850704274 139.6980502818663 35.539 35.6917308881297 139.69804795240358 35.539</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69182965066727 139.6981758901787 32.584 35.691891577986354 139.698170747259 32.584 35.691891577986354 139.698170747259 35.539 35.69182965066727 139.6981758901787 35.539 35.69182965066727 139.6981758901787 32.584</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69177300557311 139.6981812120156 32.584 35.69182965066727 139.6981758901787 32.584 35.69182965066727 139.6981758901787 35.539 35.69177300557311 139.6981812120156 35.539 35.69177300557311 139.6981812120156 32.584</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69173081430672 139.698046427739 35.539 35.69173081430672 139.698046427739 32.584 35.6917308881297 139.69804795240358 32.584 35.6917308881297 139.69804795240358 35.539 35.69173081430672 139.698046427739 35.539</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.691883824069635 139.69803243517578 35.539 35.691883824069635 139.69803243517578 32.584 35.69173081430672 139.698046427739 32.584 35.69173081430672 139.698046427739 35.539 35.691883824069635 139.69803243517578 35.539</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.691891577986354 139.698170747259 35.539 35.691891577986354 139.698170747259 32.584 35.691883824069635 139.69803243517578 32.584 35.691883824069635 139.69803243517578 35.539 35.691891577986354 139.698170747259 35.539</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:Shell>
							</gml:exterior>
						</gml:Solid>
					</core:lod3Solid>
				</bldg:BuildingRoom>
			</bldg:buildingRoom>
			<bldg:buildingRoom>
				<bldg:BuildingRoom gml:id="room_d6748b54-10d2-476c-9c72-425f626f7401">
					<core:lod3Solid>
						<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
							<gml:exterior>
								<gml:Shell>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380506578057 139.70118082347412 27.189 35.69381536726927 139.70113871899065 27.189 35.69381536726927 139.70113871899065 27.324 35.69380506578057 139.70118082347412 27.324 35.69380506578057 139.70118082347412 27.189</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69382440325686 139.70117544362205 27.189 35.693824932479885 139.7011730892083 27.189 35.69383139047944 139.7011440736495 27.189 35.69381536726927 139.70113871899065 27.189 35.69380506578057 139.70118082347412 27.189 35.693821963558015 139.70118639768774 27.189 35.69382263627228 139.70118338006824 27.189 35.69382440325686 139.70117544362205 27.189</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693821963558015 139.70118639768774 25.038 35.69382263627228 139.70118338006824 25.038 35.69382263627228 139.70118338006824 27.189 35.693821963558015 139.70118639768774 27.189 35.693821963558015 139.70118639768774 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380411900502 139.70118051565046 25.271 35.69380411900502 139.70118051565046 25.038 35.693821963558015 139.70118639768774 25.038 35.693821963558015 139.70118639768774 27.189 35.69380411900502 139.70118051565046 25.271</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380506578057 139.70118082347412 27.189 35.69380411900502 139.70118051565046 25.271 35.693821963558015 139.70118639768774 27.189 35.69380506578057 139.70118082347412 27.189</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380242378986 139.7011881425906 26.023 35.693801553732825 139.701192033444 26.023 35.69380411900502 139.70118051565046 25.271 35.69380242378986 139.7011881425906 26.023</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693801553732825 139.701192033444 26.023 35.69379621565996 139.70119024118367 26.024 35.69380411900502 139.70118051565046 25.271 35.693801553732825 139.701192033444 26.023</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693801553732825 139.701192033444 26.023 35.69380242378986 139.7011881425906 26.023 35.69379621565996 139.70119024118367 26.024 35.693801553732825 139.701192033444 26.023</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380242378986 139.7011881425906 26.023 35.69378879905619 139.70118354638245 26.026 35.69379621565996 139.70119024118367 26.024 35.69380242378986 139.7011881425906 26.023</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380242378986 139.7011881425906 27.323 35.69378879905619 139.70118354638245 26.026 35.69380242378986 139.7011881425906 26.023 35.69380242378986 139.7011881425906 27.323</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380242378986 139.7011881425906 26.023 35.69380411900502 139.70118051565046 25.271 35.69380411900502 139.70118051565046 27.324 35.69380242378986 139.7011881425906 27.323 35.69380242378986 139.7011881425906 26.023</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380411900502 139.70118051565046 27.324 35.69380411900502 139.70118051565046 25.271 35.69380506578057 139.70118082347412 27.189 35.69380506578057 139.70118082347412 27.324 35.69380411900502 139.70118051565046 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381544340524 139.70113417752657 27.324 35.69381544340524 139.70113417752657 25.918 35.693794442801284 139.7011271515638 25.918 35.693794442801284 139.7011271515638 27.324 35.69381544340524 139.70113417752657 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381616994541 139.7011309277789 25.918 35.69381544340524 139.70113417752657 25.918 35.69381616994541 139.7011309277789 25.038 35.69381616994541 139.7011309277789 25.918</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381616994541 139.7011309277789 25.918 35.69379516934128 139.7011239018169 25.918 35.693794442801284 139.7011271515638 25.918 35.69381544340524 139.70113417752657 25.918 35.69381616994541 139.7011309277789 25.918</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379516934128 139.7011239018169 27.324 35.69379341101928 139.7011233080468 27.324 35.693778884898705 139.70118436934933 27.324 35.693794442801284 139.7011271515638 27.324 35.69379516934128 139.7011239018169 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381544340524 139.70113417752657 27.324 35.693794442801284 139.7011271515638 27.324 35.69380411900502 139.70118051565046 27.324 35.69381544340524 139.70113417752657 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381536726927 139.70113871899065 27.324 35.69381450164096 139.7011384331325 27.324 35.69381544340524 139.70113417752657 27.324 35.69380411900502 139.70118051565046 27.324 35.69380506578057 139.70118082347412 27.324 35.69381536726927 139.70113871899065 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381450164096 139.7011384331325 27.324 35.69381450164096 139.7011384331325 25.038 35.69381544340524 139.70113417752657 25.918 35.69381544340524 139.70113417752657 27.324 35.69381450164096 139.7011384331325 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381536726927 139.70113871899065 27.189 35.69381450164096 139.7011384331325 25.038 35.69381450164096 139.7011384331325 27.324 35.69381536726927 139.70113871899065 27.324 35.69381536726927 139.70113871899065 27.189</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69383139047944 139.7011440736495 27.189 35.69383139047944 139.7011440736495 25.038 35.69381536726927 139.70113871899065 27.189 35.69383139047944 139.7011440736495 27.189</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693824932479885 139.7011730892083 25.038 35.69383139047944 139.7011440736495 25.038 35.69383139047944 139.7011440736495 27.189 35.693824932479885 139.7011730892083 27.189 35.693824932479885 139.7011730892083 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69382263627228 139.70118338006824 25.038 35.69382440325686 139.70117544362205 25.038 35.69382440325686 139.70117544362205 27.189 35.69382263627228 139.70118338006824 27.189 35.69382263627228 139.70118338006824 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374707819925 139.70117033373694 25.038 35.69373980148217 139.70116790380817 25.038 35.6937420803527 139.7011582428071 25.038 35.69374707819925 139.70117033373694 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.6937420803527 139.7011582428071 25.038 35.69375755040413 139.70111132330067 25.038 35.69377482697769 139.70111709576176 25.038 35.693785890847124 139.70112080117633 25.038 35.6937420803527 139.7011582428071 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69375755040413 139.70111132330067 25.038 35.6937420803527 139.7011582428071 25.038 35.6937534296374 139.7011099489176 25.038 35.69375755040413 139.70111132330067 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69382440325686 139.70117544362205 25.038 35.69379878093198 139.70117872339085 25.038 35.693785890847124 139.70112080117633 25.038 35.69382440325686 139.70117544362205 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381450164096 139.7011384331325 25.038 35.69382440325686 139.70117544362205 25.038 35.693785890847124 139.70112080117633 25.038 35.69381450164096 139.7011384331325 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379341101928 139.7011233080468 25.038 35.69381450164096 139.7011384331325 25.038 35.693785890847124 139.70112080117633 25.038 35.69379341101928 139.7011233080468 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381616994541 139.7011309277789 25.038 35.69381450164096 139.7011384331325 25.038 35.69379341101928 139.7011233080468 25.038 35.69381616994541 139.7011309277789 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693824932479885 139.7011730892083 25.038 35.69382440325686 139.70117544362205 25.038 35.69381450164096 139.7011384331325 25.038 35.693824932479885 139.7011730892083 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69383139047944 139.7011440736495 25.038 35.693824932479885 139.7011730892083 25.038 35.69381450164096 139.7011384331325 25.038 35.69383139047944 139.7011440736495 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69382263627228 139.70118338006824 25.038 35.69379878093198 139.70117872339085 25.038 35.69382440325686 139.70117544362205 25.038 35.69382263627228 139.70118338006824 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380411900502 139.70118051565046 25.038 35.69379878093198 139.70117872339085 25.038 35.69382263627228 139.70118338006824 25.038 35.693821963558015 139.70118639768774 25.038 35.69380411900502 139.70118051565046 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379878093198 139.70117872339085 25.271 35.69379878093198 139.70117872339085 25.038 35.69380411900502 139.70118051565046 25.038 35.69380411900502 139.70118051565046 25.271 35.69379878093198 139.70117872339085 25.271</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380411900502 139.70118051565046 25.271 35.69379621565996 139.70119024118367 26.024 35.69379878093198 139.70117872339085 25.271 35.69380411900502 139.70118051565046 25.271</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379621565996 139.70119024118367 25.038 35.69379878093198 139.70117872339085 25.038 35.69379878093198 139.70117872339085 25.271 35.69379621565996 139.70119024118367 26.024 35.69379621565996 139.70119024118367 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379621565996 139.70119024118367 26.024 35.69378792899929 139.70118743723523 26.026 35.69379621565996 139.70119024118367 25.038 35.69379621565996 139.70119024118367 26.024</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379621565996 139.70119024118367 26.024 35.69378879905619 139.70118354638245 26.026 35.69378792899929 139.70118743723523 26.026 35.69379621565996 139.70119024118367 26.024</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69378792899929 139.70118743723523 26.026 35.69378879905619 139.70118354638245 26.026 35.693788799044064 139.701183535333 27.323 35.69378792898716 139.70118742618578 27.324 35.69378792899929 139.70118743723523 26.026</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380242378986 139.7011881425906 27.323 35.693788799044064 139.701183535333 27.323 35.69378879905619 139.70118354638245 26.026 35.69380242378986 139.7011881425906 27.323</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380242378986 139.7011881425906 27.323 35.69380411900502 139.70118051565046 27.324 35.693788799044064 139.701183535333 27.323 35.69380242378986 139.7011881425906 27.323</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69380411900502 139.70118051565046 27.324 35.693794442801284 139.7011271515638 27.324 35.693788799044064 139.701183535333 27.323 35.69380411900502 139.70118051565046 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693794442801284 139.7011271515638 27.324 35.693794442801284 139.7011271515638 25.918 35.69379516934128 139.7011239018169 25.918 35.69379516934128 139.7011239018169 27.324 35.693794442801284 139.7011271515638 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69375953414091 139.7011119829951 27.607 35.69375755040413 139.70111132330067 27.607 35.6937534296374 139.7011099489176 27.607 35.6937420803527 139.7011582428071 27.607 35.69373980148217 139.70116790380817 27.607 35.69374009002896 139.7011680027774 27.607 35.69374035152261 139.70116809074176 27.607 35.69375953414091 139.7011119829951 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374654619829 139.70117015782301 27.607 35.69375953414091 139.7011119829951 27.607 35.69374410259456 139.70116934419332 27.607 35.69374654619829 139.70117015782301 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374410259456 139.70116934419332 27.607 35.69375953414091 139.7011119829951 27.607 35.69374035152261 139.70116809074176 27.607 35.69374410259456 139.70116934419332 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374707819925 139.70117033373694 27.607 35.69375953414091 139.7011119829951 27.607 35.69374654619829 139.70117015782301 27.607 35.69374707819925 139.70117033373694 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693785890847124 139.70112080117633 27.607 35.69377482697769 139.70111709576176 27.607 35.69375953414091 139.7011119829951 27.607 35.69374707819925 139.70117033373694 27.607 35.6937463786141 139.70117350609067 27.607 35.69376109435476 139.7011784209297 27.607 35.69377254595596 139.70118225830623 27.607 35.693778884898705 139.70118436934933 27.607 35.69379341101928 139.7011233080468 27.607 35.693785890847124 139.70112080117633 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379341101928 139.7011233080468 27.324 35.69379516934128 139.7011239018169 25.918 35.69379341101928 139.7011233080468 25.038 35.69379341101928 139.7011233080468 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379516934128 139.7011239018169 27.324 35.69379516934128 139.7011239018169 25.918 35.69379341101928 139.7011233080468 27.324 35.69379516934128 139.7011239018169 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379341101928 139.7011233080468 27.324 35.69379341101928 139.7011233080468 25.038 35.693785890847124 139.70112080117633 25.038 35.693785890847124 139.70112080117633 27.607 35.69379341101928 139.7011233080468 27.607 35.69379341101928 139.7011233080468 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379516934128 139.7011239018169 25.918 35.69381616994541 139.7011309277789 25.918 35.69379341101928 139.7011233080468 25.038 35.69379516934128 139.7011239018169 25.918</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381616994541 139.7011309277789 25.918 35.69381616994541 139.7011309277789 25.038 35.69379341101928 139.7011233080468 25.038 35.69381616994541 139.7011309277789 25.918</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381544340524 139.70113417752657 25.918 35.69381450164096 139.7011384331325 25.038 35.69381616994541 139.7011309277789 25.038 35.69381544340524 139.70113417752657 25.918</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69381536726927 139.70113871899065 27.189 35.69383139047944 139.7011440736495 25.038 35.69381450164096 139.7011384331325 25.038 35.69381536726927 139.70113871899065 27.189</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69382440325686 139.70117544362205 25.038 35.693824932479885 139.7011730892083 25.038 35.693824932479885 139.7011730892083 27.189 35.69382440325686 139.70117544362205 27.189 35.69382440325686 139.70117544362205 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379621565996 139.70119024118367 25.038 35.693778884898705 139.70118436934933 25.082 35.69379878093198 139.70117872339085 25.038 35.69379621565996 139.70119024118367 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69378792899929 139.70118743723523 26.026 35.693778884898705 139.70118436934933 25.082 35.69379621565996 139.70119024118367 25.038 35.69378792899929 139.70118743723523 26.026</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693778884898705 139.70118436934933 27.324 35.69379341101928 139.7011233080468 27.324 35.69379341101928 139.7011233080468 27.607 35.693778884898705 139.70118436934933 27.607 35.693778884898705 139.70118436934933 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693778884898705 139.70118436934933 27.324 35.693778884898705 139.70118436934933 25.082 35.69378792899929 139.70118743723523 26.026 35.693778884898705 139.70118436934933 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69378792898716 139.70118742618578 27.324 35.693778884898705 139.70118436934933 27.324 35.69378792899929 139.70118743723523 26.026 35.69378792898716 139.70118742618578 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69378792898716 139.70118742618578 27.324 35.693788799044064 139.701183535333 27.323 35.693778884898705 139.70118436934933 27.324 35.69378792898716 139.70118742618578 27.324</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693788799044064 139.701183535333 27.323 35.693794442801284 139.7011271515638 27.324 35.693778884898705 139.70118436934933 27.324 35.693788799044064 139.701183535333 27.323</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69377254595596 139.70118225830623 25.082 35.693778884898705 139.70118436934933 25.082 35.693778884898705 139.70118436934933 27.324 35.693778884898705 139.70118436934933 27.607 35.69377254595596 139.70118225830623 27.607 35.69377254595596 139.70118225830623 25.082</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.6937463786141 139.70117350609067 27.607 35.6937463786141 139.70117350609067 25.038 35.69376107632741 139.70117842095942 25.063 35.6937463786141 139.70117350609067 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69376109435476 139.7011784209297 27.607 35.6937463786141 139.70117350609067 27.607 35.69376107632741 139.70117842095942 25.063 35.69376109435476 139.7011784209297 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693785890847124 139.70112080117633 25.038 35.69377482697769 139.70111709576176 25.038 35.69377482697769 139.70111709576176 27.607 35.693785890847124 139.70112080117633 27.607 35.693785890847124 139.70112080117633 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69376109435476 139.7011784209297 27.607 35.69376107632741 139.70117842095942 25.063 35.69377254595596 139.70118225830623 25.082 35.69376109435476 139.7011784209297 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69379878093198 139.70117872339085 25.038 35.69377254595596 139.70118225830623 25.082 35.693785890847124 139.70112080117633 25.038 35.69379878093198 139.70117872339085 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.693778884898705 139.70118436934933 25.082 35.69377254595596 139.70118225830623 25.082 35.69379878093198 139.70117872339085 25.038 35.693778884898705 139.70118436934933 25.082</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69377254595596 139.70118225830623 27.607 35.69376109435476 139.7011784209297 27.607 35.69377254595596 139.70118225830623 25.082 35.69377254595596 139.70118225830623 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69377254595596 139.70118225830623 25.082 35.69376107632741 139.70117842095942 25.063 35.69374707819925 139.70117033373694 25.038 35.69377254595596 139.70118225830623 25.082</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69376107632741 139.70117842095942 25.063 35.6937463786141 139.70117350609067 25.038 35.69374707819925 139.70117033373694 25.038 35.69376107632741 139.70117842095942 25.063</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374707819925 139.70117033373694 27.607 35.69374707819925 139.70117033373694 25.038 35.6937463786141 139.70117350609067 25.038 35.6937463786141 139.70117350609067 27.607 35.69374707819925 139.70117033373694 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374707819925 139.70117033373694 27.607 35.69374654619829 139.70117015782301 27.607 35.69374707819925 139.70117033373694 25.038 35.69374707819925 139.70117033373694 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374654619829 139.70117015782301 27.607 35.69374410259456 139.70116934419332 27.607 35.69374707819925 139.70117033373694 25.038 35.69374654619829 139.70117015782301 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374410259456 139.70116934419332 27.607 35.69373980148217 139.70116790380817 25.038 35.69374707819925 139.70117033373694 25.038 35.69374410259456 139.70116934419332 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374035152261 139.70116809074176 27.607 35.69373980148217 139.70116790380817 25.038 35.69374410259456 139.70116934419332 27.607 35.69374035152261 139.70116809074176 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69374009002896 139.7011680027774 27.607 35.69373980148217 139.70116790380817 25.038 35.69374035152261 139.70116809074176 27.607 35.69374009002896 139.7011680027774 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69373980148217 139.70116790380817 27.607 35.69373980148217 139.70116790380817 25.038 35.69374009002896 139.7011680027774 27.607 35.69373980148217 139.70116790380817 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69377482697769 139.70111709576176 27.607 35.69377482697769 139.70111709576176 25.038 35.69375953414091 139.7011119829951 27.607 35.69377482697769 139.70111709576176 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.6937420803527 139.7011582428071 27.607 35.6937420803527 139.7011582428071 25.082 35.6937420803527 139.7011582428071 25.038 35.69373980148217 139.70116790380817 25.038 35.69373980148217 139.70116790380817 27.607 35.6937420803527 139.7011582428071 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69377254595596 139.70118225830623 25.082 35.6937420803527 139.7011582428071 25.038 35.693785890847124 139.70112080117633 25.038 35.69377254595596 139.70118225830623 25.082</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69377254595596 139.70118225830623 25.082 35.69374707819925 139.70117033373694 25.038 35.6937420803527 139.7011582428071 25.038 35.69377254595596 139.70118225830623 25.082</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69375755040413 139.70111132330067 27.607 35.69375953414091 139.7011119829951 27.607 35.69375755040413 139.70111132330067 25.038 35.69375755040413 139.70111132330067 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69375953414091 139.7011119829951 27.607 35.69377482697769 139.70111709576176 25.038 35.69375755040413 139.70111132330067 25.038 35.69375953414091 139.7011119829951 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.69375755040413 139.70111132330067 25.038 35.6937534296374 139.7011099489176 25.038 35.6937534296374 139.7011099489176 25.082 35.6937534296374 139.7011099489176 27.607 35.69375755040413 139.70111132330067 27.607 35.69375755040413 139.70111132330067 25.038</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.6937534296374 139.7011099489176 27.607 35.6937534296374 139.7011099489176 25.082 35.6937420803527 139.7011582428071 25.082 35.6937420803527 139.7011582428071 27.607 35.6937534296374 139.7011099489176 27.607</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
									<gml:surfaceMember>
										<gml:Polygon>
											<gml:exterior>
												<gml:LinearRing>
													<gml:posList>35.6937534296374 139.7011099489176 25.082 35.6937534296374 139.7011099489176 25.038 35.6937420803527 139.7011582428071 25.038 35.6937420803527 139.7011582428071 25.082 35.6937534296374 139.7011099489176 25.082</gml:posList>
												</gml:LinearRing>
											</gml:exterior>
										</gml:Polygon>
									</gml:surfaceMember>
								</gml:Shell>
							</gml:exterior>
						</gml:Solid>
					</core:lod3Solid>
				</bldg:BuildingRoom>
			</bldg:buildingRoom>
		</uro:UndergroundBuilding>
	</core:cityObjectMember>
</core:CityModel>
