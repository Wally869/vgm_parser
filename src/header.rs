use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{VgmError, VgmResult},
    traits::{VgmParser, VgmWriter},
    utils::{bcd_from_bytes, decimal_to_bcd},
};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ChipClockEntry {
    pub chip_id: u8,
    pub clock: u32,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ChipVolumeEntry {
    pub chip_id: u8,
    pub flags: u8,
    pub volume: u16,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ExtraHeaderData {
    pub header_size: u32,
    pub chip_clock_offset: u32,
    pub chip_vol_offset: u32,
    pub chip_clock_entries: Vec<ChipClockEntry>,
    pub chip_volume_entries: Vec<ChipVolumeEntry>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct HeaderData {
    pub end_of_file_offset: u32,
    pub version: u32,
    pub sn76489_clock: u32, // also called Sega PSG?

    // 0x10
    pub ym2413_clock: u32,
    pub gd3_offset: u32,
    pub total_nb_samples: u32,
    pub loop_offset: u32,

    // 0x20
    pub loop_nb_samples: u32,
    pub rate: u32,
    pub sn76489_feedback: u16,
    pub sn76489_shift_register_width: u8,
    pub sn76489_flags: u8,
    pub ym2612_clock: u32,

    // 0x30
    pub ym2151_clock: u32,
    pub vgm_data_offset: u32,
    pub sega_pcm_clock: u32,
    pub spcm_interface: u32,

    // 0x40
    pub rf5_c68_clock: u32,
    pub ym2203_clock: u32,
    pub ym2608_clock: u32,
    pub ym2610_b_clock: u32,

    // 0x50
    pub ym3812_clock: u32,
    pub ym3526_clock: u32,
    pub y8950_clock: u32,
    pub ymf262_clock: u32,

    // 0x60
    pub ymf278_b_clock: u32,
    pub ymf271_clock: u32,
    pub ymz280_b_clock: u32,
    pub rf5_c164_clock: u32,

    // 0x70
    pub pwm_clock: u32,
    pub ay8910_clock: u32,
    pub ay8910_chip_type: u8,
    pub ay8910_flags: u8,
    pub ym2203_ay8910_flags: u8,
    pub ym2608_ay8910_flags: u8,
    pub volume_modifier: u8,
    pub loop_base: u8,
    pub loop_modifier: u8,

    // 0x80
    pub gb_dmg_clock: u32,
    pub nes_apu_clock: u32,
    pub multi_pcm_clock: u32,
    pub u_pd7759_clock: u32,

    // 0x90
    pub okim6258_clock: u32,
    pub okim6258_flags: u8,
    pub k054539_flags: u8,
    pub c140_chip_type: u8,
    pub okim6295_clock: u32,
    pub k051649_k052539_clock: u32,

    // 0xA0
    pub k054539_clock: u32,
    pub hu_c6280_clock: u32,
    pub c140_clock: u32,
    pub k053260_clock: u32,

    // 0xB0
    pub pokey_clock: u32,
    pub qsound_clock: u32,
    pub scsp_clock: u32,
    pub extra_header_offset: u32,

    // 0xC0
    pub wonder_swan_clock: u32,
    pub vsu_clock: u32,
    pub saa1099_clock: u32,
    pub es5503_clock: u32,

    // 0xD0
    pub es5506_clock: u32,
    pub es5503_nb_channels: u8,
    pub es5505_es5506_nb_channels: u8,
    pub c352_clock_divider: u8,
    pub x1010_clock: u32,
    pub c352_clock: u32,

    // 0xE0
    pub ga20_clock: u32,

    // TODO: extra headers
    /// With VGM v1.70, there was an extra header added. This one has to be placed between the usual header and the actual VGM data.
    pub extra_header: ExtraHeaderData,
}

impl HeaderData {
    /// Parse VGM header with resource limits and allocation tracking
    pub fn from_bytes_with_config(data: &mut Bytes, config: &crate::ParserConfig, tracker: &mut crate::ResourceTracker) -> VgmResult<Self> {
        // Enter parsing context for depth tracking
        tracker.enter_parsing_context(config)?;
        
        let result = Self::from_bytes_internal_with_config(data, config, tracker);
        
        // Exit parsing context regardless of success/failure
        tracker.exit_parsing_context();
        
        result
    }
    
    fn from_bytes_internal_with_config(data: &mut Bytes, config: &crate::ParserConfig, _tracker: &mut crate::ResourceTracker) -> VgmResult<Self> {
        let mut header = HeaderData::default();
        // get length of data for position calculation
        let len_data = data.len();

        // validate magic
        let magic = data.get_u32();
        let magic_bytes = magic.to_be_bytes();
        if magic_bytes != *b"Vgm " {
            let expected = String::from_utf8_lossy(b"Vgm ").to_string();
            let found = String::from_utf8_lossy(&magic_bytes).to_string();
            return Err(VgmError::InvalidMagicBytes { 
                expected, 
                found, 
                offset: len_data - data.remaining() - 4 
            });
        }
        header.end_of_file_offset = data.get_u32_le();

        header.version = bcd_from_bytes(&data.get_u32().to_be_bytes()[..]); //(&data.get_u32().to_be_bytes()[..]);
        header.sn76489_clock = data.get_u32_le();

        // 0x10
        header.ym2413_clock = data.get_u32_le();
        header.gd3_offset = data.get_u32_le();
        header.total_nb_samples = data.get_u32_le();
        header.loop_offset = data.get_u32_le();

        // 0x20
        header.loop_nb_samples = data.get_u32_le();
        header.rate = data.get_u32_le();
        header.sn76489_feedback = data.get_u16_le();
        header.sn76489_shift_register_width = data.get_u8();
        header.sn76489_flags = data.get_u8();
        header.ym2612_clock = data.get_u32_le();

        // 0x30
        header.ym2151_clock = data.get_u32_le();
        header.vgm_data_offset = data.get_u32_le();
        header.sega_pcm_clock = data.get_u32_le();
        header.spcm_interface = data.get_u32_le();

        // Security: Prevent integer overflow in VGM data position calculation
        let pos_start_vgm = header.vgm_data_offset
            .checked_add(0x34)
            .ok_or(VgmError::IntegerOverflow {
                operation: "VGM data position calculation".to_string(),
                details: format!("vgm_data_offset {} + 0x34", header.vgm_data_offset),
            })?;
        
        // Security: Convert pos_start_vgm to usize safely
        let pos_start_vgm_usize = usize::try_from(pos_start_vgm)
            .map_err(|_| VgmError::IntegerOverflow {
                operation: "VGM position usize conversion".to_string(),
                details: format!("pos_start_vgm {} cannot fit in usize", pos_start_vgm),
            })?;

        // 0x40
        // From here, need to check if is still header, or start of vgm data
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.rf5_c68_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2203_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2608_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2610_b_clock = data.get_u32_le();

        // 0x50
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym3812_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym3526_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.y8950_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ymf262_clock = data.get_u32_le();

        // 0x60
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ymf278_b_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ymf271_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ymz280_b_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.rf5_c164_clock = data.get_u32_le();

        // 0x70
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.pwm_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ay8910_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ay8910_chip_type = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2203_ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2608_ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.volume_modifier = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.loop_base = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.loop_modifier = data.get_u8();

        // 0x80
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.gb_dmg_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.nes_apu_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.multi_pcm_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.u_pd7759_clock = data.get_u32_le();

        // 0x90
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.okim6258_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.okim6258_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.k054539_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.c140_chip_type = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.okim6295_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.k051649_k052539_clock = data.get_u32_le();

        // 0xA0
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.k054539_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.hu_c6280_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.c140_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.k053260_clock = data.get_u32_le();

        // 0xB0
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.pokey_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.qsound_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.scsp_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.extra_header_offset = data.get_u32_le();

        let pos_extra_header = if header.extra_header_offset == 0 {
            None
        } else {
            // Security: Prevent integer overflow in extra header position calculation
            Some(header.extra_header_offset
                .checked_add(0xBC)
                .and_then(|v| usize::try_from(v).ok())
                .ok_or(VgmError::IntegerOverflow {
                    operation: "Extra header position calculation".to_string(),
                    details: format!("extra_header_offset {} + 0xBC", header.extra_header_offset),
                })?)
        };

        // 0xC0
        // from here need to also check for extra header data
        // can assume that after extra header is vgm data?
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.wonder_swan_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.vsu_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.saa1099_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.es5503_clock = data.get_u32_le();

        // 0xD0
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.es5506_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.es5503_nb_channels = data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.es5505_es5506_nb_channels = data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.c352_clock_divider = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.x1010_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.c352_clock = data.get_u32_le();

        // 0xE0
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header_with_config(data, pos_extra_header, config)?;
                return Ok(header);
            }
        }
        header.ga20_clock = data.get_u32_le();

        Ok(header)
    }
    
    fn parse_extra_header_with_config(&mut self, data: &mut Bytes, extra_header_pos: usize, config: &crate::ParserConfig) -> VgmResult<()> {
        // use this to track pos in the extra header?
        let remaining_bytes = data.remaining();

        let mut extra_header = ExtraHeaderData::default();

        extra_header.header_size = data.get_u32_le();
        extra_header.chip_clock_offset = data.get_u32_le();
        extra_header.chip_vol_offset = data.get_u32_le();

        // should be options, no guarantee that both are set
        let chip_clock_pos = if extra_header.chip_clock_offset == 0 {
            None
        } else {
            // Security: Prevent integer overflow in chip clock position calculation
            Some(extra_header_pos
                .checked_add(4)
                .and_then(|v| v.checked_add(extra_header.chip_clock_offset as usize))
                .ok_or(VgmError::IntegerOverflow {
                    operation: "Chip clock position calculation".to_string(),
                    details: format!("extra_header_pos {} + 4 + chip_clock_offset {}", extra_header_pos, extra_header.chip_clock_offset),
                })?)
        };

        let chip_vol_pos = if extra_header.chip_vol_offset == 0 {
            None
        } else {
            // Security: Prevent integer overflow in chip volume position calculation
            Some(extra_header_pos
                .checked_add(8)
                .and_then(|v| v.checked_add(extra_header.chip_vol_offset as usize))
                .ok_or(VgmError::IntegerOverflow {
                    operation: "Chip volume position calculation".to_string(),
                    details: format!("extra_header_pos {} + 8 + chip_vol_offset {}", extra_header_pos, extra_header.chip_vol_offset),
                })?)
        };

        let mut chip_clock_entries: Vec<ChipClockEntry> = vec![];
        let mut chip_vol_entries: Vec<ChipVolumeEntry> = vec![];
        // iter twice, no guarantees on ordering of both headers here?
        // should be contiguous
        for _ in 0..2 {
            let curr_pos = extra_header_pos + remaining_bytes - data.remaining();
            if let Some(chip_clock_pos) = chip_clock_pos {
                if chip_clock_pos == curr_pos {
                    let nb_entries = data.get_u8();
                    
                    // Security: Check chip clock entry count against config limits
                    config.check_chip_entries(nb_entries, 0)?;
                    
                    for _i in 0..nb_entries {
                        let curr_entry = ChipClockEntry {
                            chip_id: data.get_u8(),
                            clock: data.get_u32_le(),
                        };

                        chip_clock_entries.push(curr_entry);
                    }
                }
            }

            if let Some(chip_vol_pos) = chip_vol_pos {
                if chip_vol_pos == curr_pos {
                    let nb_entries = data.get_u8();
                    
                    // Security: Check chip volume entry count against config limits  
                    config.check_chip_entries(0, nb_entries)?;
                    
                    for _i in 0..nb_entries {
                        let curr_entry = ChipVolumeEntry {
                            chip_id: data.get_u8(),
                            flags: data.get_u8(),
                            volume: data.get_u16_le(),
                        };

                        chip_vol_entries.push(curr_entry);
                    }
                }
            }
        }

        extra_header.chip_clock_entries = chip_clock_entries;
        extra_header.chip_volume_entries = chip_vol_entries;

        self.extra_header = extra_header;
        Ok(())
    }

    fn parse_extra_header(&mut self, data: &mut Bytes, extra_header_pos: usize) -> VgmResult<()> {
        // use this to track pos in the extra header?
        let remaining_bytes = data.remaining();

        let mut extra_header = ExtraHeaderData::default();

        extra_header.header_size = data.get_u32_le();
        extra_header.chip_clock_offset = data.get_u32_le();
        extra_header.chip_vol_offset = data.get_u32_le();

        // should be options, no guarantee that both are set
        let chip_clock_pos = if extra_header.chip_clock_offset == 0 {
            None
        } else {
            // Security: Prevent integer overflow in chip clock position calculation
            Some(extra_header_pos
                .checked_add(4)
                .and_then(|v| v.checked_add(extra_header.chip_clock_offset as usize))
                .ok_or(VgmError::IntegerOverflow {
                    operation: "Chip clock position calculation".to_string(),
                    details: format!("extra_header_pos {} + 4 + chip_clock_offset {}", extra_header_pos, extra_header.chip_clock_offset),
                })?)
        };

        let chip_vol_pos = if extra_header.chip_vol_offset == 0 {
            None
        } else {
            // Security: Prevent integer overflow in chip volume position calculation
            Some(extra_header_pos
                .checked_add(8)
                .and_then(|v| v.checked_add(extra_header.chip_vol_offset as usize))
                .ok_or(VgmError::IntegerOverflow {
                    operation: "Chip volume position calculation".to_string(),
                    details: format!("extra_header_pos {} + 8 + chip_vol_offset {}", extra_header_pos, extra_header.chip_vol_offset),
                })?)
        };

        let mut chip_clock_entries: Vec<ChipClockEntry> = vec![];
        let mut chip_vol_entries: Vec<ChipVolumeEntry> = vec![];
        // iter twice, no guarantees on ordering of both headers here?
        // should be contiguous
        for _ in 0..2 {
            let curr_pos = extra_header_pos + remaining_bytes - data.remaining();
            if let Some(chip_clock_pos) = chip_clock_pos {
                if chip_clock_pos == curr_pos {
                    let nb_entries = data.get_u8();
                    for _i in 0..nb_entries {
                        let curr_entry = ChipClockEntry {
                            chip_id: data.get_u8(),
                            clock: data.get_u32_le(),
                        };

                        chip_clock_entries.push(curr_entry);
                    }
                }
            }

            if let Some(chip_vol_pos) = chip_vol_pos {
                if chip_vol_pos == curr_pos {
                    let nb_entries = data.get_u8();
                    for _i in 0..nb_entries {
                        let curr_entry = ChipVolumeEntry {
                            chip_id: data.get_u8(),
                            flags: data.get_u8(),
                            volume: data.get_u16_le(),
                        };

                        chip_vol_entries.push(curr_entry);
                    }
                }
            }
        }

        extra_header.chip_clock_entries = chip_clock_entries;
        extra_header.chip_volume_entries = chip_vol_entries;

        self.extra_header = extra_header;
        Ok(())
    }

    fn write_extra_header(&self, buffer: &mut BytesMut, vgm_data_pos: usize) {
        // write header
        buffer.put(&self.extra_header.header_size.to_le_bytes()[..]);
        buffer.put(&self.extra_header.chip_clock_offset.to_le_bytes()[..]);
        buffer.put(&self.extra_header.chip_vol_offset.to_le_bytes()[..]);

        if self.extra_header.chip_clock_offset != 0 {
            if self.extra_header.chip_vol_offset == 0 {
                // just write the chip clocks
                // nb entries
                buffer.put(&(self.extra_header.chip_clock_entries.len() as u8).to_le_bytes()[..]);
                for chip_entry in &self.extra_header.chip_clock_entries {
                    buffer.put(&chip_entry.chip_id.to_le_bytes()[..]);
                    buffer.put(&chip_entry.clock.to_le_bytes()[..]);
                }
            } else {
                // volume and clocks are defined, need to check which goes first
                // we assume that there is no space between offset definition and chip clock / chip vol headers
                // so can check offset values directly
                if self.extra_header.chip_vol_offset == 4 {
                    // chip vol directly
                    buffer.put(
                        &(self.extra_header.chip_volume_entries.len() as u8).to_le_bytes()[..],
                    );
                    for chip_entry in &self.extra_header.chip_volume_entries {
                        buffer.put(&chip_entry.chip_id.to_le_bytes()[..]);
                        buffer.put(&chip_entry.flags.to_le_bytes()[..]);
                        buffer.put(&chip_entry.volume.to_le_bytes()[..])
                    }

                    // then chip clock
                    buffer
                        .put(&(self.extra_header.chip_clock_entries.len() as u8).to_le_bytes()[..]);
                    for chip_entry in &self.extra_header.chip_clock_entries {
                        buffer.put(&chip_entry.chip_id.to_le_bytes()[..]);
                        buffer.put(&chip_entry.clock.to_le_bytes()[..]);
                    }
                } else {
                    // chip clock directly
                    buffer
                        .put(&(self.extra_header.chip_clock_entries.len() as u8).to_le_bytes()[..]);
                    for chip_entry in &self.extra_header.chip_clock_entries {
                        buffer.put(&chip_entry.chip_id.to_le_bytes()[..]);
                        buffer.put(&chip_entry.clock.to_le_bytes()[..]);
                    }

                    // then chip vol
                    buffer.put(
                        &(self.extra_header.chip_volume_entries.len() as u8).to_le_bytes()[..],
                    );
                    for chip_entry in &self.extra_header.chip_volume_entries {
                        buffer.put(&chip_entry.chip_id.to_le_bytes()[..]);
                        buffer.put(&chip_entry.flags.to_le_bytes()[..]);
                        buffer.put(&chip_entry.volume.to_le_bytes()[..])
                    }
                }
            }
        } else {
            // shouldn't be an extra header if nothing in the extra header, but let's be safe
            if self.extra_header.chip_vol_offset != 0 {
                buffer.put(&(self.extra_header.chip_volume_entries.len() as u8).to_le_bytes()[..]);
                for chip_entry in &self.extra_header.chip_volume_entries {
                    buffer.put(&chip_entry.chip_id.to_le_bytes()[..]);
                    buffer.put(&chip_entry.flags.to_le_bytes()[..]);
                    buffer.put(&chip_entry.volume.to_le_bytes()[..])
                }
            }
        }

        // pad until start of VGM?
        while buffer.len() < vgm_data_pos {
            buffer.put(&[0x00][..]);
        }
    }
}

impl VgmParser for HeaderData {
    /// Read header data
    /// From 1.5 onwards, any length of header is valid as long as it is at least 64 bytes long
    fn from_bytes(data: &mut Bytes) -> VgmResult<Self> {
        let mut header = HeaderData::default();
        // get length of data for position calculation
        let len_data = data.len();

        // validate magic
        let magic = data.get_u32();
        let magic_bytes = magic.to_be_bytes();
        if magic_bytes != *b"Vgm " {
            let expected = String::from_utf8_lossy(b"Vgm ").to_string();
            let found = String::from_utf8_lossy(&magic_bytes).to_string();
            return Err(VgmError::InvalidMagicBytes { 
                expected, 
                found, 
                offset: len_data - data.remaining() - 4 
            });
        }
        header.end_of_file_offset = data.get_u32_le();

        header.version = bcd_from_bytes(&data.get_u32().to_be_bytes()[..]); //(&data.get_u32().to_be_bytes()[..]);
        header.sn76489_clock = data.get_u32_le();

        // 0x10
        header.ym2413_clock = data.get_u32_le();
        header.gd3_offset = data.get_u32_le();
        header.total_nb_samples = data.get_u32_le();
        header.loop_offset = data.get_u32_le();

        // 0x20
        header.loop_nb_samples = data.get_u32_le();
        header.rate = data.get_u32_le();
        header.sn76489_feedback = data.get_u16_le();
        header.sn76489_shift_register_width = data.get_u8();
        header.sn76489_flags = data.get_u8();
        header.ym2612_clock = data.get_u32_le();

        // 0x30
        header.ym2151_clock = data.get_u32_le();
        header.vgm_data_offset = data.get_u32_le();
        header.sega_pcm_clock = data.get_u32_le();
        header.spcm_interface = data.get_u32_le();

        // Security: Prevent integer overflow in VGM data position calculation
        let pos_start_vgm = header.vgm_data_offset
            .checked_add(0x34)
            .ok_or(VgmError::IntegerOverflow {
                operation: "VGM data position calculation".to_string(),
                details: format!("vgm_data_offset {} + 0x34", header.vgm_data_offset),
            })?;
        
        // Security: Convert pos_start_vgm to usize safely
        let pos_start_vgm_usize = usize::try_from(pos_start_vgm)
            .map_err(|_| VgmError::IntegerOverflow {
                operation: "VGM position usize conversion".to_string(),
                details: format!("pos_start_vgm {} cannot fit in usize", pos_start_vgm),
            })?;

        // 0x40
        // From here, need to check if is still header, or start of vgm data
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.rf5_c68_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2203_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2608_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2610_b_clock = data.get_u32_le();

        // 0x50
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym3812_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym3526_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.y8950_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ymf262_clock = data.get_u32_le();

        // 0x60
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ymf278_b_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ymf271_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ymz280_b_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.rf5_c164_clock = data.get_u32_le();

        // 0x70
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.pwm_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ay8910_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ay8910_chip_type = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2203_ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.ym2608_ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.volume_modifier = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.loop_base = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.loop_modifier = data.get_u8();

        // 0x80
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.gb_dmg_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.nes_apu_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.multi_pcm_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.u_pd7759_clock = data.get_u32_le();

        // 0x90
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.okim6258_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.okim6258_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.k054539_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.c140_chip_type = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.okim6295_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.k051649_k052539_clock = data.get_u32_le();

        // 0xA0
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.k054539_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.hu_c6280_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.c140_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.k053260_clock = data.get_u32_le();

        // 0xB0
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.pokey_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.qsound_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.scsp_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        }
        header.extra_header_offset = data.get_u32_le();

        let pos_extra_header = if header.extra_header_offset == 0 {
            None
        } else {
            // Security: Prevent integer overflow in extra header position calculation
            Some(header.extra_header_offset
                .checked_add(0xBC)
                .and_then(|v| usize::try_from(v).ok())
                .ok_or(VgmError::IntegerOverflow {
                    operation: "Extra header position calculation".to_string(),
                    details: format!("extra_header_offset {} + 0xBC", header.extra_header_offset),
                })?)
        };

        // 0xC0
        // from here need to also check for extra header data
        // can assume that after extra header is vgm data?
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.wonder_swan_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.vsu_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.saa1099_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.es5503_clock = data.get_u32_le();

        // 0xD0
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.es5506_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.es5503_nb_channels = data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.es5505_es5506_nb_channels = data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.c352_clock_divider = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.x1010_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.c352_clock = data.get_u32_le();

        // 0xE0
        if (len_data - data.remaining()) == pos_start_vgm_usize {
            return Ok(header);
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header)?;
                return Ok(header);
            }
        }
        header.ga20_clock = data.get_u32_le();

        Ok(header)
    }
}

