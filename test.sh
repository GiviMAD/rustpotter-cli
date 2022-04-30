cargo run -- build-model \
-r 16000 \
--model-name "oye casa" \
--model-path samples_16000/oye_casa.rpw  \
samples_16000/oye_casa.wav samples_16000/oye_casa1.wav samples_16000/oye_casa2.wav

cargo run -- build-model \
-r 44100 \
--model-name "oye casa" \
--model-path samples_44100/oye_casa.rpw  \
samples_44100/oye_casa.wav samples_44100/oye_casa1.wav samples_44100/oye_casa2.wav

cargo run -- build-model \
-r 48000 \
--model-name "oye casa" \
--model-path samples_48000/oye_casa.rpw  \
samples_48000/oye_casa.wav samples_48000/oye_casa1.wav samples_48000/oye_casa2.wav

cargo run -- test-model \
-r 16000 \
-t 0 \
samples_16000/oye_casa.rpw samples_16000/oye_casa1.wav 

cargo run -- test-model \
-r 44100 \
-t 0 \
samples_44100/oye_casa.rpw samples_44100/oye_casa1.wav 

cargo run -- test-model \
-r 48000 \
-t 0 \
samples_48000/oye_casa.rpw samples_48000/oye_casa1.wav 


cargo run -- test-model \
-r 16000 \
-t 0 \
samples_48000/oye_casa.rpw samples_16000/oye_casa.wav 

cargo run -- test-model \
-r 48000 \
-t 0 \
samples_16000/oye_casa.rpw samples_48000/oye_casa.wav 


cargo run -- test-model \
-r 44100 \
-t 0 \
samples_16000/oye_casa.rpw samples_44100/oye_casa.wav 