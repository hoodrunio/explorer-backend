use bech32::ToBase32;

/// Converts given HEX string to bytes.
pub fn from_hex(hex_str: &str) -> Option<Vec<u8>> {
    let hex_bytes = hex_str.as_bytes();

    if hex_bytes.len() % 2 != 0 || hex_bytes.len() != hex_str.len() {
        None
    } else {
        let mut i = 0;

        let mut bytes = vec![];

        loop {
            if i == hex_bytes.len() {
                break;
            }

            bytes.push(hex_to_byte(hex_bytes[i])? * 16 + hex_to_byte(hex_bytes[i + 1])?);

            i += 2;
        }

        Some(bytes)
    }
}

/// Converts given HEX bytes to BASE 64 string.
pub fn to_base64(hex_bytes: &[u8]) -> String {
    base64::encode(hex_bytes)
}

/// Parses HEX encoded adresses to bech32 adresses.
pub fn to_bech32(hex_str: &str) -> Option<String> {
    bech32::encode("evmosvaloper", to_base64(&from_hex(hex_str)?).to_base32(), bech32::Variant::Bech32).ok()
}

/// Converts a single hex byte to byte.
fn hex_to_byte(hex_byte: u8) -> Option<u8> {
    // Between 0..9 in ANSI table.
    if hex_byte > 47 && hex_byte < 58 {
        Some(hex_byte - 48)
    }
    // Between uppercase A..F in ANSI table.
    else if hex_byte > 64 && hex_byte < 71 {
        Some(hex_byte - 55)
    }
    // Lowercase letters are not supported.
    else {
        None
    }
}
