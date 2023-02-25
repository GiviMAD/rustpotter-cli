# Rustpotter CLI

## CLI for Rustpotter, an open source wakeword spotter forged in rust

<div align="center">
    <img src="./logo.png?raw=true" width="400px"</img> 
</div>

## Description

This is a client for using the [rustpotter](https://github.com/GiviMAD/rustpotter) library on the command line. 

## Use Example 

```sh
# print help
rustpotter-cli -h
# print command help
rustpotter-cli spot -h
# list available input devices and configs
rustpotter-cli devices
# record samples, you should press "ctrl + c" to stop after saying your wakeword
rustpotter-cli record hey_home.wav
rustpotter-cli record hey_home1.wav
rustpotter-cli record hey_home2.wav
# check that your samples are correctly trimmed and without noise using any player
...
# build a model, this op is idempotent (same input samples with same options = same model)
rustpotter-cli build-model \
--model-path hey_home.rpw \
--model-name "hey home" \
hey_home.wav hey_home1.wav hey_home2.wav
# test the model accuracy over the samples in verbose mode to print partial detections
rustpotter-cli test-model -v hey_home.rpw hey_home.wav
rustpotter-cli test-model -v hey_home.rpw hey_home1.wav
rustpotter-cli test-model -v hey_home.rpw hey_home2.wav
# test the spot functionality in real time, customizing
# the default detection threshold
rustpotter-cli spot -t 0.563 hey_home.rpw
# rebuild a model adding a custom threshold for the word,
# this one has prevalence over the spot configuration
rustpotter-cli build-model \
--averaged-threshold 0.54 \
--threshold 0.54 \
--model-path hey_home.rpw \
--model-name "hey home" \
hey_home.wav hey_home1.wav hey_home2.wav hey_home3.wav
# you can spot using multiple models
rustpotter-cli spot hey_home.rpw good_morning.rpw ...
# you will get an output like this in your terminal on each spot event
Wakeword detection: 10:58:53
RustpotterDetection { name: "hey home", avg_score: 0.37827095, score: 0.5000453, scores: {"hey_home2.wav": 0.43628272, "hey_home.wav": 0.5000453, "hey_home1.wav": 0.4230849}, counter: 7 }
```
