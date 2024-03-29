# Rustpotter CLI

CLI for Rustpotter, an open source wakeword spotter forged in rust

<div align="center">
    <img src="./logo.png?raw=true" width="400px"</img> 
</div>

## Description

This is a client for using the [rustpotter](https://github.com/GiviMAD/rustpotter) library on the command line.

You can use it to record wav samples, create rustpotter wakeword files and test them.

# Installation

Some pre-build executables for the supported platforms can be found on the 'Assets' tab of the [releases](https://github.com/GiviMAD/rustpotter-cli/releases).

# Basic usage.

## Listing available audio input devices and formats.

Your can list the available audio sources with the `devices` command,
the `--configs` option can be added to display the default and available record formats for each source.

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
To use a different input device provide the  `--device-index` argument with the id returned by the `devices` commands. 
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

## Creating a Wakeword Model

The `train` command allows to create wakeword models.

It's required to setup a training and testing folders containing wav records which need to be tagged (contains [label] in its file name, where 'label' is the tag the network should predict for that audio segment) or untagged (equivalent to contain [none] on the filename).
The tag `none` is reserved because it will not emit a detections, but doesn't need to be added to the filenames.

The size and cpu usage of a wakeword model is based on the model type you choose, and the audio duration it was trained on (which is defined by the max audio duration found on the training set) and the number of labels in the training set. The train command prints these information at the beginning.

Example train directory (truncated output):

```bash
$ ls train
'[ok_home]11.wav'                         building112.wav   kitchen21.wav    kitchen489.wav   kitchen91.wav   room98.wav
'[ok_home]12.wav'                         building113.wav   kitchen210.wav   kitchen49.wav    kitchen92.wav   room99.wav
'[ok_home]13.wav'                         building114.wav   kitchen211.wav   kitchen490.wav   kitchen93.wav   speaker0.wav
'[ok_home]14.wav'                         building115.wav   kitchen212.wav   kitchen491.wav   kitchen94.wav   speaker1.wav
'[ok_home]15.wav'                         building116.wav   kitchen213.wav   kitchen492.wav   kitchen95.wav   speaker10.wav
'[ok_home]16.wav'                         building117.wav   kitchen214.wav   kitchen493.wav   kitchen96.wav   speaker100.wav
'[ok_home]1692561880432.wav'              building118.wav   kitchen215.wav   kitchen494.wav   kitchen97.wav   speaker101.wav
...
```

Those will train a model to spot "ok_home". These dataset includes about 250 records of the wakeword (the records prefixed by "[ok_home]") and about 1800 records of noises or silence, as you can see I have named those depending on what or where I was recording, but that doesn't matter as long as they do not include the delimiters "[" and "]" those are threated as if they include "[none]".

The files in the test folder should follow same rules. In this case it contains 41 records of the wakeword and around 78 random records.

It's recommended to have records of the same duration in both folders, if not the data will be truncated or padded with silence  by the max record duration on the train folder (this happens in-memory, it does not modifies the files).

Example run:

```sh
$ rustpotter-cli train -t small --train-dir train.wav/train --test-dir train.wav/test --test-epochs 10 --epochs 2500 -l 0.017 trained-small.rpw 
Start training trained-small.rpw!
Model type: small.
Labels: ["none", "ok_casa"].
Training with 2042 records.
Testing with 119 records.
Training on 1950ms of audio.
  10 train loss:  0.12944 test acc: 90.76%
  20 train loss:  0.06484 test acc: 93.28%
  30 train loss:  0.04454 test acc: 94.12%
  40 train loss:  0.03361 test acc: 94.12%
  50 train loss:  0.02687 test acc: 94.12%
  60 train loss:  0.02227 test acc: 94.12%
  70 train loss:  0.01916 test acc: 94.12%
  80 train loss:  0.01681 test acc: 94.12%
  90 train loss:  0.01499 test acc: 94.12%
 100 train loss:  0.01354 test acc: 94.12%
 110 train loss:  0.01232 test acc: 94.96%
...  
 160 train loss:  0.00822 test acc: 94.96%
 170 train loss:  0.00766 test acc: 94.96%
 180 train loss:  0.00717 test acc: 95.80%
 190 train loss:  0.00673 test acc: 95.80%
...
 470 train loss:  0.00234 test acc: 95.80%
 480 train loss:  0.00229 test acc: 95.80%
 490 train loss:  0.00224 test acc: 96.64%
 500 train loss:  0.00219 test acc: 96.64%
...
1180 train loss:  0.00083 test acc: 96.64%
1190 train loss:  0.00082 test acc: 96.64%
1200 train loss:  0.00081 test acc: 97.48%
1210 train loss:  0.00081 test acc: 97.48%
...
2340 train loss:  0.00034 test acc: 97.48%
2350 train loss:  0.00034 test acc: 97.48%
2360 train loss:  0.00034 test acc: 98.32%
2370 train loss:  0.00033 test acc: 98.32%
...
2480 train loss:  0.00031 test acc: 98.32%
2490 train loss:  0.00031 test acc: 98.32%
2500 train loss:  0.00031 test acc: 98.32%
trained-small.rpw created!
```

Be aware that you can obtain different results on different executions with the same training set as the initialization of the weights is not constant.

You can continue training from another model using the `-m` option, in that case the options used to create that model (and the audio duration) are used instead.

To get a correct idea about the accuracy of the model, do not share records between the train and test folders.

One last tip, you can take advantage of the`spot` command option for creating records on partial spot, it's an easy way to record samples.
For example creating a wakeword reference to use for capturing records for later training a wakeword model,
but also it's a great way of capturing records of false positives detected by a wakeword model, which are very valuable for training a better version.

## Creating a Wakeword Reference

The `build` command allows to create a wakeword reference file from some records.

This wakeword type requires a low number of records to be created but offers more inconsistent results than the wakeword models. 

As an example example:

```
rustpotter-cli build --model-name "ok home" --model-path ok_home.rpw ok_home1.wav ok_home2.wav
```

This is an example run on macOS:

```bash
$ WAKEWORD="ok home"
$ WAKEWORD_FILENAME="${WAKEWORD// /_}"
$ rustpotter-cli build --model-name "$WAKEWORD" --model-path $WAKEWORD_FILENAME.rpw $WAKEWORD_FILENAME*.wav
ok_home1.wav: WavSpec { channels: 2, sample_rate: 44100, bits_per_sample: 32, sample_format: Float }
ok home created!
```

## Using a model

You can use the commands `spot` to test a model in real time using the available audio inputs,
or `test` to do it against an audio file.
Both expose similar options to make change from one to the other simpler.

This way you can record an example record and tune the options there to then test those on real time. 

This is an example run on macOS:
```bash
$ rustpotter-cli test -g --gain-ref 0.004 ok_home_test.rpw test_audio.wav
Testing file test_audio.wav against model ok_home_test.rpw!
Wakeword detection: [11:06:11] RustpotterDetection { name: "ok_home_test", avg_score: 0.0, score: 0.5261932, scores: {"ok_home1-bandpass1000_2000.wav": 0.5261932}, counter: 12, gain: 0.9 }
```

The more relevant options for the `spot` and `test` commands are:

* `-d` parameter enables the called 'debug mode' so you can see the partial detections.
* `-t` sets the threshold value (defaults to 0.5).
* `-m 6` require at least 6 frames of positive scoring (compared against the detection `counter` field).
* `-e` enables the eager mode so detection is emitted as soon as possible (on min positive scores).
* `-g` enables gain normalization. To debug the gain normalization you can use `--debug-gain`, or look at the gain reflected on the detection.
* `--gain-ref` changes the gain normalization reference. (the default value is printed at the beginning when `--debug-gain` is provided, depends on the wakeword)

### Record on Partial Detections

Rustpotter can create audio records every partial detection, this can be useful to collect samples or to debug the behavior of the library.

This can be enabled by providing a folder path to the `spot` or `test` commands in the `--record-path` option.
The folder must exists and be writable for the records to be created.