impl VgmWriter for HeaderData {
    fn to_bytes(&self, buffer: &mut BytesMut) -> VgmResult<()> {
        let vgm_data_pos = (self.vgm_data_offset + 0x34) as usize;
        let extra_header_pos = if self.extra_header_offset == 0 {
            None
        } else {
            Some((self.extra_header_offset + 0xBC) as usize)
        };

        buffer.put(&b"Vgm "[..]);
        buffer.put(&self.end_of_file_offset.to_le_bytes()[..]);
        buffer.put(&decimal_to_bcd(self.version)[..]);

        buffer.put(&self.sn76489_clock.to_le_bytes()[..]);

        // 0x10
        buffer.put(&self.ym2413_clock.to_le_bytes()[..]);
        buffer.put(&self.gd3_offset.to_le_bytes()[..]);
        buffer.put(&self.total_nb_samples.to_le_bytes()[..]);
        buffer.put(&self.loop_offset.to_le_bytes()[..]);

        // 0x20
        buffer.put(&self.loop_nb_samples.to_le_bytes()[..]);
        buffer.put(&self.rate.to_le_bytes()[..]);
        buffer.put(&self.sn76489_feedback.to_le_bytes()[..]);
        buffer.put(&self.sn76489_shift_register_width.to_le_bytes()[..]);
        buffer.put(&self.sn76489_flags.to_le_bytes()[..]);
        buffer.put(&self.ym2612_clock.to_le_bytes()[..]);

        // 0x30
        buffer.put(&self.ym2151_clock.to_le_bytes()[..]);
        buffer.put(&self.vgm_data_offset.to_le_bytes()[..]);
        buffer.put(&self.sega_pcm_clock.to_le_bytes()[..]);
        buffer.put(&self.spcm_interface.to_le_bytes()[..]);

        // 0x40
        // From here, need to check if is still header, or start of vgm data
        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.rf5_c68_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ym2203_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ym2608_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ym2610_b_clock.to_le_bytes()[..]);

