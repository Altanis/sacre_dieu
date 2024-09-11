use bytemuck::cast_slice;
use strum::EnumCount;

use super::{board::Bitboard, piece::{PieceType, PieceColor}};

pub const MAX_LEGAL_MOVES: usize = 218;
pub const PIECE_INDICES: usize = PieceType::COUNT + PieceColor::COUNT;

pub const fn get_piece_type(piece_code: char) -> PieceType {
    match piece_code {
        'p' => PieceType::Pawn,
        'n' => PieceType::Knight,
        'b' => PieceType::Bishop,
        'r' => PieceType::Rook,
        'q' => PieceType::Queen,
        'k' => PieceType::King,
        _ => panic!("invalid piece code")
    }
}

pub const MAX_DEPTH: i32 = 127;

// Constants which represent evaluation thresholds.
pub const WORST_EVAL: i32 = -i32::MAX;
pub const SHALLOWEST_PROVEN_LOSS: i32 = -100000;
pub const DEEPEST_PROVEN_LOSS: i32 = SHALLOWEST_PROVEN_LOSS + MAX_DEPTH;
pub const DEEPEST_PROVEN_WIN: i32 = SHALLOWEST_PROVEN_WIN - MAX_DEPTH;
pub const SHALLOWEST_PROVEN_WIN: i32 = 100000;
pub const BEST_EVAL: i32 = i32::MAX;


// Values of each piece, in centipawns.
pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 300;
pub const BISHOP_VALUE: i32 = 320;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 0;

// Reverse Futility Pruning constants.
pub const RFP_DEPTH: usize = 5;
pub const RFP_THRESHOLD: usize = 200;

