use core::{
    fmt::Debug,
    ops::{Index, IndexMut},
    ptr::null,
};

use crate::aes::AES_128_BLOCK_SIZE;

pub type Id = u32;
pub type ContentId = u32;
pub type AesKey = [u8; AES_128_BLOCK_SIZE];
pub type AesIv = [u8; AES_128_BLOCK_SIZE];
pub type EccPublicKey = [u8; 64];
pub type EccPrivateKey = [u8; 32];
pub type RsaPublicKey2048 = [u8; 256];
pub type RsaPublicKey4096 = [u8; 512];
pub type RsaExponent = u32;
pub type RsaSig2048 = [u8; 256];
pub type RsaSig4096 = [u8; 512];
pub type EccSig = [u8; 64];
pub type OwnerId = u32;
pub type TicketId = u16;

pub type ShaHash = [u8; 20];

pub type Name = [u8; 64];
pub type ServerName = [u8; 64];
pub type ServerSuffix = [u8; 64];

pub type ContentDesc = [u8; 0x2800];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Virage2 {
    pub sk_hash: ShaHash,

    pub rom_patch: [u32; 16],

    pub pub_key: EccPublicKey,

    pub bbid: Id,

    pub priv_key: EccPrivateKey,

    pub boot_app_key: AesKey,
    pub recrypt_list_key: AesKey,
    pub app_state_key: AesKey,
    pub self_msg_key: AesKey,

    pub csum_adjust: u32,

    pub jtag_enable: u32,
}

impl Default for Virage2 {
    fn default() -> Self {
        Self {
            sk_hash: Default::default(),
            rom_patch: Default::default(),
            pub_key: [0; 64],
            bbid: Default::default(),
            priv_key: Default::default(),
            boot_app_key: Default::default(),
            recrypt_list_key: Default::default(),
            app_state_key: Default::default(),
            self_msg_key: Default::default(),
            csum_adjust: Default::default(),
            jtag_enable: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CertBase {
    pub cert_type: u32,
    pub sig_type: u32,
    pub date: u32,
    pub issuer: ServerName,
    pub name: Name,
}

impl Default for CertBase {
    fn default() -> Self {
        Self {
            cert_type: Default::default(),
            sig_type: Default::default(),
            date: Default::default(),
            issuer: [0; 64],
            name: [0; 64],
        }
    }
}

pub type CertId = CertBase;

#[repr(C)]
#[derive(Clone, Copy)]
pub union GenericSig {
    pub rsa2048: RsaSig2048,
    pub rsa4096: RsaSig4096,
    pub ecc: EccSig,
}

impl Debug for GenericSig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "GenericSig")
    }
}

