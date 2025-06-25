use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum System {
    SN76489,
    YM2413,
    YM2612,
    YM2151,
    SegaPcm,
    RF5C68,
    YM2203,
    YM2608,
    YM2610, // Bit 31 is used to set whether it is an YM2610 or an YM2610B chip
    YM3812,
    YM3526,
    Y8950,
    YMF262,
    YMF278B,
    YMF271,
    YMZ280B,
    RF5C164,
    Pwm,
    AY8910,
    GameboyDmg,
    NesApu,
    MultiPcm,
    UPD7759,
    OKIM6258,
    K054539,
    C140,
    OKIM6295,
    K051649, // If bit 31 is set it is a K052539.
    K052539,
    HuC6280,
    K053260,
    Pokey,
    QSound,
    SCSP,
    WonderSwan,
    VSU,
    SAA1099,
    ES5503,
    ES5505, // If bit 31 is set it is an ES5506, if bit 31 is clear it is an ES5505.
    ES5506,
    C352,
    X1_010,
    GA20,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn test_system_variants_exist() {
        // Test that all documented system variants are accessible
        let _sn76489 = System::SN76489;
        let _ym2413 = System::YM2413;
        let _ym2612 = System::YM2612;
        let _ym2151 = System::YM2151;
        let _sega_pcm = System::SegaPcm;
        let _rf5c68 = System::RF5C68;
        let _ym2203 = System::YM2203;
        let _ym2608 = System::YM2608;
        let _ym2610 = System::YM2610;
        let _ym3812 = System::YM3812;
        let _ym3526 = System::YM3526;
        let _y8950 = System::Y8950;
        let _ymf262 = System::YMF262;
        let _ymf278b = System::YMF278B;
        let _ymf271 = System::YMF271;
        let _ymz280b = System::YMZ280B;
        let _rf5c164 = System::RF5C164;
        let _pwm = System::Pwm;
        let _ay8910 = System::AY8910;
        let _gameboy_dmg = System::GameboyDmg;
        let _nes_apu = System::NesApu;
        let _multi_pcm = System::MultiPcm;
        let _upd7759 = System::UPD7759;
        let _okim6258 = System::OKIM6258;
        let _k054539 = System::K054539;
        let _c140 = System::C140;
        let _okim6295 = System::OKIM6295;
        let _k051649 = System::K051649;
        let _k052539 = System::K052539;
        let _huc6280 = System::HuC6280;
        let _k053260 = System::K053260;
        let _pokey = System::Pokey;
        let _qsound = System::QSound;
        let _scsp = System::SCSP;
        let _wonderswan = System::WonderSwan;
        let _vsu = System::VSU;
        let _saa1099 = System::SAA1099;
        let _es5503 = System::ES5503;
        let _es5505 = System::ES5505;
        let _es5506 = System::ES5506;
        let _c352 = System::C352;
        let _x1_010 = System::X1_010;
        let _ga20 = System::GA20;
    }

    #[test]
    fn test_system_variant_count() {
        // Create a vector with all system variants to verify count
        let all_systems = vec![
            System::SN76489,
            System::YM2413,
            System::YM2612,
            System::YM2151,
            System::SegaPcm,
            System::RF5C68,
            System::YM2203,
            System::YM2608,
            System::YM2610,
            System::YM3812,
            System::YM3526,
            System::Y8950,
            System::YMF262,
            System::YMF278B,
            System::YMF271,
            System::YMZ280B,
            System::RF5C164,
            System::Pwm,
            System::AY8910,
            System::GameboyDmg,
            System::NesApu,
            System::MultiPcm,
            System::UPD7759,
            System::OKIM6258,
            System::K054539,
            System::C140,
            System::OKIM6295,
            System::K051649,
            System::K052539,
            System::HuC6280,
            System::K053260,
            System::Pokey,
            System::QSound,
            System::SCSP,
            System::WonderSwan,
            System::VSU,
            System::SAA1099,
            System::ES5503,
            System::ES5505,
            System::ES5506,
            System::C352,
            System::X1_010,
            System::GA20,
        ];

        // Verify we have 43 different systems
        assert_eq!(all_systems.len(), 43);

        // Verify all systems are unique
        let mut unique_systems = std::collections::HashSet::new();
        for system in &all_systems {
            assert!(unique_systems.insert(system.clone()), "Duplicate system found: {:?}", system);
        }
        assert_eq!(unique_systems.len(), 43);
    }

    #[test]
    fn test_partial_eq_trait() {
        // Test equality
        assert_eq!(System::SN76489, System::SN76489);
        assert_eq!(System::YM2612, System::YM2612);
        assert_eq!(System::GameboyDmg, System::GameboyDmg);

        // Test inequality
        assert_ne!(System::SN76489, System::YM2612);
        assert_ne!(System::YM2413, System::YM2151);
        assert_ne!(System::K051649, System::K052539);
        assert_ne!(System::ES5505, System::ES5506);

        // Test special chip pairs that are closely related
        assert_ne!(System::YM2610, System::YM2608);
        assert_ne!(System::RF5C68, System::RF5C164);
        assert_ne!(System::OKIM6258, System::OKIM6295);
    }

    #[test]
    fn test_eq_trait() {
        // Test reflexivity
        let sn76489 = System::SN76489;
        assert_eq!(sn76489, sn76489);

        // Test symmetry
        let ym2612_a = System::YM2612;
        let ym2612_b = System::YM2612;
        assert_eq!(ym2612_a, ym2612_b);
        assert_eq!(ym2612_b, ym2612_a);

        // Test transitivity
        let gameboy_a = System::GameboyDmg;
        let gameboy_b = System::GameboyDmg;
        let gameboy_c = System::GameboyDmg;
        assert_eq!(gameboy_a, gameboy_b);
        assert_eq!(gameboy_b, gameboy_c);
        assert_eq!(gameboy_a, gameboy_c);
    }

    #[test]
    fn test_hash_trait() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Test that equal systems have equal hashes
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        
        System::YM2612.hash(&mut hasher1);
        System::YM2612.hash(&mut hasher2);
        
        assert_eq!(hasher1.finish(), hasher2.finish());

        // Test that different systems have different hashes (usually)
        let mut hasher3 = DefaultHasher::new();
        let mut hasher4 = DefaultHasher::new();
        
        System::SN76489.hash(&mut hasher3);
        System::YM2413.hash(&mut hasher4);
        
        // Note: Hash collisions are possible but unlikely for these distinct values
        assert_ne!(hasher3.finish(), hasher4.finish());
    }

    #[test]
    fn test_hash_map_usage() {
        // Test that System can be used as HashMap keys
        let mut system_map: HashMap<System, &str> = HashMap::new();
        
        system_map.insert(System::SN76489, "PSG");
        system_map.insert(System::YM2612, "FM");
        system_map.insert(System::GameboyDmg, "DMG");
        system_map.insert(System::NesApu, "APU");
        
        assert_eq!(system_map.get(&System::SN76489), Some(&"PSG"));
        assert_eq!(system_map.get(&System::YM2612), Some(&"FM"));
        assert_eq!(system_map.get(&System::GameboyDmg), Some(&"DMG"));
        assert_eq!(system_map.get(&System::NesApu), Some(&"APU"));
        assert_eq!(system_map.get(&System::YM2413), None);
        
        assert_eq!(system_map.len(), 4);
    }

    #[test]
    fn test_clone_trait() {
        let original = System::SCSP;
        let cloned = original.clone();
        
        // Test that clone produces an equal object
        assert_eq!(original, cloned);
        
        // Test that clone works with collections
        let systems = vec![System::QSound, System::C352, System::X1_010];
        let cloned_systems = systems.clone();
        
        assert_eq!(systems, cloned_systems);
        assert_eq!(systems.len(), cloned_systems.len());
        
        for (orig, clone) in systems.iter().zip(cloned_systems.iter()) {
            assert_eq!(orig, clone);
        }
    }

    #[test]
    fn test_debug_trait() {
        // Test Debug formatting for various systems
        let debug_output = format!("{:?}", System::SN76489);
        assert_eq!(debug_output, "SN76489");
        
        let debug_output = format!("{:?}", System::YM2612);
        assert_eq!(debug_output, "YM2612");
        
        let debug_output = format!("{:?}", System::GameboyDmg);
        assert_eq!(debug_output, "GameboyDmg");
        
        let debug_output = format!("{:?}", System::X1_010);
        assert_eq!(debug_output, "X1_010");
        
        // Test debug formatting in collections
        let systems = vec![System::Pokey, System::SAA1099];
        let debug_output = format!("{:?}", systems);
        assert!(debug_output.contains("Pokey"));
        assert!(debug_output.contains("SAA1099"));
    }

    #[test]
    fn test_serialize() {
        // Test serialization of individual systems
        let sn76489_json = serde_json::to_string(&System::SN76489).unwrap();
        assert_eq!(sn76489_json, "\"SN76489\"");
        
        let ym2612_json = serde_json::to_string(&System::YM2612).unwrap();
        assert_eq!(ym2612_json, "\"YM2612\"");
        
        let gameboy_json = serde_json::to_string(&System::GameboyDmg).unwrap();
        assert_eq!(gameboy_json, "\"GameboyDmg\"");
        
        // Test serialization of collection
        let systems = vec![System::ES5503, System::ES5505, System::ES5506];
        let systems_json = serde_json::to_string(&systems).unwrap();
        assert!(systems_json.contains("ES5503"));
        assert!(systems_json.contains("ES5505"));
        assert!(systems_json.contains("ES5506"));
    }

    #[test]
    fn test_deserialize() {
        // Test deserialization of individual systems
        let sn76489: System = serde_json::from_str("\"SN76489\"").unwrap();
        assert_eq!(sn76489, System::SN76489);
        
        let ym2612: System = serde_json::from_str("\"YM2612\"").unwrap();
        assert_eq!(ym2612, System::YM2612);
        
        let gameboy: System = serde_json::from_str("\"GameboyDmg\"").unwrap();
        assert_eq!(gameboy, System::GameboyDmg);
        
        // Test deserialization of collection
        let systems_json = "[\"C140\", \"C352\", \"GA20\"]";
        let systems: Vec<System> = serde_json::from_str(systems_json).unwrap();
        assert_eq!(systems.len(), 3);
        assert_eq!(systems[0], System::C140);
        assert_eq!(systems[1], System::C352);
        assert_eq!(systems[2], System::GA20);
    }

    #[test]
    fn test_serialize_deserialize_round_trip() {
        let test_systems = vec![
            System::SN76489,
            System::YM2413,
            System::YM2612,
            System::YM2151,
            System::SegaPcm,
            System::RF5C68,
            System::GameboyDmg,
            System::NesApu,
            System::VSU,
            System::WonderSwan,
        ];
        
        for system in &test_systems {
            // Serialize
            let json = serde_json::to_string(system).unwrap();
            
            // Deserialize
            let deserialized: System = serde_json::from_str(&json).unwrap();
            
            // Verify round-trip
            assert_eq!(*system, deserialized);
        }
    }

    #[test]
    fn test_deserialize_invalid_system() {
        // Test that invalid system names fail to deserialize
        let result: Result<System, _> = serde_json::from_str("\"InvalidSystemName\"");
        assert!(result.is_err());
        
        let result: Result<System, _> = serde_json::from_str("\"ym2612\""); // Wrong case
        assert!(result.is_err());
        
        let result: Result<System, _> = serde_json::from_str("\"YM2612_EXTRA\""); // Extra text
        assert!(result.is_err());
        
        let result: Result<System, _> = serde_json::from_str("123"); // Not a string
        assert!(result.is_err());
    }

    #[test]
    fn test_classic_chip_families() {
        // Test PSG family
        let psg_chips = vec![System::SN76489, System::AY8910, System::GameboyDmg];
        for chip in &psg_chips {
            assert!(matches!(chip, System::SN76489 | System::AY8910 | System::GameboyDmg));
        }
        
        // Test Yamaha FM family
        let yamaha_fm_chips = vec![
            System::YM2413,
            System::YM2612,
            System::YM2151,
            System::YM2203,
            System::YM2608,
            System::YM2610,
            System::YM3812,
            System::YM3526,
            System::Y8950,
            System::YMF262,
            System::YMF278B,
            System::YMF271,
        ];
        
        for chip in &yamaha_fm_chips {
            assert!(matches!(chip,
                System::YM2413 | System::YM2612 | System::YM2151 | System::YM2203 |
                System::YM2608 | System::YM2610 | System::YM3812 | System::YM3526 |
                System::Y8950 | System::YMF262 | System::YMF278B | System::YMF271
            ));
        }
        
        // Test OKI family
        let oki_chips = vec![System::OKIM6258, System::OKIM6295];
        for chip in &oki_chips {
            assert!(matches!(chip, System::OKIM6258 | System::OKIM6295));
        }
        
        // Test Konami family
        let konami_chips = vec![System::K054539, System::K051649, System::K052539, System::K053260];
        for chip in &konami_chips {
            assert!(matches!(chip, System::K054539 | System::K051649 | System::K052539 | System::K053260));
        }
    }

    #[test]
    fn test_special_chip_variants() {
        // Test chips with special bit 31 flags mentioned in comments
        
        // YM2610 vs YM2610B (comment mentions bit 31 usage)
        assert_ne!(System::YM2610, System::YM2608);
        assert_eq!(System::YM2610, System::YM2610); // Self equality
        
        // K051649 vs K052539 (comment mentions bit 31)
        assert_ne!(System::K051649, System::K052539);
        assert_eq!(System::K051649, System::K051649);
        assert_eq!(System::K052539, System::K052539);
        
        // ES5505 vs ES5506 (comment mentions bit 31)
        assert_ne!(System::ES5505, System::ES5506);
        assert_eq!(System::ES5505, System::ES5505);
        assert_eq!(System::ES5506, System::ES5506);
    }

    #[test]
    fn test_system_name_patterns() {
        // Test that system names follow expected patterns
        let systems_with_numbers = vec![
            System::YM2413,
            System::YM2612,
            System::YM2151,
            System::YM2203,
            System::YM2608,
            System::YM2610,
            System::YM3812,
            System::YM3526,
            System::Y8950,
            System::RF5C68,
            System::RF5C164,
            System::HuC6280,
            System::OKIM6258,
            System::OKIM6295,
            System::K054539,
            System::K051649,
            System::K052539,
            System::K053260,
            System::SAA1099,
            System::ES5503,
            System::ES5505,
            System::ES5506,
            System::C140,
            System::C352,
            System::X1_010,
            System::GA20,
        ];
        
        // Just verify these exist and are accessible
        for system in &systems_with_numbers {
            let debug_str = format!("{:?}", system);
            assert!(!debug_str.is_empty());
        }
        
        // Test systems with letter suffixes
        let systems_with_letters = vec![
            System::YMF262,
            System::YMF278B,
            System::YMF271,
            System::YMZ280B,
            System::UPD7759,
        ];
        
        for system in &systems_with_letters {
            let debug_str = format!("{:?}", system);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_console_specific_systems() {
        // Test console-specific sound systems
        let console_systems = vec![
            (System::GameboyDmg, "Game Boy"),
            (System::NesApu, "NES"),
            (System::WonderSwan, "WonderSwan"),
            (System::VSU, "Virtual Boy"),
        ];
        
        for (system, _console_name) in &console_systems {
            assert_eq!(*system, system.clone());
            let debug_str = format!("{:?}", system);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_arcade_specific_systems() {
        // Test arcade-specific sound systems
        let arcade_systems = vec![
            System::QSound,        // Capcom
            System::MultiPcm,      // Sega
            System::SCSP,          // Sega Saturn/arcade
            System::C140,          // Namco
            System::C352,          // Namco
            System::Pokey,         // Atari
        ];
        
        for system in &arcade_systems {
            assert_eq!(*system, system.clone());
            let debug_str = format!("{:?}", system);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_all_systems_comprehensive() {
        // Create a comprehensive list to ensure no systems are missed
        let all_systems = [
            System::SN76489,    // PSG
            System::YM2413,     // OPLL
            System::YM2612,     // OPN2
            System::YM2151,     // OPM
            System::SegaPcm,    // Sega PCM
            System::RF5C68,     // Ricoh
            System::YM2203,     // OPN
            System::YM2608,     // OPNA
            System::YM2610,     // OPNB
            System::YM3812,     // OPL2
            System::YM3526,     // OPL
            System::Y8950,      // MSX-Audio
            System::YMF262,     // OPL3
            System::YMF278B,    // OPL4
            System::YMF271,     // OPX
            System::YMZ280B,    // PCMD8
            System::RF5C164,    // Ricoh
            System::Pwm,        // PWM
            System::AY8910,     // PSG
            System::GameboyDmg, // DMG
            System::NesApu,     // 2A03/2A07
            System::MultiPcm,   // Sega MultiPCM
            System::UPD7759,    // NEC
            System::OKIM6258,   // OKI ADPCM
            System::K054539,    // Konami
            System::C140,       // Namco
            System::OKIM6295,   // OKI ADPCM
            System::K051649,    // Konami SCC
            System::K052539,    // Konami SCC+
            System::HuC6280,    // Hudson/NEC
            System::K053260,    // Konami
            System::Pokey,      // Atari
            System::QSound,     // Capcom
            System::SCSP,       // Yamaha
            System::WonderSwan, // Bandai
            System::VSU,        // Nintendo Virtual Boy
            System::SAA1099,    // Philips
            System::ES5503,     // Ensoniq
            System::ES5505,     // Ensoniq
            System::ES5506,     // Ensoniq
            System::C352,       // Namco
            System::X1_010,     // Seta
            System::GA20,       // Irem
        ];
        
        // Verify all systems are unique and accessible
        assert_eq!(all_systems.len(), 43);
        
        let unique_count = all_systems.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 43);
        
        // Test that all systems can be serialized and deserialized
        for system in &all_systems {
            let json = serde_json::to_string(system).unwrap();
            let deserialized: System = serde_json::from_str(&json).unwrap();
            assert_eq!(*system, deserialized);
        }
    }

    #[test]
    fn test_enum_memory_layout() {
        use std::mem;
        
        // Test that enum has reasonable memory footprint
        let size = mem::size_of::<System>();
        
        // Enum should be reasonably small (typically 1 byte for simple enums)
        assert!(size <= 8, "System enum size {} bytes is too large", size);
        
        // Test alignment
        let align = mem::align_of::<System>();
        assert!(align <= 8, "System enum alignment {} is too large", align);
    }

    #[test]
    fn test_system_ordering_consistency() {
        // Test that PartialEq is consistent with itself
        let systems = vec![
            System::SN76489,
            System::YM2612,
            System::GameboyDmg,
            System::C352,
        ];
        
        for (i, system_a) in systems.iter().enumerate() {
            for (j, system_b) in systems.iter().enumerate() {
                if i == j {
                    assert_eq!(system_a, system_b, "System should equal itself");
                } else {
                    assert_ne!(system_a, system_b, "Different systems should not be equal");
                }
            }
        }
    }
}
