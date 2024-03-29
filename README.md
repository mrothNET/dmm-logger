# DMM Logger

Connects via TCP to a digital multimeter and logs measurements from the DMM using SCPI into CSV files.

The goal is to provide an easy-to-use tool for taking measurements over a longer period of time and logging the results in a CSV file.

Comments are added to the CSV file, making it easily understandable by someone else or even by yourself after a few months. You can also add your own comments, such as a short description of what was measured.

## Install from source code

DMM Logger is written in Rust. To install it from the source code, you will first need to have Rust and its package manager `cargo` installed on your system. You can obtain Rust using the official Rust toolchain installer `rustup`.

Visit the `rustup` homepage at [https://rustup.rs](https://rustup.rs) and follow the instructions provided to install Rust and Cargo for your platform. Once Rust and Cargo are installed and configured, you can proceed with installing DMM Logger.

To install DMM Logger, run the following command in your terminal:

```console
cargo install --git https://github.com/mrothNET/dmm-logger
```

## Usage

Basic usage is straightforward. The instrument settings remain unchanged, and one measurement per second is performed and written to the CSV file:

```console
dmm-logger 10.1.2.3 example.csv
```

Replace `10.1.2.3` with the IP address or hostname of your instrument.

The resulting CSV file can be plotted in a Python notebook with:

```python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('example.csv', comment='#', parse_dates=[['date', 'time']])
df["reading"].plot()
```

### Command line arguments

Usage: `dmm-logger [OPTIONS] <HOST> [FILE]`

#### Arguments

<dl>
<dt><code>&lt;HOST&gt;</code></dt>
<dd>Network name or IP address of the instrument.</dd>

<dt><code>[FILE]</code></dt>
<dd>Filename to save the CSV lines into. If omitted, lines are written to stdout.</dd>
</dl>

#### Options

<dl>

<dt><code>--interval &lt;SECONDS&gt;</code> | <code>--rate &lt;HERTZ&gt;</code></dt>
<dd>Set the sampling interval respectively the sampling rate. Default interval is 1 second or 1 hertz.</dd>

<dt><code>-n &lt;COUNT&gt;</code></dt>
<dd>Restricts the number of samples to take. Default is unlimited until you hit `CTRL-C`</dd>

<dt><code>--display-off</code> | <code>--display-text [&lt;TEXT&gt;]</code></dt>
<dd>Switch off instruments display or displays a text message on instrument during logging. Makes most instruments faster.</dd>

<dt><code>--drop-slow-samples</code></dt>
<dd>Drop delayed samples or samples with high latency. Helps with fast sampling because lack of realtime behaviour.</dd>

<dt><code>-U, --voltage &lt;RANGE&gt;</code> | <code>-I, --current &lt;RANGE&gt;</code> | <code>-R, --resistance &lt;RANGE&gt;</code></dt>
<dd>Configures instrument for voltage, current or resistant measurement.</dd>

<dt><code>--DC</code> | <code>--AC</code></dt>
<dd>Selects between DC- or AC-mode for voltage or current measurement.</dd>

<dt><code>-2, --two-wire</code> | <code>-4, --four-wire</code></dt>
<dd>Selects between 2-wire or 4-wire resistance measurement.</dd>

<dt><code>--resolution &lt;VALUE&gt;</code></dt>
<dd>Resolution in units as the measurement function. For example `0.001`.</dd>

<dt><code>--nplc &lt;NPLC&gt;</code></dt>
<dd>Integration time in number of power line cycles. Typical integration cycles are 10 or 1.</dd>

<dt><code>-m, --message &lt;TEXT&gt;</code> | <code>--message-from &lt;FILE&gt;</code></dt>
<dd>Add a custom message to the CSV file as comment. Use this to add a short reminder into the CSV file to help you recall the experiment a few month later.</dd>

<dt><code>--beep</code></dt>
<dd>Beep instrument when logging finished.</dd>

<dt><code>--reset</code></dt>
<dd>Performs instrument reset before logging. Helps to start with a known state of all instrument settings.</dd>

<dt><code>--port <PORT></code></dt>
<dd><PORT> Network port for SCPI. Most instruments use the default port 5025.</dd>

<dt><code>--debug</code></dt>
<dd>Print SCPI communication to stderr. If you have problems with an instrument working together with DMM logger this will help to sort out the issues.</dd>

</dl>

## CSV file Format

The created CSV file contains 7 columns:

<dl>

<dt><code>sequence</code></dt>
<dd>Sequential sample number starting at 0</dd>

<dt><code>date</code></dt>
<dd>Local date of measurement: YEAR-MONTH-DAY</dd>

<dt><code>time</code></dt>
<dd>Local clock time of measurement: HOURS:MINUTES:SECONDS.MILLISECONDS</dd>

<dt><code>moment</code></dt>
<dd>Time in seconds since first measurement</dd>

<dt><code>delay</code></dt>
<dd>Delay of the measurement in seconds, caused by non-real-time behavior or fast logging rate</dd>

<dt><code>latency</code></dt>
<dd>Measurement duration in seconds including network roundtrip time</dd>

<dt><code>reading</code></dt>
<dd>Measured value returned from instrument</dd>

</dl>

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
