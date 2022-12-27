# DMM Logger

Connects via TCP to a digital multimeter and log measurements from DMM using SCPI into CSV files.

The goal was to have an easy-to-use tool for measurements over a longer period of time that logs the measurements in a CSV file.

Comments are added into the CSV file so that the CSV file can be understood by someone else without problems or maybe yourself a few months later. You can also add your own comment, e.g. for a short description of what was measured.

## Install from source code

DMM logger is written in Rust. Currently you have to install using `cargo`:

```console
cargo install --git https://github.com/mrothNET/dmm-logger
```

## Usage

Basic usage is pretty simple. The settings of the instrument remain unchanged and one measurement per second is performed and written to the CSV file:

```console
dmm-logger 10.1.2.3 example.csv
```

Where `10.1.2.3` should be replaced with the IP address or hostname of your intrument.

The resulting CSV file can be plotted in python notebook with:

```python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('example.csv', comment='#', parse_dates=[['date', 'time']])
df["reading"].plot()
```

### Command line arguments

Usage: `dmm-logger [OPTIONS] <HOST> [FILE]`

#### Arguments

`<HOST>`
: Network name or IP address of the instrument.

`[FILE]`
: Filename to save the CSV lines into. If omitted, lines are written to stdout.

#### Options

`--interval <SECONDS>` | `--rate <HERTZ>`
: Set the sampling interval respectively the sampling rate. Default interval is 1 second or 1 hertz.

`-n <COUNT>`
: Restricts the number of samples to take. Default is unlimited until you hit `CTRL-C`.

`--display-off` | `--display-text [<TEXT>]`
: Switch off instruments display or displays a text message on instrument during logging. Makes most instruments faster.

`--drop-slow-samples`
: Drop delayed samples or samples with high latency. Helps with fast sampling because lack of realtime behaviour.

`-U, --voltage <RANGE>` | `-I, --current <RANGE>` | `-R, --resistance <RANGE>`
: Configures instrument for voltage, current or resistant measurement.

`--DC` | `--AC`
: Selects between DC- or AC-mode for voltage or current measurement.

`-2, --two-wire` | `-4, --four-wire`
: Selects between 2-wire or 4-wire resistance measurement.

`--resolution <VALUE>`
: Resolution in units as the measurement function. For example `0.001`.

`--nplc <NPLC>`
: Integration time in number of power line cycles. Typical integration cycles are 10 or 1.

`-m, --message <TEXT>` | `--message-from <FILE>`
: Add a custom message to the CSV file as comment. Use this to add a short reminder into the CSV file to help you recall the experiment a few month later.

`--beep`
: Beep instrument when logging finished.

`--reset`
: Performs instrument reset before logging. Helps to start with a known state of all instrument settings.

`--port`
: <PORT> Network port for SCPI. Most instruments use the default port 5025.

`--debug`
: Print SCPI communication to stderr. If you have problems with an instrument working together with DMM logger this will help to sort out the issues.

## CSV file Format

The creates CSV file contains 7 columns:

`sequence`
: Sequential sample number starting at 0

`date`
: Local date of measurement: YEAR-MONTH-DAY

`time`
: Local clock time of measurement: HOURS:MINUTES:SECONDS.MILLISECONDS

`moment`
: Time in seconds since first measurement

`delay`
: Delay of the measurement in seconds, caused by non-real-time behavior or fast logging rate

`latency`
: Measurement duration in seconds including network roundtrip time

`reading`
: Measured value returned from instrument

### Example CSV file

The following excerpt was created by:

```
dmm-logger -R 10 -m "Example measurement of an 11 Ohms resistor" -n 10 10.1.2.3 example.csv
```

The contents of the resulting CSV file are (redacted):

```
# File created by DMM logger (1.0.0) on 2022-12-27 18:04:28 (+01:00)
#
# ------------------------------------------
# Example measurement of an 11 Ohms resistor
# ------------------------------------------
#
# Settings
# --------
# Sampling interval   : 1 seconds
# Sample rate         : 1 Hz
# Resistance (2-wire) : 10 Ohms
#
# Instrument
# ----------
# Manufacturer        : Keysight Technologies
# Model               : 34461A
# Serial              : xxxxxxxxxx
# Firmware            : A.02.14-02.40-02.14-00.49-01-01
#
# Fields
# ------
# sequence            : Sequential sample number starting at 0
# date                : Local date of measurement: YEAR-MONTH-DAY
# time                : Local clock time of measurement: HOURS:MINUTES:SECONDS.MILLISECONDS
# moment              : Time in seconds since first measurement
# delay               : Delay of the measurement in seconds, caused by non-real-time behavior or fast logging rate
# latency             : Measurement duration in seconds including network roundtrip time
# reading             : Measured value returned from instrument
#
sequence,date,time,moment,delay,latency,reading
0,2022-12-27,18:04:28.477,0.0000,0.0000,0.4077,11.0293809
1,2022-12-27,18:04:29.477,1.0002,0.0002,0.4076,11.0294049
2,2022-12-27,18:04:30.477,2.0002,0.0002,0.4076,11.0293892
3,2022-12-27,18:04:31.477,3.0001,0.0001,0.4075,11.0293611
4,2022-12-27,18:04:32.477,4.0004,0.0004,0.4078,11.0293718
5,2022-12-27,18:04:33.477,5.0005,0.0005,0.4075,11.0294192
6,2022-12-27,18:04:34.477,6.0004,0.0004,0.4080,11.0294847
7,2022-12-27,18:04:35.477,7.0001,0.0001,0.4078,11.0293944
8,2022-12-27,18:04:36.477,8.0004,0.0004,0.4077,11.0294834
9,2022-12-27,18:04:37.477,9.0004,0.0004,0.4075,11.0294869
```
