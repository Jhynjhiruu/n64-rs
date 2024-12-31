use crate::{mi::mi, pi::Pi};

pub const BYTES_PER_PAGE: u32 = 512;
pub const PAGES_PER_BLOCK: u32 = 32;
pub const BYTES_PER_BLOCK: usize = (BYTES_PER_PAGE * PAGES_PER_BLOCK) as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardStatus {
    Ok,
    NotPresent,
    DoubleBitError,
}

#[macro_export]
macro_rules! page_to_addr {
    ($e:expr) => {
        ($e as u32) * $crate::card::BYTES_PER_PAGE
    };
}

#[macro_export]
macro_rules! block_to_page {
    ($e:expr) => {
        ($e as u32) * $crate::card::PAGES_PER_BLOCK
    };
}

impl Pi {
    pub fn read_page(&mut self, page: u32) -> CardStatus {
        self.set_bb_nand_addr(page_to_addr!(page));

        self.set_bb_nand_ctrl(0x9F008A10);

        loop {
            if mi().bb_interrupt() & (1 << 25) != 0 {
                self.set_bb_nand_ctrl(0);
                return CardStatus::NotPresent;
            }

            if self.bb_nand_ctrl() & (1 << 31) == 0 {
                break;
            }
        }

        if self.bb_nand_ctrl() & (1 << 10) != 0 {
            CardStatus::DoubleBitError
        } else {
            CardStatus::Ok
        }
    }
}