// PSQT table, stolen from Pesto.
// NOTE: These PSQT tables assume A8 = 0.
pub const PIECE_SQUARE_TABLE: [[(i32, i32); 64]; PieceType::COUNT] = [
    // Pawn
    [
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        (98, 178), (134, 173), (61, 158), (95, 134), (68, 147), (126, 132), (34, 165), (-11, 187),
        (-6, 94), (7, 100), (26, 85), (31, 67), (65, 56), (56, 53), (25, 82), (-20, 84),
        (-14, 32), (13, 24), (6, 13), (21, 5), (23, -2), (12, 4), (17, 17), (-23, 17),
        (-27, 13), (-2, 9), (-5, -3), (12, -7), (17, -7), (6, -8), (10, 3), (-25, -1),
        (-26, 4), (-4, 7), (-4, -6), (-10, 1), (3, 0), (3, -5), (33, -1), (-12, -8),
        (-35, 13), (-1, 8), (-20, 8), (-23, 10), (-15, 13), (24, 0), (38, 2), (-22, -7),
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
    ],
    // Knight
    [
        (-167, -58), (-89, -38), (-34, -13), (-49, -28), (61, -31), (-97, -27), (-15, -63), (-107, -99),
        (-73, -25), (-41, -8), (72, -25), (36, -2), (23, -9), (62, -25), (7, -24), (-17, -52),
        (-47, -24), (60, -20), (37, 10), (65, 9), (84, -1), (129, -9), (73, -19), (44, -41),
        (-9, -17), (17, 3), (19, 22), (53, 22), (37, 22), (69, 11), (18, 8), (22, -18),
        (-13, -18), (4, -6), (16, 16), (13, 25), (28, 16), (19, 17), (21, 4), (-8, -18),
        (-23, -23), (-9, -3), (12, -1), (10, 15), (19, 10), (17, -3), (25, -20), (-16, -22),
        (-29, -42), (-53, -20), (-12, -10), (-3, -5), (-1, -2), (18, -20), (-14, -23), (-19, -44),
        (-105, -29), (-21, -51), (-58, -23), (-33, -15), (-17, -22), (-28, -18), (-19, -50), (-23, -64),
    ],
    // Bishop
    [
        (-29, -14), (4, -21), (-82, -11), (-37, -8), (-25, -7), (-42, -9), (7, -17), (-8, -24),
        (-26, -8), (16, -4), (-18, 7), (-13, -12), (30, -3), (59, -13), (18, -4), (-47, -14),
        (-16, 2), (37, -8), (43, 0), (40, -1), (35, -2), (50, 6), (37, 0), (-2, 4),
        (-4, -3), (5, 9), (19, 12), (50, 9), (37, 14), (37, 10), (7, 3), (-2, 2),
        (-6, -6), (13, 3), (13, 13), (26, 19), (34, 7), (12, 10), (10, -3), (4, -9),
        (0, -12), (15, -3), (15, 8), (15, 10), (14, 13), (27, 3), (18, -7), (10, -15),
        (4, -14), (15, -18), (16, -7), (0, -1), (7, 4), (21, -9), (33, -15), (1, -27),
        (-33, -23), (-3, -9), (-14, -23), (-21, -5), (-13, -9), (-12, -16), (-39, -5), (-21, -17),
    ],
    // Rook
    [
        (32, 13), (42, 10), (32, 18), (51, 15), (63, 12), (9, 12), (31, 8), (43, 5),
        (27, 11), (32, 13), (58, 13), (62, 11), (80, -3), (67, 3), (26, 8), (44, 3),
        (-5, 7), (19, 7), (26, 7), (36, 5), (17, 4), (45, -3), (61, -5), (16, -3),
        (-24, 4), (-11, 3), (7, 13), (26, 1), (24, 2), (35, 1), (-8, -1), (-20, 2),
        (-36, 3), (-26, 5), (-12, 8), (-1, 4), (9, -5), (-7, -6), (6, -8), (-23, -11),
        (-45, -4), (-25, 0), (-16, -5), (-17, -1), (3, -7), (0, -12), (-5, -8), (-33, -16),
        (-44, -6), (-16, -6), (-20, 0), (-9, 2), (-1, -9), (11, -9), (-6, -11), (-71, -3),
        (-19, -9), (-13, 2), (1, 3), (17, -1), (16, -5), (7, -13), (-37, 4), (-26, -20),
    ],
    // Queen
    [
        (-28, -9), (0, 22), (29, 22), (12, 27), (59, 27), (44, 19), (43, 10), (45, 20),
        (-24, -17), (-39, 20), (-5, 32), (1, 41), (-16, 58), (57, 25), (28, 30), (54, 0),
        (-13, -20), (-17, 6), (7, 9), (8, 49), (29, 47), (56, 35), (47, 19), (57, 9),
        (-27, 3), (-27, 22), (-16, 24), (-16, 45), (-1, 57), (17, 40), (-2, 57), (1, 36),
        (-9, -18), (-26, 28), (-9, 19), (-10, 47), (-2, 31), (-4, 34), (3, 39), (-3, 23),
        (-14, -16), (2, -27), (-11, 15), (-2, 6), (-5, 9), (2, 17), (14, 10), (5, 5),
        (-35, -22), (-8, -23), (11, -30), (2, -16), (8, -16), (15, -23), (-3, -36), (1, -32),
        (-1, -33), (-18, -28), (-9, -22), (10, -43), (-15, -5), (-25, -32), (-31, -20), (-50, -41),
    ],
    // King
    [
        (-65, -74), (23, -35), (16, -18), (-15, -18), (-56, -11), (-34, 15), (2, 4), (13, -17),
        (29, -12), (-1, 17), (-20, 14), (-7, 17), (-8, 17), (-4, 38), (-38, 23), (-29, 11),
        (-9, 10), (24, 17), (2, 23), (-16, 15), (-20, 20), (6, 45), (22, 44), (-22, 13),
        (-17, -8), (-20, 22), (-12, 24), (-27, 27), (-30, 26), (-25, 33), (-14, 26), (-36, 3),
        (-49, -18), (-1, -4), (-27, 21), (-39, 24), (-46, 27), (-44, 23), (-33, 9), (-51, -11),
        (-14, -19), (-14, -3), (-22, 11), (-46, 21), (-44, 23), (-30, 16), (-15, 7), (-27, -9),
        (1, -27), (7, -11), (-8, 4), (-64, 13), (-43, 14), (-16, 4), (9, -5), (8, -17),
        (-15, -53), (36, -34), (12, -21), (-54, -11), (8, -28), (-28, -14), (24, -24), (14, -43),
    ]
];

