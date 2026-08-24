pub fn seed_to_tick(seed: &str) -> u32 {
    if seed.is_empty() {
        return 0;
    }
    let mut hash = 0_u32;
    for byte in seed.bytes() {
        hash = hash
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(u32::from(byte));
    }
    hash.max(1)
}

pub const fn mix(seed: u32, salt: u32) -> u32 {
    seed.wrapping_mul(0x9E37_79B9)
        .wrapping_add(salt.wrapping_mul(0x85EB_CA6B))
}
