//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu heap
use {
    memmap,
    modular_bitfield::specifiers::{B11, B4},
    std::{
        fs::{remove_file, OpenOptions},
        io::{Seek, SeekFrom, Write},
    },
};

#[cfg(feature = "async")]
use {futures::executor::block_on, futures_locks::RwLock};

#[cfg(not(feature = "async"))]
use std::cell::RefCell;

// (type, total-size, alloc, in-use)
type AllocMap = (u8, usize, usize, usize);

pub struct Heap {
    pub mmap: Box<memmap::MmapMut>,

    #[cfg(feature = "async")]
    pub alloc_map: RwLock<Vec<AllocMap>>,
    #[cfg(not(feature = "async"))]
    pub alloc_map: RefCell<Vec<AllocMap>>,
    pub page_size: usize,
    pub npages: usize,
    pub size: usize,
    pub write_barrier: usize,
}

#[bitfield]
#[repr(align(8))]
pub struct Info {
    pub reloc: u32, // relocation
    #[skip]
    __: B11, // expansion
    pub mark: bool, // reference counting
    pub len: u16,   // in bytes
    pub tag_type: B4, // tag type
}

impl Heap {
    pub fn new(pages: usize) -> Self {
        let path = "/var/tmp/thorn.heap";

        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .expect("unable to open heap mmap");

        f.seek(SeekFrom::Start((pages * 4096) as u64)).unwrap();
        f.write_all(&[0]).unwrap();
        f.rewind().unwrap();

        remove_file(path).unwrap();

        let data = unsafe {
            memmap::MmapOptions::new()
                .map_mut(&f)
                .expect("Could not access data from memory mapped file")
        };

        let heap = Heap {
            mmap: Box::new(data),
            page_size: 4096,
            npages: pages,
            size: pages * 4096,
            #[cfg(feature = "async")]
            alloc_map: RwLock::new(Vec::new()),
            #[cfg(not(feature = "async"))]
            alloc_map: RefCell::new(Vec::new()),
            write_barrier: 0,
        };

        {
            #[cfg(feature = "async")]
            let mut alloc_ref = block_on(heap.alloc_map.write());
            #[cfg(not(feature = "async"))]
            let mut alloc_ref = heap.alloc_map.borrow_mut();

            for id in 0..256 {
                alloc_ref.push((id as u8, 0, 0, 0))
            }
        }

        heap
    }

    // allocation statistics
    pub fn alloc_id(&self, id: u8) -> (usize, usize, usize) {
        #[cfg(feature = "async")]
        let alloc_ref = block_on(self.alloc_map.read());
        #[cfg(not(feature = "async"))]
        let alloc_ref = self.alloc_map.borrow();

        let (_, total_size, alloc, in_use) = alloc_ref[id as usize];
        (total_size, alloc, in_use)
    }

    fn alloc_map(&self, id: u8, size: usize) {
        #[cfg(feature = "async")]
        let mut alloc_ref = block_on(self.alloc_map.write());
        #[cfg(not(feature = "async"))]
        let mut alloc_ref = self.alloc_map.borrow_mut();

        let (_, total_size, alloc, in_use) = alloc_ref[id as usize];
        alloc_ref[id as usize] = (id, total_size + size, alloc + 1, in_use + 1);
    }

    // rewrite object data
    pub fn write_image(&mut self, image: &[[u8; 8]], offset: usize) {
        let mut index = offset;

        for n in image {
            self.mmap[index..(index + 8)].copy_from_slice(n);
            index += 8;
        }
    }

    // allocate
    pub fn alloc(&mut self, src: &[[u8; 8]], id: u8) -> usize {
        let ntypes = src.len() as u64;
        let base = self.write_barrier;

        if base + (((ntypes + 1) * 8) as usize) > self.size {
            panic!();
        } else {
            let data = &mut self.mmap;
            let hinfo = Info::new()
                .with_reloc(0)
                .with_len(((ntypes + 1) * 8) as u16)
                .with_mark(false)
                .with_tag_type(id)
                .into_bytes();

            data[self.write_barrier..(self.write_barrier + 8)].copy_from_slice(&hinfo);
            self.write_barrier += 8;

            let image = self.write_barrier;
            for n in src {
                data[self.write_barrier..(self.write_barrier + 8)].copy_from_slice(n);
                self.write_barrier += 8;
            }

            self.alloc_map(id, src.len() * 8);

            image
        }
    }

    pub fn valloc(&mut self, src: &[[u8; 8]], vdata: &[u8], id: u8) -> usize {
        let ntypes = src.len() as u64;
        let base = self.write_barrier;
        let len_to_8: usize = vdata.len() + (8 - (vdata.len() & 7));

        if base + (((ntypes + 1) * 8) as usize) > self.size {
            panic!();
        } else {
            let data = &mut self.mmap;
            let hinfo = Info::new()
                .with_reloc(0)
                .with_len((((ntypes + 1) * 8) + (len_to_8 as u64)) as u16)
                .with_mark(false)
                .with_tag_type(id)
                .into_bytes();

            data[self.write_barrier..(self.write_barrier + 8)].copy_from_slice(&hinfo);
            self.write_barrier += 8;

            let image = self.write_barrier;
            for n in src {
                data[self.write_barrier..(self.write_barrier + 8)].copy_from_slice(n);
                self.write_barrier += 8;
            }

            data[self.write_barrier..(self.write_barrier + vdata.len())].copy_from_slice(vdata);
            self.write_barrier += len_to_8;

            self.alloc_map(id, src.len() * 8 + vdata.len());

            image
        }
    }

    // object header
    pub fn info(&self, offset: usize) -> Option<Info> {
        if offset == 0 || offset > self.write_barrier {
            None
        } else {
            let data = &self.mmap;
            let mut info = 0u64.to_le_bytes();

            info.copy_from_slice(&data[(offset - 8)..offset]);
            Some(Info::from_bytes(info))
        }
    }

    pub fn of_length(&self, offset: usize, len: usize) -> Option<&[u8]> {
        if offset == 0 || offset > self.write_barrier {
            None
        } else {
            let data = &self.mmap;
            Some(&data[offset..offset + len])
        }
    }
}
