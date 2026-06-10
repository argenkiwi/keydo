pub const MOD_ALT: u8 = 0x1;
pub const MOD_SUPER: u8 = 0x2;
pub const MOD_SHIFT: u8 = 0x4;
pub const MOD_CTRL: u8 = 0x8;
pub const MOD_ALT_GR: u8 = 0x10;

pub const MAX_MOD: usize = 5;

pub struct Modifier {
    pub mask: u8,
    pub key: u8,
}

pub const MODIFIERS: [Modifier; MAX_MOD] = [
    Modifier { mask: MOD_ALT, key: KEYD_LEFTALT },
    Modifier { mask: MOD_ALT_GR, key: KEYD_RIGHTALT },
    Modifier { mask: MOD_SHIFT, key: KEYD_LEFTSHIFT },
    Modifier { mask: MOD_SUPER, key: KEYD_LEFTMETA },
    Modifier { mask: MOD_CTRL, key: KEYD_LEFTCTRL },
];

pub struct KeycodeTableEnt {
    pub name: Option<&'static str>,
    pub alt_name: Option<&'static str>,
    pub shifted_name: Option<&'static str>,
}

pub const KEYD_ESC: u8 = 1;
pub const KEYD_1: u8 = 2;
pub const KEYD_2: u8 = 3;
pub const KEYD_3: u8 = 4;
pub const KEYD_4: u8 = 5;
pub const KEYD_5: u8 = 6;
pub const KEYD_6: u8 = 7;
pub const KEYD_7: u8 = 8;
pub const KEYD_8: u8 = 9;
pub const KEYD_9: u8 = 10;
pub const KEYD_0: u8 = 11;
pub const KEYD_MINUS: u8 = 12;
pub const KEYD_EQUAL: u8 = 13;
pub const KEYD_BACKSPACE: u8 = 14;
pub const KEYD_TAB: u8 = 15;
pub const KEYD_Q: u8 = 16;
pub const KEYD_W: u8 = 17;
pub const KEYD_E: u8 = 18;
pub const KEYD_R: u8 = 19;
pub const KEYD_T: u8 = 20;
pub const KEYD_Y: u8 = 21;
pub const KEYD_U: u8 = 22;
pub const KEYD_I: u8 = 23;
pub const KEYD_O: u8 = 24;
pub const KEYD_P: u8 = 25;
pub const KEYD_LEFTBRACE: u8 = 26;
pub const KEYD_RIGHTBRACE: u8 = 27;
pub const KEYD_ENTER: u8 = 28;
pub const KEYD_LEFTCTRL: u8 = 29;
pub const KEYD_A: u8 = 30;
pub const KEYD_S: u8 = 31;
pub const KEYD_D: u8 = 32;
pub const KEYD_F: u8 = 33;
pub const KEYD_G: u8 = 34;
pub const KEYD_H: u8 = 35;
pub const KEYD_J: u8 = 36;
pub const KEYD_K: u8 = 37;
pub const KEYD_L: u8 = 38;
pub const KEYD_SEMICOLON: u8 = 39;
pub const KEYD_APOSTROPHE: u8 = 40;
pub const KEYD_GRAVE: u8 = 41;
pub const KEYD_LEFTSHIFT: u8 = 42;
pub const KEYD_BACKSLASH: u8 = 43;
pub const KEYD_Z: u8 = 44;
pub const KEYD_X: u8 = 45;
pub const KEYD_C: u8 = 46;
pub const KEYD_V: u8 = 47;
pub const KEYD_B: u8 = 48;
pub const KEYD_N: u8 = 49;
pub const KEYD_M: u8 = 50;
pub const KEYD_COMMA: u8 = 51;
pub const KEYD_DOT: u8 = 52;
pub const KEYD_SLASH: u8 = 53;
pub const KEYD_RIGHTSHIFT: u8 = 54;
pub const KEYD_KPASTERISK: u8 = 55;
pub const KEYD_LEFTALT: u8 = 56;
pub const KEYD_SPACE: u8 = 57;
pub const KEYD_CAPSLOCK: u8 = 58;
pub const KEYD_F1: u8 = 59;
pub const KEYD_F2: u8 = 60;
pub const KEYD_F3: u8 = 61;
pub const KEYD_F4: u8 = 62;
pub const KEYD_F5: u8 = 63;
pub const KEYD_F6: u8 = 64;
pub const KEYD_F7: u8 = 65;
pub const KEYD_F8: u8 = 66;
pub const KEYD_F9: u8 = 67;
pub const KEYD_F10: u8 = 68;
pub const KEYD_NUMLOCK: u8 = 69;
pub const KEYD_SCROLLLOCK: u8 = 70;
pub const KEYD_KP7: u8 = 71;
pub const KEYD_KP8: u8 = 72;
pub const KEYD_KP9: u8 = 73;
pub const KEYD_KPMINUS: u8 = 74;
pub const KEYD_KP4: u8 = 75;
pub const KEYD_KP5: u8 = 76;
pub const KEYD_KP6: u8 = 77;
pub const KEYD_KPPLUS: u8 = 78;
pub const KEYD_KP1: u8 = 79;
pub const KEYD_KP2: u8 = 80;
pub const KEYD_KP3: u8 = 81;
pub const KEYD_KP0: u8 = 82;
pub const KEYD_KPDOT: u8 = 83;
pub const KEYD_IS_LEVEL3_SHIFT: u8 = 84;
pub const KEYD_ZENKAKUHANKAKU: u8 = 85;
pub const KEYD_102ND: u8 = 86;
pub const KEYD_F11: u8 = 87;
pub const KEYD_F12: u8 = 88;
pub const KEYD_RO: u8 = 89;
pub const KEYD_KATAKANA: u8 = 90;
pub const KEYD_HIRAGANA: u8 = 91;
pub const KEYD_HENKAN: u8 = 92;
pub const KEYD_KATAKANAHIRAGANA: u8 = 93;
pub const KEYD_MUHENKAN: u8 = 94;
pub const KEYD_KPJPCOMMA: u8 = 95;
pub const KEYD_KPENTER: u8 = 96;
pub const KEYD_RIGHTCTRL: u8 = 97;
pub const KEYD_KPSLASH: u8 = 98;
pub const KEYD_SYSRQ: u8 = 99;
pub const KEYD_RIGHTALT: u8 = 100;
pub const KEYD_LINEFEED: u8 = 101;
pub const KEYD_HOME: u8 = 102;
pub const KEYD_UP: u8 = 103;
pub const KEYD_PAGEUP: u8 = 104;
pub const KEYD_LEFT: u8 = 105;
pub const KEYD_RIGHT: u8 = 106;
pub const KEYD_END: u8 = 107;
pub const KEYD_DOWN: u8 = 108;
pub const KEYD_PAGEDOWN: u8 = 109;
pub const KEYD_INSERT: u8 = 110;
pub const KEYD_DELETE: u8 = 111;
pub const KEYD_MACRO: u8 = 112;
pub const KEYD_MUTE: u8 = 113;
pub const KEYD_VOLUMEDOWN: u8 = 114;
pub const KEYD_VOLUMEUP: u8 = 115;
pub const KEYD_POWER: u8 = 116;
pub const KEYD_KPEQUAL: u8 = 117;
pub const KEYD_KPPLUSMINUS: u8 = 118;
pub const KEYD_PAUSE: u8 = 119;
pub const KEYD_SCALE: u8 = 120;
pub const KEYD_KPCOMMA: u8 = 121;
pub const KEYD_HANGEUL: u8 = 122;
pub const KEYD_HANJA: u8 = 123;
pub const KEYD_YEN: u8 = 124;
pub const KEYD_LEFTMETA: u8 = 125;
pub const KEYD_RIGHTMETA: u8 = 126;
pub const KEYD_COMPOSE: u8 = 127;
pub const KEYD_STOP: u8 = 128;
pub const KEYD_AGAIN: u8 = 129;
pub const KEYD_PROPS: u8 = 130;
pub const KEYD_UNDO: u8 = 131;
pub const KEYD_FRONT: u8 = 132;
pub const KEYD_COPY: u8 = 133;
pub const KEYD_OPEN: u8 = 134;
pub const KEYD_PASTE: u8 = 135;
pub const KEYD_FIND: u8 = 136;
pub const KEYD_CUT: u8 = 137;
pub const KEYD_HELP: u8 = 138;
pub const KEYD_MENU: u8 = 139;
pub const KEYD_CALC: u8 = 140;
pub const KEYD_SETUP: u8 = 141;
pub const KEYD_SLEEP: u8 = 142;
pub const KEYD_WAKEUP: u8 = 143;
pub const KEYD_FILE: u8 = 144;
pub const KEYD_SENDFILE: u8 = 145;
pub const KEYD_DELETEFILE: u8 = 146;
pub const KEYD_XFER: u8 = 147;
pub const KEYD_SCROLL_DOWN: u8 = 148;
pub const KEYD_SCROLL_UP: u8 = 149;
pub const KEYD_WWW: u8 = 150;
pub const KEYD_MSDOS: u8 = 151;
pub const KEYD_COFFEE: u8 = 152;
pub const KEYD_ROTATE_DISPLAY: u8 = 153;
pub const KEYD_CYCLEWINDOWS: u8 = 154;
pub const KEYD_MAIL: u8 = 155;
pub const KEYD_BOOKMARKS: u8 = 156;
pub const KEYD_COMPUTER: u8 = 157;
pub const KEYD_BACK: u8 = 158;
pub const KEYD_FORWARD: u8 = 159;
pub const KEYD_CLOSECD: u8 = 160;
pub const KEYD_EJECTCD: u8 = 161;
pub const KEYD_EJECTCLOSECD: u8 = 162;
pub const KEYD_NEXTSONG: u8 = 163;
pub const KEYD_PLAYPAUSE: u8 = 164;
pub const KEYD_PREVIOUSSONG: u8 = 165;
pub const KEYD_STOPCD: u8 = 166;
pub const KEYD_RECORD: u8 = 167;
pub const KEYD_REWIND: u8 = 168;
pub const KEYD_PHONE: u8 = 169;
pub const KEYD_ISO: u8 = 170;
pub const KEYD_CONFIG: u8 = 171;
pub const KEYD_HOMEPAGE: u8 = 172;
pub const KEYD_REFRESH: u8 = 173;
pub const KEYD_EXIT: u8 = 174;
pub const KEYD_MOVE: u8 = 175;
pub const KEYD_EDIT: u8 = 176;
pub const KEYD_ZOOM: u8 = 177;
pub const KEYD_MOUSE_BACK: u8 = 178;
pub const KEYD_KPLEFTPAREN: u8 = 179;
pub const KEYD_KPRIGHTPAREN: u8 = 180;
pub const KEYD_NEW: u8 = 181;
pub const KEYD_REDO: u8 = 182;
pub const KEYD_F13: u8 = 183;
pub const KEYD_F14: u8 = 184;
pub const KEYD_F15: u8 = 185;
pub const KEYD_F16: u8 = 186;
pub const KEYD_F17: u8 = 187;
pub const KEYD_F18: u8 = 188;
pub const KEYD_F19: u8 = 189;
pub const KEYD_F20: u8 = 190;
pub const KEYD_F21: u8 = 191;
pub const KEYD_F22: u8 = 192;
pub const KEYD_F23: u8 = 193;
pub const KEYD_F24: u8 = 194;
pub const KEYD_NOOP: u8 = 195;
pub const KEYD_EXTERNAL_MOUSE_BUTTON: u8 = 196;
pub const KEYD_CHORD_1: u8 = 197;
pub const KEYD_CHORD_2: u8 = 198;
pub const KEYD_CHORD_MAX: u8 = 199;
pub const KEYD_PLAYCD: u8 = 200;
pub const KEYD_PAUSECD: u8 = 201;
pub const KEYD_SCROLL_LEFT: u8 = 202;
pub const KEYD_SCROLL_RIGHT: u8 = 203;
pub const KEYD_DASHBOARD: u8 = 204;
pub const KEYD_SUSPEND: u8 = 205;
pub const KEYD_CLOSE: u8 = 206;
pub const KEYD_PLAY: u8 = 207;
pub const KEYD_FASTFORWARD: u8 = 208;
pub const KEYD_BASSBOOST: u8 = 209;
pub const KEYD_PRINT: u8 = 210;
pub const KEYD_HP: u8 = 211;
pub const KEYD_CAMERA: u8 = 212;
pub const KEYD_SOUND: u8 = 213;
pub const KEYD_QUESTION: u8 = 214;
pub const KEYD_EMAIL: u8 = 215;
pub const KEYD_CHAT: u8 = 216;
pub const KEYD_SEARCH: u8 = 217;
pub const KEYD_CONNECT: u8 = 218;
pub const KEYD_FINANCE: u8 = 219;
pub const KEYD_SPORT: u8 = 220;
pub const KEYD_SHOP: u8 = 221;
pub const KEYD_VOICECOMMAND: u8 = 222;
pub const KEYD_CANCEL: u8 = 223;
pub const KEYD_BRIGHTNESSDOWN: u8 = 224;
pub const KEYD_BRIGHTNESSUP: u8 = 225;
pub const KEYD_MEDIA: u8 = 226;
pub const KEYD_SWITCHVIDEOMODE: u8 = 227;
pub const KEYD_KBDILLUMTOGGLE: u8 = 228;
pub const KEYD_KBDILLUMDOWN: u8 = 229;
pub const KEYD_KBDILLUMUP: u8 = 230;
pub const KEYD_SEND: u8 = 231;
pub const KEYD_REPLY: u8 = 232;
pub const KEYD_FORWARDMAIL: u8 = 233;
pub const KEYD_SAVE: u8 = 234;
pub const KEYD_DOCUMENTS: u8 = 235;
pub const KEYD_BATTERY: u8 = 236;
pub const KEYD_BLUETOOTH: u8 = 237;
pub const KEYD_WLAN: u8 = 238;
pub const KEYD_UWB: u8 = 239;
pub const KEYD_UNKNOWN: u8 = 240;
pub const KEYD_VIDEO_NEXT: u8 = 241;
pub const KEYD_VIDEO_PREV: u8 = 242;
pub const KEYD_BRIGHTNESS_CYCLE: u8 = 243;
pub const KEYD_BRIGHTNESS_AUTO: u8 = 244;
pub const KEYD_DISPLAY_OFF: u8 = 245;
pub const KEYD_WWAN: u8 = 246;
pub const KEYD_RFKILL: u8 = 247;
pub const KEYD_MICMUTE: u8 = 248;
pub const KEYD_LEFT_MOUSE: u8 = 249;
pub const KEYD_MIDDLE_MOUSE: u8 = 250;
pub const KEYD_RIGHT_MOUSE: u8 = 251;
pub const KEYD_MOUSE_1: u8 = 252;
pub const KEYD_MOUSE_2: u8 = 253;
pub const KEYD_FN: u8 = 254;
pub const KEYD_MOUSE_FORWARD: u8 = 255;

