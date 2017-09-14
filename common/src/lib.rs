extern crate openssl;
extern crate rmp_serde as rmps;
extern crate serde;
#[macro_use] extern crate serde_derive;

pub mod encrypter;
pub use encrypter::*;

use openssl::rsa::Rsa;
use std::io;

pub const DEFAULT_PORT: u16 = 8439;
pub const RSA_KEY_BIT_LEN: u32 = 3072;

pub const ERR_LOGIN_INVALID: u8 = 0;
pub const ERR_LOGIN_BANNED:  u8 = 1;

// TYPES
#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub id: usize,
    pub name: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: usize,
    pub bot: bool,
    pub name: String,
    pub nick: Option<String>,
    pub attributes: Vec<usize>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    pub id: usize,
    pub condition: Option<usize>
}

// CLIENT PACKETS
#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub name: String,
    pub password: Option<String>,
    pub token: Option<String>,
    pub public_key: Vec<u8>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelList {}
#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelCreate {
    pub channel: Channel
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelUpdate {
    pub channel: Channel
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelDelete {
    pub channel: usize
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageList {
    pub around: Option<usize>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageCreate {
    pub channel: usize,
    pub text: Vec<u8>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageUpdate {
    pub id: usize,
    pub channel: usize,
    pub text: Vec<u8>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageDelete {
    pub id: usize
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub author: usize,
    pub recipient: usize,
    pub command: String,
    pub args: Vec<String>
}

// SERVER PACKETS
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginSuccess {
    pub created: bool,
    pub token: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CommandReceive {
    pub author: User,
    pub recipient: User,
    pub command: String,
    pub args: Vec<String>
}

macro_rules! packet {
    ($($type:ident),+) => {
        #[derive(Serialize, Deserialize, Debug)]
        #[serde(/*tag = "type",*/ rename_all = "snake_case")]
        pub enum Packet {
            Err(u8),
            $($type($type),)+
        }
    }
}
packet! {
    Login,
    ChannelList,
    ChannelCreate,
    ChannelUpdate,
    ChannelDelete,
    MessageList,
    MessageCreate,
    MessageUpdate,
    MessageDelete,
    Command,

    LoginSuccess,
    CommandReceive
}

pub fn serialize(packet: &Packet) -> Result<Vec<u8>, rmps::encode::Error> {
    rmps::to_vec(&packet)
}
pub fn deserialize<'a>(buf: &'a [u8]) -> Result<Packet, rmps::decode::Error> {
    rmps::from_slice(buf)
}
pub fn deserialize_stream<T: io::Read>(buf: T) -> Result<Packet, rmps::decode::Error> {
    rmps::from_read(buf)
}
pub fn read<T: io::Read>(reader: &mut T, rsa: &Rsa) -> Result<Packet, Box<std::error::Error>> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;

    let (size_rsa, size_aes) = decode_size(&buf);
    let (size_rsa, size_aes) = (size_rsa as usize, size_aes as usize);

    let mut buf = vec![0; size_rsa+size_aes];
    reader.read_exact(&mut buf)?;
    decrypt(&buf, rsa, size_rsa)
}
pub fn write<T: io::Write>(writer: &mut T, rsa: &Rsa, packet: &Packet) -> Result<(), Box<std::error::Error>> {
    let encrypted = encrypt(packet, rsa)?;
    writer.write_all(&encrypted)?;
    writer.flush()?;

    Ok(())
}
