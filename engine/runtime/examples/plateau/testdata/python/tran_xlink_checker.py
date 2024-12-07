import sys
import json
import xml.etree.ElementTree as ET

XMLNS = {
    "core": "http://www.opengis.net/citygml/2.0",
    "gml": "http://www.opengis.net/gml",
    "tran": "http://www.opengis.net/citygml/transportation/2.0",
    "xlink": "http://www.w3.org/1999/xlink",
    "uro": "https://www.geospatial.jp/iur/uro/3.0",
    "urf": "https://www.geospatial.jp/iur/urf/3.0",
}


def process_attributes(attributes, xml_root):
    for road in xml_root.findall('*//tran:Road', XMLNS):
        gml_id = road.get('{http://www.opengis.net/gml}id')
        lod2_trf = []
        lod3_trf = []
        for x in road.findall('.//tran:TrafficArea', XMLNS):
            for p in x.findall('./tran:lod2MultiSurface/gml:MultiSurface/gml:surfaceMember/*[@gml:id]', XMLNS):
                lod2_trf.append(p.get('{http://www.opengis.net/gml}id'))
            for p in x.findall('./tran:lod3MultiSurface/gml:MultiSurface/gml:surfaceMember/*[@gml:id]', XMLNS):
                lod3_trf.append(p.get('{http://www.opengis.net/gml}id'))

        lod2_aux = []
        lod3_aux = []
        for x in road.findall('.//tran:AuxiliaryTrafficArea', XMLNS):
            for p in x.findall('./tran:lod2MultiSurface/gml:MultiSurface/gml:surfaceMember/*[@gml:id]', XMLNS):
                lod2_aux.append(p.get('{http://www.opengis.net/gml}id'))
            for p in x.findall('./tran:lod3MultiSurface/gml:MultiSurface/gml:surfaceMember/*[@gml:id]', XMLNS):
                lod3_aux.append(p.get('{http://www.opengis.net/gml}id'))

        lod2xlinks = []
        lod3xlinks = []
        for x in road.findall('./tran:lod2MultiSurface//*[@xlink:href]', XMLNS):
            lod2xlinks.append(x.get('{http://www.w3.org/1999/xlink}href')[1:])
        for x in road.findall('./tran:lod3MultiSurface//*[@xlink:href]', XMLNS):
            lod3xlinks.append(x.get('{http://www.w3.org/1999/xlink}href')[1:])

        lod2_unref = set(lod2_trf + lod2_aux) - set(lod2xlinks)
        lod3_unref = set(lod3_trf + lod3_aux) - set(lod3xlinks)

        if lod2_trf:
            attributes.update({
                "glmId": {"String": gml_id},
                "featureType": {"String": "Road"},
                "lod": {"String": "2"},
                "unreferencedSurfaceNum": {"Number": len(lod2_unref)},
                "unreferencedIds": {"Array": [{"String": str(id)} for id in lod2_unref]},
            })

        if lod3_trf:
            attributes.update({
                "glmId": {"String": gml_id},
                "featureType": {"String": "Road"},
                "lod": {"String": "3"},
                "unreferencedSurfaceNum": {"Number": len(lod3_unref)},
                "unreferencedIds": {"Array": [{"String": str(id)} for id in lod3_unref]},
            })
    return attributes


def main():
    try:
        stdin_input = sys.stdin.read()  # xml
        xml_root = ET.fromstring(stdin_input)

        arg_input = sys.argv[1]  # attributes
        parsed_arg_input = json.loads(arg_input)

        updated_attributes = process_attributes(parsed_arg_input, xml_root)

        output = {"status": "success", "attributes": updated_attributes, "error": None}
    except Exception as e:
        output = {"status": "error", "attributes": None, "error": str(e)}

    print(json.dumps(output), file=sys.stdout)


if __name__ == "__main__":
    main()