// pub const PIECE_SQUARE_TABLE: [[(i32, i32); 64]; PieceType::COUNT] = [
//     // WHITE
//     // Pawn
//     [
//         (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), 
//         (50, 80), (50, 80), (50, 80), (50, 80), (50, 80), (50, 80), (50, 80), (50, 80), 
//         (10, 50), (10, 50), (20, 50), (30, 30), (30, 30), (20, 50), (10, 50), (10, 50), 
//         (5, 30), (5, 30), (10, 30), (25, 20), (25, 20), (10, 30), (5, 30), (5, 30), 
//         (0, 20), (0, 20), (0, 20), (20, 20), (20, 20), (0, 20), (0, 20), (0, 20), 
//         (5, 10), (-5, 10), (-10, 10), (0, 10), (0, 10), (-10, 10), (-5, 10), (5, 10), 
//         (5, 10), (10, 10), (10, 10), (-20, 0), (-20, 0), (10, 10), (10, 10), (5, 10), 
//         (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), 
//     ],
//     // Knight
//     [
//         (-50, -50), (-40, -40), (-30, -30), (-30, -30), (-30, -30), (-30, -30), (-40, -40), (-50, -50), 
//         (-40, -40), (-20, -20), (0, 0), (0, 0), (0, 0), (0, 0), (-20, -20), (-40, -40), 
//         (-30, -30), (0, 0), (10, 10), (15, 15), (15, 15), (10, 10), (0, 0), (-30, -30), 
//         (-30, -30), (5, 5), (15, 15), (20, 20), (20, 20), (15, 15), (5, 5), (-30, -30), 
//         (-30, -30), (0, 0), (15, 15), (20, 20), (20, 20), (15, 15), (0, 0), (-30, -30), 
//         (-30, -30), (5, 5), (10, 10), (15, 15), (15, 15), (10, 10), (5, 5), (-30, -30), 
//         (-40, -40), (-20, -20), (0, 0), (5, 5), (5, 5), (0, 0), (-20, -20), (-40, -40), 
//         (-50, -50), (-40, -40), (-30, -30), (-30, -30), (-30, -30), (-30, -30), (-40, -40), (-50, -50), 
//     ],
//     // Bishop
//     [
//         (-20, -20), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-20, -20), 
//         (-10, -10), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (-10, -10), 
//         (-10, -10), (0, 5), (5, 10), (10, 10), (10, 10), (5, 10), (0, 5), (-10, -10), 
//         (-10, -10), (5, 5), (5, 10), (10, 10), (10, 10), (5, 10), (5, 5), (-10, -10), 
//         (-10, -10), (0, 10), (10, 10), (10, 10), (10, 10), (10, 10), (0, 10), (-10, -10), 
//         (-10, -10), (10, 10), (10, 10), (10, 10), (10, 10), (10, 10), (10, 10), (-10, -10), 
//         (-10, -10), (5, 5), (0, 0), (0, 0), (0, 0), (0, 0), (5, 5), (-10, -10), 
//         (-20, -20), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-20, -20), 
//     ],
//     // Rook
//     [
//         (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), 
//         (5, 5), (10, 10), (10, 10), (10, 10), (10, 10), (10, 10), (10, 10), (5, 5), 
//         (-5, -5), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (-5, -5), 
//         (-5, -5), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (-5, -5), 
//         (-5, -5), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (-5, -5), 
//         (-5, -5), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (-5, -5), 
//         (-5, -5), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (-5, -5), 
//         (0, 0), (0, 0), (0, 0), (5, 5), (5, 5), (0, 0), (0, 0), (0, 0), 
//     ],
//     // Queen
//     [
//         (-20, -20), (-10, -10), (-10, -10), (-5, -5), (-5, -5), (-10, -10), (-10, -10), (-20, -20), 
//         (-10, -10), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (-10, -10), 
//         (-10, -10), (0, 5), (5, 5), (5, 5), (5, 5), (5, 5), (0, 0), (-10, -10), 
//         (-5, -5), (0, 5), (5, 5), (5, 5), (5, 5), (5, 5), (0, 0), (-5, -5), 
//         (0, -5), (0, 5), (5, 5), (5, 5), (5, 5), (5, 5), (0, 0), (-5, -5), 
//         (-10, -10), (5, 5), (5, 5), (5, 5), (5, 5), (5, 5), (0, 0), (-10, -10), 
//         (-10, -10), (0, 5), (0, 5), (0, 0), (0, 0), (0, 0), (0, 0), (-10, -10), 
//         (-20, -20), (-10, -10), (-10, -10), (-5, -5), (-5, -5), (-10, -10), (-10, -10), (-20, -20), 
//     ],
//     // King
//     [

//         (-80, -20), (-70, -10), (-70, -10), (-70, -10), (-70, -10), (-70, -10), (-70, -10), (-80, -20), 
//         (-60, -5), (-60, 0), (-60, 5), (-60, 5), (-60, 5), (-60, 5), (-60, 0), (-60, -5), 
//         (-40, -10), (-50, 20), (-50, 30), (-60, 30), (-60, 30), (-50, 30), (-50, 20), (-40, -10), 
//         (-30, -15), (-40, 35), (-40, 45), (-50, 45), (-50, 45), (-40, 35), (-40, 30), (-30, -15), 
//         (-20, -20), (-30, 30), (-30, 40), (-40, 40), (-40, 40), (-30, 30), (-30, -15), (-20, -20), 
//         (-10, -25), (-20, 20), (-20, 25), (-20, 25), (-20, 25), (-20, 20), (-20, -20), (-10, -25), 
//         (20, -30), (20, -25), (-5, 0), (-5, 0), (-5, 0), (-5, 0), (-25, -30), (-30, -50), 
//         (20, -50), (30, -30), (10, -30), (0, -30), (0, -30), (10, -30), (30, -30), (20, -50), 
//     ]
// ];


// Magics taken from Analog Hors.

