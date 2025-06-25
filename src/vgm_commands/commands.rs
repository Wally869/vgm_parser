//! VGM Commands Enum Definition
//!
//! Contains the core Commands enum that represents all possible VGM sound chip commands
//! and special operations like wait commands, data blocks, and streaming control.

use super::data_blocks::DataBlockContent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub enum Commands {
    AY8910StereoMask {
        value: u8,
    },
    GameGearPSGStereo {
        value: u8,
        chip_index: u8,
    },
    PSGWrite {
        value: u8,
        chip_index: u8,
    },
    YM2413Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM2612Port0Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM2612Port1Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM2151Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM2203Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM2608Port0Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM2608Port1Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM2610Port0Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM2610Port1Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM3812Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YM3526Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    Y8950Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YMZ280BWrite {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YMF262Port0Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    YMF262Port1Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    WaitNSamples {
        n: u16,
    },
    Wait735Samples,
    Wait882Samples,
    EndOfSoundData,
    DataBlock {
        block_type: u8,
        data: DataBlockContent,
    },
    PCMRAMWrite {
        chip_type: u8,
        read_offset: u32,  // 24-bit in VGM spec
        write_offset: u32, // 24-bit in VGM spec
        size: u32,         // 24-bit in VGM spec
        data: Vec<u8>,
    },
    WaitNSamplesPlus1 {
        n: u8,
    },
    YM2612Port0Address2AWriteWait {
        n: u8,
    },
    // DAC Stream Control Commands (0x90-0x95)
    DACStreamSetupControl {
        stream_id: u8,
        chip_type: u8,
        port: u8,
        command: u8,
        chip_index: u8,
    },
    DACStreamSetData {
        stream_id: u8,
        data_bank_id: u8,
        step_size: u8,
        step_base: u8,
    },
    DACStreamSetFrequency {
        stream_id: u8,
        frequency: u32,
    },
    DACStreamStart {
        stream_id: u8,
        data_start_offset: u32,
        length_mode: u8,
        data_length: u32,
    },
    DACStreamStop {
        stream_id: u8,
    },
    DACStreamStartFast {
        stream_id: u8,
        block_id: u16,
        flags: u8,
    },
    AY8910Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    RF5C68Write {
        register: u8,
        value: u8,
    },
    RF5C164Write {
        register: u8,
        value: u8,
    },
    PWMWrite {
        register: u8,
        value: u16,
    },
    GameBoyDMGWrite {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    NESAPUWrite {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    MultiPCMWrite {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    uPD7759Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    OKIM6258Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    OKIM6295Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    HuC6280Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    K053260Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    PokeyWrite {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    WonderSwanWrite {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    SAA1099Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    ES5506Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    GA20Write {
        register: u8,
        value: u8,
        chip_index: u8,
    },
    SegaPCMWrite {
        offset: u16,
        value: u8,
    },
    MultiPCMSetBank {
        channel: u8,
        offset: u16,
    },
    QSoundWrite {
        register: u8,
        value: u16,
    },
    SCSPWrite {
        offset: u16,
        value: u8,
    },
    WonderSwanWrite16 {
        offset: u16,
        value: u8,
    },
    VSUWrite {
        offset: u16,
        value: u8,
    },
    X1010Write {
        offset: u16,
        value: u8,
    },
    YMF278BWrite {
        port: u8,
        register: u8,
        value: u8,
    },
    YMF271Write {
        port: u8,
        register: u8,
        value: u8,
    },
    SCC1Write {
        port: u8,
        register: u8,
        value: u8,
    },
    K054539Write {
        register: u16,
        value: u8,
    },
    C140Write {
        register: u16,
        value: u8,
    },
    ES5503Write {
        register: u16,
        value: u8,
    },
    ES5506Write16 {
        register: u8,
        value: u16,
    },
    SeekPCM {
        offset: u32,
    },
    C352Write {
        register: u16,
        value: u16,
    },

    // offset write
    RF5C68WriteOffset {
        offset: u16,
        value: u8,
    },
    RF5C164WriteOffset {
        offset: u16,
        value: u8,
    },
}
