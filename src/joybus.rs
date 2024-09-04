use crate::{si::Si, text::Colour, vi::Vi};

#[derive(Clone, Copy)]
enum JoybusCommand {
    Info,
    ReadState,
    Reset,
}

impl JoybusCommand {
    fn cmd_size(self) -> u8 {
        match self {
            Self::Info => 1,
            Self::ReadState => 1,
            Self::Reset => 1,
        }
    }

    fn response_size(self) -> u8 {
        match self {
            Self::Info => 3,
            Self::ReadState => 4,
            Self::Reset => 3,
        }
    }

    fn cmd(self) -> u8 {
        match self {
            Self::Info => 0x00,
            Self::ReadState => 0x01,
            Self::Reset => 0xFF,
        }
    }
}

fn make_joybus_packet(cmd: JoybusCommand) -> [u8; 64] {
    let mut packet = [0; 64];

    let mut write_ptr = 0;

    let mut write_byte = |b| {
        packet[write_ptr] = b;
        write_ptr += 1;
    };

    for _ in 0..4 {
        write_byte(0xFF);
        write_byte(cmd.cmd_size());
        write_byte(cmd.response_size());
        write_byte(cmd.cmd());

        write_byte(0xFF);
        write_byte(0xFF);
        write_byte(0xFF);
        write_byte(0xFF);
    }

    packet[32] = 0xFE;
    packet[63] = 0x01;

    packet
}

#[derive(Clone, Copy)]
pub enum ControllerStatus {
    StandardController(u8),
    UnknownDevice(u16, u8),
    None,
    Error,
}

impl ControllerStatus {
    fn parse_response(data: &[u8]) -> Self {
        assert_eq!(data.len(), 8);

        match data[2] & 0xC0 {
            0x80 => return Self::None,
            0x40 => return Self::Error,
            _ => {}
        }

        let device = u16::from_be_bytes([data[4], data[5]]);
        match device {
            0x0500 => Self::StandardController(data[6]),
            0x0000 => Self::None,
            _ => Self::UnknownDevice(device, data[6]),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ControllerData([u8; 4]);

impl ControllerData {
    pub const A: u16 = 0x8000;
    pub const B: u16 = 0x4000;
    pub const Z: u16 = 0x2000;
    pub const START: u16 = 0x1000;
    pub const D_UP: u16 = 0x0800;
    pub const D_DOWN: u16 = 0x0400;
    pub const D_LEFT: u16 = 0x0200;
    pub const D_RIGHT: u16 = 0x0100;
    pub const RESET: u16 = 0x0080;

    pub const L: u16 = 0x0020;
    pub const R: u16 = 0x0010;
    pub const C_UP: u16 = 0x0008;
    pub const C_DOWN: u16 = 0x0004;
    pub const C_LEFT: u16 = 0x0002;
    pub const C_RIGHT: u16 = 0x0001;

    fn parse_response(data: &[u8]) -> Result<Option<Self>, ()> {
        assert_eq!(data.len(), 8);

        match data[2] & 0xC0 {
            0x80 => return Ok(None),
            0x40 | 0xC0 => return Err(()),
            _ => {}
        }

        Ok(Some(Self([data[4], data[5], data[6], data[7]])))
    }

    pub fn x(&self) -> i8 {
        self.0[2] as _
    }

    pub fn y(&self) -> i8 {
        self.0[3] as _
    }

    pub fn sticks(&self) -> (i8, i8) {
        (self.x(), self.y())
    }

    pub fn buttons(&self) -> u16 {
        u16::from_be_bytes([self.0[0], self.0[1]])
    }

    pub fn a(&self) -> bool {
        self.buttons() & Self::A != 0
    }

    pub fn b(&self) -> bool {
        self.buttons() & Self::B != 0
    }

    pub fn z(&self) -> bool {
        self.buttons() & Self::Z != 0
    }

    pub fn start(&self) -> bool {
        self.buttons() & Self::START != 0
    }

    pub fn d_up(&self) -> bool {
        self.buttons() & Self::D_UP != 0
    }

    pub fn d_down(&self) -> bool {
        self.buttons() & Self::D_DOWN != 0
    }

    pub fn d_left(&self) -> bool {
        self.buttons() & Self::D_LEFT != 0
    }

    pub fn d_right(&self) -> bool {
        self.buttons() & Self::D_RIGHT != 0
    }

    pub fn reset(&self) -> bool {
        self.buttons() & Self::RESET != 0
    }

    pub fn l(&self) -> bool {
        self.buttons() & Self::L != 0
    }

    pub fn r(&self) -> bool {
        self.buttons() & Self::R != 0
    }

    pub fn c_up(&self) -> bool {
        self.buttons() & Self::C_UP != 0
    }

    pub fn c_down(&self) -> bool {
        self.buttons() & Self::C_DOWN != 0
    }

    pub fn c_left(&self) -> bool {
        self.buttons() & Self::C_LEFT != 0
    }

    pub fn c_right(&self) -> bool {
        self.buttons() & Self::C_RIGHT != 0
    }
}

impl Si {
    pub fn query_controllers(&mut self) -> [ControllerStatus; 4] {
        let packet = make_joybus_packet(JoybusCommand::Info);

        self.write(&packet);

        let response = self.read();

        [
            ControllerStatus::parse_response(&response[0..8]),
            ControllerStatus::parse_response(&response[8..16]),
            ControllerStatus::parse_response(&response[16..24]),
            ControllerStatus::parse_response(&response[24..32]),
        ]
    }

    pub fn read_controllers(&mut self) -> [Result<Option<ControllerData>, ()>; 4] {
        let packet = make_joybus_packet(JoybusCommand::ReadState);
        self.write(&packet);

        let response = self.read();

        [
            ControllerData::parse_response(&response[0..8]),
            ControllerData::parse_response(&response[8..16]),
            ControllerData::parse_response(&response[16..24]),
            ControllerData::parse_response(&response[24..32]),
        ]
    }
}
