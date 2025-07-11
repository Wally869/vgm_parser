[[VGM]] (Video Game Music) is a sample-accurate sound logging format for the [[Sega Master System]], the [[Sega Game Gear]] and possibly many other machines (e.g. [[Sega Genesis]]).

The normal file extension is <code>.vgm</code> but files can also be GZip compressed into <code>.vgz</code> files.
However, a [[:Category:VGM Players|VGM player]] should attempt to support compressed and uncompressed files with either extension (ZLib's GZIO library makes this trivial to implement).

== Header ==

The format starts with a 256 byte header:

{| class="wikitable"
 |-
 ! width="4%" | 
 ! width="6%" | 00
 ! width="6%" | 01
 ! width="6%" | 02
 ! width="6%" | 03
 ! width="6%" | 04
 ! width="6%" | 05
 ! width="6%" | 06
 ! width="6%" | 07
 ! width="6%" | 08
 ! width="6%" | 09
 ! width="6%" | 0A
 ! width="6%" | 0B
 ! width="6%" | 0C
 ! width="6%" | 0D
 ! width="6%" | 0E
 ! width="6%" | 0F
 |-
 ! 0x00
 | colspan="4" | "Vgm " ident
 | colspan="4" | EoF offset
 | colspan="4" | Version
 | colspan="4" | [[SN76489]] clock
 |-
 ! 0x10
 | colspan="4" | [[YM2413]] clock
 | colspan="4" | [[GD3]] offset
 | colspan="4" | Total # samples
 | colspan="4" | Loop offset
 |-
 ! 0x20
 | colspan="4" | Loop # samples
 | colspan="4" | Rate
 | colspan="2" | SN FB
 || SNW
 || SF
 | colspan="4" | [[YM2612]] clock
 |-
 ! 0x30 
 | colspan="4" | [[YM2151]] clock    
 | colspan="4" | VGM data offset 
 | colspan="4" | [[Sega PCM]] clock  
 | colspan="4" | SPCM Interface
 |-
 ! 0x40 
 | colspan="4" | [[RF5C68]] clock    
 | colspan="4" | [[YM2203]] clock    
 | colspan="4" | [[YM2608]] clock    
 | colspan="4" | [[YM2610]]/B clock
 |-
 ! 0x50 
 | colspan="4" | [[YM3812]] clock    
 | colspan="4" | [[YM3526]] clock    
 | colspan="4" | [[Y8950]] clock     
 | colspan="4" | [[YMF262]] clock
 |-
 ! 0x60 
 | colspan="4" | [[YMF278B]] clock   
 | colspan="4" | [[YMF271]] clock    
 | colspan="4" | [[YMZ280B]] clock   
 | colspan="4" | [[RF5C164]] clock
 |-
 ! 0x70 
 | colspan="4" | [[PWM]] clock
 | colspan="4" | [[AY8910]] clock    || AYT 
 | colspan="3" | AY Flags   || VM || *** || LB || LM
 |-
 ! 0x80 
 | colspan="4" | [[GB DMG]] clock    
 | colspan="4" | [[NES APU]] clock   
 | colspan="4" | [[MultiPCM]] clock 
 | colspan="4" | [[uPD7759]] clock
 |-
 ! 0x90 
 | colspan="4" | [[OKIM6258]] clock  || OF || KF || CF || ***  
 | colspan="4" | [[OKIM6295]] clock  
 | colspan="4" | [[K051649]] clock
 |-
 ! 0xA0 
 | colspan="4" | [[K054539]] clock   
 | colspan="4" | [[HuC6280]] clock   
 | colspan="4" | [[C140]] clock 
 | colspan="4" | [[K053260]] clock
 |-
 ! 0xB0 
 | colspan="4" | [[Pokey]] clock     
 | colspan="4" | [[QSound]] clock
 | colspan="4" | [[SCSP]] clock
 | colspan="4" | Extra Hdr ofs
 |-
 ! 0xC0
 | colspan="4" | [[WonderSwan]] clock     
 | colspan="4" | [[VSU]] clock
 | colspan="4" | [[SAA1099]] clock
 | colspan="4" | [[ES5503]] clock
 |-
 ! 0xD0
 | colspan="4" | [[ES5506]] clock
 | colspan="2" | ES chns || CD || ***
 | colspan="4" | [[X1-010]] clock
 | colspan="4" | [[C352]] clock
 |-
 ! 0xE0
 | colspan="4" | [[GA20]] clock
 | colspan="4" | [[Mikey]] clock || *** || *** || *** || *** || *** || *** || *** || ***
 |-
 ! 0xF0
 | *** || *** || *** || *** || *** || *** || *** || *** || *** || *** || *** || *** || *** || *** || *** || ***

 |}


* Unused space (marked with *) is reserved for future expansion, and must be zero.
* All integer values are ''unsigned'' and written in "Intel" byte order (Little Endian), so for example "0x12345678" is written as <code>0x78 0x56 0x34 0x12</code>.
* All pointer offsets are written as relative to the current position in the file, so for example the [[GD3 Specification|GD3]] offset at 0x14 in the header is the file position of the GD3 tag minus 0x14.
* All header sizes are valid for all versions from 1.50 on, as long as header has at least 64 bytes. If the VGM data starts at an offset that is lower than 0x100, all overlapping header bytes have to be handled as they were zero.
* VGMs run with a rate of 44100 samples per second. All sample values use this unit.

{| class="wikitable topAlign"
|-
! Ofs
! style="width: 50px" | Size
! Field
! Description
|-
! 0x00
| 32 bits 
| <code>"Vgm "</code>
| file identification (<code>0x56 0x67 0x6d 0x20</code>)
|-
! 0x04
| 32 bits 
| Eof offset 
|
Relative offset to end of file (i.e. file length - 4).
This is mainly used to find the next track when concatenating player stubs and multiple files.
|-
! 0x08
| 32 bits 
| Version number 
|
Version number in BCD-Code. e.g. Version 1.71 is stored as <code>0x00000171</code>.
This is used for backwards compatibility in players, and defines which header values are valid.
|-
! 0x0C
| 32 bits 
| SN76489 clock 
|
Input clock rate in Hz for the [[SN76489]] PSG chip. A typical value is 3579545.

It should be 0 if there is no PSG chip used.

{{Note|Bit 31 (0x80000000) is used on combination with the dual-chip-bit to indicate that this is a [[T6W28]]. (PSG variant used in [[Neo Geo Pocket]])}}
|-
! 0x10
| 32 bits 
| YM2413 clock 
|
Input clock rate in Hz for the [[YM2413]] chip. A typical value is 3579545.

It should be 0 if there is no YM2413 chip used.

For version 1.01 and earlier files, this may in fact be the clock for the YM2151 or YM2612. If above 5000000 it must be the YM2612 clock.

|-
! 0x14
| 32 bits 
| GD3 offset 
|
Relative offset to GD3 tag. 0 if no GD3 tag.
GD3 tags are descriptive tags similar in use to ID3 tags in MP3 files.
See the GD3 specification for more details. The GD3 tag is usually stored at the end of the file, immediately after the VGM data.
|-
! 0x18
| 32 bits 
| Total # samples 
|
Total of all wait values in the file.
|-
! 0x1C
| 32 bits 
| Loop offset 
|
Relative offset to loop point, or 0 if no loop.
For example, if the data for the one-off intro to a song was in bytes <code>0x0040 - 0x3FFF</code> of the file, but the main looping section started at <code>0x4000</code>, this would contain the value <code>0x4000 - 0x1C = 0x00003FE4</code>.

{{Note|A VGM file parser should be aware that some tools may write invalid loop offsets, resulting in out-of-range file offsets or 0-sample loops and treat those as "no loop". (and possibly throw a warning)}}
|-
! 0x20
| 32 bits 
| Loop # samples 
|
Number of samples in one loop, or 0 if there is no loop.
Total of all wait values between the loop point and the end of the file.
|-
| colspan="4" | '''VGM 1.01 additions:'''
|-
! 0x24
| 32 bits 
| Rate 
|
"Rate" of recording in Hz, used for rate scaling on playback. It is typically 50 for PAL systems and 60 for NTSC systems. It should be set to zero if rate scaling is not appropriate - for example, if the game adjusts its music engine for the system's speed.
VGM 1.00 files will have a value of 0.
|-
| colspan="4" | '''VGM 1.10 additions:'''
|-
! 0x28
| 16 bits 
| SN76489 feedback 
|
The white noise feedback pattern for the SN76489 PSG. For anachronistic reasons this field is based directly on MAME's feedback taps which includes latency bits and differs from the physical value. Known values are:
{|
| 0x0003 || SN76489, SN94624
|-
| 0x0006 || Used incorrectly by some packs to refer to SN76489A, SN76494, SN76496, and Y204 in combination with the LFSR width value 16. Should be feedback pattern 0xC and LFSR width 17. The latter correctly reflects 1 additional bit of latency between shift register and output.
|-
| 0x0009 || Sega Master System 2/Game Gear/Mega Drive (SN76489/SN76496 integrated into Sega VDP chip)
|-
| 0x000C || SN76489A, SN76494, SN76496, Y204
|-
| 0x0022 || NCR8496, PSSJ3
|}
For version 1.01 and earlier files, the feedback pattern should be assumed to be 0x0009. If the PSG is not used then this may be omitted (left at zero).
|-
! 0x2A
| 8 bits 
| SN76489 shift register width 
|
The noise feedback shift register width, in bits. For anachronistic reasons this field is based directly on MAME's feedback mask which includes latency bits and differs from the physical value. Known values are:
{|
| 15 || SN76489, SN94624
|-
| 16 || Sega Master System 2/Game Gear/Mega Drive (SN76489/SN76496 integrated into Sega VDP chip), NCR8496, PSSJ3
|-
| 17 || SN76489A, SN76494, SN76496, Y204
|}
For version 1.01 and earlier files, the shift register width should be
assumed to be 16. If the PSG is not used then this may be omitted (left
at zero).
|-
| colspan="4" | '''VGM 1.51 additions:'''
|-
! 0x2B
| 8 bits 
| SN76489 Flags 
|
Misc flags for the SN76489. Most of them don't make audible changes and
can be ignored, if the SN76489 emulator lacks the features.
{|
| bit 0   || frequency 0 is 0x400 (should set for all chips but SEGA PSG)
|-
| bit 1   || output negate flag
|-
| bit 2   || GameGear stereo on/off (on when bit clear)
|-
| bit 3   || /8 Clock Divider on/off (on when bit clear)
|-
| bit 4   || XNOR noise mode (for NCR8496/PSSJ-3)
|-
| bit 5-7 || reserved (must be zero)
|}
For version 1.51 and earlier files, all the flags should not be set.
If the PSG is not used then this may be omitted (left at zero).
|-
| colspan="4" | '''VGM 1.10 additions:'''
|-
! 0x2C
| 32 bits 
| YM2612 clock 
|
Input clock rate in Hz for the YM2612 chip. A typical value is 7670454 or 8053975.

It should be 0 if there us no YM2612 chip used.

For version 1.01 and earlier files, the YM2413 clock rate should be used for the clock rate of the YM2612 if it is greater than 5000000.

For version 1.51 and later, bit 31 is set to indicate the YM3438 variant.
|-
! 0x30
| 32 bits 
| YM2151 clock 
|
Input clock rate in Hz for the YM2151 chip. A typical value is 3579545 or 4000000.

It should be 0 if there us no YM2151 chip used.

For version 1.01 and earlier files, the YM2413 clock rate should be used for the clock rate of the YM2151 if it is less than 5000000.

For version 1.51 and later, bit 31 is set to indicate the YM2164 variant.
|-
| colspan="4" | '''VGM 1.50 additions:'''
|-
! 0x34
| 32 bits 
| VGM data offset 
|
Relative offset to VGM data stream.

If the VGM data starts at absolute offset 0x40, this will contain value 0x0000000C. For versions prior to 1.50, it should be 0 and the VGM data must start at offset 0x40.
|-
| colspan="4" | '''VGM 1.51 additions:'''
|-
! 0x38
| 32 bits 
| Sega PCM clock 
|
Input clock rate in Hz for the Sega PCM chip. A typical value is 4000000.

It should be 0 if there is no Sega PCM chip used.
|-
! 0x3C
| 32 bits 
| Sega PCM interface register 
|
The interface register for the Sega PCM chip.

It should be 0 if there is no Sega PCM chip used.
|-
! 0x40
| 32 bits 
| RF5C68 clock 
|
Input clock rate in Hz for the RF5C68 PCM chip. A typical value is 10000000 or 12500000.

It should be 0 if there is no RF5C68 chip used.
|-
! 0x44
| 32 bits 
| YM2203 clock 
|
Input clock rate in Hz for the YM2203 chip. A typical value is 3000000 or 4000000.

It should be 0 if there is no YM2203 chip used.
|-
! 0x48
| 32 bits 
| YM2608 clock 
|
Input clock rate in Hz for the YM2608 chip. A typical value is 7987000 or 8000000.

It should be 0 if there is no YM2608 chip used.
|-
! 0x4C
| 32 bits 
| YM2610/YM2610B clock 
|
Input clock rate in Hz for the YM2610/B chip. A typical value is 8000000.
It should be 0 if there is no YM2610/B chip used.
{{Note|Bit 31 is used to set whether it is an YM2610 or an YM2610B chip.
If bit 31 is set it is an YM2610B, if bit 31 is clear it is an YM2610.}}
|-
! 0x50
| 32 bits 
| YM3812 clock 
|
Input clock rate in Hz for the YM3812 chip. A typical value is 3579545.

It should be 0 if there is no YM3812 chip used.
|-
! 0x54
| 32 bits 
| YM3526 clock 
|
Input clock rate in Hz for the YM3526 chip. A typical value is 3579545.

It should be 0 if there is no YM3526 chip used.
|-
! 0x58
| 32 bits 
| Y8950 clock 
|
Input clock rate in Hz for the Y8950 chip. A typical value is 3579545.

It should be 0 if there is no Y8950 chip used.
|-
! 0x5C
| 32 bits 
| YMF262 clock 
|
Input clock rate in Hz for the YMF262 chip. A typical value is 14318180.

It should be 0 if there is no YMF262 chip used.
|-
! 0x60
| 32 bits 
| YMF278B clock 
|
Input clock rate in Hz for the YMF278B chip. A typical value is 33868800.

It should be 0 if there is no YMF278B chip used.
|-
! 0x64
| 32 bits 
| YMF271 clock 
|
Input clock rate in Hz for the YMF271 chip. A typical value is 16934400.

It should be 0 if there is no YMF271 chip used.
|-
! 0x68
| 32 bits 
| YMZ280B clock 
|
Input clock rate in Hz for the YMZ280B chip. A typical value is 16934400.

It should be 0 if there is no YMZ280B chip used.
|-
! 0x6C
| 32 bits 
| RF5C164 clock 
|
Input clock rate in Hz for the RF5C164 PCM chip. A typical value is 12500000.

It should be 0 if there is no RF5C164 chip used.
|-
! 0x70
| 32 bits 
| PWM clock 
|
Input clock rate in Hz for the PWM chip. A typical value is 23011361.

It should be 0 if there is no PWM chip used.
|-
! 0x74
| 32 bits 
| AY8910 clock 
|
Input clock rate in Hz for the AY8910 chip. A typical value is 1789772.

It should be 0 if there is no AY8910 chip used.
|-
! 0x78
| 8 bits 
| AY8910 Chip Type 
|
Defines the exact type of AY8910. The values are:
{|
! 0x00
| AY8910
|-
! 0x01
| AY8912
|-
! 0x02
| AY8913
|-
! 0x03
| AY8930
|-
! 0x04
| AY8914
|-
! 0x10
| YM2149
|-
! 0x11
| YM3439
|-
! 0x12
| YMZ284
|-
! 0x13
| YMZ294
|}

If the AY8910 is not used then this may be omitted (left at zero).
|-
! 0x79
| 8 bits 
| AY8910 Flags 
|
Misc flags for the AY8910. Default is 0x01.
For additional description see ay8910.h in MAME source code.
{|
| bit 0   || Legacy Output
|-
| bit 1   || Single Output
|-
| bit 2   || Discrete Output
|-
| bit 3   || RAW Output
|-
| bit 4   || YMxxxx pin 26 (clock divider) low
|-
| bit 5-7 || reserved (must be zero)
|}

If the AY8910 is not used then this may be omitted (left at zero).
|-
! 0x7A
| 8 bits 
| YM2203/AY8910 Flags 
|
Misc flags for the AY8910. This one is specific for the AY8910 that's connected with/part of the YM2203.
|-
! 0x7B
| 8 bits 
| YM2608/AY8910 Flags 
|
Misc flags for the AY8910. This one is specific for the AY8910 that's connected with/part of the YM2608.
|-
| colspan="4" | '''VGM 1.60 additions:'''
|-
! 0x7C
| 8 bits 
| Volume Modifier 
|
<code>Volume = 2 ^ (VolumeModifier / 0x20)</code> where VolumeModifier is a number
from -63 to 192 (-63 = 0xC1, 0 = 0x00, 192 = 0xC0). Also the value -63
gets replaced with -64 in order to make factor of 0.25 possible.
Therefore the volume can reach levels between 0.25 and 64.
Default is 0, which is equal to a factor of 1 or 100%.

{{Note|Players should support the Volume Modifier in v1.50 files and higher. This way Mega Drive VGMs can use the Volume Modifier without breaking compatibility with old players.}}
|-
! 0x7D
| 8 bits
| reserved
| Reserved byte for future use. It must be 0.
|-
! 0x7E
| 8 bits 
| Loop Base 
|
Modifies the number of loops that are played before the playback ends.
Set this value to eg. 1 to reduce the number of played loops by one.
This is useful, if the song is looped twice in the vgm, because there
are minor differences between the first and second loop and the song
repeats just the second loop.
The resulting number of loops that are played is calculated as
following: NumLoops = NumLoopsModified - LoopBase
Default is 0. Negative numbers are possible (80h...FFh = -128...-1)
|-
| colspan="4" | '''VGM 1.51 additions:'''
|-
! 0x7F
| 8 bits 
| Loop Modifier 
|
Modifies the number of loops that are played before the playback ends.
You may want to use this, e.g. if a tune has a very short, but non-
repetitive loop (then set it to 0x20 double the loop number).
The resulting number of loops that are played is calculated as
following:
  NumLoops = ProgramNumLoops * LoopModifier / 0x10

Default is 0, which is equal to 0x10.
|-
| colspan="4" | '''VGM 1.61 additions:'''
|-
! 0x80
| 32 bits 
| GameBoy DMG clock 
|
Input clock rate in Hz for the GameBoy DMG chip, LR35902. A typical value is 4194304.

It should be 0 if there is no GB DMG chip used.
|-
! 0x84
| 32 bits 
| NES APU clock 
|
Input clock rate in Hz for the NES APU chip, N2A03. A typical value is 1789772.

It should be 0 if there is no NES APU chip used.

{{Note|Bit 31 (0x80000000) is used to enable the FDS sound addon. Set to enable, clear to disable.}}
|-
! 0x88
| 32 bits 
| MultiPCM clock 
|
Input clock rate in Hz for the MultiPCM chip. A typical value is 8053975.

It should be 0 if there is no MultiPCM chip used.
|-
! 0x8C
| 32 bits 
| uPD7759 clock 
|
Input clock rate in Hz for the uPD7759 chip. A typical value is 640000.

It should be 0 if there is no uPD7759 chip used.
|-
! 0x90
| 32 bits 
| OKIM6258 clock 
|
Input clock rate in Hz for the OKIM6258 chip. A typical value is 4000000.

It should be 0 if there is no OKIM6258 chip used.
|-
! 0x94
| 8 bits 
| OKIM6258 Flags 
|
Misc flags for the OKIM6258. Default is 0x00.
{|
| bit 0-1 || Clock Divider (clock dividers are 1024, 768, 512, 512)
|-
| bit 2   || 3/4-bit ADPCM select (default is 4-bit, doesn't work currently)
|-
| bit 3   || 10/12-bit Output (default is 10-bit)
|-
| bit 4-7 || reserved (must be zero)
|}
If the OKIM6258 is not used then this may be omitted (left at zero).
|-
! 0x95
| 8 bits 
| K054539 Flags 
|
Misc flags for the K054539. Default is 0x01.
See also k054539.h in MAME source code.
{|
| bit 0   || Reverse Stereo
|-
| bit 1   || Disable Reverb
|-
| bit 2   || Update at KeyOn
|-
| bit 3-7 || reserved (must be zero)
|}
If the K054539 is not used then this may be omitted (left at zero).
|-
! 0x96
| 8 bits 
| C140 Chip Type 
|
Defines the exact type of C140 and its banking method. The values are:
{|
! 0x00
| C140, [[Namco System 2]]
|-
! 0x01
| C140, [[Namco System 21]]
|-
! 0x02
| 219 ASIC, Namco NA-1/2
|}
If the C140 is not used then this may be omitted (left at zero).
|-
! 0x97
| 8 bits
| reserved
| Reserved byte for future use. It must be 0.
|-
! 0x98
| 32 bits 
| OKIM6295 clock 
|
Input clock rate in Hz for the OKIM6295 chip. A typical value is 8000000 or 8448000.

It should be 0 if there is no OKIM6295 chip used.
|-
! 0x9C
| 32 bits 
| K051649/K052539 clock 
|
Input clock rate in Hz for the K051649 chip. A typical value is 1789773.

It should be 0 if there is no K051649 chip used.

If bit 31 is set it is a K052539.
|-
! 0xA0
| 32 bits 
| K054539 clock 
|
Input clock rate in Hz for the K054539 chip. A typical value is 18432000.

It should be 0 if there is no K054539 chip used.
|-
! 0xA4
| 32 bits 
| HuC6280 clock 
|
Input clock rate in Hz for the HuC6280 chip. A typical value is 3579545.

It should be 0 if there is no HuC6280 chip used.
|-
! 0xA8
| 32 bits 
| C140 clock 
|
Input clock rate in Hz for the C140 chip. A typical value is 21390.

It should be 0 if there is no C140 chip used.
|-
! 0xAC
| 32 bits 
| K053260 clock 
|
Input clock rate in Hz for the K053260 chip. A typical value is 3579545.

It should be 0 if there is no K053260 chip used.
|-
! 0xB0
| 32 bits 
| Pokey clock 
|
Input clock rate in Hz for the Pokey chip. A typical value is 1789772.

It should be 0 if there is no Pokey chip used.
|-
! 0xB4
| 32 bits 
| QSound clock 
|
Input clock rate in Hz for the QSound chip. A typical value is 4000000.

It should be 0 if there is no QSound chip used.
|-
| colspan="4" | '''VGM 1.71 additions:'''
|-
! 0xB8
| 32 bits
| SCSP clock
|
Input clock rate in Hz for the SCSP chip. A typical value is 22579200.

It should be 0 if there is no SCSP chip used.
|-
| colspan="4" | '''VGM 1.70 additions:'''
|-
! 0xBC
| 32 bits
| Extra Header Offset
| Relative offset to the extra header or 0 if no extra header is present.
|-
| colspan="4" | '''VGM 1.71 additions:'''
|-
! 0xC0
| 32 bits
| WonderSwan clock
|
Input clock rate in Hz for the WonderSwan chip. A typical value is 3072000.

It should be 0 if there is no WonderSwan chip used.
|-
! 0xC4
| 32 bits
| VSU clock
|
Input clock rate in Hz for the VSU chip. A typical value is 5000000.

It should be 0 if there is no VSU chip used.
|-
! 0xC8
| 32 bits
| SAA1099 clock
|
Input clock rate in Hz for the SAA1099 chip. A typical value is 8000000 (or 7159000/7159090).

It should be 0 if there is no SAA1099 chip used.
|-
! 0xCC
| 32 bits
| ES5503 clock
|
Input clock rate in Hz for the ES5503 chip. A typical value is 7159090.

It should be 0 if there is no ES5503 chip used.
|-
! 0xD0
| 32 bits
| ES5505/ES5506 clock
|
Input clock rate in Hz for the ES5505/ES5506 chip. A typical value is 16000000.

It should be 0 if there is no ES5505/ES5506 chip used.
{{Note|Bit 31 is used to set whether it is an ES5505 or an ES5506 chip.
If bit 31 is set it is an ES5506, if bit 31 is clear it is an ES5505.}}
|-
! 0xD4
| 8 bits
| ES5503 amount of output channels
|
Defines the internal number of output channels for the ES5503.

Possible values are 1 to 8. A typical value is 2.

If the ES5503 is not used then this may be omitted (left at zero).
|-
! 0xD5
| 8 bits
| ES5505/ES5506 amount of output channels
|
Defines the internal number of output channels for the ES5506.

Possible values are 1 to 4 for the ES5505 and 1 to 8 for the ES5506. A typical value is 1.

If the ES5505/ES5506 is not used then this may be omitted (left at zero).
|-
! 0xD6
| 8 bits
| C352 clock divider
|
Defines the clock divider for the C352 chip, divided by 4 in order to achieve a divider range of 0 to 1020. A typical value is 288.

If the C352 is not used then this may be omitted (left at zero).

|-
! 0xD8
| 32 bits
| X1-010 clock
|
Input clock rate in Hz for the X1-010 chip. A typical value is 16000000.

It should be 0 if there is no X1-010 chip used.
|-
! 0xDC
| 32 bits
| C352 clock
|
Input clock rate in Hz for the C352 chip. A typical value is 24192000.

It should be 0 if there is no C352 chip used.
|-
! 0xE0
| 32 bits
| GA20 clock
|
Input clock rate in Hz for the GA20 chip. A typical value is 3579545.

It should be 0 if there is no GA20 chip used.
|-
| colspan="4" | '''VGM 1.72 additions:'''
|-
! 0xE4
| 32 bits
| Mikey clock
|
Input clock rate in Hz for the Mikey (Atari Lynx) chip. A typical value is 16000000.

It should be 0 if there is no Mikey chip used.
|}

== Commands ==
Starting at the location specified by the VGM data offset (or, offset 0x40 for
file versions below 1.50) is found a sequence of commands containing data
written to the chips or timing information. A command is one of:
{| class="wikitable"
| 0x31 || dd || AY8910 stereo mask, dd is a bit mask of <code>i y r3 l3 r2 l2 r1 l1</code> (bit 7 ... 0)
{|
| <code>i</code>        || chip instance (0 or 1)
|-
| <code>y</code>        || set stereo mask for YM2203 SSG (1) or AY8910 (0)
|-
| <code>l1/l2/l3</code> || enable channel 1/2/3 on left speaker
|-
| <code>r1/r2/r3</code> || enable channel 1/2/3 on right speaker
|}
|-
| 0x40 || aa dd || [[Mikey]], write value <code>dd</code> to register <code>aa</code>
|-
| 0x4F || dd || Game Gear PSG stereo, write <code>dd</code> to port <code>0x06</code>
|-
| 0x50 || dd    || [[PSG]] (SN76489/SN76496) write value <code>dd</code>
|-
| 0x51 || aa dd || [[YM2413]], write value <code>dd</code> to register <code>aa</code>
|-
| 0x52 || aa dd || [[YM2612]] port 0, write value <code>dd</code> to register <code>aa</code>
|-
| 0x53 || aa dd || YM2612 port 1, write value <code>dd</code> to register <code>aa</code>
|-
| 0x54 || aa dd || [[YM2151]], write value <code>dd</code> to register <code>aa</code>
|-
| 0x55 || aa dd || [[YM2203]], write value <code>dd</code> to register <code>aa</code>
|-
| 0x56 || aa dd || [[YM2608]] port 0, write value <code>dd</code> to register <code>aa</code>
|-
| 0x57 || aa dd || YM2608 port 1, write value <code>dd</code> to register <code>aa</code>
|-
| 0x58 || aa dd || [[YM2610]] port 0, write value <code>dd</code> to register <code>aa</code>
|-
| 0x59 || aa dd || YM2610 port 1, write value <code>dd</code> to register <code>aa</code>
|-
| 0x5A || aa dd || [[YM3812]], write value <code>dd</code> to register <code>aa</code>
|-
| 0x5B || aa dd || [[YM3526]], write value <code>dd</code> to register <code>aa</code>
|-
| 0x5C || aa dd || [[Y8950]], write value <code>dd</code> to register <code>aa</code>
|-
| 0x5D || aa dd || [[YMZ280B]], write value <code>dd</code> to register <code>aa</code>
|-
| 0x5E || aa dd || [[YMF262]] port 0, write value <code>dd</code> to register <code>aa</code>
|-
| 0x5F || aa dd || YMF262 port 1, write value <code>dd</code> to register <code>aa</code>
|-
| 0x61 || nn nn || Wait <code>n</code> samples, <code>n</code> can range from 0 to 65535 (approx 1.49 seconds). Longer pauses than this are represented by multiple wait commands.
|-
| 0x62 || || wait 735 samples (60th of a second), a shortcut for <code>0x61 0xdf 0x02</code>
|-
| 0x63 || || wait 882 samples (50th of a second), a shortcut for <code>0x61 0x72 0x03</code>
|-
| 0x66 ||     || end of sound data
|-
| 0x67 || ... || data block: see below
|-
| 0x68 || ... || PCM RAM write: see below
|-
| 0x7n ||     || wait <code>n+1</code> samples, n can range from 0 to 15.
|-
| 0x8n ||     || YM2612 port 0 address 2A write from the data bank, then wait n samples; n can range from 0 to 15. Note that the wait is n, NOT n+1. See also command <code>0xE0</code>.
{{Note|Written to first chip instance only.}}
|-
| colspan="2" | 0x90 - 0x95
| DAC Stream Control Write: see below
|-
| 0xA0 || aa dd || [[AY8910]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xB0 || aa dd || [[RF5C68]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xB1 || aa dd || [[RF5C164]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xB2 || ad dd || [[PWM]], write value <code>ddd</code> to register <code>a</code> (<code>d</code> is MSB, <code>dd</code> is LSB)
|-
| 0xB3 || aa dd || [[GameBoy DMG]], write value <code>dd</code> to register <code>aa</code>
{{Note|Register 00 equals GameBoy address FF10.}}
|-
| 0xB4 || aa dd || [[NES APU]], write value <code>dd</code> to register <code>aa</code>
{{Note|Registers 00-1F equal NES address 4000-401F,
registers 20-3E equal NES address 4080-409E,
register 3F equals NES address 4023,
registers 40-7F equal NES address 4040-407F.}}
|-
| 0xB5 || aa dd || [[MultiPCM]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xB6 || aa dd || [[uPD7759]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xB7 || aa dd || [[OKIM6258]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xB8 || aa dd || [[OKIM6295]], write value <code>dd</code> to register aa
|-
| 0xB9 || aa dd || [[HuC6280]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xBA || aa dd || [[K053260]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xBB || aa dd || [[Pokey]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xBC || aa dd || [[WonderSwan]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xBD || aa dd || [[SAA1099]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xBE || aa dd || [[ES5506]], write value <code>dd</code> to register <code>aa</code>
{{Note|This command writes 8-bit data. For 16-bit data, use command 0xD6}}
|-
| 0xBF || aa dd || [[GA20]], write value <code>dd</code> to register <code>aa</code>
|-
| 0xC0 || bbaa dd || [[Sega PCM]], write value <code>dd</code> to memory offset <code>aabb</code>
|-
| 0xC1 || bbaa dd || [[RF5C68]], write value <code>dd</code> to memory offset <code>aabb</code>
|-
| 0xC2 || bbaa dd || [[RF5C164]], write value <code>dd</code> to memory offset <code>aabb</code>
|-
| 0xC3 || cc bbaa || [[MultiPCM]], write set bank offset <code>aabb</code> to channel <code>cc</code>
|-
| 0xC4 || mmll rr || [[QSound]], write value <code>mmll</code> to register <code>rr</code> (<code>mm</code> - data MSB, <code>ll</code> - data LSB)
|-
| 0xC5 || mmll dd || [[SCSP]], write value <code>dd</code> to memory offset <code>mmll</code> (<code>mm</code> - offset MSB, <code>ll</code> - offset LSB)
|-
| 0xC6 || mmll dd || [[WonderSwan]], write value <code>dd</code> to memory offset <code>mmll</code> (<code>mm</code> - offset MSB, <code>ll</code> - offset LSB)
|-
| 0xC7 || mmll dd || [[VSU]], write value <code>dd</code> to memory offset <code>mmll</code> (<code>mm</code> - offset MSB, <code>ll</code> - offset LSB)
|-
| 0xC8 || mmll dd || [[X1-010]], write value <code>dd</code> to memory offset <code>mmll</code> (<code>mm</code> - offset MSB, <code>ll</code> - offset LSB)
|-
| 0xD0 || pp aa dd || [[YMF278B]], port <code>pp</code>, write value <code>dd</code> to register <code>aa</code>
|-
| 0xD1 || pp aa dd || [[YMF271]], port <code>pp</code>, write value <code>dd</code> to register <code>aa</code>
|-
| 0xD2 || pp aa dd || [[SCC1]], port <code>pp</code>, write value <code>dd</code> to register <code>aa</code>
|-
| 0xD3 || pp aa dd || [[K054539]], write value <code>dd</code> to register <code>ppaa</code>
|-
| 0xD4 || pp aa dd || [[C140]], write value <code>dd</code> to register <code>ppaa</code>
|-
| 0xD5 || pp aa dd || [[ES5503]], write value <code>dd</code> to register <code>ppaa</code>
|-
| 0xD6 || pp aa dd || [[ES5506]], write value <code>aadd</code> to register <code>pp</code>
{{Note|This command writes 16-bit data. For 8-bit data, use command 0xBE}}
|-
| 0xE0 || dddddddd || Seek to offset <code>dddddddd</code> (Intel byte order) in PCM data bank of data block type 0 (YM2612).
{{Note|This command is used with <code>8x</code> commands. The usual order is:
# Data block <code>00</code> (<code>0x67</code> <code>0x66</code> <code>0x00</code> ...)
# Use <code>E0</code> to set the read pointer
# Use <code>8x</code> to play a sample and advance the pointer by 1
}}
|-
| 0xE1 || mmll aadd|| [[C352]], write value <code>aadd</code> to register <code>mmll</code>
|-
|}

Some ranges are reserved for future use, with different numbers of operands:
{| class="wikitable topAlign"
| <code>0x30..0x3F</code> || dd          || one operand, reserved for future use
{{Note|used for dual-chip support (see below)}}
|-
| <code>0x41..0x4E</code> || dd dd       || two operands, reserved for future use
{{Note|was one operand only til v1.60}}
|-
| <code>0xC9..0xCF</code> || dd dd dd    || three operands, reserved for future use
|-
| <code>0xD7..0xDF</code> || dd dd dd    || three operands, reserved for future use
|-
| <code>0xE2..0xFF</code> || dd dd dd dd || four operands, reserved for future use
|}

On encountering these, the correct number of bytes should be skipped.

== Data blocks ==

VGM command 0x67 specifies a data block. These are used to store large amounts
of data, which can be used in parallel with the normal VGM data stream. The
data block format is:

  0x67 0x66 tt ss ss ss ss (data)

where:
{| class="wikitable"
| <code>0x67</code>
|
| VGM command
|-
| <code>0x66</code>
|
| compatibility command to make older players stop parsing the stream
|-
| <code>tt</code>
| 8 bits
| data type (see below)
|-
| <code>ss ss ss ss</code>
| 32 bits
| size of data, in bytes
|-
| <code>data</code>
|
| data, of size previously specified
|}

Data blocks of recorded streams, if present, should be at the very start of the VGM data.
Multiple data blocks expand the data bank. (The start offset and length of the block in the data bank should be saved for command 0x95.)
Because data blocks can happen anywhere in the stream, players must be able to parse data blocks anywhere in the stream.

The data block type specifies what type of data it contains. Currently defined types are:

00..3F : data of recorded streams (uncompressed)

40..7E : data of recorded streams (compressed)

  data block format for compressed streams:
    tt (8 bits) = compression type
                    00 - bit packing compression
                    01 - DPCM compression
    ss ss ss ss (32 bits) = size of uncompressed data (for memory allocation)
                            It is assumed that each decompressed value uses
                            ceil(bd/8) bytes.
    (attr) = attribute bytes used by the decompression-algorithm
    bit packing compression:
        bd (8 bits) = Bits decompressed
        bc (8 bits) = Bits compressed
        st (8 bits) = compression sub-type
                        00 - copy (high bits aren't used)
                        01 - shift left (low bits aren't used)
                        02 - use table (data = index into decompression table,
                                        see data block 7F)
        aa aa (16 bits) = value that is added (ignored if table is used)
        The data block is treated as a bitstream with bc bits per value. The
        top bits in each byte are read first. The extracted bits of each value
        are transformed into a value with at least bd bits using method st.
        Finally, aaaa is added to get the resulting value.
    DPCM-Compression: (uses a decompression table)
        bd (8 bits) = Bits decompressed
        bc (8 bits) = Bits compressed
        st (8 bits) = [reserved for future use, must be 00]
        aa aa (16 bits) = start value
        The data is read as a bitstream (see bit packing). The read value is used as
        index into a delta-table (defined by data block 7F). The delta value
        is added to the "state" value, which is also the result value.
        The "state" value is initialized with aaaa at the beginning.
    (data) = compressed data, of size (block size - 0x0A - attr size)

7F     : Decompression Table

    tt (8 bits) = compression type (see data block 40..7E)
    st (8 bits) = compression sub-type (see data block 40..7E)
    bd (8 bits) = Bits decompressed
    bc (8 bits) = Bits compressed (only used for verifying against
                  block 40..7E)
    cc cc (16 bits) = number of following values (with each of size
                      ceil(bd / 8))
    (data) = table data, cccc values with a total size of (block size - 0x06)
    Note: Multiple decompression tables are valid. The player should keep a
          list of one table per tt and st combination. If there are multiple
          tables of the same tt/st type, the new one overrides the old one and
          all following compressed data blocks will use the new table.

80..BF : ROM/RAM Image dumps (contain usually samples)

  data block format for ROM dumps:
    rr rr rr rr (32 bits) = size of the entire ROM
    ss ss ss ss (32 bits) = start address of data
    (data) = ROM data, of size (block size - 0x08)
  The size of the VGM can be decreased a lot by saving only the used parts
  of the ROM. This is done by saving multiple small ROM data blocks.
  The start address is the ROM offset where the data will be written, the
  ROM size is used to allocate space for the ROM (and some chips rely on it).

C0..DF : RAM writes (for RAM with up to 64 KB)

  data block format for direct RAM writes:
    ss ss (16 bits) = start address of data (affected by a chip's banking
                      registers)
    (data) = RAM data, of size (block size - 0x02)

E0..FF : RAM writes (for RAM with more than 64 KB)

  data block format for direct RAM writes:
    ss ss ss ss (32 bits) = start address of data (affected by a chip's banking
                            registers)
    (data) = RAM data, of size (block size - 0x04)


{| class="wikitable"
| 00 || YM2612 PCM data for use with associated commands
|-
| 01 || RF5C68 PCM data for use with associated commands
|-
| 02 || RF5C164 PCM data for use with associated commands
|-
| 03 || PWM PCM data for use with associated commands
|-
| 04 || OKIM6258 ADPCM data for use with associated commands
|-
| 05 || HuC6280 PCM data for use with associated commands
|-
| 06 || SCSP PCM data for use with associated commands
|-
| 07 || NES APU DPCM data for use with associated commands
|-
| 08 || Mikey PCM data for use with associated commands
|-
| 40..7E || same as 00..3E, just compressed
|-
| 80 || Sega PCM ROM data
|-
| 81 || YM2608 DELTA-T ROM data
|-
| 82 || YM2610 ADPCM ROM data
|-
| 83 || YM2610 DELTA-T ROM data
|-
| 84 || YMF278B ROM data
|-
| 85 || YMF271 ROM data
|-
| 86 || YMZ280B ROM data
|-
| 87 || YMF278B RAM data
|-
| 88 || Y8950 DELTA-T ROM data
|-
| 89 || MultiPCM ROM data
|-
| 8A || uPD7759 ROM data
|-
| 8B || OKIM6295 ROM data
|-
| 8C || K054539 ROM data
|-
| 8D || C140 ROM data
|-
| 8E || K053260 ROM data
|-
| 8F || Q-Sound ROM data
|-
| 90 || ES5505/ES5506 ROM data
|-
| 91 || X1-010 ROM data
|-
| 92 || C352 ROM data
|-
| 93 || GA20 ROM data
|-
| C0 || RF5C68 RAM write
|-
| C1 || RF5C164 RAM write
|-
| C2 || NES APU RAM write
|-
| E0 || SCSP RAM write
|-
| E1 || ES5503 RAM write
|}

Unknown ROM/RAM blocks must be skipped by the player.
Data for blocks 0x00..0x7F must be stored by the player in any case.

{{Note|The "Stream Control" system is able to use any of the 64 block types with any of the sound chips. The data block types may be used arbitrarily in these cases.<br/>
The data block type list must be followed if VGM commands 0x68 and 0x80..0x8F are used.}}

{{Note|It is valid to have multiple data blocks of the same type. The player must consolidate those into a single block of memory and keep track of the start/end offsets of the data block instances for use with VGM command 0x95.}}

= PCM RAM writes =

VGM command 0x68 specifies a PCM RAM write. These are used to write data from
data blocks to the RAM of a PCM chip. The data block format is:

  0x68 0x66 cc oo oo oo dd dd dd ss ss ss

where:
  0x68 = VGM command
  0x66 = compatibility command to make older players stop parsing the stream
  cc   = chip type (see data block types 00..3F)
  oo oo oo (24 bits) = read offset in data block
  dd dd dd (24 bits) = write offset in chip's ram (affected by chip's
                        registers)
  ss ss ss (24 bits) = size of data, in bytes
    Since size can't be zero, a size of 0 bytes means 0x0100 0000 bytes.

Unknown chip types must be skipped by the player.

== DAC Stream Control Write ==

VGM commands 0x90 to 0x95 specify writes to the DAC Stream Control Driver.
These are used to stream data from data blocks to the chips via chip writes.
To use it you must:
# Setup the Stream (set chip type and command) - this activates the stream
# Set the Stream Data Bank
# Set the Stream Frequency
# Now you can start the stream, change its frequency, start it again, stop it, etc ...

There are the following commands:

{{Note|Stream ID 0xFF is reserved and ignored unless noted otherwise.}}

Setup Stream Control:
  0x90 ss tt pp cc
      ss = Stream ID
      tt = Chip Type (see clock-order in header, e.g. YM2612 = 0x02)
            bit 7 is used to select the 2nd chip
      pp cc = write command/register cc at port pp
      Note: For chips that use Channel Select Registers (like the RF5C-family
            and the HuC6280), the format is pp cd where pp is the channel
            number, c is the channel register and d is the data register.
            If you set pp to FF, the channel select write is skipped.

Set Stream Data:
  0x91 ss dd ll bb
      ss = Stream ID
      dd = Data Bank ID (see data block types 0x00..0x3f)
      ll = Step Size (how many data is skipped after every write, usually 1)
            Set to 2, if you're using an interleaved stream (e.g. for
             left/right channel).
      bb = Step Base (data offset added to the Start Offset when starting
            stream playback, usually 0)
            If you're using an interleaved stream, set it to 0 in one stream
            and to 1 in the other one.
      Note: Step Size/Step Step are given in command-data-size
             (i.e. 1 for YM2612, 2 for PWM), not bytes

Set Stream Frequency:
  0x92 ss ff ff ff ff
      ss = Stream ID
      ff = Frequency (or Sample Rate, in Hz) at which the writes are done

Start Stream:
  0x93 ss aa aa aa aa mm ll ll ll ll
      ss = Stream ID
      aa = Data Start offset in data bank (byte offset in data bank)
            Note: if set to -1, the Data Start offset is ignored
      mm = Length Mode (how the Data Length is calculated)
            00 - ignore (just change current data position)
            01 - length = number of commands
            02 - length in msec
            03 - play until end of data
            1? - (bit 4) Reverse Mode
            8? - (bit 7) Loop (automatically restarts when finished)
      ll = Data Length

Stop Stream:
  0x94 ss
      ss = Stream ID
            Note: 0xFF stops all streams

Start Stream (fast call):
  0x95 ss bb bb ff
      ss = Stream ID
      bb = Block ID (number of the data block that is part of the data bank set
            with command 0x91)
      ff = Flags
            bit 0 - Loop (see command 0x93, mm bit 7)
            bit 4 - Reverse Mode (see command 0x93)

General Note to the DAC Stream Control:<br/>
Although it may be quite hard to press already streamed data into these
commands, it makes it very easy to write vgm-creation tools that need to stream
something. (like YM2612 DAC drums/voices/etc.)
The DAC Stream Control can use with almost all chips and is NOT limited to
chips such as YM2612 and PWM.

== Dual Chip Support ==

These chips support two instances of a chip in one vgm:
[[PSG]], [[YM2413]], [[YM2612]], [[YM2151]], [[YM2203]], [[YM2608]], [[YM2610]], [[YM3812]], [[YM3526]], [[Y8950]],
[[YMZ280B]], [[YMF262]], [[YMF278B]], [[YMF271]], [[AY8910]], [[GameBoy DMG]], [[NES APU]], [[MultiPCM]],
[[uPD7759]], [[OKIM6258]], [[OKIM6295]], [[K051649]], [[K054539]], [[HuC6280]], [[C140]], [[K053260]], [[Pokey]],
[[SCSP]], [[WonderSwan]], [[VSU]], [[SAA1099]], [[ES5503]], [[ES5506]], [[X1-010]], [[C352]], [[GA20]].   

Dual chip support is activated by setting bit 30 (0x40000000) in the chip's
clock value. (Note: The PSG needs this bit set for T6W28 mode.)

'''Dual Chip Support #1'''<br/>
The second chip instance is controlled via separate commands.

The second SN76489 PSG uses <code>0x30</code> (<code>0x3F</code> for GG Stereo).
All chips of the YM-family that use command <code>0x5n</code> use <code>0xAn</code> for the second chip. n is the last digit of the main command.
e.g. <code>0x52</code> (1st chip) -> <code>0xA2</code> (2nd chip)

'''Dual Chip Support #2'''<br/>
All other chips use bit 7 (0x80) of the first parameter byte to distinguish
between the 1st and 2nd chip. (<code>0x00-7F</code> = chip 1, <code>0x80-0xFF</code> = chip 2)

Note: The SegaPCM chip has the 2nd-chip-bit in the high byte of the address
parameter. This is the second parameter byte.

== Extra Header ==

With VGM v1.70, there was an extra header added. This one has to be placed
between the usual header and the actual VGM data.

This is the format of the extra header:

{| class="wikitable"
 ! !! 00 !! 01 !! 02 !! 03 !! 04 !! 05 !! 06 !! 07 !! 08 !! 09 !! 0A !! 0B !! 0C !! 0D !! 0E !! 0F
 |-
 ! 0x00
 | colspan="4" | Header Size    
 | colspan="4" | ChpClock Offset
 | colspan="4" | ChpVol Offset
 | || || ||
 |}

'''Header Size''' is the size of the extra header, including the length value itself. It has to be 4 or larger,
depending in the needed offsets.

Then there are two offsets that point to extra header data for:
* additional Chip Clocks for second chips
* user-defined chip volumes

'''Chip Clock Header'''

  1 byte   - Entry Count (chips with extra clocks)
  [5 bytes - List Entry 1]
  [5 bytes - List Entry 2]
  ...

Each list entry has the format:
  1 byte  - Chip ID (chip order follows the header)
  4 bytes - clock for second chip of the type above


'''Chip Volume Header'''

{{Note|This controls the balance between sound chips. A global normalization is applied after adding all sound chip volumes together.}}

  1 byte   - Entry Count (chips with user-defined volumes)
  [4 bytes - List Entry 1]
  [4 bytes - List Entry 2]
  ...

Each list entry has the format:

1 byte  - Chip ID (chip order follows the header)
{{Note|If bit 7 is set, it's the volume for a paired chip.
(e.g. the AY-part of the YM2203)}}
1 byte  - Flags
{{Note|If bit 0 is set, it's the volume for the second chip.}}
2 bytes - volume for the chip
{{Note|If Bit 15 is 0, this is an absolute volume setting.
If Bit 15 is 1, it's relative and the chip volume gets multiplied by <code>((Value & 0x7FFF) / 0x0100)</code>.}}

== History ==

{| class="wikitable"
 |-
 ! 1.00
 | Initial public release by Dave
 |-
 ! 1.01
 | Rate value added by Maxim; 1.00 files are fully compatible
 |-
 ! 1.10
 | PSG white noise feedback and shift register width parameters added by Maxim, with note on how to handle earlier version files.
Additional wait command added by Maxim with thanks to Steve Snake for the suggestion.
1.01 files are fully compatible but 1.01 players might have problems with 1.10 files, hence the 0.1 version change.
 |-
 ! 1.50
 | VGM data offset added to header by Maxim.
Data block support added by blargg, to allow for better handling of YM2612 PCM data.
Both of these changes have the potential to cause problems, but are really good changes, so the version number has been increased all the way to 1.50.
 |-
 ! 1.51
 | Sega PCM, RF5C68, YM2203, YM2608, YM2610/B, YM3812, YM3526, Y8950, YMF262, YMF278B, YMF271, YMZ280B, RF5C164, PWM and AY8910 chips and commands added.
Additional data block types RF5C68 RAM write, RF5C164 RAM write, Sega PCM ROM, YM2608 DELTA-T ROM, YM2610 ADPCM ROM, YM2610 DELTA-T ROM, YMF278B ROM, YMF271 ROM, YMF271 RAM, YMZ280B ROM and Y8950 DELTA-T ROM Data added.
Data Block Types splitted into 4 categories. (PCM Stream, compressed PCM Stream, ROM/RAM Dump, RAM write) SN76489 Flags and Loop Modifier added.
It is the first time the header size exceeds 0x40 bytes.
1.51 files are fully compatible to 1.50 players, but there may be problems with the new commands.<br/>
Note: Dual chip support was added too, but as a "cheat"-feature. The dual-chip-bits in the clock values are not compatible to 1.50, but the rest is.
All changes done by Valley Bell.
 |-
 ! 1.60
 | RF5C68, RF5C164 and PWM PCM blocks and compressed data blocks added.
A whole bunch of new commands (PCM RAM write and DAC Stream Control) added.
Volume Modifier and Loop Base added.
The new commands (especially 0x9?) may cause problems with older players.
All changes done by Valley Bell.
 |-
 ! 1.61
 | GameBoy DMG, NES APU, MultiPCM, uPD7759, OKIM6258, OKIM6295, K051649, K051649, HuC6280, C140, K053260, Pokey and Q-Sound chips added. (including necessary data blocks)
Changed number of operands from 1 to 2 for reserved commands 0x40-0x4E.
Although they're still unused, old players might handle future vgm versions wrongly.
All changes done by Valley Bell.
 |-
 ! 1.70
 | Added extra header with separate chip clocks for the second one of dual chips and chip volume adjustments.
All changes done by Valley Bell.
 |-
 ! 1.71
 | SCSP, WonderSwan, Virtual Boy VSU, SAA1099, ES5503, ES5506, Seta X1-010, Namco C352, Irem GA20 added. (including necessary ROM data blocks)
Data blocks (type 0x) for OKIM6258, HuC6280, SCSP and NES added.
VGM v1.61 players should support the data block of their respective chips despite their late addition.
Documented command 0x31 added by NewRisingSun.
All changes done by Valley Bell.
|-
! 1.72 (beta)
| Mikey chip clock and command 0x40 added.
|}

[[Category:Logged Music Formats]]
[[Category:Informational Documents]]