        // 0x50
        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ym3812_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ym3526_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.y8950_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ymf262_clock.to_le_bytes()[..]);

        // 0x60
        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ymf278_b_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ymf271_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ymz280_b_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.rf5_c164_clock.to_le_bytes()[..]);

        // 0x70
        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.pwm_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ay8910_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ay8910_chip_type.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ay8910_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ym2203_ay8910_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.ym2608_ay8910_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.volume_modifier.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&[0x00][..]); // reserved

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.loop_base.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.loop_modifier.to_le_bytes()[..]);

        // 0x80
        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.gb_dmg_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.nes_apu_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.multi_pcm_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.u_pd7759_clock.to_le_bytes()[..]);

        // 0x90
        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.okim6258_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.okim6258_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.k054539_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.c140_chip_type.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&[0x00][..]); // reserved

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.okim6295_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.k051649_k052539_clock.to_le_bytes()[..]);

        // 0xA0
        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.k054539_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.hu_c6280_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.c140_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.k053260_clock.to_le_bytes()[..]);

        // 0xB0
        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.pokey_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.qsound_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.scsp_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        }
        buffer.put(&self.extra_header_offset.to_le_bytes()[..]);

        // 0xC0
        // from here need to also check for extra header data
        // can assume that after extra header is vgm data?
        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.wonder_swan_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.vsu_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.saa1099_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.es5503_clock.to_le_bytes()[..]);

        // 0xD0
        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.es5506_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.es5503_nb_channels.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.es5505_es5506_nb_channels.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.c352_clock_divider.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&[0x00][..]); // reserved

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.x1010_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.c352_clock.to_le_bytes()[..]);

        // 0xE0
        if buffer.len() == vgm_data_pos {
            return Ok(());
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return Ok(());
            }
        }
        buffer.put(&self.ga20_clock.to_le_bytes()[..]);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use bytes::{Bytes, BytesMut};

    use crate::traits::{VgmParser, VgmWriter};

    use super::HeaderData;

    /// Get project root directory for test file paths
    fn get_project_root() -> PathBuf {
        // Try to find project root by looking for Cargo.toml
        let mut current = std::env::current_dir().expect("Failed to get current directory");
        loop {
            if current.join("Cargo.toml").exists() {
                return current;
            }
            if !current.pop() {
                // If we can't find Cargo.toml, assume current directory is project root
                return std::env::current_dir().expect("Failed to get current directory");
            }
        }
    }

    /// Get path relative to project root
    fn project_path(relative_path: &str) -> PathBuf {
        get_project_root().join(relative_path)
    }

    #[test]
    fn header_170() {
        // Use project-relative paths
        let filename = project_path("vgm_files/Into Battle.vgm");
        
        // Skip test if file doesn't exist
        if !filename.exists() {
            println!("Skipping header_170 test - test VGM file not found at {:?}", filename);
            return;
        }
        
        let data = match fs::read(&filename) {
            Ok(data) => data,
            Err(e) => {
                println!("Skipping header_170 test - failed to read file {:?}: {}", filename, e);
                return;
            }
        };
        
        let mut mem = Bytes::from(data.clone());

        let header = match HeaderData::from_bytes(&mut mem) {
            Ok(header) => header,
            Err(e) => {
                println!("Skipping header_170 test - failed to parse header: {}", e);
                return;
            }
        };
        println!("clock: {}", header.ym2608_clock);

        let mut out_buffer = BytesMut::new();
        match header.to_bytes(&mut out_buffer) {
            Ok(()) => {},
            Err(e) => {
                println!("Skipping header_170 test - failed to serialize header: {}", e);
                return;
            }
        };

        // Ensure generated directory exists before writing
        let generated_dir = project_path("generated");
        if let Err(e) = fs::create_dir_all(&generated_dir) {
            println!("Warning: Could not create generated directory {:?}: {}", generated_dir, e);
            return;
        }

        // Write output files with better error handling
        let bin_path = project_path("generated/Into Battle.bin");
        if let Err(e) = fs::write(&bin_path, &out_buffer) {
            println!("Warning: Could not write binary file {:?}: {}", bin_path, e);
        }
        
        let json_path = project_path("generated/Into Battle.json");
        match serde_json::to_string(&header) {
            Ok(json_str) => {
                if let Err(e) = fs::write(&json_path, json_str) {
                    println!("Warning: Could not write JSON file {:?}: {}", json_path, e);
                }
            }
            Err(e) => {
                println!("Warning: Could not serialize header to JSON: {}", e);
            }
        }
    }
}

