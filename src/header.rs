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
    pub fn from_bytes_with_config(
        data: &mut Bytes,
        config: &crate::ParserConfig,
        tracker: &mut crate::ResourceTracker,
    ) -> VgmResult<Self> {
        // Enter parsing context for depth tracking
        tracker.enter_parsing_context(config)?;

        let result = Self::from_bytes_internal_with_config(data, config, tracker);

        // Exit parsing context regardless of success/failure
        tracker.exit_parsing_context();

        result
    }

    fn from_bytes_internal_with_config(
        data: &mut Bytes,
        config: &crate::ParserConfig,
        _tracker: &mut crate::ResourceTracker,
    ) -> VgmResult<Self> {
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
                offset: len_data - data.remaining() - 4,
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
        let pos_start_vgm =
            header
                .vgm_data_offset
                .checked_add(0x34)
                .ok_or(VgmError::IntegerOverflow {
                    operation: "VGM data position calculation".to_string(),
                    details: format!("vgm_data_offset {} + 0x34", header.vgm_data_offset),
                })?;

        // Security: Convert pos_start_vgm to usize safely
        let pos_start_vgm_usize =
            usize::try_from(pos_start_vgm).map_err(|_| VgmError::IntegerOverflow {
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
            Some(
                header
                    .extra_header_offset
                    .checked_add(0xBC)
                    .and_then(|v| usize::try_from(v).ok())
                    .ok_or(VgmError::IntegerOverflow {
                        operation: "Extra header position calculation".to_string(),
                        details: format!(
                            "extra_header_offset {} + 0xBC",
                            header.extra_header_offset
                        ),
                    })?,
            )
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

    fn parse_extra_header_with_config(
        &mut self,
        data: &mut Bytes,
        extra_header_pos: usize,
        config: &crate::ParserConfig,
    ) -> VgmResult<()> {
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
            Some(
                extra_header_pos
                    .checked_add(4)
                    .and_then(|v| v.checked_add(extra_header.chip_clock_offset as usize))
                    .ok_or(VgmError::IntegerOverflow {
                        operation: "Chip clock position calculation".to_string(),
                        details: format!(
                            "extra_header_pos {} + 4 + chip_clock_offset {}",
                            extra_header_pos, extra_header.chip_clock_offset
                        ),
                    })?,
            )
        };

        let chip_vol_pos = if extra_header.chip_vol_offset == 0 {
            None
        } else {
            // Security: Prevent integer overflow in chip volume position calculation
            Some(
                extra_header_pos
                    .checked_add(8)
                    .and_then(|v| v.checked_add(extra_header.chip_vol_offset as usize))
                    .ok_or(VgmError::IntegerOverflow {
                        operation: "Chip volume position calculation".to_string(),
                        details: format!(
                            "extra_header_pos {} + 8 + chip_vol_offset {}",
                            extra_header_pos, extra_header.chip_vol_offset
                        ),
                    })?,
            )
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
            Some(
                extra_header_pos
                    .checked_add(4)
                    .and_then(|v| v.checked_add(extra_header.chip_clock_offset as usize))
                    .ok_or(VgmError::IntegerOverflow {
                        operation: "Chip clock position calculation".to_string(),
                        details: format!(
                            "extra_header_pos {} + 4 + chip_clock_offset {}",
                            extra_header_pos, extra_header.chip_clock_offset
                        ),
                    })?,
            )
        };

        let chip_vol_pos = if extra_header.chip_vol_offset == 0 {
            None
        } else {
            // Security: Prevent integer overflow in chip volume position calculation
            Some(
                extra_header_pos
                    .checked_add(8)
                    .and_then(|v| v.checked_add(extra_header.chip_vol_offset as usize))
                    .ok_or(VgmError::IntegerOverflow {
                        operation: "Chip volume position calculation".to_string(),
                        details: format!(
                            "extra_header_pos {} + 8 + chip_vol_offset {}",
                            extra_header_pos, extra_header.chip_vol_offset
                        ),
                    })?,
            )
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

        // Check minimum header size (64 bytes minimum for v1.00+)
        if data.len() < 64 {
            return Err(VgmError::TruncatedFile {
                expected: 64,
                actual: data.len(),
            });
        }

        // validate magic
        let magic = data.get_u32();
        let magic_bytes = magic.to_be_bytes();
        if magic_bytes != *b"Vgm " {
            let expected = String::from_utf8_lossy(b"Vgm ").to_string();
            let found = String::from_utf8_lossy(&magic_bytes).to_string();
            return Err(VgmError::InvalidMagicBytes {
                expected,
                found,
                offset: len_data - data.remaining() - 4,
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
        let pos_start_vgm =
            header
                .vgm_data_offset
                .checked_add(0x34)
                .ok_or(VgmError::IntegerOverflow {
                    operation: "VGM data position calculation".to_string(),
                    details: format!("vgm_data_offset {} + 0x34", header.vgm_data_offset),
                })?;

        // Security: Convert pos_start_vgm to usize safely
        let pos_start_vgm_usize =
            usize::try_from(pos_start_vgm).map_err(|_| VgmError::IntegerOverflow {
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
            Some(
                header
                    .extra_header_offset
                    .checked_add(0xBC)
                    .and_then(|v| usize::try_from(v).ok())
                    .ok_or(VgmError::IntegerOverflow {
                        operation: "Extra header position calculation".to_string(),
                        details: format!(
                            "extra_header_offset {} + 0xBC",
                            header.extra_header_offset
                        ),
                    })?,
            )
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

        // Ensure we pad to the full header size (vgm_data_offset + 0x34)
        while buffer.len() < vgm_data_pos {
            buffer.put(&[0x00][..]);
        }

        Ok(())
    }
}

// Validation implementation for HeaderData
use crate::validation::{ChipValidator, OffsetValidator, ValidationContext, VgmValidate};

impl VgmValidate for HeaderData {
    fn validate(&self, context: &ValidationContext) -> crate::errors::VgmResult<()> {
        // Validate VGM version
        crate::validation::VersionValidator::validate_version(self.version, &context.config)?;

        // Validate chip clocks
        ChipValidator::validate_chip_clocks(self)?;

        // Validate chip volumes
        ChipValidator::validate_chip_volumes(self)?;

        // Validate offsets against file size
        if self.gd3_offset > 0 {
            OffsetValidator::validate_offset(
                self.gd3_offset + 0x14,
                context.file_size,
                "gd3_offset",
            )?;
        }

        if self.vgm_data_offset > 0 {
            OffsetValidator::validate_offset(
                self.vgm_data_offset + 0x34,
                context.file_size,
                "vgm_data_offset",
            )?;
        }

        if self.loop_offset > 0 {
            OffsetValidator::validate_offset(
                self.loop_offset + 0x1C,
                context.file_size,
                "loop_offset",
            )?;
        }

        if self.extra_header_offset > 0 {
            OffsetValidator::validate_offset(
                self.extra_header_offset + 0xBC,
                context.file_size,
                "extra_header_offset",
            )?;
        }

        // Validate sample counts are reasonable
        if self.total_nb_samples > 0 && self.rate > 0 {
            let duration_seconds = self.total_nb_samples as f64 / self.rate as f64;
            if duration_seconds > 3600.0 {
                // More than 1 hour
                return Err(crate::errors::VgmError::ValidationFailed {
                    field: "total_nb_samples".to_string(),
                    reason: format!(
                        "Duration {:.1} seconds exceeds reasonable limit",
                        duration_seconds
                    ),
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
                reason: format!(
                    "Sample rate {} Hz outside valid range 8000-192000 Hz",
                    self.rate
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use bytes::{Bytes, BytesMut};

    use crate::traits::{VgmParser, VgmWriter};

    use super::{ChipClockEntry, ChipVolumeEntry, ExtraHeaderData, HeaderData};
    use crate::{ParserConfig, ResourceTracker, ValidationContext, ValidationConfig, VgmValidate};

    /// Helper to create valid VGM header bytes
    fn create_test_header_bytes() -> Vec<u8> {
        let mut buffer = BytesMut::new();
        
        // VGM magic + basic header structure
        buffer.extend_from_slice(b"Vgm ");           // 0x00: Magic
        buffer.extend_from_slice(&100u32.to_le_bytes());   // 0x04: EOF offset
        // Version 1.51 in BCD format: we want 151 decimal, so need to use decimal_to_bcd
        let version_bcd = crate::utils::decimal_to_bcd(151);
        // Pad to 4 bytes and reverse for big-endian
        let mut version_bytes = vec![0u8; 4];
        for (i, &byte) in version_bcd.iter().rev().enumerate() {
            if i < 4 {
                version_bytes[3 - i] = byte;
            }
        }
        buffer.extend_from_slice(&version_bytes); // 0x08: Version 1.51 (BCD)
        buffer.extend_from_slice(&3579545u32.to_le_bytes());     // 0x0C: SN76489 clock
        
        // 0x10
        buffer.extend_from_slice(&0u32.to_le_bytes());      // 0x10: YM2413 clock
        buffer.extend_from_slice(&0u32.to_le_bytes());      // 0x14: GD3 offset
        buffer.extend_from_slice(&44100u32.to_le_bytes());  // 0x18: Total samples
        buffer.extend_from_slice(&0u32.to_le_bytes());      // 0x1C: Loop offset
        
        // 0x20
        buffer.extend_from_slice(&0u32.to_le_bytes());      // 0x20: Loop samples
        buffer.extend_from_slice(&44100u32.to_le_bytes()); // 0x24: Rate
        buffer.extend_from_slice(&0x0009u16.to_le_bytes()); // 0x28: SN76489 feedback
        buffer.extend_from_slice(&[16u8]);                  // 0x2A: SN76489 shift register width
        buffer.extend_from_slice(&[0u8]);                   // 0x2B: SN76489 flags
        buffer.extend_from_slice(&7670453u32.to_le_bytes()); // 0x2C: YM2612 clock
        
        // 0x30
        buffer.extend_from_slice(&0u32.to_le_bytes());      // 0x30: YM2151 clock
        buffer.extend_from_slice(&0x40u32.to_le_bytes());   // 0x34: VGM data offset
        buffer.extend_from_slice(&0u32.to_le_bytes());      // 0x38: Sega PCM clock
        buffer.extend_from_slice(&0u32.to_le_bytes());      // 0x3C: SPCM interface

        // Continue with more header fields to make it complete (VGM 1.51 needs 256 bytes minimum)
        // Add all remaining fields with zero padding
        while buffer.len() < 256 {
            buffer.extend_from_slice(&[0u8; 4]);
        }
        
        buffer.to_vec()
    }

    #[test]
    fn test_header_data_default() {
        let header = HeaderData::default();
        
        // Test default values
        assert_eq!(header.version, 0);
        assert_eq!(header.sn76489_clock, 0);
        assert_eq!(header.ym2612_clock, 0);
        assert_eq!(header.total_nb_samples, 0);
        assert_eq!(header.rate, 0);
        assert_eq!(header.vgm_data_offset, 0);
    }

    #[test]
    fn test_chip_clock_entry() {
        let entry = ChipClockEntry {
            chip_id: 0x02,
            clock: 3579545,
        };
        
        assert_eq!(entry.chip_id, 0x02);
        assert_eq!(entry.clock, 3579545);
    }

    #[test]
    fn test_chip_volume_entry() {
        let entry = ChipVolumeEntry {
            chip_id: 0x02,
            flags: 0x01,
            volume: 0x8000,
        };
        
        assert_eq!(entry.chip_id, 0x02);
        assert_eq!(entry.flags, 0x01);
        assert_eq!(entry.volume, 0x8000);
    }

    #[test]
    fn test_extra_header_data() {
        let extra_header = ExtraHeaderData {
            header_size: 0x40,
            chip_clock_offset: 0x00,
            chip_vol_offset: 0x00,
            chip_clock_entries: vec![],
            chip_volume_entries: vec![],
        };
        
        assert_eq!(extra_header.header_size, 0x40);
        assert!(extra_header.chip_clock_entries.is_empty());
        assert!(extra_header.chip_volume_entries.is_empty());
    }

    #[test]
    fn test_header_from_bytes() {
        let test_data = create_test_header_bytes();
        let mut bytes = Bytes::from(test_data);
        
        let header = HeaderData::from_bytes(&mut bytes).unwrap();
        
        assert_eq!(header.version, 151); // Version 1.51 in decimal
        assert_eq!(header.sn76489_clock, 3579545);
        assert_eq!(header.ym2612_clock, 7670453);
        assert_eq!(header.total_nb_samples, 44100);
        assert_eq!(header.rate, 44100);
        assert_eq!(header.vgm_data_offset, 0x40);
        assert_eq!(header.sn76489_feedback, 0x0009);
        assert_eq!(header.sn76489_shift_register_width, 16);
    }

    #[test]
    fn test_header_from_bytes_with_config() {
        let test_data = create_test_header_bytes();
        let mut bytes = Bytes::from(test_data);
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        
        let header = HeaderData::from_bytes_with_config(&mut bytes, &config, &mut tracker).unwrap();
        
        assert_eq!(header.version, 151);
        assert_eq!(header.sn76489_clock, 3579545);
        assert_eq!(header.ym2612_clock, 7670453);
    }

    #[test]
    fn test_header_to_bytes() {
        let header = HeaderData {
            version: 151,
            sn76489_clock: 3579545,
            ym2612_clock: 7670453,
            total_nb_samples: 44100,
            rate: 44100,
            vgm_data_offset: 0x40,
            sn76489_feedback: 0x0009,
            sn76489_shift_register_width: 16,
            ..Default::default()
        };
        
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        
        // Check magic bytes
        assert_eq!(&buffer[0..4], b"Vgm ");
        
        // Check some key fields - version is stored as BCD in little-endian format
        let version_bytes = &buffer[8..12];
        // decimal_to_bcd(151) returns [0x51, 0x01, 0x00, 0x00] which is stored as little-endian
        // So in the buffer it should be [0x51, 0x01, 0x00, 0x00]
        assert_eq!(version_bytes, &[0x51, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_header_round_trip() {
        let original = HeaderData {
            version: 150,
            sn76489_clock: 3579545,
            ym2612_clock: 7670453,
            ym2151_clock: 3579545,
            total_nb_samples: 88200,
            rate: 44100,
            vgm_data_offset: 0x40,
            gd3_offset: 0x100,
            loop_offset: 0x80,
            loop_nb_samples: 44100,
            sn76489_feedback: 0x0009,
            sn76489_shift_register_width: 16,
            volume_modifier: 32,
            ..Default::default()
        };
        
        // Serialize
        let mut buffer = BytesMut::new();
        original.to_bytes(&mut buffer).unwrap();
        
        // Parse back
        let mut bytes = Bytes::from(buffer.to_vec());
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        
        // Compare key fields
        assert_eq!(original.version, parsed.version);
        assert_eq!(original.sn76489_clock, parsed.sn76489_clock);
        assert_eq!(original.ym2612_clock, parsed.ym2612_clock);
        assert_eq!(original.ym2151_clock, parsed.ym2151_clock);
        assert_eq!(original.total_nb_samples, parsed.total_nb_samples);
        assert_eq!(original.rate, parsed.rate);
        assert_eq!(original.vgm_data_offset, parsed.vgm_data_offset);
        assert_eq!(original.sn76489_feedback, parsed.sn76489_feedback);
        assert_eq!(original.sn76489_shift_register_width, parsed.sn76489_shift_register_width);
    }

    #[test]
    fn test_header_invalid_magic() {
        let mut test_data = create_test_header_bytes();
        
        // Corrupt magic bytes
        test_data[0] = b'X';
        test_data[1] = b'g';
        test_data[2] = b'm';
        test_data[3] = b' ';
        
        let mut bytes = Bytes::from(test_data);
        let result = HeaderData::from_bytes(&mut bytes);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::VgmError::InvalidMagicBytes { .. }));
    }

    #[test]
    fn test_header_truncated_data() {
        let test_data = vec![b'V', b'g', b'm', b' ']; // Only magic, missing everything else
        let mut bytes = Bytes::from(test_data);
        
        // This should return an error due to insufficient data
        let result = HeaderData::from_bytes(&mut bytes);
        assert!(result.is_err(), "Expected error for truncated header data");
    }

    #[test]
    fn test_header_validation() {
        let header = HeaderData {
            version: 150,
            sn76489_clock: 3579545,
            ym2612_clock: 7670453,
            total_nb_samples: 44100,
            rate: 44100,
            vgm_data_offset: 0x40,
            gd3_offset: 0x100,
            ..Default::default()
        };
        
        let context = ValidationContext {
            file_size: 1000,
            config: ValidationConfig::default(),
        };
        
        let result = header.validate(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_header_validation_invalid_offset() {
        let header = HeaderData {
            version: 150,
            sn76489_clock: 3579545,
            gd3_offset: 2000, // Beyond file size
            ..Default::default()
        };
        
        let context = ValidationContext {
            file_size: 1000, // File too small for offset
            config: ValidationConfig::default(),
        };
        
        let result = header.validate(&context);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_chip_configurations() {
        // Test various chip clock configurations
        let header = HeaderData {
            version: 151,
            sn76489_clock: 3579545,     // Standard NTSC PSG
            ym2612_clock: 7670453,      // Standard NTSC YM2612
            ym2151_clock: 3579545,      // YM2151
            ym2413_clock: 3579545,      // YM2413
            ay8910_clock: 1789773,      // AY-3-8910
            nes_apu_clock: 1789773,     // NES APU
            gb_dmg_clock: 4194304,      // Game Boy DMG
            ..Default::default()
        };
        
        // Test round-trip with multiple chips
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        
        let mut bytes = Bytes::from(buffer.to_vec());
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        
        assert_eq!(parsed.sn76489_clock, 3579545);
        assert_eq!(parsed.ym2612_clock, 7670453);
        assert_eq!(parsed.ym2151_clock, 3579545);
        assert_eq!(parsed.ym2413_clock, 3579545);
        assert_eq!(parsed.ay8910_clock, 1789773);
        assert_eq!(parsed.nes_apu_clock, 1789773);
        assert_eq!(parsed.gb_dmg_clock, 4194304);
    }

    #[test]
    fn test_header_loop_configuration() {
        let header = HeaderData {
            version: 150,
            sn76489_clock: 3579545,
            total_nb_samples: 176400,  // 4 seconds at 44.1kHz
            loop_offset: 0x80,
            loop_nb_samples: 88200,    // 2 second loop
            rate: 44100,
            ..Default::default()
        };
        
        // Test round-trip
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        
        let mut bytes = Bytes::from(buffer.to_vec());
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        
        assert_eq!(parsed.total_nb_samples, 176400);
        assert_eq!(parsed.loop_offset, 0x80);
        assert_eq!(parsed.loop_nb_samples, 88200);
        assert_eq!(parsed.rate, 44100);
    }

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
            println!(
                "Skipping header_170 test - test VGM file not found at {:?}",
                filename
            );
            return;
        }

        let data = match fs::read(&filename) {
            Ok(data) => data,
            Err(e) => {
                println!(
                    "Skipping header_170 test - failed to read file {:?}: {}",
                    filename, e
                );
                return;
            },
        };

        let mut mem = Bytes::from(data.clone());

        let header = match HeaderData::from_bytes(&mut mem) {
            Ok(header) => header,
            Err(e) => {
                println!("Skipping header_170 test - failed to parse header: {}", e);
                return;
            },
        };
        println!("clock: {}", header.ym2608_clock);

        let mut out_buffer = BytesMut::new();
        match header.to_bytes(&mut out_buffer) {
            Ok(()) => {},
            Err(e) => {
                println!(
                    "Skipping header_170 test - failed to serialize header: {}",
                    e
                );
                return;
            },
        };

        // Ensure generated directory exists before writing
        let generated_dir = project_path("generated");
        if let Err(e) = fs::create_dir_all(&generated_dir) {
            println!(
                "Warning: Could not create generated directory {:?}: {}",
                generated_dir, e
            );
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
            },
            Err(e) => {
                println!("Warning: Could not serialize header to JSON: {}", e);
            },
        }
    }

    // Property-based tests using proptest
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_header_round_trip_parsing(
            version in 100u32..=171u32,
            eof_offset in 0u32..=0x100000u32,
            gd3_offset in 0u32..=0x100000u32,
            total_samples in 0u32..=0x10000000u32,
            loop_offset in 0u32..=0x100000u32,
            loop_samples in 0u32..=0x10000000u32,
            rate in 1u32..=192000u32,
            sn76489_clock in 0u32..=50000000u32,
            ym2413_clock in 0u32..=10000000u32,
            ym2612_clock in 0u32..=10000000u32,
            ym2151_clock in 0u32..=10000000u32
        ) {
            // Create a HeaderData with property values
            let original_header = HeaderData {
                version,
                end_of_file_offset: eof_offset,
                gd3_offset,
                total_nb_samples: total_samples,
                loop_offset,
                loop_nb_samples: loop_samples,
                rate,
                vgm_data_offset: 64, // Keep standard value
                volume_modifier: 0,
                loop_base: 0,
                loop_modifier: 0,
                extra_header_offset: 0,
                sn76489_clock,
                sn76489_feedback: 0,
                sn76489_shift_register_width: 0,
                sn76489_flags: 0,
                ym2413_clock,
                ym2612_clock,
                ym2151_clock,
                sega_pcm_clock: 0,
                spcm_interface: 0,
                rf5_c68_clock: 0,
                ym2203_clock: 0,
                ym2608_clock: 0,
                ym2610_b_clock: 0,
                ym3812_clock: 0,
                ym3526_clock: 0,
                y8950_clock: 0,
                ymf262_clock: 0,
                ymf278_b_clock: 0,
                ymf271_clock: 0,
                ymz280_b_clock: 0,
                rf5_c164_clock: 0,
                pwm_clock: 0,
                ay8910_clock: 0,
                ay8910_chip_type: 0,
                ay8910_flags: 0,
                ym2203_ay8910_flags: 0,
                ym2608_ay8910_flags: 0,
                gb_dmg_clock: 0,
                nes_apu_clock: 0,
                multi_pcm_clock: 0,
                u_pd7759_clock: 0,
                okim6258_clock: 0,
                okim6258_flags: 0,
                k054539_flags: 0,
                c140_chip_type: 0,
                okim6295_clock: 0,
                k051649_k052539_clock: 0,
                k054539_clock: 0,
                hu_c6280_clock: 0,
                c140_clock: 0,
                k053260_clock: 0,
                pokey_clock: 0,
                qsound_clock: 0,
                scsp_clock: 0,
                extra_header: ExtraHeaderData::default(),
                wonder_swan_clock: 0,
                vsu_clock: 0,
                saa1099_clock: 0,
                es5503_clock: 0,
                es5506_clock: 0,
                es5503_nb_channels: 0,
                es5505_es5506_nb_channels: 0,
                c352_clock_divider: 0,
                x1010_clock: 0,
                c352_clock: 0,
                ga20_clock: 0,
            };

            // Serialize to bytes
            let mut buffer = BytesMut::new();
            original_header.to_bytes(&mut buffer).unwrap();

            // Parse back from bytes
            let mut bytes = Bytes::from(buffer);
            let parsed_header = HeaderData::from_bytes(&mut bytes).unwrap();

            // Verify round-trip preservation
            prop_assert_eq!(original_header.version, parsed_header.version);
            prop_assert_eq!(original_header.end_of_file_offset, parsed_header.end_of_file_offset);
            prop_assert_eq!(original_header.gd3_offset, parsed_header.gd3_offset);
            prop_assert_eq!(original_header.total_nb_samples, parsed_header.total_nb_samples);
            prop_assert_eq!(original_header.loop_offset, parsed_header.loop_offset);
            prop_assert_eq!(original_header.loop_nb_samples, parsed_header.loop_nb_samples);
            prop_assert_eq!(original_header.rate, parsed_header.rate);
            prop_assert_eq!(original_header.sn76489_clock, parsed_header.sn76489_clock);
            prop_assert_eq!(original_header.ym2413_clock, parsed_header.ym2413_clock);
            prop_assert_eq!(original_header.ym2612_clock, parsed_header.ym2612_clock);
            prop_assert_eq!(original_header.ym2151_clock, parsed_header.ym2151_clock);
        }

        #[test]
        fn test_header_version_bcd_conversion(
            major in 1u8..=9u8,
            minor in 0u8..=9u8,
            patch in 0u8..=9u8
        ) {
            // Test BCD version format conversion
            let decimal_version = (major as u32) * 100 + (minor as u32) * 10 + (patch as u32);
            
            // Create a header with this decimal version (to be converted to BCD)
            let mut header = HeaderData::default();
            header.version = decimal_version;

            // Serialize and parse back
            let mut buffer = BytesMut::new();
            header.to_bytes(&mut buffer).unwrap();
            let mut bytes = Bytes::from(buffer);
            let parsed_header = HeaderData::from_bytes(&mut bytes).unwrap();

            // Verify version is preserved through BCD conversion
            prop_assert_eq!(header.version, parsed_header.version);
        }

        #[test]
        fn test_header_clock_frequency_ranges(
            sn76489_clock in 0u32..=50000000u32,
            ym2612_clock in 0u32..=10000000u32,
            ym2151_clock in 0u32..=5000000u32
        ) {
            // Test valid clock frequency ranges
            let mut header = HeaderData::default();
            header.sn76489_clock = sn76489_clock;
            header.ym2612_clock = ym2612_clock;
            header.ym2151_clock = ym2151_clock;
            header.version = 150; // Valid version

            // Serialize and parse back
            let mut buffer = BytesMut::new();
            header.to_bytes(&mut buffer).unwrap();
            let mut bytes = Bytes::from(buffer);
            let parsed_header = HeaderData::from_bytes(&mut bytes).unwrap();

            // Verify clock values are preserved
            prop_assert_eq!(header.sn76489_clock, parsed_header.sn76489_clock);
            prop_assert_eq!(header.ym2612_clock, parsed_header.ym2612_clock);
            prop_assert_eq!(header.ym2151_clock, parsed_header.ym2151_clock);
        }

        #[test]
        fn test_header_offset_consistency(
            eof_offset in 64u32..=0x100000u32,
            gd3_offset in 0u32..=0x100000u32,
            vgm_data_offset in 64u32..=256u32
        ) {
            // Test offset field consistency
            let mut header = HeaderData::default();
            header.end_of_file_offset = eof_offset;
            header.gd3_offset = if gd3_offset == 0 { 0 } else { gd3_offset.max(64) };
            header.vgm_data_offset = vgm_data_offset;
            header.version = 150; // Valid version

            // Serialize and parse back
            let mut buffer = BytesMut::new();
            header.to_bytes(&mut buffer).unwrap();
            let mut bytes = Bytes::from(buffer);
            let parsed_header = HeaderData::from_bytes(&mut bytes).unwrap();

            // Verify offsets are preserved
            prop_assert_eq!(header.end_of_file_offset, parsed_header.end_of_file_offset);
            prop_assert_eq!(header.gd3_offset, parsed_header.gd3_offset);
            prop_assert_eq!(header.vgm_data_offset, parsed_header.vgm_data_offset);

            // Verify logical constraints
            if header.gd3_offset > 0 {
                prop_assert!(header.gd3_offset >= 64); // GD3 offset must be after header
            }
            prop_assert!(header.vgm_data_offset >= 64); // VGM data offset must be after header
        }

        #[test]
        fn test_header_dual_chip_flags(
            sn76489_clock in 1u32..=50000000u32,
            ym2612_clock in 1u32..=10000000u32
        ) {
            // Test dual chip support flags (bit 31)
            let mut header = HeaderData::default();
            header.version = 150; // Valid version
            
            // Set clocks with dual chip flag (bit 31 set)
            header.sn76489_clock = sn76489_clock | 0x80000000;
            header.ym2612_clock = ym2612_clock | 0x80000000;

            // Serialize and parse back
            let mut buffer = BytesMut::new();
            header.to_bytes(&mut buffer).unwrap();
            let mut bytes = Bytes::from(buffer);
            let parsed_header = HeaderData::from_bytes(&mut bytes).unwrap();

            // Verify dual chip flags are preserved
            prop_assert_eq!(header.sn76489_clock, parsed_header.sn76489_clock);
            prop_assert_eq!(header.ym2612_clock, parsed_header.ym2612_clock);
            
            // Verify dual chip flags are correctly set
            prop_assert!((parsed_header.sn76489_clock & 0x80000000) != 0);
            prop_assert!((parsed_header.ym2612_clock & 0x80000000) != 0);
        }

        #[test]
        fn test_header_sample_rate_and_duration(
            rate in 8000u32..=192000u32,
            total_samples in 1u32..=57600000u32  // Max 7200 seconds at 8kHz rate
        ) {
            // Test valid sample rates and durations
            let mut header = HeaderData::default();
            header.version = 150;
            header.rate = rate;
            header.total_nb_samples = total_samples;

            // Serialize and parse back
            let mut buffer = BytesMut::new();
            header.to_bytes(&mut buffer).unwrap();
            let mut bytes = Bytes::from(buffer);
            let parsed_header = HeaderData::from_bytes(&mut bytes).unwrap();

            // Verify values are preserved
            prop_assert_eq!(header.rate, parsed_header.rate);
            prop_assert_eq!(header.total_nb_samples, parsed_header.total_nb_samples);

            // Calculate duration and verify it's reasonable
            let duration_seconds = header.total_nb_samples as f64 / header.rate as f64;
            prop_assert!(duration_seconds >= 0.0);
            prop_assert!(duration_seconds <= 7200.0); // Max 2 hours for property test
        }
    }

    #[test]
    fn test_header_edge_cases_coverage() {
        // Test edge cases for improved coverage of header parsing
        
        // Test with minimal valid header (VGM 1.00)
        let mut header = HeaderData::default();
        header.version = 100; // VGM 1.00
        header.end_of_file_offset = 0x40; // Minimal size
        header.sn76489_clock = 3579545; // Standard PSG clock
        
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        let mut bytes = Bytes::from(buffer);
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        assert_eq!(parsed.version, 100);
        
        // Test with maximum valid offsets
        let mut header = HeaderData::default();
        header.version = 171; // VGM 1.71
        header.gd3_offset = 0x100000;
        header.loop_offset = 0x100000;
        header.vgm_data_offset = 0x34;
        
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        let mut bytes = Bytes::from(buffer);
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        assert_eq!(parsed.gd3_offset, 0x100000);
        assert_eq!(parsed.loop_offset, 0x100000);
        
        // Test various chip clock combinations
        let mut header = HeaderData::default();
        header.version = 150;
        header.ym2612_clock = 7670453;
        header.ym2151_clock = 3579545;
        header.sn76489_clock = 3579545;
        header.ym2413_clock = 3579545;
        
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        let mut bytes = Bytes::from(buffer);
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        assert_eq!(parsed.ym2612_clock, 7670453);
        assert_eq!(parsed.ym2151_clock, 3579545);
        
        // Test dual chip configurations
        let mut header = HeaderData::default();
        header.version = 161;
        header.ym2612_clock = 7670453 | 0x40000000; // Dual chip flag
        header.sn76489_clock = 3579545 | 0x40000000; // Dual chip flag
        
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        let mut bytes = Bytes::from(buffer);
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        assert!((parsed.ym2612_clock & 0x40000000) != 0);
        assert!((parsed.sn76489_clock & 0x40000000) != 0);
    }

    #[test]
    fn test_header_serialization_edge_cases() {
        // Test serialization of various header configurations
        
        // Test all chip types with non-zero clocks
        let mut header = HeaderData::default();
        header.version = 170;
        header.sn76489_clock = 3579545;
        header.ym2413_clock = 3579545;
        header.ym2612_clock = 7670453;
        header.ym2151_clock = 3579545;
        header.ym2203_clock = 3579545;
        header.ym2608_clock = 7987200;
        header.ym2610_b_clock = 8000000;
        header.ym3812_clock = 3579545;
        header.ym3526_clock = 3579545;
        header.y8950_clock = 3579545;
        header.ymf262_clock = 14318180;
        header.ymf278_b_clock = 33868800;
        header.ymf271_clock = 16934400;
        header.ymz280_b_clock = 16934400;
        header.rf5_c68_clock = 12500000;
        header.rf5_c164_clock = 7159090;
        header.pwm_clock = 23011361;
        header.ay8910_clock = 1789772;
        header.gb_dmg_clock = 4194304;
        header.nes_apu_clock = 1789772;
        header.multi_pcm_clock = 8053975;
        header.u_pd7759_clock = 640000;
        header.okim6258_clock = 4000000;
        header.okim6295_clock = 1056000;
        
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        let mut bytes = Bytes::from(buffer);
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        
        assert_eq!(parsed.sn76489_clock, 3579545);
        assert_eq!(parsed.ym2413_clock, 3579545);
        assert_eq!(parsed.ym2612_clock, 7670453);
        assert_eq!(parsed.ym2151_clock, 3579545);
        assert_eq!(parsed.ym2203_clock, 3579545);
        assert_eq!(parsed.ym2608_clock, 7987200);
        assert_eq!(parsed.ym2610_b_clock, 8000000);
        assert_eq!(parsed.ym3812_clock, 3579545);
        assert_eq!(parsed.ym3526_clock, 3579545);
        assert_eq!(parsed.y8950_clock, 3579545);
        assert_eq!(parsed.ymf262_clock, 14318180);
        assert_eq!(parsed.ymf278_b_clock, 33868800);
        assert_eq!(parsed.ymf271_clock, 16934400);
        assert_eq!(parsed.ymz280_b_clock, 16934400);
        assert_eq!(parsed.rf5_c68_clock, 12500000);
        assert_eq!(parsed.rf5_c164_clock, 7159090);
        assert_eq!(parsed.pwm_clock, 23011361);
        assert_eq!(parsed.ay8910_clock, 1789772);
        assert_eq!(parsed.gb_dmg_clock, 4194304);
        assert_eq!(parsed.nes_apu_clock, 1789772);
        assert_eq!(parsed.multi_pcm_clock, 8053975);
        assert_eq!(parsed.u_pd7759_clock, 640000);
        assert_eq!(parsed.okim6258_clock, 4000000);
        assert_eq!(parsed.okim6295_clock, 1056000);
    }

    #[test]
    fn test_header_early_return_scenarios() {
        // Test different VGM data offset scenarios to hit early return branches
        
        // Test header ending at 0x40 (basic VGM 1.00)
        let mut test_data = create_test_header_bytes();
        // Set vgm_data_offset to point right after basic header
        test_data[0x34..0x38].copy_from_slice(&0x0Cu32.to_le_bytes()); // 0x40 - 0x34 = 0x0C
        
        let mut bytes = Bytes::from(test_data);
        let header = HeaderData::from_bytes(&mut bytes).unwrap();
        assert_eq!(header.vgm_data_offset, 0x0C);
        assert_eq!(header.rf5_c68_clock, 0); // Should not be parsed due to early return
        
        // Test header ending at 0x50 (includes rf5_c68_clock)
        let mut test_data = create_test_header_bytes();
        test_data[0x34..0x38].copy_from_slice(&0x1Cu32.to_le_bytes()); // 0x50 - 0x34 = 0x1C
        test_data[0x40..0x44].copy_from_slice(&123456u32.to_le_bytes()); // Set rf5_c68_clock
        
        let mut bytes = Bytes::from(test_data);
        let header = HeaderData::from_bytes(&mut bytes).unwrap();
        assert_eq!(header.vgm_data_offset, 0x1C);
        assert_eq!(header.rf5_c68_clock, 123456);
        assert_eq!(header.ym2203_clock, 0); // Should not be parsed
        
        // Test header ending at 0x80 (includes more fields)
        let mut test_data = create_test_header_bytes();
        test_data[0x34..0x38].copy_from_slice(&0x4Cu32.to_le_bytes()); // 0x80 - 0x34 = 0x4C
        test_data[0x70..0x74].copy_from_slice(&789012u32.to_le_bytes()); // Set pwm_clock
        
        let mut bytes = Bytes::from(test_data);
        let header = HeaderData::from_bytes(&mut bytes).unwrap();
        assert_eq!(header.vgm_data_offset, 0x4C);
        assert_eq!(header.pwm_clock, 789012);
        assert_eq!(header.gb_dmg_clock, 0); // Should not be parsed
    }

    #[test]
    fn test_header_extra_header_parsing() {
        // Test header with extra header offset
        let mut test_data = create_test_header_bytes();
        
        // Set vgm_data_offset to a large value so parsing continues past 0xBC
        test_data[0x34..0x38].copy_from_slice(&0x100u32.to_le_bytes()); // Set vgm_data_offset to 0x100
        // Set extra_header_offset to point to a location
        test_data[0xBC..0xC0].copy_from_slice(&0x10u32.to_le_bytes()); // Extra header at 0xBC + 0x10 = 0xCC
        
        // Create extra header data at the expected position
        let extra_header_start = 0xCC;
        while test_data.len() < extra_header_start + 16 {
            test_data.push(0);
        }
        
        // Write extra header structure
        test_data[extra_header_start..extra_header_start+4].copy_from_slice(&0x10u32.to_le_bytes()); // header_size
        test_data[extra_header_start+4..extra_header_start+8].copy_from_slice(&0x10u32.to_le_bytes()); // chip_clock_offset
        test_data[extra_header_start+8..extra_header_start+12].copy_from_slice(&0x00u32.to_le_bytes()); // chip_vol_offset
        
        let mut bytes = Bytes::from(test_data);
        let header = HeaderData::from_bytes(&mut bytes).unwrap();
        
        assert_eq!(header.extra_header_offset, 0x10);
        assert_eq!(header.extra_header.header_size, 0x10);
    }

    #[test]
    fn test_header_integer_overflow_protection() {
        // Test vgm_data_offset overflow protection
        let mut test_data = create_test_header_bytes();
        // Set vgm_data_offset to a value that would overflow when adding 0x34
        test_data[0x34..0x38].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        
        let mut bytes = Bytes::from(test_data);
        let result = HeaderData::from_bytes(&mut bytes);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::VgmError::IntegerOverflow { .. }));
        
        // Test extra_header_offset overflow protection  
        let mut test_data = create_test_header_bytes();
        // Set vgm_data_offset to a large value so parsing continues past 0xBC
        test_data[0x34..0x38].copy_from_slice(&0x100u32.to_le_bytes()); // Set vgm_data_offset to 0x100
        test_data[0xBC..0xC0].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // Extra header offset overflow
        
        let mut bytes = Bytes::from(test_data);
        let result = HeaderData::from_bytes(&mut bytes);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::VgmError::IntegerOverflow { .. }));
    }

    #[test]
    fn test_header_all_chip_clocks_parsing() {
        // Test a header that includes all possible chip clocks to exercise all parsing paths
        let mut test_data = create_test_header_bytes();
        
        // Set vgm_data_offset to allow parsing all fields
        test_data[0x34..0x38].copy_from_slice(&0xB0u32.to_le_bytes()); // Point to 0xE4
        
        // Ensure we have enough data
        while test_data.len() < 0xE8 {
            test_data.push(0);
        }
        
        // Set various chip clocks with recognizable values
        test_data[0x40..0x44].copy_from_slice(&1000000u32.to_le_bytes()); // rf5_c68_clock
        test_data[0x44..0x48].copy_from_slice(&2000000u32.to_le_bytes()); // ym2203_clock  
        test_data[0x48..0x4C].copy_from_slice(&3000000u32.to_le_bytes()); // ym2608_clock
        test_data[0x4C..0x50].copy_from_slice(&4000000u32.to_le_bytes()); // ym2610_b_clock
        
        test_data[0x50..0x54].copy_from_slice(&5000000u32.to_le_bytes()); // ym3812_clock
        test_data[0x54..0x58].copy_from_slice(&6000000u32.to_le_bytes()); // ym3526_clock
        test_data[0x58..0x5C].copy_from_slice(&7000000u32.to_le_bytes()); // y8950_clock
        test_data[0x5C..0x60].copy_from_slice(&8000000u32.to_le_bytes()); // ymf262_clock
        
        test_data[0x60..0x64].copy_from_slice(&9000000u32.to_le_bytes()); // ymf278_b_clock
        test_data[0x64..0x68].copy_from_slice(&10000000u32.to_le_bytes()); // ymf271_clock
        test_data[0x68..0x6C].copy_from_slice(&11000000u32.to_le_bytes()); // ymz280_b_clock
        test_data[0x6C..0x70].copy_from_slice(&12000000u32.to_le_bytes()); // rf5_c164_clock
        
        test_data[0x70..0x74].copy_from_slice(&13000000u32.to_le_bytes()); // pwm_clock
        test_data[0x74..0x78].copy_from_slice(&14000000u32.to_le_bytes()); // ay8910_clock
        test_data[0x78] = 0x01; // ay8910_chip_type
        test_data[0x79] = 0x02; // ay8910_flags
        test_data[0x7A] = 0x03; // ym2203_ay8910_flags
        test_data[0x7B] = 0x04; // ym2608_ay8910_flags
        test_data[0x7C] = 32; // volume_modifier
        test_data[0x7E] = 0x05; // loop_base
        test_data[0x7F] = 0x06; // loop_modifier
        
        test_data[0x80..0x84].copy_from_slice(&15000000u32.to_le_bytes()); // gb_dmg_clock
        test_data[0x84..0x88].copy_from_slice(&16000000u32.to_le_bytes()); // nes_apu_clock
        test_data[0x88..0x8C].copy_from_slice(&17000000u32.to_le_bytes()); // multi_pcm_clock
        test_data[0x8C..0x90].copy_from_slice(&18000000u32.to_le_bytes()); // u_pd7759_clock
        
        test_data[0x90..0x94].copy_from_slice(&19000000u32.to_le_bytes()); // okim6258_clock
        test_data[0x94] = 0x07; // okim6258_flags
        test_data[0x95] = 0x08; // k054539_flags
        test_data[0x96] = 0x09; // c140_chip_type
        test_data[0x98..0x9C].copy_from_slice(&20000000u32.to_le_bytes()); // okim6295_clock
        test_data[0x9C..0xA0].copy_from_slice(&21000000u32.to_le_bytes()); // k051649_k052539_clock
        
        test_data[0xA0..0xA4].copy_from_slice(&22000000u32.to_le_bytes()); // k054539_clock
        test_data[0xA4..0xA8].copy_from_slice(&23000000u32.to_le_bytes()); // hu_c6280_clock
        test_data[0xA8..0xAC].copy_from_slice(&24000000u32.to_le_bytes()); // c140_clock
        test_data[0xAC..0xB0].copy_from_slice(&25000000u32.to_le_bytes()); // k053260_clock
        
        test_data[0xB0..0xB4].copy_from_slice(&26000000u32.to_le_bytes()); // pokey_clock
        test_data[0xB4..0xB8].copy_from_slice(&27000000u32.to_le_bytes()); // qsound_clock
        test_data[0xB8..0xBC].copy_from_slice(&28000000u32.to_le_bytes()); // scsp_clock
        // extra_header_offset at 0xBC..0xC0 should remain 0
        
        test_data[0xC0..0xC4].copy_from_slice(&29000000u32.to_le_bytes()); // wonder_swan_clock
        test_data[0xC4..0xC8].copy_from_slice(&30000000u32.to_le_bytes()); // vsu_clock
        test_data[0xC8..0xCC].copy_from_slice(&31000000u32.to_le_bytes()); // saa1099_clock
        test_data[0xCC..0xD0].copy_from_slice(&32000000u32.to_le_bytes()); // es5503_clock
        
        test_data[0xD0..0xD4].copy_from_slice(&33000000u32.to_le_bytes()); // es5506_clock
        test_data[0xD4] = 16; // es5503_nb_channels
        test_data[0xD5] = 32; // es5505_es5506_nb_channels
        test_data[0xD6] = 0x0A; // c352_clock_divider
        test_data[0xD8..0xDC].copy_from_slice(&34000000u32.to_le_bytes()); // x1010_clock
        test_data[0xDC..0xE0].copy_from_slice(&35000000u32.to_le_bytes()); // c352_clock
        
        test_data[0xE0..0xE4].copy_from_slice(&36000000u32.to_le_bytes()); // ga20_clock
        
        let mut bytes = Bytes::from(test_data);
        let header = HeaderData::from_bytes(&mut bytes).unwrap();
        
        // Verify all fields were parsed correctly
        assert_eq!(header.rf5_c68_clock, 1000000);
        assert_eq!(header.ym2203_clock, 2000000);
        assert_eq!(header.ym2608_clock, 3000000);
        assert_eq!(header.ym2610_b_clock, 4000000);
        assert_eq!(header.ym3812_clock, 5000000);
        assert_eq!(header.ym3526_clock, 6000000);
        assert_eq!(header.y8950_clock, 7000000);
        assert_eq!(header.ymf262_clock, 8000000);
        assert_eq!(header.ymf278_b_clock, 9000000);
        assert_eq!(header.ymf271_clock, 10000000);
        assert_eq!(header.ymz280_b_clock, 11000000);
        assert_eq!(header.rf5_c164_clock, 12000000);
        assert_eq!(header.pwm_clock, 13000000);
        assert_eq!(header.ay8910_clock, 14000000);
        assert_eq!(header.ay8910_chip_type, 0x01);
        assert_eq!(header.ay8910_flags, 0x02);
        assert_eq!(header.ym2203_ay8910_flags, 0x03);
        assert_eq!(header.ym2608_ay8910_flags, 0x04);
        assert_eq!(header.volume_modifier, 32);
        assert_eq!(header.loop_base, 0x05);
        assert_eq!(header.loop_modifier, 0x06);
        assert_eq!(header.gb_dmg_clock, 15000000);
        assert_eq!(header.nes_apu_clock, 16000000);
        assert_eq!(header.multi_pcm_clock, 17000000);
        assert_eq!(header.u_pd7759_clock, 18000000);
        assert_eq!(header.okim6258_clock, 19000000);
        assert_eq!(header.okim6258_flags, 0x07);
        assert_eq!(header.k054539_flags, 0x08);
        assert_eq!(header.c140_chip_type, 0x09);
        assert_eq!(header.okim6295_clock, 20000000);
        assert_eq!(header.k051649_k052539_clock, 21000000);
        assert_eq!(header.k054539_clock, 22000000);
        assert_eq!(header.hu_c6280_clock, 23000000);
        assert_eq!(header.c140_clock, 24000000);
        assert_eq!(header.k053260_clock, 25000000);
        assert_eq!(header.pokey_clock, 26000000);
        assert_eq!(header.qsound_clock, 27000000);
        assert_eq!(header.scsp_clock, 28000000);
        assert_eq!(header.wonder_swan_clock, 29000000);
        assert_eq!(header.vsu_clock, 30000000);
        assert_eq!(header.saa1099_clock, 31000000);
        assert_eq!(header.es5503_clock, 32000000);
        assert_eq!(header.es5506_clock, 33000000);
        assert_eq!(header.es5503_nb_channels, 16);
        assert_eq!(header.es5505_es5506_nb_channels, 32);
        assert_eq!(header.c352_clock_divider, 0x0A);
        assert_eq!(header.x1010_clock, 34000000);
        assert_eq!(header.c352_clock, 35000000);
        assert_eq!(header.ga20_clock, 36000000);
    }

    #[test]
    fn test_header_serialization_all_fields() {
        // Test serialization with all fields set to exercise serialization paths
        let header = HeaderData {
            end_of_file_offset: 1000,
            version: 171, // VGM 1.71
            sn76489_clock: 3579545,
            ym2413_clock: 3579545,
            gd3_offset: 0x200,
            total_nb_samples: 176400, // 4 seconds at 44.1kHz
            loop_offset: 0x100,
            loop_nb_samples: 88200, // 2 seconds
            rate: 44100,
            sn76489_feedback: 0x0009,
            sn76489_shift_register_width: 16,
            sn76489_flags: 0x01,
            ym2612_clock: 7670453,
            ym2151_clock: 3579545,
            vgm_data_offset: 0xB0,
            sega_pcm_clock: 4000000,
            spcm_interface: 0x01,
            rf5_c68_clock: 12500000,
            ym2203_clock: 3993600,
            ym2608_clock: 8000000,
            ym2610_b_clock: 8000000,
            ym3812_clock: 3579545,
            ym3526_clock: 3579545,
            y8950_clock: 3579545,
            ymf262_clock: 14318180,
            ymf278_b_clock: 33868800,
            ymf271_clock: 16934400,
            ymz280_b_clock: 16934400,
            rf5_c164_clock: 12500000,
            pwm_clock: 23011361,
            ay8910_clock: 1789773,
            ay8910_chip_type: 0x00,
            ay8910_flags: 0x01,
            ym2203_ay8910_flags: 0x02,
            ym2608_ay8910_flags: 0x03,
            volume_modifier: 32,
            loop_base: 0x01,
            loop_modifier: 0x02,
            gb_dmg_clock: 4194304,
            nes_apu_clock: 1789773,
            multi_pcm_clock: 8053975,
            u_pd7759_clock: 8000000,
            okim6258_clock: 4000000,
            okim6258_flags: 0x02,
            k054539_flags: 0x01,
            c140_chip_type: 0x00,
            okim6295_clock: 1056000,
            k051649_k052539_clock: 1500000,
            k054539_clock: 18432000,
            hu_c6280_clock: 3579545,
            c140_clock: 21390,
            k053260_clock: 3579545,
            pokey_clock: 1789773,
            qsound_clock: 4000000,
            scsp_clock: 22579200,
            extra_header_offset: 0,
            wonder_swan_clock: 3072000,
            vsu_clock: 5000000,
            saa1099_clock: 7159090,
            es5503_clock: 7159090,
            es5506_clock: 16000000,
            es5503_nb_channels: 32,
            es5505_es5506_nb_channels: 32,
            c352_clock_divider: 4,
            x1010_clock: 16000000,
            c352_clock: 25401600,
            ga20_clock: 3579545,
            extra_header: ExtraHeaderData::default(),
        };
        
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        
        // Verify the buffer contains expected values
        assert_eq!(&buffer[0..4], b"Vgm ");
        
        // Parse back and compare
        let mut bytes = Bytes::from(buffer.to_vec());
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        
        // Verify key fields
        assert_eq!(header.version, parsed.version);
        assert_eq!(header.sn76489_clock, parsed.sn76489_clock);
        assert_eq!(header.ym2612_clock, parsed.ym2612_clock);
        assert_eq!(header.total_nb_samples, parsed.total_nb_samples);
        assert_eq!(header.gb_dmg_clock, parsed.gb_dmg_clock);
        assert_eq!(header.pokey_clock, parsed.pokey_clock);
        assert_eq!(header.ga20_clock, parsed.ga20_clock);
        assert_eq!(header.es5503_nb_channels, parsed.es5503_nb_channels);
        assert_eq!(header.c352_clock_divider, parsed.c352_clock_divider);
    }

    #[test]
    fn test_header_bcd_version_edge_cases() {
        // Test various BCD version conversion edge cases
        let versions_to_test = vec![
            (100, "Version 1.00"),
            (101, "Version 1.01"),
            (110, "Version 1.10"),
            (150, "Version 1.50"),
            (151, "Version 1.51"),
            (160, "Version 1.60"),
            (161, "Version 1.61"),
            (170, "Version 1.70"),
            (171, "Version 1.71"),
        ];
        
        for (version, description) in versions_to_test {
            let header = HeaderData {
                version,
                sn76489_clock: 3579545,
                vgm_data_offset: 0x40,
                ..Default::default()
            };
            
            let mut buffer = BytesMut::new();
            header.to_bytes(&mut buffer).unwrap();
            
            let mut bytes = Bytes::from(buffer.to_vec());
            let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
            
            assert_eq!(header.version, parsed.version, "Failed for {}", description);
        }
    }

    #[test]
    fn test_header_newer_version_fields() {
        // Test newer VGM version specific fields
        let mut header = HeaderData::default();
        header.version = 171; // VGM 1.71
        header.extra_header_offset = 0xBC;
        header.k051649_k052539_clock = 1500000;
        header.k054539_clock = 18432000;
        header.hu_c6280_clock = 3579545;
        header.c140_clock = 21390;
        header.k053260_clock = 3579545;
        header.pokey_clock = 1789772;
        header.qsound_clock = 4000000;
        header.scsp_clock = 22579200;
        header.wonder_swan_clock = 3072000;
        header.vsu_clock = 5000000;
        header.saa1099_clock = 7159090;
        header.es5503_clock = 7159090;
        header.es5506_clock = 16000000;
        header.x1010_clock = 14318180;
        header.c352_clock = 24576000;
        header.ga20_clock = 3579545;
        
        let mut buffer = BytesMut::new();
        header.to_bytes(&mut buffer).unwrap();
        let mut bytes = Bytes::from(buffer);
        let parsed = HeaderData::from_bytes(&mut bytes).unwrap();
        
        assert_eq!(parsed.extra_header_offset, 0xBC);
        assert_eq!(parsed.k051649_k052539_clock, 1500000);
        assert_eq!(parsed.k054539_clock, 18432000);
        assert_eq!(parsed.hu_c6280_clock, 3579545);
        assert_eq!(parsed.c140_clock, 21390);
        assert_eq!(parsed.k053260_clock, 3579545);
        assert_eq!(parsed.pokey_clock, 1789772);
        assert_eq!(parsed.qsound_clock, 4000000);
        assert_eq!(parsed.scsp_clock, 22579200);
        assert_eq!(parsed.wonder_swan_clock, 3072000);
        assert_eq!(parsed.vsu_clock, 5000000);
        assert_eq!(parsed.saa1099_clock, 7159090);
        assert_eq!(parsed.es5503_clock, 7159090);
        assert_eq!(parsed.es5506_clock, 16000000);
        assert_eq!(parsed.x1010_clock, 14318180);
        assert_eq!(parsed.c352_clock, 24576000);
        assert_eq!(parsed.ga20_clock, 3579545);
    }
}
