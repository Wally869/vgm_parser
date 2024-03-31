# VGM Parser  

Parse VGM data using Rust.  

## Overview  

VGM files contain instruction for soundschips of retro systems to play music.  

There is a lot of variance between VGM files:   
- Headers can have different sizes  
- Files can contain Data blocks  
- Metadata is listed at the end of the file and contains track information in utf-16 encoding  

I created this repo to parse VGM data and try to prepare the data for use in a transformer for music generation.    


## References  
VGM Specification
https://vgmrips.net/wiki/VGM_Specification

Gd3 Metadata
https://www.smspower.org/uploads/Music/gd3spec100.txt  
