use core::{array, mem::size_of, slice};

use crate::{aes, types::*, v2::virage2};

impl RecryptList {
    pub fn sign(&mut self) {
        // todo
    }

    pub fn decrypt_entry(&self, index: u32) -> RecryptListEntry {
        let iv = array::from_fn::<_, 4, _>(|i| unsafe { virage2.read() }.bbid + i as u32)
            .map(|e| e.to_be_bytes())
            .flatten()
            .try_into()
            .unwrap();

        let mut entry = RecryptListEntry::default();

        aes::decrypt(
            unsafe {
                slice::from_raw_parts(
                    (&self[index] as *const RecryptListEntry).cast::<u8>(),
                    size_of::<RecryptListEntry>(),
                )
            },
            unsafe {
                slice::from_raw_parts_mut(
                    (&mut entry as *mut RecryptListEntry).cast::<u8>(),
                    size_of::<RecryptListEntry>(),
                )
            },
            unsafe { virage2.read() }.recrypt_list_key,
            iv,
        );

        entry
    }

    pub fn add_entry(&mut self, entry: &RecryptListEntry, index: u32) {
        let iv = array::from_fn::<_, 4, _>(|i| unsafe { virage2.read() }.bbid + i as u32)
            .map(|e| e.to_be_bytes())
            .flatten()
            .try_into()
            .unwrap();

        aes::encrypt(
            unsafe {
                slice::from_raw_parts(
                    (entry as *const RecryptListEntry).cast::<u8>(),
                    size_of::<RecryptListEntry>(),
                )
            },
            unsafe {
                slice::from_raw_parts_mut(
                    (&mut self[index] as *mut RecryptListEntry).cast::<u8>(),
                    size_of::<RecryptListEntry>(),
                )
            },
            unsafe { virage2.read() }.recrypt_list_key,
            iv,
        );
    }

    pub fn get_entry_for_cid(&self, cid: ContentId) -> Option<(u32, RecryptListEntry)> {
        for i in 0..self.num_entries {
            let entry = self.decrypt_entry(i);
            if entry.content_id == cid {
                return Some((i, entry));
            }
        }

        None
    }

    pub fn get_key_for_cid(&mut self, cid: ContentId) -> (AesKey, RecryptState) {
        match self.get_entry_for_cid(cid) {
            Some((_, entry)) => (entry.content_key, entry.state),
            None => {
                let new_entry = RecryptListEntry {
                    content_id: cid,
                    content_key: AesKey::default(), // generating an actual key sounds way too hard
                    state: RecryptState::Unfinished,
                    padding: Default::default(),
                };

                self.num_entries += 1;
                self.add_entry(&new_entry, self.num_entries - 1);

                self.sign();

                (new_entry.content_key, RecryptState::New)
            }
        }
    }
}
