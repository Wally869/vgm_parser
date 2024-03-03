

/*
need to add <start_of_file> and <empty> tokens
maybe <end_of_file> too? not needed since have "end of file" token?
*/

use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{vgm_commands::Commands, header::HeaderData, systems::System};


#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ChipPayload {
    pub system: System,
    pub clock_value: u32
}

impl ChipPayload {
    fn new(system: System, clock_value: u32) -> Self {
        return ChipPayload { system: system, clock_value: clock_value };
    }
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum ExtendedInstructionSet {
    StartFile, 
    EndHeader,
    SetChip(ChipPayload),
    VgmCommand(Commands)
}


#[derive(Serialize, Deserialize)]
pub struct Registry {
    instruction_to_token: HashMap<ExtendedInstructionSet, usize>,
    token_to_instruction: Vec<ExtendedInstructionSet> //HashMap<u64, ExtendedInstructionSet>
}

impl Registry {
    pub fn new() -> Self {
        let mut registry = Registry { instruction_to_token: HashMap::new(), token_to_instruction: vec![] };
        let mut curr_id = 0;
        for instruction in vec![
            ExtendedInstructionSet::StartFile,
            ExtendedInstructionSet::EndHeader,
        ] {
            registry.instruction_to_token.insert(instruction.clone(), curr_id);
            registry.token_to_instruction.push(instruction);
            curr_id += 1;
        }
        return registry;
    }
}

pub fn find_clock_commands(header: &HeaderData) -> Vec<ExtendedInstructionSet> {
    let mut chip_payloads: Vec<ChipPayload> = vec![];

    if header.SN76489_clock != 0 {
        chip_payloads.push(
            ChipPayload::new(System::SN76489, header.SN76489_clock)
        );
    }

    // 0x10
    chip_payloads.push(
        ChipPayload::new(System::YM2413, header.YM2413_clock)
    );



    // 0x20
    chip_payloads.push(
        ChipPayload::new(System::YM2612, header.YM2612_clock)
    );


    //pub SN76489_feedback: u16,
    //pub SN76489_shift_register_width: u8,
    //pub SN76489_flags: u8,

    // 0x30
    chip_payloads.push(
        ChipPayload::new(System::YM2151, header.YM2151_clock)
    );

    chip_payloads.push(
        ChipPayload::new(System::SegaPcm, header.SegaPCM_clock)
    );

    //pub SPCM_interface: u32,

    // 0x40
    chip_payloads.push(
        ChipPayload::new(System::RF5C68, header.RF5C68_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::YM2203, header.YM2203_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::YM2608, header.YM2608_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::YM2610, header.YM2610B_clock)
    );


    // 0x50
    chip_payloads.push(
        ChipPayload::new(System::YM3812, header.YM3812_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::YM3526, header.YM3526_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::Y8950, header.Y8950_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::YMF262, header.YMF262_clock)
    );


    // 0x60
    chip_payloads.push(
        ChipPayload::new(System::YMF278B, header.YMF278B_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::YMF271, header.YMF271_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::YMZ280B, header.YMZ280B_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::RF5C164, header.RF5C164_clock)
    );


    // 0x70
    chip_payloads.push(
        ChipPayload::new(System::Pwm, header.PWM_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::AY8910, header.AY8910_clock)
    );

    //pub AY8910_chip_type: u8,
    //pub AY8910_flags: u8,
    //pub YM2203_AY8910_flags: u8,
    //pub YM2608_AY8910_flags: u8,
    //pub volume_modifier: u8,
    //pub loop_base: u8,
    //pub loop_modifier: u8,

    // 0x80
    chip_payloads.push(
        ChipPayload::new(System::GameboyDmg, header.GB_DMG_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::NesApu, header.NES_APU_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::MultiPcm, header.MultiPCM_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::UPD7759, header.uPD7759_clock)
    );



    // 0x90
    chip_payloads.push(
        ChipPayload::new(System::OKIM6258, header.OKIM6258_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::OKIM6295, header.OKIM6295_clock)
    );

    chip_payloads.push(
        // pub K051649_K052539_clock: u32,
        ChipPayload::new(System::K051649, header.K051649_K052539_clock)
    );

    // pub OKIM6258_flags: u8,
    //  pub K054539_flags: u8,
    // pub C140_chip_type: u8,

    // 0xA0
    chip_payloads.push(
        ChipPayload::new(System::K054539, header.K054539_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::HuC6280, header.HuC6280_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::C140, header.C140_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::K053260, header.K053260_clock)
    );


    // 0xB0
    chip_payloads.push(
        ChipPayload::new(System::Pokey, header.Pokey_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::QSound, header.QSound_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::SCSP, header.SCSP_clock)
    );

    // pub extra_header_offset: u32,

    // 0xC0
    chip_payloads.push(
        ChipPayload::new(System::WonderSwan, header.WonderSwan_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::VSU, header.VSU_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::SAA1099, header.SAA1099_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::ES5503, header.ES5503_clock)
    );

    // 0xD0
    chip_payloads.push(
        ChipPayload::new(System::ES5506, header.ES5506_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::X1_010, header.X1010_clock)
    );
    chip_payloads.push(
        ChipPayload::new(System::C352, header.C352_clock)
    );

    // pub ES5503_nb_channels: u8,
    // pub ES5505_ES5506_nb_channels: u8,
    // pub C352_clock_divider: u8,

    // 0xE0
    chip_payloads.push(
        ChipPayload::new(System::GA20, header.GA20_clock)
    );

    // prune chips with 0 clock 
    return chip_payloads.into_iter().filter_map(
        |payload| if payload.clock_value == 0 {
            None
        } else {
            Some(ExtendedInstructionSet::SetChip(payload))
        }
    ).collect();

}


pub fn allocate_commands(vgm_command: Commands, registry: &mut Registry, curr_id: usize) -> Option<usize> {
    let wrapped_command = ExtendedInstructionSet::VgmCommand(vgm_command);
    if registry.instruction_to_token.contains_key(&wrapped_command) {
        return None;
    } else {
        registry.instruction_to_token.insert(wrapped_command.clone(), curr_id);
        registry.token_to_instruction.push(wrapped_command);

        return Some(curr_id + 1);
    }
}


