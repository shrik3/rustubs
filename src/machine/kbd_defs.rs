use core::ffi::c_uchar;

// bit masks for modifier keys
pub enum Mbit {
    Shift = 0b00000001,
    AltLeft = 0b00000010,
    AltRight = 0b00000100,
    CtrlLeft = 0b00001000,
    CtrlRight = 0b00010000,
    CapsLock = 0b00100000,
    NumLock = 0b01000000,
    ScrollLock = 0b10000000,
}

// scan codes of a few specific keys
pub enum Scan {
    F1 = 0x3b,
    Del = 0x53,
    Up = 72,
    Down = 80,
    Left = 75,
    Right = 77,
    Div = 8,
}

// Decoding tables ... this shit is so ugly, thanks to rust's strong typing system!!!
// Also, this is a german layout keyboard
// oh btw, the code translation is done by ChatGPT if it's wrong complain to the AI!
const NORMAL_TAB: [c_uchar; 89] = [
    0, 0, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 225, 39, 8, 0, 113, 119, 101, 114, 116, 122, 117,
    105, 111, 112, 129, 43, 10, 0, 97, 115, 100, 102, 103, 104, 106, 107, 108, 148, 132, 94, 0, 35,
    121, 120, 99, 118, 98, 110, 109, 44, 46, 45, 0, 42, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 45, 0, 0, 0, 43, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0,
];

const SHIFT_TAB: [c_uchar; 89] = [
    0, 0, 33, 34, 21, 36, 37, 38, 47, 40, 41, 61, 63, 96, 0, 0, 81, 87, 69, 82, 84, 90, 85, 73, 79,
    80, 154, 42, 0, 0, 65, 83, 68, 70, 71, 72, 74, 75, 76, 153, 142, 248, 0, 39, 89, 88, 67, 86,
    66, 78, 77, 59, 58, 95, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 0, 0,
];

const ALT_TAB: [c_uchar; 89] = [
    0, 0, 0, 253, 0, 0, 0, 0, 123, 91, 93, 125, 92, 0, 0,   0, 64, 0,
    0, 0, 0, 0,   0, 0, 0, 0, 0,   126, 0,   0,   0,    0, 0,   0, 0,   0,
    0, 0, 0, 0,   0, 0, 0, 0, 0,   0,   0,   0,   0,    0, 230, 0, 0,   0,
    0, 0, 0, 0,   0, 0, 0, 0, 0,   0,   0,   0,   0,    0, 0,   0, 0,   0,
    0, 0, 0, 0,   0, 0, 0, 0, 0,   0,   0,   0,   0,    0, 124, 0, 0
];

const ASC_NUM_TAB: [c_uchar; 13] = [55, 56, 57, 45, 52, 53, 54, 43, 49, 50, 51, 48, 44];
const SCAN_NUM_TAB: [c_uchar; 13] = [8,  9, 10, 53, 5,  6, 7, 27, 2, 3,  4,  11, 51];

// I think constants are more handy than enum for these...
// Keyboard controller commands
pub const KC_CMD_SET_LED: u8 = 0xed;
pub const KC_CMD_SET_SPEED: u8 = 0xf3;

// CPU reset .. (reboot)
pub const KC_CPU_RESET: u8 = 0xfe;

// Status register bits
pub const KC_SR_OUTB: u8 = 0x01;
pub const KC_SR_INPB: u8 = 0x02;
pub const KC_SR_AUXB: u8 = 0x20;

// Keyboard Controller LED bits
pub const KC_LED_CAPS_LOCK: u8 = 4;
pub const KC_LED_NUM_LOCK: u8 = 2;
pub const KC_LED_SCROLL_LOCK: u8 = 1;

// ACK
pub const KC_REPLY_ACK: u8 = 0xfa;

// some stuffs for decoding
pub const BREAK_BIT: u8 = 0x80;
pub const PREFIX1: u8 = 0xe0;
pub const PREFIX2: u8 = 0xe1;
