# DMM Logger

Connects via TCP using LXI/SCPI to a digital multimeter and log measurements from DMM into CSV files.

## Options

- `-U, --voltage <RANGE>`: Measures voltage
  - `--dc | --ac`: Selects DC or AC measurements, DC is default
- `-R, --resistance <RANGE>`: Measures resistance
  - `-2, | -4`: Selects between 2-wire-measurements or 4-wire-measurements. Default is 2-wire-measurements.
- `-I, --current <RANGE>`: Measures current
  - `--dc | --ac`: Selects DC or AC measurements, DC is default

## CSV-Format

Example:

```
# File created by DMM Logger (1.2.3) on 2022-12-18 15:18:42 (CET)
#
# Measurement: Voltage, DC
# Range: 10 Volt
#
sequence,date,time,moment,latency,voltage
0,2022-12-18,15:18:42.123,0,0.033,1.23456
1,2022-12-18,15:18:42.165,0.042,0.031,1.32109
2,2022-12-18,15:18:42.201,0.078,0.034,1.317643
3,2022-12-18,15:18:42.239,0.116,0.032,1.320016
...
```
