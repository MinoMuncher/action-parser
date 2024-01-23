const ATTACK_TABLE: [[[usize; 21]; 9]; 5] = [
    // straight taken from osk's table: https://cdn.discordapp.com/attachments/674421736162197515/716081165886423110/2020-05-30_02-07-18.png
    [
        [
            0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3,
        ], // Single // hits 4 at 43
        [
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6,
        ], // Double. Increments every 4 combos.
        [
            2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
        ], // Triple. Increments every 2 combos.
        [
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ], // Quad. Increments every combo.
        [
            0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3,
        ], // T-Spin Mini Single. Same as single, but gets B2B.
        [
            2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
        ], // T-Spin Single. Same as triple, but gets B2B.
        [
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6,
        ], // T-Spin Mini Double. Same as double, but gets B2B.
        [
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ], // T-Spin Double. Same as quad.
        [
            6, 7, 9, 10, 12, 13, 15, 16, 18, 19, 21, 22, 24, 25, 27, 28, 30, 31, 33, 34, 36,
        ], // T-Spin Triple. Increments every combo and by 2 every 2 combos.
    ], // B2B 0
    [
        [0; 21],
        [0; 21],
        [0; 21], // single double triple
        [
            5, 6, 7, 8, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21, 22, 23, 25, 26, 27, 28, 30,
        ], // Quad
        [
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6,
        ], // T-Spin Mini Single
        [
            3, 3, 4, 5, 6, 6, 7, 8, 9, 9, 10, 11, 12, 12, 13, 14, 15, 15, 16, 17, 18,
        ], // T-Spin Single
        [
            2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
        ], // T-Spin Mini Double
        [
            5, 6, 7, 8, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21, 22, 23, 25, 26, 27, 28, 30,
        ], // T-Spin Double
        [
            7, 8, 10, 12, 14, 15, 17, 19, 21, 22, 24, 26, 28, 29, 31, 33, 35, 36, 38, 40, 42,
        ], // T-Spin Triple
    ], // B2B 1
    [
        [0; 21],
        [0; 21],
        [0; 21], // single double triple
        [
            6, 7, 9, 10, 12, 13, 15, 16, 18, 19, 21, 22, 24, 25, 27, 28, 30, 31, 33, 34, 36,
        ], // Quad
        [
            2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
        ], // T-Spin Mini Single
        [
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ], // T-Spin Single
        [
            3, 3, 4, 5, 6, 6, 7, 8, 9, 9, 10, 11, 12, 12, 13, 14, 15, 15, 16, 17, 18,
        ], // T-Spin Mini Double
        [
            6, 7, 9, 10, 12, 13, 15, 16, 18, 19, 21, 22, 24, 25, 27, 28, 30, 31, 33, 34, 36,
        ], // T-Spin Double
        [
            8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48,
        ], // T-Spin Triple
    ], // B2B 2
    [
        [0; 21],
        [0; 21],
        [0; 21], // single double triple
        [
            7, 8, 10, 12, 14, 15, 17, 19, 21, 22, 24, 26, 28, 29, 31, 33, 35, 36, 38, 40, 42,
        ], // Quad
        [
            3, 3, 4, 5, 6, 6, 7, 8, 9, 9, 10, 11, 12, 12, 13, 14, 15, 15, 16, 17, 18,
        ], // T-Spin Mini Single
        [
            5, 6, 7, 8, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21, 22, 23, 25, 26, 27, 28, 30,
        ], // T-Spin Single
        [
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ], // T-Spin Mini Double
        [
            7, 8, 10, 12, 14, 15, 17, 19, 21, 22, 24, 26, 28, 29, 31, 33, 35, 36, 38, 40, 42,
        ], // T-Spin Double
        [
            9, 11, 13, 15, 18, 20, 22, 24, 27, 29, 31, 33, 36, 38, 40, 42, 45, 47, 49, 51, 54,
        ], // T-Spin Triple
    ], // B2B 3
    [
        [0; 21],
        [0; 21],
        [0; 21], // single double triple
        [
            8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48,
        ], // Quad
        [
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ], // T-Spin Mini Single
        [
            6, 7, 9, 10, 12, 13, 15, 16, 18, 19, 21, 22, 24, 25, 27, 28, 30, 31, 33, 34, 36,
        ], // T-Spin Single
        [
            5, 6, 7, 8, 10, 11, 12, 13, 15, 16, 17, 18, 20, 21, 22, 23, 25, 26, 27, 28, 30,
        ], // T-Spin Mini Double
        [
            8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48,
        ], // T-Spin Double
        [
            10, 12, 15, 17, 20, 22, 25, 27, 30, 32, 35, 37, 40, 42, 45, 47, 50, 52, 55, 57, 60,
        ], // T-Spin Triple
    ], // B2B 4
];
///Get attack amount from provided tetrio attack table, accurate enough for most cases.
pub fn get_indexed_attack(clear_type_index: usize, combo: usize, btb: usize) -> usize {
    let btb_level = btb_level(btb);
    if btb_level < ATTACK_TABLE.len() {
        if combo < 21 {
            return ATTACK_TABLE[btb_level][clear_type_index][combo];
        }
        if btb_level == 0 && (clear_type_index == 0 || clear_type_index == 4) && combo >= 43 {
            return 4;
        }
        return ATTACK_TABLE[btb_level][clear_type_index][20];
    }
    get_indexed_attack(clear_type_index, combo, 0) + btb_level
}

fn btb_level(btb: usize) -> usize {
    if btb < 1 {
        return 0;
    }
    if btb < 3 {
        return 1;
    }
    if btb < 8 {
        return 2;
    }
    if btb < 24 {
        return 3;
    }
    if btb < 67 {
        return 4;
    }
    if btb < 185 {
        return 5;
    }
    if btb < 504 {
        return 6;
    }
    if btb < 1370 {
        return 7;
    }
    8// next "level" starts at ~3725 but we're keeping it oskreveal
}
