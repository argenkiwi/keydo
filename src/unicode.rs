use crate::keys::{KeyCode, KEYD_CANCEL, KEYD_0, KEYD_1, KEYD_2, KEYD_3, KEYD_4, KEYD_5, KEYD_6, KEYD_7, KEYD_8, KEYD_9, KEYD_A, KEYD_B, KEYD_C, KEYD_D, KEYD_E, KEYD_F, KEYD_G, KEYD_H, KEYD_I, KEYD_J, KEYD_K, KEYD_L, KEYD_M, KEYD_N, KEYD_O, KEYD_P, KEYD_Q, KEYD_R, KEYD_S, KEYD_T, KEYD_U, KEYD_V, KEYD_W, KEYD_X, KEYD_Y, KEYD_Z};

// This should ideally be generated or loaded from a file
// For now, we'll provide a stub and the lookup logic.
pub const UNICODE_TABLE: &[u32] = &[
    // Sample entries
    0x1F600, // 😀
    0x1F601, // 😁
];

pub fn unicode_lookup_index(codepoint: u32) -> Option<usize> {
    UNICODE_TABLE.iter().position(|&cp| cp == codepoint)
}

pub fn unicode_get_sequence(idx: usize) -> [u16; 4] {
    let chars = [
        KEYD_0, KEYD_1, KEYD_2, KEYD_3, KEYD_4, KEYD_5, KEYD_6, KEYD_7,
        KEYD_8, KEYD_9, KEYD_A, KEYD_B, KEYD_C, KEYD_D, KEYD_E, KEYD_F,
        KEYD_G, KEYD_H, KEYD_I, KEYD_J, KEYD_K, KEYD_L, KEYD_M, KEYD_N,
        KEYD_O, KEYD_P, KEYD_Q, KEYD_R, KEYD_S, KEYD_T, KEYD_U, KEYD_V,
        KEYD_W, KEYD_X, KEYD_Y, KEYD_Z
    ];

    [
        KEYD_CANCEL,
        chars[(idx / (36 * 36)) % 36],
        chars[(idx / 36) % 36],
        chars[idx % 36],
    ]
}