// Validation implementation for HeaderData
use crate::validation::{VgmValidate, ValidationContext, ChipValidator, OffsetValidator};

impl VgmValidate for HeaderData {
    fn validate(&self, context: &ValidationContext) -> crate::errors::VgmResult<()> {
        // Validate chip clocks
        ChipValidator::validate_chip_clocks(self)?;
        
        // Validate chip volumes
        ChipValidator::validate_chip_volumes(self)?;
        
        // Validate offsets against file size
        if self.gd3_offset > 0 {
            OffsetValidator::validate_offset(self.gd3_offset + 0x14, context.file_size, "gd3_offset")?;
        }
        
        if self.vgm_data_offset > 0 {
            OffsetValidator::validate_offset(self.vgm_data_offset + 0x34, context.file_size, "vgm_data_offset")?;
        }
        
        if self.loop_offset > 0 {
            OffsetValidator::validate_offset(self.loop_offset + 0x1C, context.file_size, "loop_offset")?;
        }
        
        if self.extra_header_offset > 0 {
            OffsetValidator::validate_offset(self.extra_header_offset + 0xBC, context.file_size, "extra_header_offset")?;
        }
        
        // Validate sample counts are reasonable
        if self.total_nb_samples > 0 && self.rate > 0 {
            let duration_seconds = self.total_nb_samples as f64 / self.rate as f64;
            if duration_seconds > 3600.0 { // More than 1 hour
                return Err(crate::errors::VgmError::ValidationFailed {
                    field: "total_nb_samples".to_string(),
                    reason: format!("Duration {:.1} seconds exceeds reasonable limit", duration_seconds),
                });
            }
        }
        
        // Validate loop data consistency
        if self.loop_offset > 0 && self.loop_nb_samples == 0 {
            return Err(crate::errors::VgmError::InconsistentData {
                context: "Loop configuration".to_string(),
                reason: "Loop offset specified but loop sample count is zero".to_string(),
            });
        }
        
        // Validate rate is reasonable
        if self.rate > 0 && (self.rate < 8000 || self.rate > 192000) {
            return Err(crate::errors::VgmError::ValidationFailed {
                field: "rate".to_string(),
                reason: format!("Sample rate {} Hz outside valid range 8000-192000 Hz", self.rate),
            });
        }
        
        Ok(())
    }
}
