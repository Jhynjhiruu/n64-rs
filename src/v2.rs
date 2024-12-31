use volcell::VolatileCell;

use crate::types::*;

extern "C" {
    pub static mut virage2: VolatileCell<Virage2>;
}
