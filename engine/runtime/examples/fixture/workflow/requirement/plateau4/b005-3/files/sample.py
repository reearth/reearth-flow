import sys
import json


def process_attributes(attributes):
    updated_attributes = {}
    for key, value in attributes.items():
        if isinstance(value, dict) and "String" in value:
            updated_attributes[key] = {"String": value["String"] + "_modified"}
        else:
            updated_attributes[key] = value
    return updated_attributes


def main():
    try:
        input_data = sys.argv[1]
        parsed_data = json.loads(input_data)

        updated_attributes = process_attributes(parsed_data)

        output = {"status": "success", "attributes": updated_attributes, "error": None}
    except Exception as e:
        output = {"status": "error", "attributes": None, "error": str(e)}

    print(json.dumps(output), file=sys.stdout)


if __name__ == "__main__":
    main()
