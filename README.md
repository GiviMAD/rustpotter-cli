# Rustpotter CLI

CLI for Rustpotter, an open source wakeword spotter forged in rust

<div align="center">
    <img src="./logo.png?raw=true" width="400px"</img> 
</div>

## Description

This is a client for using the [rustpotter](https://github.com/GiviMAD/rustpotter) library on the command line.

You can use it to record wav samples, build rustpotter models and tests them.

# Installation

You can download rustpotter-cli binaries for the supported platforms from the 'Assets' tab of the [releases](https://github.com/GiviMAD/rustpotter-cli/releases), it can be installed like any other executable.

For instance on debian it can be installed like:

```bash
# This command print your arch, just in case you don't remember it.
$ uname -m
# Here I used the armv7l binary
$ curl -OL https://github.com/GiviMAD/rustpotter-cli/releases/download/v2.0.5/rustpotter-cli_debian_armv7l
# Make executable
$ chmod +x rustpotter-cli_debian_armv7l
# Check simple execution
$ ./rustpotter-cli_debian_armv7l --version
# Make available as ruspotter-cli
$ sudo mv ./rustpotter-cli_debian_armv7l /usr/local/bin/rustpotter-cli
```

# How to used it.

## Listing available audio input devices and formats.

Your can list the available audio sources with the `devices` command,
the `--configs` option can be added to display the default and available record formats for each source.

Host warnings are hidden by default, you can enable them by providing the `--host-warnings` option.

Every device and config has a numerical id to the left which is the one you can use on the other commands (`record` and `spot`)
to change its audio source and format.

In some systems to many configurations are displayed you can filter them by max channel number using the parameter `--max-channels`.

This is an example run on macOS:

```bash
$ rustpotter-cli devices -c -m 1
Audio hosts:
  - CoreAudio
Default input device:
  - MacBook Pro Microphone
Available Devices: 
0 - MacBook Pro Microphone
  Default input stream config:
      - Sample Rate: 48000, Channels: 1, Format: f32, Supported: true
  All supported input stream configs:
    0 - Sample Rate: 44100, Channels: 1, Format: f32, Supported: true
    1 - Sample Rate: 48000, Channels: 1, Format: f32, Supported: true
    2 - Sample Rate: 88200, Channels: 1, Format: f32, Supported: true
    3 - Sample Rate: 96000, Channels: 1, Format: f32, Supported: true
```

## Recording audio samples

The `record` command allows to record audio samples.
You pass the id returned by the `devices` commands using the `--device-index` option to change the input device. 
You pass the configuration id returned by the `devices` commands using the `--config-index` option to change the audio format.
Once executed you need to press the `Ctrl + c` key combination to finish the record.

This is an example run on macOS:

```bash
$ rustpotter-cli record good_morning.wav
Input device: MacBook Pro Microphone
Input device config: Sample Rate: 48000, Channels: 1, Format: f32
Begin recording...
Press 'Ctrl + c' to stop.
^CRecording good_morning.wav complete!
```

You can use something like this in bash to take multiple records quickly:

```bash
WAKEWORD="ok home"
WAKEWORD_FILENAME="${WAKEWORD// /_}"
# take 10 records, waiting one second after each.
for i in {0..9}; do (rustpotter-cli record $WAKEWORD_FILENAME$i.wav && sleep 1); done
```

## Filter a file.

The available audio filters in rustpotter can be applied to a wav file using the `filter` command.
To enable the gain normalizer filter you can use the `--gain-normalizer` option.
To enable the band-pass filter you can use the `--band-pass` option.
To display the full command options you can run `rustpotter-cli filter -h`.

This is an example run on macOS:

```bash
$ rustpotter-cli filter test_noise.wav -g --gain-ref 0.005 -b --low-cutoff 1000 --high-cutoff 2000    
Creating new file test_noise-gain0.005-bandpass1000_2000.wav
```

## Creating a wakeword model

The `build-model` command allows to create a wakeword file (also referred as model in this document).
You just need to provide the `--model-name` option (which defines the detection name),
the `--model-path` with the desired output path for the file, and a list of wav audio files.

For example:

```
rustpotter-cli build-model --model-name "ok home" --model-path ok_home.rpw ok_home1.wav ok_home2.wav
```

This is an example run on macOS:

```bash
$ WAKEWORD="ok home"
$ WAKEWORD_FILENAME="${WAKEWORD// /_}"
$ rustpotter-cli build-model --model-name "$WAKEWORD" --model-path $WAKEWORD_FILENAME.rpw $WAKEWORD_FILENAME*.wav
ok_home1.wav: WavSpec { channels: 2, sample_rate: 44100, bits_per_sample: 32, sample_format: Float }
ok home created!
```

## Using a model

You can use the commands `spot` to test a model in real time using the available audio inputs,
or `test_model` to do it against an audio file.
Both expose similar options to make change from one to the other simpler.

So it's recommended to record an example file using the record command and try to tune the options there to then test those for real. 

This is an example run on macOS:
```bash
$ rustpotter-cli test-model -g -b --low-cutoff 500 --high-cutoff 1500 ok_home_test.rpw test_noise.wav
Testing file test_noise.wav against model ok_home_test.rpw!
Wakeword detection: [11:06:11] RustpotterDetection { name: "ok_home_test", avg_score: 0.0, score: 0.5261932, scores: {"ok_home1-bandpass1000_2000.wav": 0.5261932}, counter: 12, gain: 0.9 }
```

The more relevant options for the `spot` and `test-model` commands are:

* `-d` parameter enables the called 'debug mode' so you can see the partial detections.
* `-t` sets the threshold value (defaults to 0.5).
* `-a` configures the `averaged threshold`, recommended as reduces the cpu usage. (set to half of the threshold or similar)
* `-m 6` require at least 6 frames of positive scoring (compared against the detection `counter` field).
* `-s` the comparison strategy used, defines the way the detection score is calculated from the different scores.
* `-g` enables gain normalization. To debug the gain normalization you can use `--debug-gain`.
* `--gain-ref` changes the gain normalization reference (the default value is printed at the beginning when `--debug-gain` is provided, changes with the model)
* `-b --low-cutoff 500 --high-cutoff 1500` the band-pass filter configuration, helps to attenuate background noises.

