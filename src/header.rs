use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

use crate::{
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
    fn parse_extra_header(&mut self, data: &mut Bytes, extra_header_pos: usize) {
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
            Some(extra_header_pos + 4 + extra_header.chip_clock_offset as usize)
        };

        let chip_vol_pos = if extra_header.chip_vol_offset == 0 {
            None
        } else {
            Some(extra_header_pos + 4 + 4 + extra_header.chip_vol_offset as usize)
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
    fn from_bytes(data: &mut Bytes) -> Self {
        let mut header = HeaderData::default();
        // get length of data for position calculation
        let len_data = data.len();

        // validate magic
        let magic = data.get_u32();
        assert_eq!(magic.to_be_bytes(), b"Vgm "[..]);
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

        let pos_start_vgm = header.vgm_data_offset + 0x34;

        // 0x40
        // From here, need to check if is still header, or start of vgm data
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.rf5_c68_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ym2203_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ym2608_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ym2610_b_clock = data.get_u32_le();

        // 0x50
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ym3812_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ym3526_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.y8950_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ymf262_clock = data.get_u32_le();

        // 0x60
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ymf278_b_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ymf271_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ymz280_b_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.rf5_c164_clock = data.get_u32_le();

        // 0x70
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.pwm_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ay8910_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ay8910_chip_type = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ym2203_ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.ym2608_ay8910_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.volume_modifier = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.loop_base = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.loop_modifier = data.get_u8();

        // 0x80
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.gb_dmg_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.nes_apu_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.multi_pcm_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.u_pd7759_clock = data.get_u32_le();

        // 0x90
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.okim6258_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.okim6258_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.k054539_flags = data.get_u8();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.c140_chip_type = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.okim6295_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.k051649_k052539_clock = data.get_u32_le();

        // 0xA0
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.k054539_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.hu_c6280_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.c140_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.k053260_clock = data.get_u32_le();

        // 0xB0
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.pokey_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.qsound_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.scsp_clock = data.get_u32_le();
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        }
        header.extra_header_offset = data.get_u32_le();

        let pos_extra_header = if header.extra_header_offset == 0 {
            None
        } else {
            Some((header.extra_header_offset + 0xBC) as usize)
        };

        // 0xC0
        // from here need to also check for extra header data
        // can assume that after extra header is vgm data?
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.wonder_swan_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.vsu_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.saa1099_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.es5503_clock = data.get_u32_le();

        // 0xD0
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.es5506_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.es5503_nb_channels = data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.es5505_es5506_nb_channels = data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.c352_clock_divider = data.get_u8();

        // skip reserved
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        data.get_u8();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.x1010_clock = data.get_u32_le();

        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.c352_clock = data.get_u32_le();

        // 0xE0
        if (len_data - data.remaining()) == pos_start_vgm as usize {
            return header;
        } else if let Some(pos_extra_header) = pos_extra_header {
            if (len_data - data.remaining()) == pos_extra_header {
                header.parse_extra_header(data, pos_extra_header);
                return header;
            }
        }
        header.ga20_clock = data.get_u32_le();

        header
    }
}

impl VgmWriter for HeaderData {
    fn to_bytes(&self, buffer: &mut BytesMut) {
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
            return;
        }
        buffer.put(&self.rf5_c68_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ym2203_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ym2608_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ym2610_b_clock.to_le_bytes()[..]);

        // 0x50
        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ym3812_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ym3526_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.y8950_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ymf262_clock.to_le_bytes()[..]);

        // 0x60
        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ymf278_b_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ymf271_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ymz280_b_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.rf5_c164_clock.to_le_bytes()[..]);

        // 0x70
        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.pwm_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ay8910_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ay8910_chip_type.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ay8910_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ym2203_ay8910_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.ym2608_ay8910_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.volume_modifier.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&[0x00][..]); // reserved

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.loop_base.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.loop_modifier.to_le_bytes()[..]);

        // 0x80
        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.gb_dmg_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.nes_apu_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.multi_pcm_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.u_pd7759_clock.to_le_bytes()[..]);

        // 0x90
        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.okim6258_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.okim6258_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.k054539_flags.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.c140_chip_type.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&[0x00][..]); // reserved

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.okim6295_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.k051649_k052539_clock.to_le_bytes()[..]);

        // 0xA0
        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.k054539_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.hu_c6280_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.c140_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.k053260_clock.to_le_bytes()[..]);

        // 0xB0
        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.pokey_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.qsound_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.scsp_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        }
        buffer.put(&self.extra_header_offset.to_le_bytes()[..]);

        // 0xC0
        // from here need to also check for extra header data
        // can assume that after extra header is vgm data?
        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.wonder_swan_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.vsu_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.saa1099_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.es5503_clock.to_le_bytes()[..]);

        // 0xD0
        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.es5506_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.es5503_nb_channels.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.es5505_es5506_nb_channels.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.c352_clock_divider.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&[0x00][..]); // reserved

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.x1010_clock.to_le_bytes()[..]);

        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.c352_clock.to_le_bytes()[..]);

        // 0xE0
        if buffer.len() == vgm_data_pos {
            return;
        } else if let Some(extra_header_pos) = extra_header_pos {
            if buffer.len() == extra_header_pos {
                self.write_extra_header(buffer, vgm_data_pos);
                return;
            }
        }
        buffer.put(&self.ga20_clock.to_le_bytes()[..]);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use bytes::{Bytes, BytesMut};

    use crate::traits::{VgmParser, VgmWriter};

    use super::HeaderData;

    #[test]
    fn header_170() {
        let filename = "./vgm_files/Into Battle.vgm";
        let data = fs::read(filename).unwrap();
        let mut mem = Bytes::from(data.clone());

        let header = HeaderData::from_bytes(&mut mem);
        println!("clock: {}", header.ym2608_clock);

        let mut out_buffer = BytesMut::new();
        header.to_bytes(&mut out_buffer);

        let _ = fs::write("./generated/Into Battle.bin", out_buffer);
        let _ = fs::write(
            "./generated/Into Battle.json",
            serde_json::to_string(&header).unwrap(),
        );
    }
}
