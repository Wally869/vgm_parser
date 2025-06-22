use bytes::{BytesMut, BufMut};



pub fn write_string_as_u16_bytes(buffer: &mut BytesMut, value: &str) {
    buffer.put(
        &value
            .encode_utf16()
            .map(|x| x.to_le_bytes())
            .flatten()
            .collect::<Vec<u8>>()[..],
    );
}



fn bcd_to_decimal(byte: u8) -> u32 {
    (((byte >> 4) * 10) + (byte & 0x0F)) as u32
}

/// Read bytes and return bcd version 
/// For example [0x51, 0x01, 0x00, 0x00] will return 151
pub fn bcd_from_bytes(bytes_list: &[u8]) -> u32 {
    let mut bytes_list = bytes_list.to_vec();
    bytes_list.reverse();
    let mut version = 0;
    for byte in bytes_list {
        version = version * 100 + bcd_to_decimal(byte);
    }

    return version;
}


/// Read decimal and return bcd version as bytes
/// For example 151 will return [0x51, 0x01, 0x00, 0x00] 
pub fn decimal_to_bcd(decimal: u32) -> Vec<u8> {
    //let decimal = decimal as u128;
    let mut bcd_bytes = Vec::new();
    let mut remaining = decimal;

    while remaining > 0 {
        let digit = remaining % 100;
        let bcd_byte = ((digit / 10) << 4) | (digit % 10);
        bcd_bytes.push(bcd_byte as u8);
        remaining /= 100;
    }

    //bcd_bytes.reverse();
    // pad bcd_bytes to get target length 
    bcd_bytes.push(0x00);
    bcd_bytes.push(0x00);

    return bcd_bytes;
}




#[cfg(test)]
mod test_utils {
    use bytes::{Bytes, Buf};

    use crate::utils::decimal_to_bcd;

    use super::bcd_from_bytes;

    #[test]
    fn bcd_cycle() {
        let version_bytes: [u8; 4] = [0x51, 0x01, 0x00, 0x00];
        let version = bcd_from_bytes(&version_bytes[..]);
        assert_eq!(version, 151);
        let out_bytes = decimal_to_bcd(version);
        assert_eq!(version_bytes.to_vec(), out_bytes);

        let version_bytes: [u8; 4] = [0x71, 0x01, 0x00, 0x00];
        let version = bcd_from_bytes(&version_bytes[..]);
        assert_eq!(version, 171);
        let out_bytes = decimal_to_bcd(version);
        assert_eq!(version_bytes.to_vec(), out_bytes);

        let version_bytes: [u8; 4] = [0x24, 0x01, 0x00, 0x00];
        let version = bcd_from_bytes(&version_bytes[..]);
        assert_eq!(version, 124);
        let out_bytes = decimal_to_bcd(version);
        assert_eq!(version_bytes.to_vec(), out_bytes);
    }
}

