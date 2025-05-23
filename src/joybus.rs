use core::num::Wrapping;

use crate::boot::is_bbplayer;
use crate::pi::{pi, LedValue};
use crate::si::Si;
use crate::types::Align8;

#[derive(Clone, Copy)]
enum JoybusCommand {
    Info,
    ReadState,
    Reset,
    TxRx(u8, u8),
}

impl JoybusCommand {
    fn cmd_size(self) -> u8 {
        match self {
            Self::Info => 1,
            Self::ReadState => 1,
            Self::Reset => 1,
            Self::TxRx(_, _) => 3,
        }
    }

    fn response_size(self) -> u8 {
        match self {
            Self::Info => 3,
            Self::ReadState => 4,
            Self::Reset => 3,
            Self::TxRx(_, _) => 2,
        }
    }

    fn cmd(self) -> u8 {
        match self {
            Self::Info => 0x00,
            Self::ReadState => 0x01,
            Self::Reset => 0xFF,
            Self::TxRx(_, _) => 0x55,
        }
    }

    fn args(self) -> [u8; 4] {
        match self {
            Self::Info | Self::ReadState | Self::Reset => [0xFF, 0xFF, 0xFF, 0xFF],
            Self::TxRx(byte, index) => [byte, index, 0xFF, 0xFF],
        }
    }
}

fn make_joybus_packet(cmds: [JoybusCommand; 4]) -> [u8; 64] {
    let mut packet = [0; 64];

    let mut write_ptr = 0;

    let mut write_byte = |b| {
        packet[write_ptr] = b;
        write_ptr += 1;
    };

    for cmd in cmds {
        write_byte(0xFF);
        write_byte(cmd.cmd_size());
        write_byte(cmd.response_size());
        write_byte(cmd.cmd());

        for b in cmd.args() {
            write_byte(b);
        }
    }

    if !is_bbplayer() {
        packet[32] = 0xFE;
        packet[63] = 0x01;
    }

    packet
}

fn make_joybus_packet_mult(cmd: JoybusCommand) -> [u8; 64] {
    make_joybus_packet([cmd, cmd, cmd, cmd])
}

#[derive(Clone, Copy)]
pub enum ControllerStatus {
    StandardController(u8),
    UnknownDevice(u16, u8),
    DebugProbe,
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
            0xBB64 => Self::DebugProbe,
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
    #[cfg(not(feature = "sk"))]
    pub fn query_controllers(&mut self) -> [ControllerStatus; 4] {
        let packet = Align8(make_joybus_packet_mult(JoybusCommand::Info));

        self.write(&packet);

        let response = self.read();

        [
            ControllerStatus::parse_response(&response[0..8]),
            ControllerStatus::parse_response(&response[8..16]),
            ControllerStatus::parse_response(&response[16..24]),
            ControllerStatus::parse_response(&response[24..32]),
        ]
    }

    /*#[cfg(feature = "sk")]
    pub fn query_controllers(&mut self) -> [ControllerStatus; 4] {
        unsafe { DRAM_BUF }.copy_from_slice(&make_joybus_packet_mult(JoybusCommand::Info));

        self.write();

        self.read();

        [
            ControllerStatus::parse_response(&unsafe { DRAM_BUF }[0..8]),
            ControllerStatus::parse_response(&unsafe { DRAM_BUF }[8..16]),
            ControllerStatus::parse_response(&unsafe { DRAM_BUF }[16..24]),
            ControllerStatus::parse_response(&unsafe { DRAM_BUF }[24..32]),
        ]
    }*/

    #[cfg(not(feature = "sk"))]
    pub fn read_controllers(&mut self) -> [Result<Option<ControllerData>, ()>; 4] {
        let packet = Align8(make_joybus_packet_mult(JoybusCommand::ReadState));

        self.write(&packet);

        let response = self.read();

        [
            ControllerData::parse_response(&response[0..8]),
            ControllerData::parse_response(&response[8..16]),
            ControllerData::parse_response(&response[16..24]),
            ControllerData::parse_response(&response[24..32]),
        ]
    }

