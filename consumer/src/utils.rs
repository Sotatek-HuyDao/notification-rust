pub fn hex_to_dec(hex_str: String) -> u64 {
    // Remove the "0x" prefix if it exists
    let hex_number = hex_str.strip_prefix("0x").unwrap_or(&hex_str);

    // Convert the hexadecimal string to a number, returning 0 on error
    u64::from_str_radix(hex_number, 16).unwrap_or(0)
}
