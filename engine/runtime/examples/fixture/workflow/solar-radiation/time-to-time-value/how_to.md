# How To

Within engine project root folder:

```sh 
cargo run --package reearth-flow-cli run --workflow runtime/examples/fixture/workflow/solar-radiation/time-to-time-value/workflow.yml
```

## The input attributes 

- 年月日
- 出
- 方位[°]
- 南中
- 高度[°]
- 入り
- 方位[°]00

## The output attributes 

- 出
- 入り
- 方位_出
- 方位_入
- 年月日
- 南中
- 高度[°]
- 日射量[kWh/m2]

## Gradually add node to add new computed attributes 

- 出_H, hour part from attribute 出.
- 出_M, minutes part from attribute 出.
- 入り_H, hour part from attribute 入り
- 入り_M, minutes part from attribute 入り
- 南中_H, hour part from attribute 南中
- 南中_M, min part from attribute 南中
- 南中_S, seconds part from attribute 南中

## Overwrite existing attribute 

- overwrite attribute value for 出
- new value is computed as: 出_H * 3600 + 出_M * 60



```yml 
     - id: deeca816-d026-11f0-bcf9-7c70db10a7e3
        name: solar-radiation-accumulator
        type: action
        action: AttributeManager
        with:
          operations:
          # Calculate solar radiation using the time values
          - attribute: 日射量[kWh/m2]
            method: create
            expr: |
              # Parse the time strings to calculate solar radiation
              let input_str = env.get("入り");
              let output_str = env.get("出");

              # Since input_str and output_str are in format like "16:50", "7:06",
              # we need to parse them into time values in seconds
              # Split the time strings by colon
              let input_parts = string::split(input_str, ":");
              let output_parts = string::split(output_str, ":");

              # Extract hour and minute parts (assuming H:MM or HH:MM format)
              let input_hour = string::to_int(input_parts[0]);
              let input_min = string::to_int(input_parts[1]);
              let output_hour = string::to_int(output_parts[0]);
              let output_min = string::to_int(output_parts[1]);

              # Calculate total seconds from midnight
              let input_seconds = input_hour * 3600 + input_min * 60;
              let output_seconds = output_hour * 3600 + output_min * 60;

              # Calculate the day fraction
              let input_day_fraction = input_seconds / 86400.0;
              let output_day_fraction = output_seconds / 86400.0;

              # Get the elevation angle
              let hight = env.get("高度[°]");

              # Calculate solar radiation
              (input_day_fraction - output_day_fraction) * 24.0 * math::sin(math::to_radians(hight)) * (2.0/math::PI)
```