/// A struct representing a magic entry, which can
/// produce a hash to index into a lookup table.
pub struct MagicEntry {
    pub mask: u64,
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

pub const ROOK_MAGICS: &[MagicEntry; 64] = &[MagicEntry { mask: 0x000101010101017E, magic: 0x5080008011400020, shift: 52, offset: 0 }, MagicEntry { mask: 0x000202020202027C, magic: 0x0140001000402000, shift: 53, offset: 4096 }, MagicEntry { mask: 0x000404040404047A, magic: 0x0280091000200480, shift: 53, offset: 6144 }, MagicEntry { mask: 0x0008080808080876, magic: 0x0700081001002084, shift: 53, offset: 8192 }, MagicEntry { mask: 0x001010101010106E, magic: 0x0300024408010030, shift: 53, offset: 10240 }, MagicEntry { mask: 0x002020202020205E, magic: 0x510004004E480100, shift: 53, offset: 12288 }, MagicEntry { mask: 0x004040404040403E, magic: 0x0400044128020090, shift: 53, offset: 14336 }, MagicEntry { mask: 0x008080808080807E, magic: 0x8080004100012080, shift: 52, offset: 16384 }, MagicEntry { mask: 0x0001010101017E00, magic: 0x0220800480C00124, shift: 53, offset: 20480 }, MagicEntry { mask: 0x0002020202027C00, magic: 0x0020401001C02000, shift: 54, offset: 22528 }, MagicEntry { mask: 0x0004040404047A00, magic: 0x000A002204428050, shift: 54, offset: 23552 }, MagicEntry { mask: 0x0008080808087600, magic: 0x004E002040100A00, shift: 54, offset: 24576 }, MagicEntry { mask: 0x0010101010106E00, magic: 0x0102000A00041020, shift: 54, offset: 25600 }, MagicEntry { mask: 0x0020202020205E00, magic: 0x0A0880040080C200, shift: 54, offset: 26624 }, MagicEntry { mask: 0x0040404040403E00, magic: 0x0002000600018408, shift: 54, offset: 27648 }, MagicEntry { mask: 0x0080808080807E00, magic: 0x0025001200518100, shift: 53, offset: 28672 }, MagicEntry { mask: 0x00010101017E0100, magic: 0x8900328001400080, shift: 53, offset: 30720 }, MagicEntry { mask: 0x00020202027C0200, magic: 0x0848810020400100, shift: 54, offset: 32768 }, MagicEntry { mask: 0x00040404047A0400, magic: 0xC001410020010153, shift: 54, offset: 33792 }, MagicEntry { mask: 0x0008080808760800, magic: 0x4110C90020100101, shift: 54, offset: 34816 }, MagicEntry { mask: 0x00101010106E1000, magic: 0x00A0808004004800, shift: 54, offset: 35840 }, MagicEntry { mask: 0x00202020205E2000, magic: 0x401080801C000601, shift: 54, offset: 36864 }, MagicEntry { mask: 0x00404040403E4000, magic: 0x0100040028104221, shift: 54, offset: 37888 }, MagicEntry { mask: 0x00808080807E8000, magic: 0x840002000900A054, shift: 53, offset: 38912 }, MagicEntry { mask: 0x000101017E010100, magic: 0x1000348280004000, shift: 53, offset: 40960 }, MagicEntry { mask: 0x000202027C020200, magic: 0x001000404000E008, shift: 54, offset: 43008 }, MagicEntry { mask: 0x000404047A040400, magic: 0x0424410300200035, shift: 54, offset: 44032 }, MagicEntry { mask: 0x0008080876080800, magic: 0x2008C22200085200, shift: 54, offset: 45056 }, MagicEntry { mask: 0x001010106E101000, magic: 0x0005304D00080100, shift: 54, offset: 46080 }, MagicEntry { mask: 0x002020205E202000, magic: 0x000C040080120080, shift: 54, offset: 47104 }, MagicEntry { mask: 0x004040403E404000, magic: 0x8404058400080210, shift: 54, offset: 48128 }, MagicEntry { mask: 0x008080807E808000, magic: 0x0001848200010464, shift: 53, offset: 49152 }, MagicEntry { mask: 0x0001017E01010100, magic: 0x6000204001800280, shift: 53, offset: 51200 }, MagicEntry { mask: 0x0002027C02020200, magic: 0x2410004003C02010, shift: 54, offset: 53248 }, MagicEntry { mask: 0x0004047A04040400, magic: 0x0181200A80801000, shift: 54, offset: 54272 }, MagicEntry { mask: 0x0008087608080800, magic: 0x000C60400A001200, shift: 54, offset: 55296 }, MagicEntry { mask: 0x0010106E10101000, magic: 0x0B00040180802800, shift: 54, offset: 56320 }, MagicEntry { mask: 0x0020205E20202000, magic: 0xC00A000280804C00, shift: 54, offset: 57344 }, MagicEntry { mask: 0x0040403E40404000, magic: 0x4040080504005210, shift: 54, offset: 58368 }, MagicEntry { mask: 0x0080807E80808000, magic: 0x0000208402000041, shift: 53, offset: 59392 }, MagicEntry { mask: 0x00017E0101010100, magic: 0xA200400080628000, shift: 53, offset: 61440 }, MagicEntry { mask: 0x00027C0202020200, magic: 0x0021020240820020, shift: 54, offset: 63488 }, MagicEntry { mask: 0x00047A0404040400, magic: 0x1020027000848022, shift: 54, offset: 64512 }, MagicEntry { mask: 0x0008760808080800, magic: 0x0020500018008080, shift: 54, offset: 65536 }, MagicEntry { mask: 0x00106E1010101000, magic: 0x10000D0008010010, shift: 54, offset: 66560 }, MagicEntry { mask: 0x00205E2020202000, magic: 0x0100020004008080, shift: 54, offset: 67584 }, MagicEntry { mask: 0x00403E4040404000, magic: 0x0008020004010100, shift: 54, offset: 68608 }, MagicEntry { mask: 0x00807E8080808000, magic: 0x12241C0880420003, shift: 53, offset: 69632 }, MagicEntry { mask: 0x007E010101010100, magic: 0x4000420024810200, shift: 53, offset: 71680 }, MagicEntry { mask: 0x007C020202020200, magic: 0x0103004000308100, shift: 54, offset: 73728 }, MagicEntry { mask: 0x007A040404040400, magic: 0x008C200010410300, shift: 54, offset: 74752 }, MagicEntry { mask: 0x0076080808080800, magic: 0x2410008050A80480, shift: 54, offset: 75776 }, MagicEntry { mask: 0x006E101010101000, magic: 0x0820880080040080, shift: 54, offset: 76800 }, MagicEntry { mask: 0x005E202020202000, magic: 0x0044220080040080, shift: 54, offset: 77824 }, MagicEntry { mask: 0x003E404040404000, magic: 0x2040100805120400, shift: 54, offset: 78848 }, MagicEntry { mask: 0x007E808080808000, magic: 0x0129000080C20100, shift: 53, offset: 79872 }, MagicEntry { mask: 0x7E01010101010100, magic: 0x0010402010800101, shift: 52, offset: 81920 }, MagicEntry { mask: 0x7C02020202020200, magic: 0x0648A01040008101, shift: 53, offset: 86016 }, MagicEntry { mask: 0x7A04040404040400, magic: 0x0006084102A00033, shift: 53, offset: 88064 }, MagicEntry { mask: 0x7608080808080800, magic: 0x0002000870C06006, shift: 53, offset: 90112 }, MagicEntry { mask: 0x6E10101010101000, magic: 0x0082008820100402, shift: 53, offset: 92160 }, MagicEntry { mask: 0x5E20202020202000, magic: 0x0012008410050806, shift: 53, offset: 94208 }, MagicEntry { mask: 0x3E40404040404000, magic: 0x2009408802100144, shift: 53, offset: 96256 }, MagicEntry { mask: 0x7E80808080808000, magic: 0x821080440020810A, shift: 52, offset: 98304 }];
pub const BISHOP_MAGICS: &[MagicEntry; 64] = &[MagicEntry { mask: 0x0040201008040200, magic: 0x2020420401002200, shift: 58, offset: 0 }, MagicEntry { mask: 0x0000402010080400, magic: 0x05210A020A002118, shift: 59, offset: 64 }, MagicEntry { mask: 0x0000004020100A00, magic: 0x1110040454C00484, shift: 59, offset: 96 }, MagicEntry { mask: 0x0000000040221400, magic: 0x1008095104080000, shift: 59, offset: 128 }, MagicEntry { mask: 0x0000000002442800, magic: 0xC409104004000000, shift: 59, offset: 160 }, MagicEntry { mask: 0x0000000204085000, magic: 0x0002901048080200, shift: 59, offset: 192 }, MagicEntry { mask: 0x0000020408102000, magic: 0x0044040402084301, shift: 59, offset: 224 }, MagicEntry { mask: 0x0002040810204000, magic: 0x2002030188040200, shift: 58, offset: 256 }, MagicEntry { mask: 0x0020100804020000, magic: 0x0000C8084808004A, shift: 59, offset: 320 }, MagicEntry { mask: 0x0040201008040000, magic: 0x1040040808010028, shift: 59, offset: 352 }, MagicEntry { mask: 0x00004020100A0000, magic: 0x40040C0114090051, shift: 59, offset: 384 }, MagicEntry { mask: 0x0000004022140000, magic: 0x40004820802004C4, shift: 59, offset: 416 }, MagicEntry { mask: 0x0000000244280000, magic: 0x0010042420260012, shift: 59, offset: 448 }, MagicEntry { mask: 0x0000020408500000, magic: 0x10024202300C010A, shift: 59, offset: 480 }, MagicEntry { mask: 0x0002040810200000, magic: 0x000054013D101000, shift: 59, offset: 512 }, MagicEntry { mask: 0x0004081020400000, magic: 0x0100020482188A0A, shift: 59, offset: 544 }, MagicEntry { mask: 0x0010080402000200, magic: 0x0120090421020200, shift: 59, offset: 576 }, MagicEntry { mask: 0x0020100804000400, magic: 0x1022204444040C00, shift: 59, offset: 608 }, MagicEntry { mask: 0x004020100A000A00, magic: 0x0008000400440288, shift: 57, offset: 640 }, MagicEntry { mask: 0x0000402214001400, magic: 0x0008060082004040, shift: 57, offset: 768 }, MagicEntry { mask: 0x0000024428002800, magic: 0x0044040081A00800, shift: 57, offset: 896 }, MagicEntry { mask: 0x0002040850005000, magic: 0x021200014308A010, shift: 57, offset: 1024 }, MagicEntry { mask: 0x0004081020002000, magic: 0x8604040080880809, shift: 59, offset: 1152 }, MagicEntry { mask: 0x0008102040004000, magic: 0x0000802D46009049, shift: 59, offset: 1184 }, MagicEntry { mask: 0x0008040200020400, magic: 0x00500E8040080604, shift: 59, offset: 1216 }, MagicEntry { mask: 0x0010080400040800, magic: 0x0024030030100320, shift: 59, offset: 1248 }, MagicEntry { mask: 0x0020100A000A1000, magic: 0x2004100002002440, shift: 57, offset: 1280 }, MagicEntry { mask: 0x0040221400142200, magic: 0x02090C0008440080, shift: 55, offset: 1408 }, MagicEntry { mask: 0x0002442800284400, magic: 0x0205010000104000, shift: 55, offset: 1920 }, MagicEntry { mask: 0x0004085000500800, magic: 0x0410820405004A00, shift: 57, offset: 2432 }, MagicEntry { mask: 0x0008102000201000, magic: 0x8004140261012100, shift: 59, offset: 2560 }, MagicEntry { mask: 0x0010204000402000, magic: 0x0A00460000820100, shift: 59, offset: 2592 }, MagicEntry { mask: 0x0004020002040800, magic: 0x201004A40A101044, shift: 59, offset: 2624 }, MagicEntry { mask: 0x0008040004081000, magic: 0x840C024220208440, shift: 59, offset: 2656 }, MagicEntry { mask: 0x00100A000A102000, magic: 0x000C002E00240401, shift: 57, offset: 2688 }, MagicEntry { mask: 0x0022140014224000, magic: 0x2220A00800010106, shift: 55, offset: 2816 }, MagicEntry { mask: 0x0044280028440200, magic: 0x88C0080820060020, shift: 55, offset: 3328 }, MagicEntry { mask: 0x0008500050080400, magic: 0x0818030B00A81041, shift: 57, offset: 3840 }, MagicEntry { mask: 0x0010200020100800, magic: 0xC091280200110900, shift: 59, offset: 3968 }, MagicEntry { mask: 0x0020400040201000, magic: 0x08A8114088804200, shift: 59, offset: 4000 }, MagicEntry { mask: 0x0002000204081000, magic: 0x228929109000C001, shift: 59, offset: 4032 }, MagicEntry { mask: 0x0004000408102000, magic: 0x1230480209205000, shift: 59, offset: 4064 }, MagicEntry { mask: 0x000A000A10204000, magic: 0x0A43040202000102, shift: 57, offset: 4096 }, MagicEntry { mask: 0x0014001422400000, magic: 0x1011284010444600, shift: 57, offset: 4224 }, MagicEntry { mask: 0x0028002844020000, magic: 0x0003041008864400, shift: 57, offset: 4352 }, MagicEntry { mask: 0x0050005008040200, magic: 0x0115010901000200, shift: 57, offset: 4480 }, MagicEntry { mask: 0x0020002010080400, magic: 0x01200402C0840201, shift: 59, offset: 4608 }, MagicEntry { mask: 0x0040004020100800, magic: 0x001A009400822110, shift: 59, offset: 4640 }, MagicEntry { mask: 0x0000020408102000, magic: 0x2002111128410000, shift: 59, offset: 4672 }, MagicEntry { mask: 0x0000040810204000, magic: 0x8420410288203000, shift: 59, offset: 4704 }, MagicEntry { mask: 0x00000A1020400000, magic: 0x0041210402090081, shift: 59, offset: 4736 }, MagicEntry { mask: 0x0000142240000000, magic: 0x8220002442120842, shift: 59, offset: 4768 }, MagicEntry { mask: 0x0000284402000000, magic: 0x0140004010450000, shift: 59, offset: 4800 }, MagicEntry { mask: 0x0000500804020000, magic: 0xC0408860086488A0, shift: 59, offset: 4832 }, MagicEntry { mask: 0x0000201008040200, magic: 0x0090203E00820002, shift: 59, offset: 4864 }, MagicEntry { mask: 0x0000402010080400, magic: 0x0820020083090024, shift: 59, offset: 4896 }, MagicEntry { mask: 0x0002040810204000, magic: 0x1040440210900C05, shift: 58, offset: 4928 }, MagicEntry { mask: 0x0004081020400000, magic: 0x0818182101082000, shift: 59, offset: 4992 }, MagicEntry { mask: 0x000A102040000000, magic: 0x0200800080D80800, shift: 59, offset: 5024 }, MagicEntry { mask: 0x0014224000000000, magic: 0x32A9220510209801, shift: 59, offset: 5056 }, MagicEntry { mask: 0x0028440200000000, magic: 0x0000901010820200, shift: 59, offset: 5088 }, MagicEntry { mask: 0x0050080402000000, magic: 0x0000014064080180, shift: 59, offset: 5120 }, MagicEntry { mask: 0x0020100804020000, magic: 0xA001204204080186, shift: 59, offset: 5152 }, MagicEntry { mask: 0x0040201008040200, magic: 0xC04010040258C048, shift: 58, offset: 5184 }];

// pub const ROOK_TABLE_SIZE: usize = 102400;
// pub const BISHOP_TABLE_SIZE: usize = 5248;

// Attack masks generated by my own generator.
// I removed the generators because it took up space.

pub const WHITE_PAWN_MASK: [(u64, u64); 64] = [ (0x00000000010100,0x00000000000200), (0x00000000020200,0x00000000000500), (0x00000000040400,0x00000000000a00), (0x00000000080800,0x00000000001400), (0x00000000101000,0x00000000002800), (0x00000000202000,0x00000000005000), (0x00000000404000,0x0000000000a000), (0x00000000808000,0x00000000004000), (0x00000001010000,0x00000000020000), (0x00000002020000,0x00000000050000), (0x00000004040000,0x000000000a0000), (0x00000008080000,0x00000000140000), (0x00000010100000,0x00000000280000), (0x00000020200000,0x00000000500000), (0x00000040400000,0x00000000a00000), (0x00000080800000,0x00000000400000), (0x00000101000000,0x00000002000000), (0x00000202000000,0x00000005000000), (0x00000404000000,0x0000000a000000), (0x00000808000000,0x00000014000000), (0x00001010000000,0x00000028000000), (0x00002020000000,0x00000050000000), (0x00004040000000,0x000000a0000000), (0x00008080000000,0x00000040000000), (0x00010100000000,0x00000200000000), (0x00020200000000,0x00000500000000), (0x00040400000000,0x00000a00000000), (0x00080800000000,0x00001400000000), (0x00101000000000,0x00002800000000), (0x00202000000000,0x00005000000000), (0x00404000000000,0x0000a000000000), (0x00808000000000,0x00004000000000), (0x01010000000000,0x00020000000000), (0x02020000000000,0x00050000000000), (0x04040000000000,0x000a0000000000), (0x08080000000000,0x00140000000000), (0x10100000000000,0x00280000000000), (0x20200000000000,0x00500000000000), (0x40400000000000,0x00a00000000000), (0x80800000000000,0x00400000000000), (0x101000000000000,0x02000000000000), (0x202000000000000,0x05000000000000), (0x404000000000000,0x0a000000000000), (0x808000000000000,0x14000000000000), (0x1010000000000000,0x28000000000000), (0x2020000000000000,0x50000000000000), (0x4040000000000000,0xa0000000000000), (0x8080000000000000,0x40000000000000), (0x100000000000000,0x200000000000000), (0x200000000000000,0x500000000000000), (0x400000000000000,0xa00000000000000), (0x800000000000000,0x1400000000000000), (0x1000000000000000,0x2800000000000000), (0x2000000000000000,0x5000000000000000), (0x4000000000000000,0xa000000000000000), (0x8000000000000000,0x4000000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000) ];
pub const BLACK_PAWN_MASK: [(u64, u64); 64] = [ (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000000,0x00000000000000), (0x00000000000001,0x00000000000002), (0x00000000000002,0x00000000000005), (0x00000000000004,0x0000000000000a), (0x00000000000008,0x00000000000014), (0x00000000000010,0x00000000000028), (0x00000000000020,0x00000000000050), (0x00000000000040,0x000000000000a0), (0x00000000000080,0x00000000000040), (0x00000000000101,0x00000000000200), (0x00000000000202,0x00000000000500), (0x00000000000404,0x00000000000a00), (0x00000000000808,0x00000000001400), (0x00000000001010,0x00000000002800), (0x00000000002020,0x00000000005000), (0x00000000004040,0x0000000000a000), (0x00000000008080,0x00000000004000), (0x00000000010100,0x00000000020000), (0x00000000020200,0x00000000050000), (0x00000000040400,0x000000000a0000), (0x00000000080800,0x00000000140000), (0x00000000101000,0x00000000280000), (0x00000000202000,0x00000000500000), (0x00000000404000,0x00000000a00000), (0x00000000808000,0x00000000400000), (0x00000001010000,0x00000002000000), (0x00000002020000,0x00000005000000), (0x00000004040000,0x0000000a000000), (0x00000008080000,0x00000014000000), (0x00000010100000,0x00000028000000), (0x00000020200000,0x00000050000000), (0x00000040400000,0x000000a0000000), (0x00000080800000,0x00000040000000), (0x00000101000000,0x00000200000000), (0x00000202000000,0x00000500000000), (0x00000404000000,0x00000a00000000), (0x00000808000000,0x00001400000000), (0x00001010000000,0x00002800000000), (0x00002020000000,0x00005000000000), (0x00004040000000,0x0000a000000000), (0x00008080000000,0x00004000000000), (0x00010100000000,0x00020000000000), (0x00020200000000,0x00050000000000), (0x00040400000000,0x000a0000000000), (0x00080800000000,0x00140000000000), (0x00101000000000,0x00280000000000), (0x00202000000000,0x00500000000000), (0x00404000000000,0x00a00000000000), (0x00808000000000,0x00400000000000), (0x01010000000000,0x02000000000000), (0x02020000000000,0x05000000000000), (0x04040000000000,0x0a000000000000), (0x08080000000000,0x14000000000000), (0x10100000000000,0x28000000000000), (0x20200000000000,0x50000000000000), (0x40400000000000,0xa0000000000000), (0x80800000000000,0x40000000000000) ];
pub const KNIGHT_MASKS: [u64; 64] = [ 0x00000000020400, 0x00000000050800, 0x000000000a1100, 0x00000000142200, 0x00000000284400, 0x00000000508800, 0x00000000a01000, 0x00000000402000, 0x00000002040004, 0x00000005080008, 0x0000000a110011, 0x00000014220022, 0x00000028440044, 0x00000050880088, 0x000000a0100010, 0x00000040200020, 0x00000204000402, 0x00000508000805, 0x00000a1100110a, 0x00001422002214, 0x00002844004428, 0x00005088008850, 0x0000a0100010a0, 0x00004020002040, 0x00020400040200, 0x00050800080500, 0x000a1100110a00, 0x00142200221400, 0x00284400442800, 0x00508800885000, 0x00a0100010a000, 0x00402000204000, 0x02040004020000, 0x05080008050000, 0x0a1100110a0000, 0x14220022140000, 0x28440044280000, 0x50880088500000, 0xa0100010a00000, 0x40200020400000, 0x204000402000000, 0x508000805000000, 0xa1100110a000000, 0x1422002214000000, 0x2844004428000000, 0x5088008850000000, 0xa0100010a0000000, 0x4020002040000000, 0x400040200000000, 0x800080500000000, 0x1100110a00000000, 0x2200221400000000, 0x4400442800000000, 0x8800885000000000, 0x100010a000000000, 0x2000204000000000, 0x04020000000000, 0x08050000000000, 0x110a0000000000, 0x22140000000000, 0x44280000000000, 0x88500000000000, 0x10a00000000000, 0x20400000000000 ];
pub const KING_MASKS: [u64; 64] = [ 0x00000000000302, 0x00000000000705, 0x00000000000e0a, 0x00000000001c14, 0x00000000003828, 0x00000000007050, 0x0000000000e0a0, 0x0000000000c040, 0x00000000030203, 0x00000000070507, 0x000000000e0a0e, 0x000000001c141c, 0x00000000382838, 0x00000000705070, 0x00000000e0a0e0, 0x00000000c040c0, 0x00000003020300, 0x00000007050700, 0x0000000e0a0e00, 0x0000001c141c00, 0x00000038283800, 0x00000070507000, 0x000000e0a0e000, 0x000000c040c000, 0x00000302030000, 0x00000705070000, 0x00000e0a0e0000, 0x00001c141c0000, 0x00003828380000, 0x00007050700000, 0x0000e0a0e00000, 0x0000c040c00000, 0x00030203000000, 0x00070507000000, 0x000e0a0e000000, 0x001c141c000000, 0x00382838000000, 0x00705070000000, 0x00e0a0e0000000, 0x00c040c0000000, 0x03020300000000, 0x07050700000000, 0x0e0a0e00000000, 0x1c141c00000000, 0x38283800000000, 0x70507000000000, 0xe0a0e000000000, 0xc040c000000000, 0x302030000000000, 0x705070000000000, 0xe0a0e0000000000, 0x1c141c0000000000, 0x3828380000000000, 0x7050700000000000, 0xe0a0e00000000000, 0xc040c00000000000, 0x203000000000000, 0x507000000000000, 0xa0e000000000000, 0x141c000000000000, 0x2838000000000000, 0x5070000000000000, 0xa0e0000000000000, 0x40c0000000000000 ];

macro_rules! include_bytes_aligned {
    ($align_to:expr, $path:expr) => {{
        #[repr(C, align($align_to))]
        struct __Aligned<T: ?Sized>(T);

        const __DATA: &'static __Aligned<[u8]> = &__Aligned(*include_bytes!($path));

        &__DATA.0
    }};
}

pub fn get_bishop_mask(idx: usize) -> Bitboard {
    cast_slice(include_bytes_aligned!(64, "../../bitboards/bishop.bin"))[idx]
}

pub fn get_rook_mask(idx: usize) -> Bitboard {
    cast_slice(include_bytes_aligned!(64, "../../bitboards/rook.bin"))[idx]
}