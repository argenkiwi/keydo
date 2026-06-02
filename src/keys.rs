pub const MOD_ALT_GR: u8 = 0x10;
pub const MOD_CTRL: u8 = 0x08;
pub const MOD_SHIFT: u8 = 0x04;
pub const MOD_SUPER: u8 = 0x02;
pub const MOD_ALT: u8 = 0x01;

#[derive(Clone, Copy)]
pub struct KeyCodeEntry {
    pub name: &'static str,
    pub alt_name: Option<&'static str>,
    pub shifted_name: Option<&'static str>,
}

pub fn get_key_name(code: u16) -> &'static str {
    if let Some(Some(ent)) = KEYCODE_TABLE.get(code as usize) {
        ent.name
    } else {
        "UNKNOWN"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyCode(pub u16);

pub const KEYD_ESC: u16 = 1;
pub const KEYD_1: u16 = 2;
pub const KEYD_2: u16 = 3;
pub const KEYD_3: u16 = 4;
pub const KEYD_4: u16 = 5;
pub const KEYD_5: u16 = 6;
pub const KEYD_6: u16 = 7;
pub const KEYD_7: u16 = 8;
pub const KEYD_8: u16 = 9;
pub const KEYD_9: u16 = 10;
pub const KEYD_0: u16 = 11;
pub const KEYD_MINUS: u16 = 12;
pub const KEYD_EQUAL: u16 = 13;
pub const KEYD_BACKSPACE: u16 = 14;
pub const KEYD_TAB: u16 = 15;
pub const KEYD_Q: u16 = 16;
pub const KEYD_W: u16 = 17;
pub const KEYD_E: u16 = 18;
pub const KEYD_R: u16 = 19;
pub const KEYD_T: u16 = 20;
pub const KEYD_Y: u16 = 21;
pub const KEYD_U: u16 = 22;
pub const KEYD_I: u16 = 23;
pub const KEYD_O: u16 = 24;
pub const KEYD_P: u16 = 25;
pub const KEYD_LEFTBRACE: u16 = 26;
pub const KEYD_RIGHTBRACE: u16 = 27;
pub const KEYD_ENTER: u16 = 28;
pub const KEYD_LEFTCTRL: u16 = 29;
pub const KEYD_A: u16 = 30;
pub const KEYD_S: u16 = 31;
pub const KEYD_D: u16 = 32;
pub const KEYD_F: u16 = 33;
pub const KEYD_G: u16 = 34;
pub const KEYD_H: u16 = 35;
pub const KEYD_J: u16 = 36;
pub const KEYD_K: u16 = 37;
pub const KEYD_L: u16 = 38;
pub const KEYD_SEMICOLON: u16 = 39;
pub const KEYD_APOSTROPHE: u16 = 40;
pub const KEYD_GRAVE: u16 = 41;
pub const KEYD_LEFTSHIFT: u16 = 42;
pub const KEYD_BACKSLASH: u16 = 43;
pub const KEYD_Z: u16 = 44;
pub const KEYD_X: u16 = 45;
pub const KEYD_C: u16 = 46;
pub const KEYD_V: u16 = 47;
pub const KEYD_B: u16 = 48;
pub const KEYD_N: u16 = 49;
pub const KEYD_M: u16 = 50;
pub const KEYD_COMMA: u16 = 51;
pub const KEYD_DOT: u16 = 52;
pub const KEYD_SLASH: u16 = 53;
pub const KEYD_RIGHTSHIFT: u16 = 54;
pub const KEYD_KPASTERISK: u16 = 55;
pub const KEYD_LEFTALT: u16 = 56;
pub const KEYD_SPACE: u16 = 57;
pub const KEYD_CAPSLOCK: u16 = 58;
pub const KEYD_F1: u16 = 59;
pub const KEYD_F2: u16 = 60;
pub const KEYD_F3: u16 = 61;
pub const KEYD_F4: u16 = 62;
pub const KEYD_F5: u16 = 63;
pub const KEYD_F6: u16 = 64;
pub const KEYD_F7: u16 = 65;
pub const KEYD_F8: u16 = 66;
pub const KEYD_F9: u16 = 67;
pub const KEYD_F10: u16 = 68;
pub const KEYD_NUMLOCK: u16 = 69;
pub const KEYD_SCROLLLOCK: u16 = 70;
pub const KEYD_KP7: u16 = 71;
pub const KEYD_KP8: u16 = 72;
pub const KEYD_KP9: u16 = 73;
pub const KEYD_KPMINUS: u16 = 74;
pub const KEYD_KP4: u16 = 75;
pub const KEYD_KP5: u16 = 76;
pub const KEYD_KP6: u16 = 77;
pub const KEYD_KPPLUS: u16 = 78;
pub const KEYD_KP1: u16 = 79;
pub const KEYD_KP2: u16 = 80;
pub const KEYD_KP3: u16 = 81;
pub const KEYD_KP0: u16 = 82;
pub const KEYD_KPDOT: u16 = 83;
pub const KEYD_IS_LEVEL3_SHIFT: u16 = 84;
pub const KEYD_ZENKAKUHANKAKU: u16 = 85;
pub const KEYD_102ND: u16 = 86;
pub const KEYD_F11: u16 = 87;
pub const KEYD_F12: u16 = 88;
pub const KEYD_RO: u16 = 89;
pub const KEYD_KATAKANA: u16 = 90;
pub const KEYD_HIRAGANA: u16 = 91;
pub const KEYD_HENKAN: u16 = 92;
pub const KEYD_KATAKANAHIRAGANA: u16 = 93;
pub const KEYD_MUHENKAN: u16 = 94;
pub const KEYD_KPJPCOMMA: u16 = 95;
pub const KEYD_KPENTER: u16 = 96;
pub const KEYD_RIGHTCTRL: u16 = 97;
pub const KEYD_KPSLASH: u16 = 98;
pub const KEYD_SYSRQ: u16 = 99;
pub const KEYD_RIGHTALT: u16 = 100;
pub const KEYD_LINEFEED: u16 = 101;
pub const KEYD_HOME: u16 = 102;
pub const KEYD_UP: u16 = 103;
pub const KEYD_PAGEUP: u16 = 104;
pub const KEYD_LEFT: u16 = 105;
pub const KEYD_RIGHT: u16 = 106;
pub const KEYD_END: u16 = 107;
pub const KEYD_DOWN: u16 = 108;
pub const KEYD_PAGEDOWN: u16 = 109;
pub const KEYD_INSERT: u16 = 110;
pub const KEYD_DELETE: u16 = 111;
pub const KEYD_MACRO: u16 = 112;
pub const KEYD_MUTE: u16 = 113;
pub const KEYD_VOLUMEDOWN: u16 = 114;
pub const KEYD_VOLUMEUP: u16 = 115;
pub const KEYD_POWER: u16 = 116;
pub const KEYD_KPEQUAL: u16 = 117;
pub const KEYD_KPPLUSMINUS: u16 = 118;
pub const KEYD_PAUSE: u16 = 119;
pub const KEYD_SCALE: u16 = 120;
pub const KEYD_KPCOMMA: u16 = 121;
pub const KEYD_HANGEUL: u16 = 122;
pub const KEYD_HANJA: u16 = 123;
pub const KEYD_YEN: u16 = 124;
pub const KEYD_LEFTMETA: u16 = 125;
pub const KEYD_RIGHTMETA: u16 = 126;
pub const KEYD_COMPOSE: u16 = 127;
pub const KEYD_STOP: u16 = 128;
pub const KEYD_AGAIN: u16 = 129;
pub const KEYD_PROPS: u16 = 130;
pub const KEYD_UNDO: u16 = 131;
pub const KEYD_FRONT: u16 = 132;
pub const KEYD_COPY: u16 = 133;
pub const KEYD_OPEN: u16 = 134;
pub const KEYD_PASTE: u16 = 135;
pub const KEYD_FIND: u16 = 136;
pub const KEYD_CUT: u16 = 137;
pub const KEYD_HELP: u16 = 138;
pub const KEYD_MENU: u16 = 139;
pub const KEYD_CALC: u16 = 140;
pub const KEYD_SETUP: u16 = 141;
pub const KEYD_SLEEP: u16 = 142;
pub const KEYD_WAKEUP: u16 = 143;
pub const KEYD_FILE: u16 = 144;
pub const KEYD_SENDFILE: u16 = 145;
pub const KEYD_DELETEFILE: u16 = 146;
pub const KEYD_XFER: u16 = 147;
pub const KEYD_SCROLL_DOWN: u16 = 148;
pub const KEYD_SCROLL_UP: u16 = 149;
pub const KEYD_WWW: u16 = 150;
pub const KEYD_MSDOS: u16 = 151;
pub const KEYD_COFFEE: u16 = 152;
pub const KEYD_ROTATE_DISPLAY: u16 = 153;
pub const KEYD_CYCLEWINDOWS: u16 = 154;
pub const KEYD_MAIL: u16 = 155;
pub const KEYD_BOOKMARKS: u16 = 156;
pub const KEYD_COMPUTER: u16 = 157;
pub const KEYD_BACK: u16 = 158;
pub const KEYD_FORWARD: u16 = 159;
pub const KEYD_CLOSECD: u16 = 160;
pub const KEYD_EJECTCD: u16 = 161;
pub const KEYD_EJECTCLOSECD: u16 = 162;
pub const KEYD_NEXTSONG: u16 = 163;
pub const KEYD_PLAYPAUSE: u16 = 164;
pub const KEYD_PREVIOUSSONG: u16 = 165;
pub const KEYD_STOPCD: u16 = 166;
pub const KEYD_RECORD: u16 = 167;
pub const KEYD_REWIND: u16 = 168;
pub const KEYD_PHONE: u16 = 169;
pub const KEYD_ISO: u16 = 170;
pub const KEYD_CONFIG: u16 = 171;
pub const KEYD_HOMEPAGE: u16 = 172;
pub const KEYD_REFRESH: u16 = 173;
pub const KEYD_EXIT: u16 = 174;
pub const KEYD_MOVE: u16 = 175;
pub const KEYD_EDIT: u16 = 176;
pub const KEYD_ZOOM: u16 = 177;
pub const KEYD_KPLEFTPAREN: u16 = 179;
pub const KEYD_KPRIGHTPAREN: u16 = 180;
pub const KEYD_NEW: u16 = 181;
pub const KEYD_REDO: u16 = 182;
pub const KEYD_F13: u16 = 183;
pub const KEYD_F14: u16 = 184;
pub const KEYD_F15: u16 = 185;
pub const KEYD_F16: u16 = 186;
pub const KEYD_F17: u16 = 187;
pub const KEYD_F18: u16 = 188;
pub const KEYD_F19: u16 = 189;
pub const KEYD_F20: u16 = 190;
pub const KEYD_F21: u16 = 191;
pub const KEYD_F22: u16 = 192;
pub const KEYD_F23: u16 = 193;
pub const KEYD_F24: u16 = 194;
pub const KEYD_PLAYCD: u16 = 200;
pub const KEYD_PAUSECD: u16 = 201;
pub const KEYD_SCROLL_LEFT: u16 = 202;
pub const KEYD_SCROLL_RIGHT: u16 = 203;
pub const KEYD_DASHBOARD: u16 = 204;
pub const KEYD_SUSPEND: u16 = 205;
pub const KEYD_CLOSE: u16 = 206;
pub const KEYD_PLAY: u16 = 207;
pub const KEYD_FASTFORWARD: u16 = 208;
pub const KEYD_BASSBOOST: u16 = 209;
pub const KEYD_PRINT: u16 = 210;
pub const KEYD_HP: u16 = 211;
pub const KEYD_CAMERA: u16 = 212;
pub const KEYD_SOUND: u16 = 213;
pub const KEYD_QUESTION: u16 = 214;
pub const KEYD_EMAIL: u16 = 215;
pub const KEYD_CHAT: u16 = 216;
pub const KEYD_SEARCH: u16 = 217;
pub const KEYD_CONNECT: u16 = 218;
pub const KEYD_FINANCE: u16 = 219;
pub const KEYD_SPORT: u16 = 220;
pub const KEYD_SHOP: u16 = 221;
pub const KEYD_VOICECOMMAND: u16 = 222;
pub const KEYD_CANCEL: u16 = 223;
pub const KEYD_BRIGHTNESSDOWN: u16 = 224;
pub const KEYD_BRIGHTNESSUP: u16 = 225;
pub const KEYD_MEDIA: u16 = 226;
pub const KEYD_SWITCHVIDEOMODE: u16 = 227;
pub const KEYD_KBDILLUMTOGGLE: u16 = 228;
pub const KEYD_KBDILLUMDOWN: u16 = 229;
pub const KEYD_KBDILLUMUP: u16 = 230;
pub const KEYD_SEND: u16 = 231;
pub const KEYD_REPLY: u16 = 232;
pub const KEYD_FORWARDMAIL: u16 = 233;
pub const KEYD_SAVE: u16 = 234;
pub const KEYD_DOCUMENTS: u16 = 235;
pub const KEYD_BATTERY: u16 = 236;
pub const KEYD_BLUETOOTH: u16 = 237;
pub const KEYD_WLAN: u16 = 238;
pub const KEYD_UWB: u16 = 239;
pub const KEYD_UNKNOWN: u16 = 240;
pub const KEYD_VIDEO_NEXT: u16 = 241;
pub const KEYD_VIDEO_PREV: u16 = 242;
pub const KEYD_BRIGHTNESS_CYCLE: u16 = 243;
pub const KEYD_BRIGHTNESS_AUTO: u16 = 244;
pub const KEYD_DISPLAY_OFF: u16 = 245;
pub const KEYD_WWAN: u16 = 246;
pub const KEYD_RFKILL: u16 = 247;
pub const KEYD_MICMUTE: u16 = 248;

pub const KEYD_NOOP: u16 = 195;
pub const KEYD_EXTERNAL_MOUSE_BUTTON: u16 = 196;
pub const KEYD_CHORD_1: u16 = 197;
pub const KEYD_CHORD_2: u16 = 198;
pub const KEYD_CHORD_MAX: u16 = 199;
pub const KEYD_LEFT_MOUSE: u16 = 249;
pub const KEYD_MIDDLE_MOUSE: u16 = 250;
pub const KEYD_RIGHT_MOUSE: u16 = 251;
pub const KEYD_MOUSE_1: u16 = 252;
pub const KEYD_MOUSE_2: u16 = 253;
pub const KEYD_MOUSE_BACK: u16 = 178;
pub const KEYD_FN: u16 = 254;
pub const KEYD_MOUSE_FORWARD: u16 = 255;

pub const KEYCODE_TABLE: [Option<KeyCodeEntry>; 512] = {
    let mut table = [None; 512];
    // Standard keys
    table[KEYD_ESC as usize] = Some(KeyCodeEntry { name: "esc", alt_name: Some("escape"), shifted_name: None });
    table[KEYD_1 as usize] = Some(KeyCodeEntry { name: "1", alt_name: None, shifted_name: Some("!") });
    table[KEYD_2 as usize] = Some(KeyCodeEntry { name: "2", alt_name: None, shifted_name: Some("@") });
    table[KEYD_3 as usize] = Some(KeyCodeEntry { name: "3", alt_name: None, shifted_name: Some("#") });
    table[KEYD_4 as usize] = Some(KeyCodeEntry { name: "4", alt_name: None, shifted_name: Some("$") });
    table[KEYD_5 as usize] = Some(KeyCodeEntry { name: "5", alt_name: None, shifted_name: Some("%") });
    table[KEYD_6 as usize] = Some(KeyCodeEntry { name: "6", alt_name: None, shifted_name: Some("^") });
    table[KEYD_7 as usize] = Some(KeyCodeEntry { name: "7", alt_name: None, shifted_name: Some("&") });
    table[KEYD_8 as usize] = Some(KeyCodeEntry { name: "8", alt_name: None, shifted_name: Some("*") });
    table[KEYD_9 as usize] = Some(KeyCodeEntry { name: "9", alt_name: None, shifted_name: Some("(") });
    table[KEYD_0 as usize] = Some(KeyCodeEntry { name: "0", alt_name: None, shifted_name: Some(")") });
    table[KEYD_MINUS as usize] = Some(KeyCodeEntry { name: "-", alt_name: Some("minus"), shifted_name: Some("_") });
    table[KEYD_EQUAL as usize] = Some(KeyCodeEntry { name: "=", alt_name: Some("equal"), shifted_name: Some("+") });
    table[KEYD_BACKSPACE as usize] = Some(KeyCodeEntry { name: "backspace", alt_name: None, shifted_name: None });
    table[KEYD_TAB as usize] = Some(KeyCodeEntry { name: "tab", alt_name: None, shifted_name: None });
    table[KEYD_Q as usize] = Some(KeyCodeEntry { name: "q", alt_name: None, shifted_name: Some("Q") });
    table[KEYD_W as usize] = Some(KeyCodeEntry { name: "w", alt_name: None, shifted_name: Some("W") });
    table[KEYD_E as usize] = Some(KeyCodeEntry { name: "e", alt_name: None, shifted_name: Some("E") });
    table[KEYD_R as usize] = Some(KeyCodeEntry { name: "r", alt_name: None, shifted_name: Some("R") });
    table[KEYD_T as usize] = Some(KeyCodeEntry { name: "t", alt_name: None, shifted_name: Some("T") });
    table[KEYD_Y as usize] = Some(KeyCodeEntry { name: "y", alt_name: None, shifted_name: Some("Y") });
    table[KEYD_U as usize] = Some(KeyCodeEntry { name: "u", alt_name: None, shifted_name: Some("U") });
    table[KEYD_I as usize] = Some(KeyCodeEntry { name: "i", alt_name: None, shifted_name: Some("I") });
    table[KEYD_O as usize] = Some(KeyCodeEntry { name: "o", alt_name: None, shifted_name: Some("O") });
    table[KEYD_P as usize] = Some(KeyCodeEntry { name: "p", alt_name: None, shifted_name: Some("P") });
    table[KEYD_LEFTBRACE as usize] = Some(KeyCodeEntry { name: "[", alt_name: Some("leftbrace"), shifted_name: Some("{") });
    table[KEYD_RIGHTBRACE as usize] = Some(KeyCodeEntry { name: "]", alt_name: Some("rightbrace"), shifted_name: Some("}") });
    table[KEYD_ENTER as usize] = Some(KeyCodeEntry { name: "enter", alt_name: None, shifted_name: None });
    table[KEYD_LEFTCTRL as usize] = Some(KeyCodeEntry { name: "control", alt_name: Some("leftcontrol"), shifted_name: None });
    table[KEYD_A as usize] = Some(KeyCodeEntry { name: "a", alt_name: None, shifted_name: Some("A") });
    table[KEYD_S as usize] = Some(KeyCodeEntry { name: "s", alt_name: None, shifted_name: Some("S") });
    table[KEYD_D as usize] = Some(KeyCodeEntry { name: "d", alt_name: None, shifted_name: Some("D") });
    table[KEYD_F as usize] = Some(KeyCodeEntry { name: "f", alt_name: None, shifted_name: Some("F") });
    table[KEYD_G as usize] = Some(KeyCodeEntry { name: "g", alt_name: None, shifted_name: Some("G") });
    table[KEYD_H as usize] = Some(KeyCodeEntry { name: "h", alt_name: None, shifted_name: Some("H") });
    table[KEYD_J as usize] = Some(KeyCodeEntry { name: "j", alt_name: None, shifted_name: Some("J") });
    table[KEYD_K as usize] = Some(KeyCodeEntry { name: "k", alt_name: None, shifted_name: Some("K") });
    table[KEYD_L as usize] = Some(KeyCodeEntry { name: "l", alt_name: None, shifted_name: Some("L") });
    table[KEYD_SEMICOLON as usize] = Some(KeyCodeEntry { name: ";", alt_name: Some("semicolon"), shifted_name: Some(":") });
    table[KEYD_APOSTROPHE as usize] = Some(KeyCodeEntry { name: "'", alt_name: Some("apostrophe"), shifted_name: Some("\"") });
    table[KEYD_GRAVE as usize] = Some(KeyCodeEntry { name: "`", alt_name: Some("grave"), shifted_name: Some("~") });
    table[KEYD_LEFTSHIFT as usize] = Some(KeyCodeEntry { name: "shift", alt_name: Some("leftshift"), shifted_name: None });
    table[KEYD_BACKSLASH as usize] = Some(KeyCodeEntry { name: "\\", alt_name: Some("backslash"), shifted_name: Some("|") });
    table[KEYD_Z as usize] = Some(KeyCodeEntry { name: "z", alt_name: None, shifted_name: Some("Z") });
    table[KEYD_X as usize] = Some(KeyCodeEntry { name: "x", alt_name: None, shifted_name: Some("X") });
    table[KEYD_C as usize] = Some(KeyCodeEntry { name: "c", alt_name: None, shifted_name: Some("C") });
    table[KEYD_V as usize] = Some(KeyCodeEntry { name: "v", alt_name: None, shifted_name: Some("V") });
    table[KEYD_B as usize] = Some(KeyCodeEntry { name: "b", alt_name: None, shifted_name: Some("B") });
    table[KEYD_N as usize] = Some(KeyCodeEntry { name: "n", alt_name: None, shifted_name: Some("N") });
    table[KEYD_M as usize] = Some(KeyCodeEntry { name: "m", alt_name: None, shifted_name: Some("M") });
    table[KEYD_COMMA as usize] = Some(KeyCodeEntry { name: ",", alt_name: Some("comma"), shifted_name: Some("<") });
    table[KEYD_DOT as usize] = Some(KeyCodeEntry { name: ".", alt_name: Some("dot"), shifted_name: Some(">") });
    table[KEYD_SLASH as usize] = Some(KeyCodeEntry { name: "/", alt_name: Some("slash"), shifted_name: Some("?") });
    table[KEYD_RIGHTSHIFT as usize] = Some(KeyCodeEntry { name: "rightshift", alt_name: Some("shift"), shifted_name: None });
    table[KEYD_KPASTERISK as usize] = Some(KeyCodeEntry { name: "kpasterisk", alt_name: None, shifted_name: None });
    table[KEYD_LEFTALT as usize] = Some(KeyCodeEntry { name: "alt", alt_name: Some("leftalt"), shifted_name: None });
    table[KEYD_SPACE as usize] = Some(KeyCodeEntry { name: "space", alt_name: None, shifted_name: None });
    table[KEYD_CAPSLOCK as usize] = Some(KeyCodeEntry { name: "capslock", alt_name: None, shifted_name: None });
    table[KEYD_F1 as usize] = Some(KeyCodeEntry { name: "f1", alt_name: None, shifted_name: None });
    table[KEYD_F2 as usize] = Some(KeyCodeEntry { name: "f2", alt_name: None, shifted_name: None });
    table[KEYD_F3 as usize] = Some(KeyCodeEntry { name: "f3", alt_name: None, shifted_name: None });
    table[KEYD_F4 as usize] = Some(KeyCodeEntry { name: "f4", alt_name: None, shifted_name: None });
    table[KEYD_F5 as usize] = Some(KeyCodeEntry { name: "f5", alt_name: None, shifted_name: None });
    table[KEYD_F6 as usize] = Some(KeyCodeEntry { name: "f6", alt_name: None, shifted_name: None });
    table[KEYD_F7 as usize] = Some(KeyCodeEntry { name: "f7", alt_name: None, shifted_name: None });
    table[KEYD_F8 as usize] = Some(KeyCodeEntry { name: "f8", alt_name: None, shifted_name: None });
    table[KEYD_F9 as usize] = Some(KeyCodeEntry { name: "f9", alt_name: None, shifted_name: None });
    table[KEYD_F10 as usize] = Some(KeyCodeEntry { name: "f10", alt_name: None, shifted_name: None });
    table[KEYD_NUMLOCK as usize] = Some(KeyCodeEntry { name: "numlock", alt_name: None, shifted_name: None });
    table[KEYD_SCROLLLOCK as usize] = Some(KeyCodeEntry { name: "scrolllock", alt_name: None, shifted_name: None });
    table[KEYD_KP7 as usize] = Some(KeyCodeEntry { name: "kp7", alt_name: None, shifted_name: None });
    table[KEYD_KP8 as usize] = Some(KeyCodeEntry { name: "kp8", alt_name: None, shifted_name: None });
    table[KEYD_KP9 as usize] = Some(KeyCodeEntry { name: "kp9", alt_name: None, shifted_name: None });
    table[KEYD_KPMINUS as usize] = Some(KeyCodeEntry { name: "kpminus", alt_name: None, shifted_name: None });
    table[KEYD_KP4 as usize] = Some(KeyCodeEntry { name: "kp4", alt_name: None, shifted_name: None });
    table[KEYD_KP5 as usize] = Some(KeyCodeEntry { name: "kp5", alt_name: None, shifted_name: None });
    table[KEYD_KP6 as usize] = Some(KeyCodeEntry { name: "kp6", alt_name: None, shifted_name: None });
    table[KEYD_KPPLUS as usize] = Some(KeyCodeEntry { name: "kpplus", alt_name: None, shifted_name: None });
    table[KEYD_KP1 as usize] = Some(KeyCodeEntry { name: "kp1", alt_name: None, shifted_name: None });
    table[KEYD_KP2 as usize] = Some(KeyCodeEntry { name: "kp2", alt_name: None, shifted_name: None });
    table[KEYD_KP3 as usize] = Some(KeyCodeEntry { name: "kp3", alt_name: None, shifted_name: None });
    table[KEYD_KP0 as usize] = Some(KeyCodeEntry { name: "kp0", alt_name: None, shifted_name: None });
    table[KEYD_KPDOT as usize] = Some(KeyCodeEntry { name: "kpdot", alt_name: None, shifted_name: None });
    table[KEYD_ZENKAKUHANKAKU as usize] = Some(KeyCodeEntry { name: "zenkakuhankaku", alt_name: None, shifted_name: None });
    table[KEYD_102ND as usize] = Some(KeyCodeEntry { name: "102nd", alt_name: None, shifted_name: None });
    table[KEYD_F11 as usize] = Some(KeyCodeEntry { name: "f11", alt_name: None, shifted_name: None });
    table[KEYD_F12 as usize] = Some(KeyCodeEntry { name: "f12", alt_name: None, shifted_name: None });
    table[KEYD_RO as usize] = Some(KeyCodeEntry { name: "ro", alt_name: None, shifted_name: None });
    table[KEYD_KATAKANA as usize] = Some(KeyCodeEntry { name: "katakana", alt_name: None, shifted_name: None });
    table[KEYD_HIRAGANA as usize] = Some(KeyCodeEntry { name: "hiragana", alt_name: None, shifted_name: None });
    table[KEYD_HENKAN as usize] = Some(KeyCodeEntry { name: "henkan", alt_name: None, shifted_name: None });
    table[KEYD_KATAKANAHIRAGANA as usize] = Some(KeyCodeEntry { name: "katakanahiragana", alt_name: None, shifted_name: None });
    table[KEYD_MUHENKAN as usize] = Some(KeyCodeEntry { name: "muhenkan", alt_name: None, shifted_name: None });
    table[KEYD_KPJPCOMMA as usize] = Some(KeyCodeEntry { name: "kpjpcomma", alt_name: None, shifted_name: None });
    table[KEYD_KPENTER as usize] = Some(KeyCodeEntry { name: "kpenter", alt_name: None, shifted_name: None });
    table[KEYD_RIGHTCTRL as usize] = Some(KeyCodeEntry { name: "rightcontrol", alt_name: Some("control"), shifted_name: None });
    table[KEYD_KPSLASH as usize] = Some(KeyCodeEntry { name: "kpslash", alt_name: None, shifted_name: None });
    table[KEYD_SYSRQ as usize] = Some(KeyCodeEntry { name: "sysrq", alt_name: None, shifted_name: None });
    table[KEYD_RIGHTALT as usize] = Some(KeyCodeEntry { name: "rightalt", alt_name: Some("alt"), shifted_name: None });
    table[KEYD_LINEFEED as usize] = Some(KeyCodeEntry { name: "linefeed", alt_name: None, shifted_name: None });
    table[KEYD_HOME as usize] = Some(KeyCodeEntry { name: "home", alt_name: None, shifted_name: None });
    table[KEYD_UP as usize] = Some(KeyCodeEntry { name: "up", alt_name: None, shifted_name: None });
    table[KEYD_PAGEUP as usize] = Some(KeyCodeEntry { name: "pageup", alt_name: None, shifted_name: None });
    table[KEYD_LEFT as usize] = Some(KeyCodeEntry { name: "left", alt_name: None, shifted_name: None });
    table[KEYD_RIGHT as usize] = Some(KeyCodeEntry { name: "right", alt_name: None, shifted_name: None });
    table[KEYD_END as usize] = Some(KeyCodeEntry { name: "end", alt_name: None, shifted_name: None });
    table[KEYD_DOWN as usize] = Some(KeyCodeEntry { name: "down", alt_name: None, shifted_name: None });
    table[KEYD_PAGEDOWN as usize] = Some(KeyCodeEntry { name: "pagedown", alt_name: None, shifted_name: None });
    table[KEYD_INSERT as usize] = Some(KeyCodeEntry { name: "insert", alt_name: None, shifted_name: None });
    table[KEYD_DELETE as usize] = Some(KeyCodeEntry { name: "delete", alt_name: None, shifted_name: None });
    table[KEYD_MACRO as usize] = Some(KeyCodeEntry { name: "macro", alt_name: None, shifted_name: None });
    table[KEYD_MUTE as usize] = Some(KeyCodeEntry { name: "mute", alt_name: None, shifted_name: None });
    table[KEYD_VOLUMEDOWN as usize] = Some(KeyCodeEntry { name: "volumedown", alt_name: None, shifted_name: None });
    table[KEYD_VOLUMEUP as usize] = Some(KeyCodeEntry { name: "volumeup", alt_name: None, shifted_name: None });
    table[KEYD_POWER as usize] = Some(KeyCodeEntry { name: "power", alt_name: None, shifted_name: None });
    table[KEYD_KPEQUAL as usize] = Some(KeyCodeEntry { name: "kpequal", alt_name: None, shifted_name: None });
    table[KEYD_KPPLUSMINUS as usize] = Some(KeyCodeEntry { name: "kpplusminus", alt_name: None, shifted_name: None });
    table[KEYD_PAUSE as usize] = Some(KeyCodeEntry { name: "pause", alt_name: None, shifted_name: None });
    table[KEYD_SCALE as usize] = Some(KeyCodeEntry { name: "scale", alt_name: None, shifted_name: None });
    table[KEYD_KPCOMMA as usize] = Some(KeyCodeEntry { name: "kpcomma", alt_name: None, shifted_name: None });
    table[KEYD_HANGEUL as usize] = Some(KeyCodeEntry { name: "hangeul", alt_name: None, shifted_name: None });
    table[KEYD_HANJA as usize] = Some(KeyCodeEntry { name: "hanja", alt_name: None, shifted_name: None });
    table[KEYD_YEN as usize] = Some(KeyCodeEntry { name: "yen", alt_name: None, shifted_name: None });
    table[KEYD_LEFTMETA as usize] = Some(KeyCodeEntry { name: "meta", alt_name: Some("leftmeta"), shifted_name: None });
    table[KEYD_RIGHTMETA as usize] = Some(KeyCodeEntry { name: "rightmeta", alt_name: Some("meta"), shifted_name: None });
    table[KEYD_COMPOSE as usize] = Some(KeyCodeEntry { name: "compose", alt_name: None, shifted_name: None });
    table[KEYD_STOP as usize] = Some(KeyCodeEntry { name: "stop", alt_name: None, shifted_name: None });
    table[KEYD_AGAIN as usize] = Some(KeyCodeEntry { name: "again", alt_name: None, shifted_name: None });
    table[KEYD_PROPS as usize] = Some(KeyCodeEntry { name: "props", alt_name: None, shifted_name: None });
    table[KEYD_UNDO as usize] = Some(KeyCodeEntry { name: "undo", alt_name: None, shifted_name: None });
    table[KEYD_FRONT as usize] = Some(KeyCodeEntry { name: "front", alt_name: None, shifted_name: None });
    table[KEYD_COPY as usize] = Some(KeyCodeEntry { name: "copy", alt_name: None, shifted_name: None });
    table[KEYD_OPEN as usize] = Some(KeyCodeEntry { name: "open", alt_name: None, shifted_name: None });
    table[KEYD_PASTE as usize] = Some(KeyCodeEntry { name: "paste", alt_name: None, shifted_name: None });
    table[KEYD_FIND as usize] = Some(KeyCodeEntry { name: "find", alt_name: None, shifted_name: None });
    table[KEYD_CUT as usize] = Some(KeyCodeEntry { name: "cut", alt_name: None, shifted_name: None });
    table[KEYD_HELP as usize] = Some(KeyCodeEntry { name: "help", alt_name: None, shifted_name: None });
    table[KEYD_MENU as usize] = Some(KeyCodeEntry { name: "menu", alt_name: None, shifted_name: None });
    table[KEYD_CALC as usize] = Some(KeyCodeEntry { name: "calc", alt_name: None, shifted_name: None });
    table[KEYD_SETUP as usize] = Some(KeyCodeEntry { name: "setup", alt_name: None, shifted_name: None });
    table[KEYD_SLEEP as usize] = Some(KeyCodeEntry { name: "sleep", alt_name: None, shifted_name: None });
    table[KEYD_WAKEUP as usize] = Some(KeyCodeEntry { name: "wakeup", alt_name: None, shifted_name: None });
    table[KEYD_FILE as usize] = Some(KeyCodeEntry { name: "file", alt_name: None, shifted_name: None });
    table[KEYD_SENDFILE as usize] = Some(KeyCodeEntry { name: "sendfile", alt_name: None, shifted_name: None });
    table[KEYD_DELETEFILE as usize] = Some(KeyCodeEntry { name: "deletefile", alt_name: None, shifted_name: None });
    table[KEYD_XFER as usize] = Some(KeyCodeEntry { name: "xfer", alt_name: None, shifted_name: None });
    table[KEYD_SCROLL_DOWN as usize] = Some(KeyCodeEntry { name: "scrolldown", alt_name: None, shifted_name: None });
    table[KEYD_SCROLL_UP as usize] = Some(KeyCodeEntry { name: "scrollup", alt_name: None, shifted_name: None });
    table[KEYD_WWW as usize] = Some(KeyCodeEntry { name: "www", alt_name: None, shifted_name: None });
    table[KEYD_MSDOS as usize] = Some(KeyCodeEntry { name: "msdos", alt_name: None, shifted_name: None });
    table[KEYD_COFFEE as usize] = Some(KeyCodeEntry { name: "coffee", alt_name: None, shifted_name: None });
    table[KEYD_ROTATE_DISPLAY as usize] = Some(KeyCodeEntry { name: "display", alt_name: None, shifted_name: None });
    table[KEYD_CYCLEWINDOWS as usize] = Some(KeyCodeEntry { name: "cyclewindows", alt_name: None, shifted_name: None });
    table[KEYD_MAIL as usize] = Some(KeyCodeEntry { name: "mail", alt_name: None, shifted_name: None });
    table[KEYD_BOOKMARKS as usize] = Some(KeyCodeEntry { name: "favorites", alt_name: Some("bookmarks"), shifted_name: None });
    table[KEYD_COMPUTER as usize] = Some(KeyCodeEntry { name: "computer", alt_name: None, shifted_name: None });
    table[KEYD_BACK as usize] = Some(KeyCodeEntry { name: "back", alt_name: None, shifted_name: None });
    table[KEYD_FORWARD as usize] = Some(KeyCodeEntry { name: "forward", alt_name: None, shifted_name: None });
    table[KEYD_CLOSECD as usize] = Some(KeyCodeEntry { name: "closecd", alt_name: None, shifted_name: None });
    table[KEYD_EJECTCD as usize] = Some(KeyCodeEntry { name: "ejectcd", alt_name: None, shifted_name: None });
    table[KEYD_EJECTCLOSECD as usize] = Some(KeyCodeEntry { name: "ejectclosecd", alt_name: None, shifted_name: None });
    table[KEYD_NEXTSONG as usize] = Some(KeyCodeEntry { name: "nextsong", alt_name: None, shifted_name: None });
    table[KEYD_PLAYPAUSE as usize] = Some(KeyCodeEntry { name: "playpause", alt_name: None, shifted_name: None });
    table[KEYD_PREVIOUSSONG as usize] = Some(KeyCodeEntry { name: "previoussong", alt_name: None, shifted_name: None });
    table[KEYD_STOPCD as usize] = Some(KeyCodeEntry { name: "stopcd", alt_name: None, shifted_name: None });
    table[KEYD_RECORD as usize] = Some(KeyCodeEntry { name: "record", alt_name: None, shifted_name: None });
    table[KEYD_REWIND as usize] = Some(KeyCodeEntry { name: "rewind", alt_name: None, shifted_name: None });
    table[KEYD_PHONE as usize] = Some(KeyCodeEntry { name: "phone", alt_name: None, shifted_name: None });
    table[KEYD_ISO as usize] = Some(KeyCodeEntry { name: "iso", alt_name: None, shifted_name: None });
    table[KEYD_CONFIG as usize] = Some(KeyCodeEntry { name: "config", alt_name: None, shifted_name: None });
    table[KEYD_HOMEPAGE as usize] = Some(KeyCodeEntry { name: "homepage", alt_name: None, shifted_name: None });
    table[KEYD_REFRESH as usize] = Some(KeyCodeEntry { name: "refresh", alt_name: None, shifted_name: None });
    table[KEYD_EXIT as usize] = Some(KeyCodeEntry { name: "exit", alt_name: None, shifted_name: None });
    table[KEYD_MOVE as usize] = Some(KeyCodeEntry { name: "move", alt_name: None, shifted_name: None });
    table[KEYD_EDIT as usize] = Some(KeyCodeEntry { name: "edit", alt_name: None, shifted_name: None });
    table[KEYD_ZOOM as usize] = Some(KeyCodeEntry { name: "zoom", alt_name: None, shifted_name: None });
    table[KEYD_KPLEFTPAREN as usize] = Some(KeyCodeEntry { name: "kpleftparen", alt_name: None, shifted_name: None });
    table[KEYD_KPRIGHTPAREN as usize] = Some(KeyCodeEntry { name: "kprightparen", alt_name: None, shifted_name: None });
    table[KEYD_NEW as usize] = Some(KeyCodeEntry { name: "new", alt_name: None, shifted_name: None });
    table[KEYD_REDO as usize] = Some(KeyCodeEntry { name: "redo", alt_name: None, shifted_name: None });
    table[KEYD_F13 as usize] = Some(KeyCodeEntry { name: "f13", alt_name: None, shifted_name: None });
    table[KEYD_F14 as usize] = Some(KeyCodeEntry { name: "f14", alt_name: None, shifted_name: None });
    table[KEYD_F15 as usize] = Some(KeyCodeEntry { name: "f15", alt_name: None, shifted_name: None });
    table[KEYD_F16 as usize] = Some(KeyCodeEntry { name: "f16", alt_name: None, shifted_name: None });
    table[KEYD_F17 as usize] = Some(KeyCodeEntry { name: "f17", alt_name: None, shifted_name: None });
    table[KEYD_F18 as usize] = Some(KeyCodeEntry { name: "f18", alt_name: None, shifted_name: None });
    table[KEYD_F19 as usize] = Some(KeyCodeEntry { name: "f19", alt_name: None, shifted_name: None });
    table[KEYD_F20 as usize] = Some(KeyCodeEntry { name: "f20", alt_name: None, shifted_name: None });
    table[KEYD_F21 as usize] = Some(KeyCodeEntry { name: "f21", alt_name: Some("prog1"), shifted_name: None });
    table[KEYD_F22 as usize] = Some(KeyCodeEntry { name: "f22", alt_name: Some("prog2"), shifted_name: None });
    table[KEYD_F23 as usize] = Some(KeyCodeEntry { name: "f23", alt_name: Some("prog3"), shifted_name: None });
    table[KEYD_F24 as usize] = Some(KeyCodeEntry { name: "f24", alt_name: Some("prog4"), shifted_name: None });
    table[KEYD_PLAYCD as usize] = Some(KeyCodeEntry { name: "playcd", alt_name: None, shifted_name: None });
    table[KEYD_PAUSECD as usize] = Some(KeyCodeEntry { name: "pausecd", alt_name: None, shifted_name: None });
    table[KEYD_SCROLL_LEFT as usize] = Some(KeyCodeEntry { name: "scrollleft", alt_name: None, shifted_name: None });
    table[KEYD_SCROLL_RIGHT as usize] = Some(KeyCodeEntry { name: "scrollright", alt_name: None, shifted_name: None });
    table[KEYD_DASHBOARD as usize] = Some(KeyCodeEntry { name: "dashboard", alt_name: None, shifted_name: None });
    table[KEYD_SUSPEND as usize] = Some(KeyCodeEntry { name: "suspend", alt_name: None, shifted_name: None });
    table[KEYD_CLOSE as usize] = Some(KeyCodeEntry { name: "close", alt_name: None, shifted_name: None });
    table[KEYD_PLAY as usize] = Some(KeyCodeEntry { name: "play", alt_name: None, shifted_name: None });
    table[KEYD_FASTFORWARD as usize] = Some(KeyCodeEntry { name: "fastforward", alt_name: None, shifted_name: None });
    table[KEYD_BASSBOOST as usize] = Some(KeyCodeEntry { name: "bassboost", alt_name: None, shifted_name: None });
    table[KEYD_PRINT as usize] = Some(KeyCodeEntry { name: "print", alt_name: None, shifted_name: None });
    table[KEYD_HP as usize] = Some(KeyCodeEntry { name: "hp", alt_name: None, shifted_name: None });
    table[KEYD_CAMERA as usize] = Some(KeyCodeEntry { name: "camera", alt_name: None, shifted_name: None });
    table[KEYD_SOUND as usize] = Some(KeyCodeEntry { name: "sound", alt_name: None, shifted_name: None });
    table[KEYD_QUESTION as usize] = Some(KeyCodeEntry { name: "question", alt_name: None, shifted_name: None });
    table[KEYD_EMAIL as usize] = Some(KeyCodeEntry { name: "email", alt_name: None, shifted_name: None });
    table[KEYD_CHAT as usize] = Some(KeyCodeEntry { name: "chat", alt_name: None, shifted_name: None });
    table[KEYD_SEARCH as usize] = Some(KeyCodeEntry { name: "search", alt_name: None, shifted_name: None });
    table[KEYD_CONNECT as usize] = Some(KeyCodeEntry { name: "connect", alt_name: None, shifted_name: None });
    table[KEYD_FINANCE as usize] = Some(KeyCodeEntry { name: "finance", alt_name: None, shifted_name: None });
    table[KEYD_SPORT as usize] = Some(KeyCodeEntry { name: "sport", alt_name: None, shifted_name: None });
    table[KEYD_SHOP as usize] = Some(KeyCodeEntry { name: "shop", alt_name: None, shifted_name: None });
    table[KEYD_VOICECOMMAND as usize] = Some(KeyCodeEntry { name: "voicecommand", alt_name: None, shifted_name: None });
    table[KEYD_CANCEL as usize] = Some(KeyCodeEntry { name: "cancel", alt_name: None, shifted_name: None });
    table[KEYD_BRIGHTNESSDOWN as usize] = Some(KeyCodeEntry { name: "brightnessdown", alt_name: None, shifted_name: None });
    table[KEYD_BRIGHTNESSUP as usize] = Some(KeyCodeEntry { name: "brightnessup", alt_name: None, shifted_name: None });
    table[KEYD_MEDIA as usize] = Some(KeyCodeEntry { name: "media", alt_name: None, shifted_name: None });
    table[KEYD_SWITCHVIDEOMODE as usize] = Some(KeyCodeEntry { name: "switchvideomode", alt_name: None, shifted_name: None });
    table[KEYD_KBDILLUMTOGGLE as usize] = Some(KeyCodeEntry { name: "kbdillumtoggle", alt_name: None, shifted_name: None });
    table[KEYD_KBDILLUMDOWN as usize] = Some(KeyCodeEntry { name: "kbdillumdown", alt_name: None, shifted_name: None });
    table[KEYD_KBDILLUMUP as usize] = Some(KeyCodeEntry { name: "kbdillumup", alt_name: None, shifted_name: None });
    table[KEYD_SEND as usize] = Some(KeyCodeEntry { name: "send", alt_name: None, shifted_name: None });
    table[KEYD_REPLY as usize] = Some(KeyCodeEntry { name: "reply", alt_name: None, shifted_name: None });
    table[KEYD_FORWARDMAIL as usize] = Some(KeyCodeEntry { name: "forwardmail", alt_name: None, shifted_name: None });
    table[KEYD_SAVE as usize] = Some(KeyCodeEntry { name: "save", alt_name: None, shifted_name: None });
    table[KEYD_DOCUMENTS as usize] = Some(KeyCodeEntry { name: "documents", alt_name: None, shifted_name: None });
    table[KEYD_BATTERY as usize] = Some(KeyCodeEntry { name: "battery", alt_name: None, shifted_name: None });
    table[KEYD_BLUETOOTH as usize] = Some(KeyCodeEntry { name: "bluetooth", alt_name: None, shifted_name: None });
    table[KEYD_WLAN as usize] = Some(KeyCodeEntry { name: "wlan", alt_name: None, shifted_name: None });
    table[KEYD_UWB as usize] = Some(KeyCodeEntry { name: "uwb", alt_name: None, shifted_name: None });
    table[KEYD_UNKNOWN as usize] = Some(KeyCodeEntry { name: "unknown", alt_name: None, shifted_name: None });
    table[KEYD_VIDEO_NEXT as usize] = Some(KeyCodeEntry { name: "next", alt_name: None, shifted_name: None });
    table[KEYD_VIDEO_PREV as usize] = Some(KeyCodeEntry { name: "prev", alt_name: None, shifted_name: None });
    table[KEYD_BRIGHTNESS_CYCLE as usize] = Some(KeyCodeEntry { name: "cycle", alt_name: None, shifted_name: None });
    table[KEYD_BRIGHTNESS_AUTO as usize] = Some(KeyCodeEntry { name: "auto", alt_name: None, shifted_name: None });
    table[KEYD_DISPLAY_OFF as usize] = Some(KeyCodeEntry { name: "off", alt_name: None, shifted_name: None });
    table[KEYD_WWAN as usize] = Some(KeyCodeEntry { name: "wwan", alt_name: None, shifted_name: None });
    table[KEYD_RFKILL as usize] = Some(KeyCodeEntry { name: "rfkill", alt_name: None, shifted_name: None });
    table[KEYD_MICMUTE as usize] = Some(KeyCodeEntry { name: "micmute", alt_name: None, shifted_name: None });

    table[KEYD_NOOP as usize] = Some(KeyCodeEntry { name: "noop", alt_name: None, shifted_name: None });
    table[KEYD_EXTERNAL_MOUSE_BUTTON as usize] = Some(KeyCodeEntry { name: "externalmousebutton", alt_name: None, shifted_name: None });
    table[KEYD_LEFT_MOUSE as usize] = Some(KeyCodeEntry { name: "leftmouse", alt_name: None, shifted_name: None });
    table[KEYD_MIDDLE_MOUSE as usize] = Some(KeyCodeEntry { name: "middlemouse", alt_name: None, shifted_name: None });
    table[KEYD_RIGHT_MOUSE as usize] = Some(KeyCodeEntry { name: "rightmouse", alt_name: None, shifted_name: None });
    table[KEYD_MOUSE_1 as usize] = Some(KeyCodeEntry { name: "mouse1", alt_name: None, shifted_name: None });
    table[KEYD_MOUSE_2 as usize] = Some(KeyCodeEntry { name: "mouse2", alt_name: None, shifted_name: None });
    table[KEYD_MOUSE_BACK as usize] = Some(KeyCodeEntry { name: "mouseback", alt_name: None, shifted_name: None });
    table[KEYD_FN as usize] = Some(KeyCodeEntry { name: "fn", alt_name: None, shifted_name: None });
    table[KEYD_MOUSE_FORWARD as usize] = Some(KeyCodeEntry { name: "mouseforward", alt_name: None, shifted_name: None });

    table
};

pub fn lookup_keycode(name: &str) -> Option<u16> {
    for (i, entry) in KEYCODE_TABLE.iter().enumerate() {
        if let Some(ent) = entry {
            if ent.name == name || ent.alt_name == Some(name) || ent.shifted_name == Some(name) {
                return Some(i as u16);
            }
        }
    }
    None
}

pub struct Modifier {
    pub mask: u8,
    pub key: u16,
}

pub fn modifiers() -> [Modifier; 5] {
    [
        Modifier { mask: MOD_ALT, key: KEYD_LEFTALT },
        Modifier { mask: MOD_ALT_GR, key: KEYD_RIGHTALT },
        Modifier { mask: MOD_SHIFT, key: KEYD_LEFTSHIFT },
        Modifier { mask: MOD_SUPER, key: KEYD_LEFTMETA },
        Modifier { mask: MOD_CTRL, key: KEYD_LEFTCTRL },
    ]
}

pub fn parse_key_sequence(s: &str, code: &mut u16, mods: &mut u8) -> bool {
    // Basic implementation for now, should be more robust
    if let Some(c) = lookup_keycode(s) {
        *code = c;
        *mods = 0;
        return true;
    }
    false
}
