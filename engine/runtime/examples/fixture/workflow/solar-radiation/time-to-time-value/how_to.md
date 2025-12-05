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

## Compute solar Energy 

- create attribute 日射量[kWh/m2]
- (@Value(入り)/86400 - @Value(出)/86400) * 24 * @sin(@degToRad(@Value(高度[°]))) * (2/@pi())


```yml 
                let entry_time_value = env.get("__value")["入り"];
                let exit_time_value = env.get("__value")["出"];
                let altitude_value = env.get("__value")["高度[°]"];
                let input_time_days = entry_time_value / 86400.0;
                let output_time_days = exit_time_value / 86400.0;
                let altitude_radians = math::to_radians(altitude_value);
                (input_time_days - output_time_days) * 24.0 * math::sin(altitude_radians) * (2.0 / math::PI)
```



