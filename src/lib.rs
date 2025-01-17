use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::io::{stdout, Write};
use std::mem::size_of;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum Sk8brdMsgs {
    MsgSelectBoard = 1,
    MsgConsole,
    MsgHardReset,
    MsgPowerOn,
    MsgPowerOff,
    MsgFastbootPresent,
    MsgFastbootDownload,
    MsgFastbootBoot,
    MsgStatusUpdate,
    MsgVbusOn,
    MsgVbusOff,
    MsgFastbootReboot,
    MsgSendBreak,
    MsgListDevices,
    MsgBoardInfo,
    MsgFastbootContinue,
}

impl TryFrom<u8> for Sk8brdMsgs {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Sk8brdMsgs::MsgSelectBoard),
            2 => Ok(Sk8brdMsgs::MsgConsole),
            3 => Ok(Sk8brdMsgs::MsgHardReset),
            4 => Ok(Sk8brdMsgs::MsgPowerOn),
            5 => Ok(Sk8brdMsgs::MsgPowerOff),
            6 => Ok(Sk8brdMsgs::MsgFastbootPresent),
            7 => Ok(Sk8brdMsgs::MsgFastbootDownload),
            8 => Ok(Sk8brdMsgs::MsgFastbootBoot),
            9 => Ok(Sk8brdMsgs::MsgStatusUpdate),
            10 => Ok(Sk8brdMsgs::MsgVbusOn),
            11 => Ok(Sk8brdMsgs::MsgVbusOff),
            12 => Ok(Sk8brdMsgs::MsgFastbootReboot),
            13 => Ok(Sk8brdMsgs::MsgSendBreak),
            14 => Ok(Sk8brdMsgs::MsgListDevices),
            15 => Ok(Sk8brdMsgs::MsgBoardInfo),
            16 => Ok(Sk8brdMsgs::MsgFastbootContinue),
            _ => Err(format!("Unknown msg package {value}")),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[repr(C)]
#[repr(packed(1))]
pub struct Sk8brdMsg {
    pub r#type: u8,
    pub len: u16,
}
pub const MSG_HDR_SIZE: usize = size_of::<Sk8brdMsg>();

pub fn send_msg(write_sink: &mut impl Write, r#type: Sk8brdMsgs, size: usize, buf: &[u8]) {
    let msg: Sk8brdMsg = Sk8brdMsg {
        r#type: r#type as u8,
        len: size as u16,
    };

    write_sink
        .write_all(&bincode::serialize(&msg).unwrap())
        .unwrap();
    let _ = write_sink.write(&buf[0..size]).unwrap();
}

pub fn send_ack(write_sink: &mut impl Write, r#type: Sk8brdMsgs) {
    send_msg(write_sink, r#type, 0, &[0])
}

pub fn parse_recv_msg(buf: &[u8]) -> Sk8brdMsg {
    let msg: Sk8brdMsg = Sk8brdMsg {
        r#type: buf[0],
        len: (buf[2] as u16) << 8 | buf[1] as u16,
    };

    // println!("{:?}", msg);

    msg
}

pub fn console_print(buf: &[u8], len: u16) {
    print!("{}", String::from_utf8_lossy(&buf[..len as usize]));
    stdout().flush().unwrap();
}

#[allow(clippy::explicit_write)]
pub fn send_image(write_sink: &mut impl Write, buf: &[u8]) {
    let mut last_percent_done: usize = 0;
    let mut bytes_sent = 0;

    while bytes_sent < buf.len() {
        let bytes_left = min(2048, buf.len() - bytes_sent);
        let percent_done = 100 * bytes_sent / buf.len();

        if percent_done != last_percent_done {
            writeln!(stdout(), " Sending image: {}%\r", percent_done).unwrap();
        }

        send_msg(
            write_sink,
            Sk8brdMsgs::MsgFastbootDownload,
            bytes_left,
            &buf[bytes_sent..],
        );

        bytes_sent += bytes_left;
        last_percent_done = percent_done;
    }

    send_ack(write_sink, Sk8brdMsgs::MsgFastbootDownload)
}

pub fn select_brd(write_sink: &mut impl Write, name: &str) {
    send_msg(
        write_sink,
        Sk8brdMsgs::MsgSelectBoard,
        name.len(),
        name.as_bytes(),
    )
}

#[allow(clippy::explicit_write)]
pub fn list_device(buf: &[u8], len: u16) {
    if len == 0 {
        return;
    }

    writeln!(
        stdout(),
        "{}\r",
        String::from_utf8_lossy(&buf[..len as usize])
    )
    .unwrap();
    stdout().flush().unwrap();
}

pub fn list_boards() {}