pub const KEYCODE_TABLE: [KeycodeTableEnt; 256] = [
    KeycodeTableEnt { name: None, alt_name: None, shifted_name: None }, // 0
    KeycodeTableEnt { name: Some("esc"), alt_name: Some("escape"), shifted_name: None }, // KEYD_ESC
    KeycodeTableEnt { name: Some("1"), alt_name: None, shifted_name: Some("!") }, // KEYD_1
    KeycodeTableEnt { name: Some("2"), alt_name: None, shifted_name: Some("@") }, // KEYD_2
    KeycodeTableEnt { name: Some("3"), alt_name: None, shifted_name: Some("#") }, // KEYD_3
    KeycodeTableEnt { name: Some("4"), alt_name: None, shifted_name: Some("$") }, // KEYD_4
    KeycodeTableEnt { name: Some("5"), alt_name: None, shifted_name: Some("%") }, // KEYD_5
    KeycodeTableEnt { name: Some("6"), alt_name: None, shifted_name: Some("^") }, // KEYD_6
    KeycodeTableEnt { name: Some("7"), alt_name: None, shifted_name: Some("&") }, // KEYD_7
    KeycodeTableEnt { name: Some("8"), alt_name: None, shifted_name: Some("*") }, // KEYD_8
    KeycodeTableEnt { name: Some("9"), alt_name: None, shifted_name: Some("(") }, // KEYD_9
    KeycodeTableEnt { name: Some("0"), alt_name: None, shifted_name: Some(")") }, // KEYD_0
    KeycodeTableEnt { name: Some("-"), alt_name: Some("minus"), shifted_name: Some("_") }, // KEYD_MINUS
    KeycodeTableEnt { name: Some("="), alt_name: Some("equal"), shifted_name: Some("+") }, // KEYD_EQUAL
    KeycodeTableEnt { name: Some("backspace"), alt_name: None, shifted_name: None }, // KEYD_BACKSPACE
    KeycodeTableEnt { name: Some("tab"), alt_name: None, shifted_name: None }, // KEYD_TAB
    KeycodeTableEnt { name: Some("q"), alt_name: None, shifted_name: Some("Q") }, // KEYD_Q
    KeycodeTableEnt { name: Some("w"), alt_name: None, shifted_name: Some("W") }, // KEYD_W
    KeycodeTableEnt { name: Some("e"), alt_name: None, shifted_name: Some("E") }, // KEYD_E
    KeycodeTableEnt { name: Some("r"), alt_name: None, shifted_name: Some("R") }, // KEYD_R
    KeycodeTableEnt { name: Some("t"), alt_name: None, shifted_name: Some("T") }, // KEYD_T
    KeycodeTableEnt { name: Some("y"), alt_name: None, shifted_name: Some("Y") }, // KEYD_Y
    KeycodeTableEnt { name: Some("u"), alt_name: None, shifted_name: Some("U") }, // KEYD_U
    KeycodeTableEnt { name: Some("i"), alt_name: None, shifted_name: Some("I") }, // KEYD_I
    KeycodeTableEnt { name: Some("o"), alt_name: None, shifted_name: Some("O") }, // KEYD_O
    KeycodeTableEnt { name: Some("p"), alt_name: None, shifted_name: Some("P") }, // KEYD_P
    KeycodeTableEnt { name: Some("["), alt_name: Some("leftbrace"), shifted_name: Some("{") }, // KEYD_LEFTBRACE
    KeycodeTableEnt { name: Some("]"), alt_name: Some("rightbrace"), shifted_name: Some("}") }, // KEYD_RIGHTBRACE
    KeycodeTableEnt { name: Some("enter"), alt_name: None, shifted_name: None }, // KEYD_ENTER
    KeycodeTableEnt { name: Some("leftcontrol"), alt_name: Some(""), shifted_name: None }, // KEYD_LEFTCTRL
    KeycodeTableEnt { name: Some("a"), alt_name: None, shifted_name: Some("A") }, // KEYD_A
    KeycodeTableEnt { name: Some("s"), alt_name: None, shifted_name: Some("S") }, // KEYD_S
    KeycodeTableEnt { name: Some("d"), alt_name: None, shifted_name: Some("D") }, // KEYD_D
    KeycodeTableEnt { name: Some("f"), alt_name: None, shifted_name: Some("F") }, // KEYD_F
    KeycodeTableEnt { name: Some("g"), alt_name: None, shifted_name: Some("G") }, // KEYD_G
    KeycodeTableEnt { name: Some("h"), alt_name: None, shifted_name: Some("H") }, // KEYD_H
    KeycodeTableEnt { name: Some("j"), alt_name: None, shifted_name: Some("J") }, // KEYD_J
    KeycodeTableEnt { name: Some("k"), alt_name: None, shifted_name: Some("K") }, // KEYD_K
    KeycodeTableEnt { name: Some("l"), alt_name: None, shifted_name: Some("L") }, // KEYD_L
    KeycodeTableEnt { name: Some(";"), alt_name: Some("semicolon"), shifted_name: Some(":") }, // KEYD_SEMICOLON
    KeycodeTableEnt { name: Some("'"), alt_name: Some("apostrophe"), shifted_name: Some("\"") }, // KEYD_APOSTROPHE
    KeycodeTableEnt { name: Some("`"), alt_name: Some("grave"), shifted_name: Some("~") }, // KEYD_GRAVE
    KeycodeTableEnt { name: Some("leftshift"), alt_name: Some(""), shifted_name: None }, // KEYD_LEFTSHIFT
    KeycodeTableEnt { name: Some("\\"), alt_name: Some("backslash"), shifted_name: Some("|") }, // KEYD_BACKSLASH
    KeycodeTableEnt { name: Some("z"), alt_name: None, shifted_name: Some("Z") }, // KEYD_Z
    KeycodeTableEnt { name: Some("x"), alt_name: None, shifted_name: Some("X") }, // KEYD_X
    KeycodeTableEnt { name: Some("c"), alt_name: None, shifted_name: Some("C") }, // KEYD_C
    KeycodeTableEnt { name: Some("v"), alt_name: None, shifted_name: Some("V") }, // KEYD_V
    KeycodeTableEnt { name: Some("b"), alt_name: None, shifted_name: Some("B") }, // KEYD_B
    KeycodeTableEnt { name: Some("n"), alt_name: None, shifted_name: Some("N") }, // KEYD_N
    KeycodeTableEnt { name: Some("m"), alt_name: None, shifted_name: Some("M") }, // KEYD_M
    KeycodeTableEnt { name: Some(","), alt_name: Some("comma"), shifted_name: Some("<") }, // KEYD_COMMA
    KeycodeTableEnt { name: Some("."), alt_name: Some("dot"), shifted_name: Some(">") }, // KEYD_DOT
    KeycodeTableEnt { name: Some("/"), alt_name: Some("slash"), shifted_name: Some("?") }, // KEYD_SLASH
    KeycodeTableEnt { name: Some("rightshift"), alt_name: None, shifted_name: None }, // KEYD_RIGHTSHIFT
    KeycodeTableEnt { name: Some("kpasterisk"), alt_name: None, shifted_name: None }, // KEYD_KPASTERISK
    KeycodeTableEnt { name: Some("leftalt"), alt_name: Some(""), shifted_name: None }, // KEYD_LEFTALT
    KeycodeTableEnt { name: Some("space"), alt_name: None, shifted_name: None }, // KEYD_SPACE
    KeycodeTableEnt { name: Some("capslock"), alt_name: None, shifted_name: None }, // KEYD_CAPSLOCK
    KeycodeTableEnt { name: Some("f1"), alt_name: None, shifted_name: None }, // KEYD_F1
    KeycodeTableEnt { name: Some("f2"), alt_name: None, shifted_name: None }, // KEYD_F2
    KeycodeTableEnt { name: Some("f3"), alt_name: None, shifted_name: None }, // KEYD_F3
    KeycodeTableEnt { name: Some("f4"), alt_name: None, shifted_name: None }, // KEYD_F4
    KeycodeTableEnt { name: Some("f5"), alt_name: None, shifted_name: None }, // KEYD_F5
    KeycodeTableEnt { name: Some("f6"), alt_name: None, shifted_name: None }, // KEYD_F6
    KeycodeTableEnt { name: Some("f7"), alt_name: None, shifted_name: None }, // KEYD_F7
    KeycodeTableEnt { name: Some("f8"), alt_name: None, shifted_name: None }, // KEYD_F8
    KeycodeTableEnt { name: Some("f9"), alt_name: None, shifted_name: None }, // KEYD_F9
    KeycodeTableEnt { name: Some("f10"), alt_name: None, shifted_name: None }, // KEYD_F10
    KeycodeTableEnt { name: Some("numlock"), alt_name: None, shifted_name: None }, // KEYD_NUMLOCK
    KeycodeTableEnt { name: Some("scrolllock"), alt_name: None, shifted_name: None }, // KEYD_SCROLLLOCK
    KeycodeTableEnt { name: Some("kp7"), alt_name: None, shifted_name: None }, // KEYD_KP7
    KeycodeTableEnt { name: Some("kp8"), alt_name: None, shifted_name: None }, // KEYD_KP8
    KeycodeTableEnt { name: Some("kp9"), alt_name: None, shifted_name: None }, // KEYD_KP9
    KeycodeTableEnt { name: Some("kpminus"), alt_name: None, shifted_name: None }, // KEYD_KPMINUS
    KeycodeTableEnt { name: Some("kp4"), alt_name: None, shifted_name: None }, // KEYD_KP4
    KeycodeTableEnt { name: Some("kp5"), alt_name: None, shifted_name: None }, // KEYD_KP5
    KeycodeTableEnt { name: Some("kp6"), alt_name: None, shifted_name: None }, // KEYD_KP6
    KeycodeTableEnt { name: Some("kpplus"), alt_name: None, shifted_name: None }, // KEYD_KPPLUS
    KeycodeTableEnt { name: Some("kp1"), alt_name: None, shifted_name: None }, // KEYD_KP1
    KeycodeTableEnt { name: Some("kp2"), alt_name: None, shifted_name: None }, // KEYD_KP2
    KeycodeTableEnt { name: Some("kp3"), alt_name: None, shifted_name: None }, // KEYD_KP3
    KeycodeTableEnt { name: Some("kp0"), alt_name: None, shifted_name: None }, // KEYD_KP0
    KeycodeTableEnt { name: Some("kpdot"), alt_name: None, shifted_name: None }, // KEYD_KPDOT
    KeycodeTableEnt { name: Some("iso-level3-shift"), alt_name: None, shifted_name: None }, // KEYD_IS_LEVEL3_SHIFT
    KeycodeTableEnt { name: Some("zenkakuhankaku"), alt_name: None, shifted_name: None }, // KEYD_ZENKAKUHANKAKU
    KeycodeTableEnt { name: Some("102nd"), alt_name: None, shifted_name: None }, // KEYD_102ND
    KeycodeTableEnt { name: Some("f11"), alt_name: None, shifted_name: None }, // KEYD_F11
    KeycodeTableEnt { name: Some("f12"), alt_name: None, shifted_name: None }, // KEYD_F12
    KeycodeTableEnt { name: Some("ro"), alt_name: None, shifted_name: None }, // KEYD_RO
    KeycodeTableEnt { name: Some("katakana"), alt_name: None, shifted_name: None }, // KEYD_KATAKANA
    KeycodeTableEnt { name: Some("hiragana"), alt_name: None, shifted_name: None }, // KEYD_HIRAGANA
    KeycodeTableEnt { name: Some("henkan"), alt_name: None, shifted_name: None }, // KEYD_HENKAN
    KeycodeTableEnt { name: Some("katakanahiragana"), alt_name: None, shifted_name: None }, // KEYD_KATAKANAHIRAGANA
    KeycodeTableEnt { name: Some("muhenkan"), alt_name: None, shifted_name: None }, // KEYD_MUHENKAN
    KeycodeTableEnt { name: Some("kpjpcomma"), alt_name: None, shifted_name: None }, // KEYD_KPJPCOMMA
    KeycodeTableEnt { name: Some("kpenter"), alt_name: None, shifted_name: None }, // KEYD_KPENTER
    KeycodeTableEnt { name: Some("rightcontrol"), alt_name: None, shifted_name: None }, // KEYD_RIGHTCTRL
    KeycodeTableEnt { name: Some("kpslash"), alt_name: None, shifted_name: None }, // KEYD_KPSLASH
    KeycodeTableEnt { name: Some("sysrq"), alt_name: None, shifted_name: None }, // KEYD_SYSRQ
    KeycodeTableEnt { name: Some("rightalt"), alt_name: None, shifted_name: None }, // KEYD_RIGHTALT
    KeycodeTableEnt { name: Some("linefeed"), alt_name: None, shifted_name: None }, // KEYD_LINEFEED
    KeycodeTableEnt { name: Some("home"), alt_name: None, shifted_name: None }, // KEYD_HOME
    KeycodeTableEnt { name: Some("up"), alt_name: None, shifted_name: None }, // KEYD_UP
    KeycodeTableEnt { name: Some("pageup"), alt_name: None, shifted_name: None }, // KEYD_PAGEUP
    KeycodeTableEnt { name: Some("left"), alt_name: None, shifted_name: None }, // KEYD_LEFT
    KeycodeTableEnt { name: Some("right"), alt_name: None, shifted_name: None }, // KEYD_RIGHT
    KeycodeTableEnt { name: Some("end"), alt_name: None, shifted_name: None }, // KEYD_END
    KeycodeTableEnt { name: Some("down"), alt_name: None, shifted_name: None }, // KEYD_DOWN
    KeycodeTableEnt { name: Some("pagedown"), alt_name: None, shifted_name: None }, // KEYD_PAGEDOWN
    KeycodeTableEnt { name: Some("insert"), alt_name: None, shifted_name: None }, // KEYD_INSERT
    KeycodeTableEnt { name: Some("delete"), alt_name: None, shifted_name: None }, // KEYD_DELETE
    KeycodeTableEnt { name: Some("macro"), alt_name: None, shifted_name: None }, // KEYD_MACRO
    KeycodeTableEnt { name: Some("mute"), alt_name: None, shifted_name: None }, // KEYD_MUTE
    KeycodeTableEnt { name: Some("volumedown"), alt_name: None, shifted_name: None }, // KEYD_VOLUMEDOWN
    KeycodeTableEnt { name: Some("volumeup"), alt_name: None, shifted_name: None }, // KEYD_VOLUMEUP
    KeycodeTableEnt { name: Some("power"), alt_name: None, shifted_name: None }, // KEYD_POWER
    KeycodeTableEnt { name: Some("kpequal"), alt_name: None, shifted_name: None }, // KEYD_KPEQUAL
    KeycodeTableEnt { name: Some("kpplusminus"), alt_name: None, shifted_name: None }, // KEYD_KPPLUSMINUS
    KeycodeTableEnt { name: Some("pause"), alt_name: None, shifted_name: None }, // KEYD_PAUSE
    KeycodeTableEnt { name: Some("scale"), alt_name: None, shifted_name: None }, // KEYD_SCALE
    KeycodeTableEnt { name: Some("kpcomma"), alt_name: None, shifted_name: None }, // KEYD_KPCOMMA
    KeycodeTableEnt { name: Some("hangeul"), alt_name: None, shifted_name: None }, // KEYD_HANGEUL
    KeycodeTableEnt { name: Some("hanja"), alt_name: None, shifted_name: None }, // KEYD_HANJA
    KeycodeTableEnt { name: Some("yen"), alt_name: None, shifted_name: None }, // KEYD_YEN
    KeycodeTableEnt { name: Some("leftmeta"), alt_name: Some(""), shifted_name: None }, // KEYD_LEFTMETA
    KeycodeTableEnt { name: Some("rightmeta"), alt_name: None, shifted_name: None }, // KEYD_RIGHTMETA
    KeycodeTableEnt { name: Some("compose"), alt_name: None, shifted_name: None }, // KEYD_COMPOSE
    KeycodeTableEnt { name: Some("stop"), alt_name: None, shifted_name: None }, // KEYD_STOP
    KeycodeTableEnt { name: Some("again"), alt_name: None, shifted_name: None }, // KEYD_AGAIN
    KeycodeTableEnt { name: Some("props"), alt_name: None, shifted_name: None }, // KEYD_PROPS
    KeycodeTableEnt { name: Some("undo"), alt_name: None, shifted_name: None }, // KEYD_UNDO
    KeycodeTableEnt { name: Some("front"), alt_name: None, shifted_name: None }, // KEYD_FRONT
    KeycodeTableEnt { name: Some("copy"), alt_name: None, shifted_name: None }, // KEYD_COPY
    KeycodeTableEnt { name: Some("open"), alt_name: None, shifted_name: None }, // KEYD_OPEN
    KeycodeTableEnt { name: Some("paste"), alt_name: None, shifted_name: None }, // KEYD_PASTE
    KeycodeTableEnt { name: Some("find"), alt_name: None, shifted_name: None }, // KEYD_FIND
    KeycodeTableEnt { name: Some("cut"), alt_name: None, shifted_name: None }, // KEYD_CUT
    KeycodeTableEnt { name: Some("help"), alt_name: None, shifted_name: None }, // KEYD_HELP
    KeycodeTableEnt { name: Some("menu"), alt_name: None, shifted_name: None }, // KEYD_MENU
    KeycodeTableEnt { name: Some("calc"), alt_name: None, shifted_name: None }, // KEYD_CALC
    KeycodeTableEnt { name: Some("setup"), alt_name: None, shifted_name: None }, // KEYD_SETUP
    KeycodeTableEnt { name: Some("sleep"), alt_name: None, shifted_name: None }, // KEYD_SLEEP
    KeycodeTableEnt { name: Some("wakeup"), alt_name: None, shifted_name: None }, // KEYD_WAKEUP
    KeycodeTableEnt { name: Some("file"), alt_name: None, shifted_name: None }, // KEYD_FILE
    KeycodeTableEnt { name: Some("sendfile"), alt_name: None, shifted_name: None }, // KEYD_SENDFILE
    KeycodeTableEnt { name: Some("deletefile"), alt_name: None, shifted_name: None }, // KEYD_DELETEFILE
    KeycodeTableEnt { name: Some("xfer"), alt_name: None, shifted_name: None }, // KEYD_XFER
    KeycodeTableEnt { name: Some("scrolldown"), alt_name: None, shifted_name: None }, // KEYD_SCROLL_DOWN
    KeycodeTableEnt { name: Some("scrollup"), alt_name: None, shifted_name: None }, // KEYD_SCROLL_UP
    KeycodeTableEnt { name: Some("www"), alt_name: None, shifted_name: None }, // KEYD_WWW
    KeycodeTableEnt { name: Some("msdos"), alt_name: None, shifted_name: None }, // KEYD_MSDOS
    KeycodeTableEnt { name: Some("coffee"), alt_name: None, shifted_name: None }, // KEYD_COFFEE
    KeycodeTableEnt { name: Some("display"), alt_name: None, shifted_name: None }, // KEYD_ROTATE_DISPLAY
    KeycodeTableEnt { name: Some("cyclewindows"), alt_name: None, shifted_name: None }, // KEYD_CYCLEWINDOWS
    KeycodeTableEnt { name: Some("mail"), alt_name: None, shifted_name: None }, // KEYD_MAIL
    KeycodeTableEnt { name: Some("favorites"), alt_name: Some("bookmarks"), shifted_name: None }, // KEYD_BOOKMARKS
    KeycodeTableEnt { name: Some("computer"), alt_name: None, shifted_name: None }, // KEYD_COMPUTER
    KeycodeTableEnt { name: Some("back"), alt_name: None, shifted_name: None }, // KEYD_BACK
    KeycodeTableEnt { name: Some("forward"), alt_name: None, shifted_name: None }, // KEYD_FORWARD
    KeycodeTableEnt { name: Some("closecd"), alt_name: None, shifted_name: None }, // KEYD_CLOSECD
    KeycodeTableEnt { name: Some("ejectcd"), alt_name: None, shifted_name: None }, // KEYD_EJECTCD
    KeycodeTableEnt { name: Some("ejectclosecd"), alt_name: None, shifted_name: None }, // KEYD_EJECTCLOSECD
    KeycodeTableEnt { name: Some("nextsong"), alt_name: None, shifted_name: None }, // KEYD_NEXTSONG
    KeycodeTableEnt { name: Some("playpause"), alt_name: None, shifted_name: None }, // KEYD_PLAYPAUSE
    KeycodeTableEnt { name: Some("previoussong"), alt_name: None, shifted_name: None }, // KEYD_PREVIOUSSONG
    KeycodeTableEnt { name: Some("stopcd"), alt_name: None, shifted_name: None }, // KEYD_STOPCD
    KeycodeTableEnt { name: Some("record"), alt_name: None, shifted_name: None }, // KEYD_RECORD
    KeycodeTableEnt { name: Some("rewind"), alt_name: None, shifted_name: None }, // KEYD_REWIND
    KeycodeTableEnt { name: Some("phone"), alt_name: None, shifted_name: None }, // KEYD_PHONE
    KeycodeTableEnt { name: Some("iso"), alt_name: None, shifted_name: None }, // KEYD_ISO
    KeycodeTableEnt { name: Some("config"), alt_name: None, shifted_name: None }, // KEYD_CONFIG
    KeycodeTableEnt { name: Some("homepage"), alt_name: None, shifted_name: None }, // KEYD_HOMEPAGE
    KeycodeTableEnt { name: Some("refresh"), alt_name: None, shifted_name: None }, // KEYD_REFRESH
    KeycodeTableEnt { name: Some("exit"), alt_name: None, shifted_name: None }, // KEYD_EXIT
    KeycodeTableEnt { name: Some("move"), alt_name: None, shifted_name: None }, // KEYD_MOVE
    KeycodeTableEnt { name: Some("edit"), alt_name: None, shifted_name: None }, // KEYD_EDIT
    KeycodeTableEnt { name: Some("zoom"), alt_name: None, shifted_name: None }, // KEYD_ZOOM
    KeycodeTableEnt { name: Some("mouseback"), alt_name: None, shifted_name: None }, // KEYD_MOUSE_BACK
    KeycodeTableEnt { name: Some("kpleftparen"), alt_name: None, shifted_name: None }, // KEYD_KPLEFTPAREN
    KeycodeTableEnt { name: Some("kprightparen"), alt_name: None, shifted_name: None }, // KEYD_KPRIGHTPAREN
    KeycodeTableEnt { name: Some("new"), alt_name: None, shifted_name: None }, // KEYD_NEW
    KeycodeTableEnt { name: Some("redo"), alt_name: None, shifted_name: None }, // KEYD_REDO
    KeycodeTableEnt { name: Some("f13"), alt_name: None, shifted_name: None }, // KEYD_F13
    KeycodeTableEnt { name: Some("f14"), alt_name: None, shifted_name: None }, // KEYD_F14
    KeycodeTableEnt { name: Some("f15"), alt_name: None, shifted_name: None }, // KEYD_F15
    KeycodeTableEnt { name: Some("f16"), alt_name: None, shifted_name: None }, // KEYD_F16
    KeycodeTableEnt { name: Some("f17"), alt_name: None, shifted_name: None }, // KEYD_F17
    KeycodeTableEnt { name: Some("f18"), alt_name: None, shifted_name: None }, // KEYD_F18
    KeycodeTableEnt { name: Some("f19"), alt_name: None, shifted_name: None }, // KEYD_F19
    KeycodeTableEnt { name: Some("f20"), alt_name: None, shifted_name: None }, // KEYD_F20
    KeycodeTableEnt { name: Some("f21"), alt_name: Some("prog1"), shifted_name: None }, // KEYD_F21
    KeycodeTableEnt { name: Some("f22"), alt_name: Some("prog2"), shifted_name: None }, // KEYD_F22
    KeycodeTableEnt { name: Some("f23"), alt_name: Some("prog3"), shifted_name: None }, // KEYD_F23
    KeycodeTableEnt { name: Some("f24"), alt_name: Some("prog4"), shifted_name: None }, // KEYD_F24
    KeycodeTableEnt { name: Some("noop"), alt_name: None, shifted_name: None }, // KEYD_NOOP
    KeycodeTableEnt { name: Some("externalmousebutton"), alt_name: None, shifted_name: None }, // KEYD_EXTERNAL_MOUSE_BUTTON (Wait, C has external_mouse_button as 196)
    KeycodeTableEnt { name: Some("chord1"), alt_name: None, shifted_name: None }, // KEYD_CHORD_1
    KeycodeTableEnt { name: Some("chord2"), alt_name: None, shifted_name: None }, // KEYD_CHORD_2
    KeycodeTableEnt { name: Some("chordmax"), alt_name: None, shifted_name: None }, // KEYD_CHORD_MAX
    KeycodeTableEnt { name: Some("playcd"), alt_name: None, shifted_name: None }, // KEYD_PLAYCD
    KeycodeTableEnt { name: Some("pausecd"), alt_name: None, shifted_name: None }, // KEYD_PAUSECD
    KeycodeTableEnt { name: Some("scrollleft"), alt_name: None, shifted_name: None }, // KEYD_SCROLL_LEFT
    KeycodeTableEnt { name: Some("scrollright"), alt_name: None, shifted_name: None }, // KEYD_SCROLL_RIGHT
    KeycodeTableEnt { name: Some("dashboard"), alt_name: None, shifted_name: None }, // KEYD_DASHBOARD
    KeycodeTableEnt { name: Some("suspend"), alt_name: None, shifted_name: None }, // KEYD_SUSPEND
    KeycodeTableEnt { name: Some("close"), alt_name: None, shifted_name: None }, // KEYD_CLOSE
    KeycodeTableEnt { name: Some("play"), alt_name: None, shifted_name: None }, // KEYD_PLAY
    KeycodeTableEnt { name: Some("fastforward"), alt_name: None, shifted_name: None }, // KEYD_FASTFORWARD
    KeycodeTableEnt { name: Some("bassboost"), alt_name: None, shifted_name: None }, // KEYD_BASSBOOST
    KeycodeTableEnt { name: Some("print"), alt_name: None, shifted_name: None }, // KEYD_PRINT
    KeycodeTableEnt { name: Some("hp"), alt_name: None, shifted_name: None }, // KEYD_HP
    KeycodeTableEnt { name: Some("camera"), alt_name: None, shifted_name: None }, // KEYD_CAMERA
    KeycodeTableEnt { name: Some("sound"), alt_name: None, shifted_name: None }, // KEYD_SOUND
    KeycodeTableEnt { name: Some("question"), alt_name: None, shifted_name: None }, // KEYD_QUESTION
    KeycodeTableEnt { name: Some("email"), alt_name: None, shifted_name: None }, // KEYD_EMAIL
    KeycodeTableEnt { name: Some("chat"), alt_name: None, shifted_name: None }, // KEYD_CHAT
    KeycodeTableEnt { name: Some("search"), alt_name: None, shifted_name: None }, // KEYD_SEARCH
    KeycodeTableEnt { name: Some("connect"), alt_name: None, shifted_name: None }, // KEYD_CONNECT
    KeycodeTableEnt { name: Some("finance"), alt_name: None, shifted_name: None }, // KEYD_FINANCE
    KeycodeTableEnt { name: Some("sport"), alt_name: None, shifted_name: None }, // KEYD_SPORT
    KeycodeTableEnt { name: Some("shop"), alt_name: None, shifted_name: None }, // KEYD_SHOP
    KeycodeTableEnt { name: Some("voicecommand"), alt_name: None, shifted_name: None }, // KEYD_VOICECOMMAND
    KeycodeTableEnt { name: Some("cancel"), alt_name: None, shifted_name: None }, // KEYD_CANCEL
    KeycodeTableEnt { name: Some("brightnessdown"), alt_name: None, shifted_name: None }, // KEYD_BRIGHTNESSDOWN
    KeycodeTableEnt { name: Some("brightnessup"), alt_name: None, shifted_name: None }, // KEYD_BRIGHTNESSUP
    KeycodeTableEnt { name: Some("media"), alt_name: None, shifted_name: None }, // KEYD_MEDIA
    KeycodeTableEnt { name: Some("switchvideomode"), alt_name: None, shifted_name: None }, // KEYD_SWITCHVIDEOMODE
    KeycodeTableEnt { name: Some("kbdillumtoggle"), alt_name: None, shifted_name: None }, // KEYD_KBDILLUMTOGGLE
    KeycodeTableEnt { name: Some("kbdillumdown"), alt_name: None, shifted_name: None }, // KEYD_KBDILLUMDOWN
    KeycodeTableEnt { name: Some("kbdillumup"), alt_name: None, shifted_name: None }, // KEYD_KBDILLUMUP
    KeycodeTableEnt { name: Some("send"), alt_name: None, shifted_name: None }, // KEYD_SEND
    KeycodeTableEnt { name: Some("reply"), alt_name: None, shifted_name: None }, // KEYD_REPLY
    KeycodeTableEnt { name: Some("forwardmail"), alt_name: None, shifted_name: None }, // KEYD_FORWARDMAIL
    KeycodeTableEnt { name: Some("save"), alt_name: None, shifted_name: None }, // KEYD_SAVE
    KeycodeTableEnt { name: Some("documents"), alt_name: None, shifted_name: None }, // KEYD_DOCUMENTS
    KeycodeTableEnt { name: Some("battery"), alt_name: None, shifted_name: None }, // KEYD_BATTERY
    KeycodeTableEnt { name: Some("bluetooth"), alt_name: None, shifted_name: None }, // KEYD_BLUETOOTH
    KeycodeTableEnt { name: Some("wlan"), alt_name: None, shifted_name: None }, // KEYD_WLAN
    KeycodeTableEnt { name: Some("uwb"), alt_name: None, shifted_name: None }, // KEYD_UWB
    KeycodeTableEnt { name: Some("unknown"), alt_name: None, shifted_name: None }, // KEYD_UNKNOWN
    KeycodeTableEnt { name: Some("next"), alt_name: None, shifted_name: Some("nextsong") }, // KEYD_VIDEO_NEXT (Wait, shifted_name is used for actual shifted keys in C)
    KeycodeTableEnt { name: Some("prev"), alt_name: None, shifted_name: Some("previoussong") }, // KEYD_VIDEO_PREV
    KeycodeTableEnt { name: Some("cycle"), alt_name: None, shifted_name: None }, // KEYD_BRIGHTNESS_CYCLE
    KeycodeTableEnt { name: Some("auto"), alt_name: None, shifted_name: None }, // KEYD_BRIGHTNESS_AUTO
    KeycodeTableEnt { name: Some("off"), alt_name: None, shifted_name: None }, // KEYD_DISPLAY_OFF
    KeycodeTableEnt { name: Some("wwan"), alt_name: None, shifted_name: None }, // KEYD_WWAN
    KeycodeTableEnt { name: Some("rfkill"), alt_name: None, shifted_name: None }, // KEYD_RFKILL
    KeycodeTableEnt { name: Some("micmute"), alt_name: None, shifted_name: None }, // KEYD_MICMUTE
    KeycodeTableEnt { name: Some("leftmouse"), alt_name: None, shifted_name: None }, // KEYD_LEFT_MOUSE
    KeycodeTableEnt { name: Some("middlemouse"), alt_name: None, shifted_name: None }, // KEYD_MIDDLE_MOUSE
    KeycodeTableEnt { name: Some("rightmouse"), alt_name: None, shifted_name: None }, // KEYD_RIGHT_MOUSE
    KeycodeTableEnt { name: Some("mouse1"), alt_name: None, shifted_name: None }, // KEYD_MOUSE_1
    KeycodeTableEnt { name: Some("mouse2"), alt_name: None, shifted_name: None }, // KEYD_MOUSE_2
    KeycodeTableEnt { name: Some("fn"), alt_name: None, shifted_name: None }, // KEYD_FN
    KeycodeTableEnt { name: Some("mouseforward"), alt_name: None, shifted_name: None }, // KEYD_MOUSE_FORWARD
];

