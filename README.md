# Rustpotter CLI

## CLI for Rustpotter, a personal keywords spotter written in Rust

<div align="center">
    <img src="./logo.png?raw=true" width="400px"</img> 
</div>

## Description

This is a CLI for using the [rustpotter](https://github.com/GiviMAD/rustpotter) library on the command line. 

## Use Example 

```sh
# quick look up to the help
rustpotter-cli -h
# list available input devices
rustpotter-cli devices
# record samples, you should press "ctrl + c" to stop after saying your keyword
rustpotter-cli record hey_home.wav
rustpotter-cli record hey_home1.wav
rustpotter-cli record hey_home2.wav
rustpotter-cli record hey_home3.wav
# check that your samples are correctly trimmed and without noise using any player
...
# build a model, this op is idempotent (same input samples with same options = same model)
rustpotter-cli build-model \
--model-path hey_home.rpw \
--model-name "hey home" \
hey_home.wav hey_home1.wav hey_home2.wav hey_home3.wav
# test the model accuracy over the samples 
rustpotter-cli test-model hey_home.rpw hey_home.wav
rustpotter-cli test-model hey_home.rpw hey_home1.wav
rustpotter-cli test-model hey_home.rpw hey_home2.wav
rustpotter-cli test-model hey_home.rpw hey_home3.wav
# test the spot functionality in real time, customizing
# the default detection threshold
rustpotter-cli spot -t 0.563 hey_home.rpw
# rebuild a model adding a custom threshold for the word,
# this one has prevalence over the default one
rustpotter-cli build-model \
--threshold 0.54 \
--model-path hey_home.rpw \
--model-name "hey home" \
hey_home.wav hey_home1.wav hey_home2.wav hey_home3.wav
# you can spot using multiple models
rustpotter-cli spot hey_home.rpw good_morning.rpw ...
# you will get an output like this in your terminal on each spot event
Detected 'good morning' with score 0.6146946!
```