impl Default for GenericSig {
    fn default() -> Self {
        Self { rsa4096: [0; 512] }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EccCert {
    pub cert_id: CertId,
    pub public_key: EccPublicKey,
    pub signature: GenericSig,
}

impl Default for EccCert {
    fn default() -> Self {
        Self {
            cert_id: Default::default(),
            public_key: [0; 64],
            signature: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RsaCert {
    pub cert_id: CertId,
    pub public_key: RsaPublicKey2048,
    pub exponent: RsaExponent,
    pub signature: GenericSig,
}

impl Default for RsaCert {
    fn default() -> Self {
        Self {
            cert_id: Default::default(),
            public_key: [0; 256],
            exponent: Default::default(),
            signature: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ContentMetaDataHead {
    pub unused_padding: u32,
    pub ca_crl_version: u32,
    pub cp_crl_version: u32,
    pub size: u32,
    pub desc_flags: u32,
    pub common_cmd_iv: AesIv,
    pub hash: ShaHash,
    pub iv: AesIv,
    pub exec_flags: u32,
    pub hw_access_rights: u32,
    pub secure_kernel_rights: u32,
    pub bbid: u32,
    pub issuer: ServerName,
    pub id: ContentId,
    pub key: AesKey,
    pub content_meta_data_sign: RsaSig2048,
}

impl Default for ContentMetaDataHead {
    fn default() -> Self {
        Self {
            unused_padding: Default::default(),
            ca_crl_version: Default::default(),
            cp_crl_version: Default::default(),
            size: Default::default(),
            desc_flags: Default::default(),
            common_cmd_iv: Default::default(),
            hash: Default::default(),
            iv: Default::default(),
            exec_flags: Default::default(),
            hw_access_rights: Default::default(),
            secure_kernel_rights: Default::default(),
            bbid: Default::default(),
            issuer: [0; 64],
            id: Default::default(),
            key: Default::default(),
            content_meta_data_sign: [0; 256],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ContentMetaData {
    pub content_desc: ContentDesc,
    pub head: ContentMetaDataHead,
}

impl Default for ContentMetaData {
    fn default() -> Self {
        Self {
            content_desc: [0; 0x2800],
            head: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TicketHead {
    pub bbid: Id,
    pub tid: TicketId,
    pub code: u16,
    pub limit: u16,
    pub reserved: u16,
    pub ts_crl_version: u32,
    pub cmd_iv: AesIv,
    pub server_key: EccPublicKey,
    pub issuer: ServerName,
    pub ticket_sign: RsaSig2048,
}

impl Default for TicketHead {
    fn default() -> Self {
        Self {
            bbid: Default::default(),
            tid: Default::default(),
            code: Default::default(),
            limit: Default::default(),
            reserved: Default::default(),
            ts_crl_version: Default::default(),
            cmd_iv: Default::default(),
            server_key: [0; 64],
            issuer: [0; 64],
            ticket_sign: [0; 256],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Ticket {
    pub cmd: ContentMetaData,
    pub head: TicketHead,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TicketBundle<'a> {
    pub ticket: Option<&'a Ticket>,
    pub ticket_chain: [Option<&'a CertBase>; 5],
    pub cmd_chain: [Option<&'a CertBase>; 5],
}

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy)]
pub enum CrlUnusedEnumType {
    #[default]
    Unused0,
    Unused1,
    Unused2,
}

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy)]
pub enum CrlNum {
    #[default]
    Ts,
    Ca,
    Cp,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CrlHead {
    pub signature: GenericSig,
    pub crl_type: u32,
    pub sig_type: u32,
    pub unused_padding: CrlUnusedEnumType,
    pub version_number: u32,
    pub date: u32,
    pub issuer: ServerName,
    pub number_revoked: u32,
}

impl Default for CrlHead {
    fn default() -> Self {
        Self {
            signature: Default::default(),
            crl_type: Default::default(),
            sig_type: Default::default(),
            unused_padding: Default::default(),
            version_number: Default::default(),
            date: Default::default(),
            issuer: [0; 64],
            number_revoked: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CrlBundle<'a> {
    pub head: Option<&'a CrlHead>,
    pub list: *const ServerSuffix,
    pub cert_chain: [Option<&'a CertBase>; 5],
}

impl<'a> Default for CrlBundle<'a> {
    fn default() -> Self {
        Self {
            head: Default::default(),
            list: null(),
            cert_chain: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct AppLaunchCrls<'a> {
    pub tsrl: CrlBundle<'a>,
    pub carl: CrlBundle<'a>,
    pub cprl: CrlBundle<'a>,
}

#[repr(u32)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum RecryptState {
    #[default]
    Success,
    NotNeeded,
    Finished,
    Unfinished,
    New,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct RecryptListEntry {
    pub content_id: ContentId,
    pub content_key: AesKey,
    pub state: RecryptState,
    pub padding: [u8; 8], // must be aligned to 16 bytes for encryption
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RecryptList {
    pub signature: EccSig,
    pub num_entries: u32,
    pub entries: [RecryptListEntry; 0],
}

impl RecryptList {
    pub fn get(&self, index: u32) -> Option<&RecryptListEntry> {
        if index < self.num_entries {
            unsafe { self.entries.as_ptr().add(index as usize).as_ref() }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut RecryptListEntry> {
        if index < self.num_entries {
            unsafe { self.entries.as_mut_ptr().add(index as usize).as_mut() }
        } else {
            None
        }
    }
}

impl Index<u32> for RecryptList {
    type Output = RecryptListEntry;

    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<u32> for RecryptList {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

#[repr(align(8))]
pub struct Align8<T: ?Sized>(pub T);

#[repr(align(64))]
pub struct Align64<T: ?Sized>(pub T);