    /*#[cfg(feature = "sk")]
    pub fn read_controllers(&mut self) -> [Result<Option<ControllerData>, ()>; 4] {
        unsafe { DRAM_BUF }.copy_from_slice(&make_joybus_packet_mult(JoybusCommand::ReadState));

        self.write();

        self.read();

        [
            ControllerData::parse_response(&unsafe { DRAM_BUF }[0..8]),
            ControllerData::parse_response(&unsafe { DRAM_BUF }[8..16]),
            ControllerData::parse_response(&unsafe { DRAM_BUF }[16..24]),
            ControllerData::parse_response(&unsafe { DRAM_BUF }[24..32]),
        ]
    }*/

    #[cfg(not(feature = "sk"))]
    pub fn txrx(&mut self, data: &[u8], mut out_buf: Option<&mut [u8]>) -> usize {
        use crate::cop0::cop0;

        let cop0 = cop0();

        cop0.disable_interrupts();

        let (data_offset, count_offset) = if is_bbplayer() { (20, 21) } else { (22, 23) };

        let mut out_index = 0;

        for byte in data {
            self.tx_index += 1;
            let packet = Align8(make_joybus_packet([
                JoybusCommand::Info,
                JoybusCommand::Info,
                JoybusCommand::TxRx(*byte, self.tx_index.0),
                JoybusCommand::Info,
            ]));

            self.write(&packet);

            let resp = self.read();

            if let Some(ref mut out) = out_buf {
                if out_index < out.len() && resp[count_offset] != self.rx_index.0 {
                    self.rx_index = Wrapping(resp[count_offset]);
                    out[out_index] = resp[data_offset];
                    out_index += 1;
                }
            }
        }

        if let Some(ref mut out) = out_buf {
            while out_index < out.len() {
                let packet = Align8(make_joybus_packet([
                    JoybusCommand::Info,
                    JoybusCommand::Info,
                    JoybusCommand::TxRx(0, self.tx_index.0),
                    JoybusCommand::Info,
                ]));

                self.write(&packet);

                let resp = self.read();

                if resp[23] == self.rx_index.0 {
                    break;
                }

                self.rx_index = Wrapping(resp[count_offset]);
                out[out_index] = resp[data_offset];
                out_index += 1;
            }
        }

        cop0.enable_interrupts();

        out_index
    }

    /*#[cfg(feature = "sk")]
    pub fn txrx(&mut self, data: &[u8], mut out_buf: Option<&mut [u8]>) -> usize {
        let mut out_index = 0;

        for byte in data {
            self.tx_index += 1;
            unsafe { DRAM_BUF }.copy_from_slice(&make_joybus_packet([
                JoybusCommand::Info,
                JoybusCommand::Info,
                JoybusCommand::TxRx(*byte, self.tx_index.0),
                JoybusCommand::Info,
            ]));

            self.write();

            self.read();

            if let Some(ref mut out) = out_buf {
                if out_index < out.len() && unsafe { DRAM_BUF }[23] != self.rx_index.0 {
                    self.rx_index = Wrapping(unsafe { DRAM_BUF }[23]);
                    out[out_index] = unsafe { DRAM_BUF }[22];
                    out_index += 1;
                }
            }
        }

        if let Some(ref mut out) = out_buf {
            while out_index < out.len() {
                unsafe { DRAM_BUF }.copy_from_slice(&make_joybus_packet([
                    JoybusCommand::Info,
                    JoybusCommand::Info,
                    JoybusCommand::TxRx(0, self.tx_index.0),
                    JoybusCommand::Info,
                ]));

                self.write();

                self.read();

                if unsafe { DRAM_BUF }[23] == self.rx_index.0 {
                    break;
                }

                self.rx_index = Wrapping(unsafe { DRAM_BUF }[23]);
                out[out_index] = unsafe { DRAM_BUF }[22];
                out_index += 1;
            }
        }

        out_index
    }*/
}
