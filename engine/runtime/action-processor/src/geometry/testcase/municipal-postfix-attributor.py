import json
import sys

# 入力ファイル名と出力ファイル名を指定
input_file = ""
output_file = ""

# コマンドライン引数を取得
if len(sys.argv) >= 3:
    input_file = sys.argv[1]
    output_file = sys.argv[2]
else:
    print("Usage: python fixtures-munic.py <input_file> <output_file>")
    sys.exit(1)

# JSONファイルを読み込む
with open(input_file, "r", encoding="utf-8") as f:
    data = json.load(f)

# 各フィーチャーに対して、名前の末尾が「市」「町」「村」であるかチェックし、新しいタグを追加
for feature in data.get("features", []):
    # ここでは、nameプロパティを使用（必要に応じて name:ja なども使える）
    name = feature.get("properties", {}).get("name", "")
    if name.endswith("市"):
        feature["properties"]["postfix"] = "市"
    elif name.endswith("町"):
        feature["properties"]["postfix"] = "町"
    elif name.endswith("村"):
        feature["properties"]["postfix"] = "村"
    else:
        feature["properties"]["postfix"] = "不明"

# 名前の末尾によってフィーチャーをソート
data["features"] = sorted(data["features"], key=lambda x: x["properties"]["postfix"])

# 結果を新しいファイルに書き出す
with open(output_file, "x", encoding="utf-8") as f:
    json.dump(data, f, ensure_ascii=False, indent=2)

print(f"処理が完了しました。結果は {output_file} に保存されました。")