pub fn modstring(mods: u8) -> String {
    let mut s = String::new();

    if mods & MOD_CTRL != 0 {
        s.push_str("C-");
    }
    if mods & MOD_SUPER != 0 {
        s.push_str("M-");
    }
    if mods & MOD_ALT_GR != 0 {
        s.push_str("G-");
    }
    if mods & MOD_SHIFT != 0 {
        s.push_str("S-");
    }
    if mods & MOD_ALT != 0 {
        s.push_str("A-");
    }

    if !s.is_empty() {
        s.pop();
    }

    s
}

pub fn parse_modset(s: &str) -> Option<u8> {
    let mut mods = 0;
    let parts = s.split('-');

    for part in parts {
        if part.len() != 1 {
            return None;
        }
        match part.chars().next().unwrap() {
            'C' => mods |= MOD_CTRL,
            'M' => mods |= MOD_SUPER,
            'A' => mods |= MOD_ALT,
            'S' => mods |= MOD_SHIFT,
            'G' => mods |= MOD_ALT_GR,
            _ => return None,
        }
    }

    Some(mods)
}

pub fn parse_key_sequence(s: &str) -> Option<(u8, u8)> {
    if s.is_empty() {
        return None;
    }

    let mut mods = 0;
    let mut c = s;

    while c.len() >= 2 && c.as_bytes()[1] == b'-' {
        match c.as_bytes()[0] {
            b'C' => mods |= MOD_CTRL,
            b'M' => mods |= MOD_SUPER,
            b'A' => mods |= MOD_ALT,
            b'S' => mods |= MOD_SHIFT,
            b'G' => mods |= MOD_ALT_GR,
            _ => return None,
        }
        c = &c[2..];
    }

    for (i, ent) in KEYCODE_TABLE.iter().enumerate() {
        if let Some(name) = ent.name {
            if let Some(shifted) = ent.shifted_name && shifted == c {
                return Some((i as u8, mods | MOD_SHIFT));
            }
            if name == c || ent.alt_name == Some(c) {
                return Some((i as u8, mods));
            }
        }
    }

    None
}
