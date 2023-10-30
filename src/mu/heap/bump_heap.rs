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

use {futures::executor::block_on, futures_locks::RwLock};

// (type, total-size, alloc, in-use)
type AllocMap = (u8, usize, usize, usize);

#[derive(Debug)]
pub struct BumpHeap {
    pub mmap: Box<memmap::MmapMut>,
    pub alloc_map: RwLock<Vec<AllocMap>>,
    pub page_size: usize,
    pub free: Vec<Vec<usize>>,
    pub npages: usize,
    pub size: usize,
    pub write_barrier: usize,
}

#[bitfield]
#[repr(align(8))]
#[derive(Debug, Copy, Clone)]
pub struct Info {
    pub reloc: u32, // relocation
    #[skip]
    __: B11, // expansion
    pub mark: bool, // reference counting
    pub len: u16,   // in bytes
    pub image_type: B4, // image type
}

impl BumpHeap {
    pub fn iter(&self) -> BumpHeapIterator {
        BumpHeapIterator {
            heap: self,
            offset: 8,
        }
    }

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

        let mut heap = BumpHeap {
            mmap: Box::new(data),
            page_size: 4096,
            npages: pages,
            free: Vec::<Vec<usize>>::new(),
            size: pages * 4096,
            alloc_map: RwLock::new(Vec::new()),
            write_barrier: 0,
        };

        for _i in 0..16 {
            heap.free.push(Vec::<usize>::new())
        }

        {
            let mut alloc_ref = block_on(heap.alloc_map.write());

            for id in 0..256 {
                alloc_ref.push((id as u8, 0, 0, 0))
            }
        }

        heap
    }

    // allocation statistics
    pub fn alloc_id(&self, id: u8) -> (usize, usize, usize) {
        let alloc_ref = block_on(self.alloc_map.read());

        let (_, total_size, alloc, in_use) = alloc_ref[id as usize];
        (total_size, alloc, in_use)
    }

    fn alloc_map(&self, id: u8, size: usize) {
        let mut alloc_ref = block_on(self.alloc_map.write());

        let (_, total_size, alloc, in_use) = alloc_ref[id as usize];
        alloc_ref[id as usize] = (id, total_size + size, alloc + 1, in_use + 1);
    }

    // allocate
    pub fn alloc(&mut self, src: &[[u8; 8]], id: u8) -> usize {
        let ntypes = src.len() as u64;
        let base = self.write_barrier;

        if base + (((ntypes + 1) * 8) as usize) > self.size {
            panic!("heap exhausted")
        }

        if let Some(image) = self.alloc_free(id) {
            let data = &mut self.mmap;
            let mut off = image;

            for n in src {
                data[off..(off + 8)].copy_from_slice(n);
                off += 8;
            }

            image
        } else {
            let data = &mut self.mmap;
            let hinfo = Info::new()
                .with_reloc(0)
                .with_len(((ntypes + 1) * 8) as u16)
                .with_mark(false)
                .with_image_type(id)
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
                .with_image_type(id)
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

    // rewrite info header
    pub fn write_info(&mut self, info: Info, off: usize) {
        self.mmap[(off - 8)..off].copy_from_slice(&(info.into_bytes()))
    }

    // rewrite image data
    pub fn write_image(&mut self, image: &[[u8; 8]], offset: usize) {
        let mut index = offset;

        for n in image {
            self.mmap[index..(index + 8)].copy_from_slice(n);
            index += 8;
        }
    }

    // gc
    pub fn clear_refbits(&mut self) {
        let mut off: usize = 8;

        while let Some(mut info) = self.image_info(off) {
            info.set_mark(false);
            self.write_info(info, off);
            off += info.len() as usize
        }
    }

    pub fn set_image_refbit(&mut self, off: usize) {
        match self.image_info(off) {
            Some(mut info) => {
                info.set_mark(true);
                self.write_info(info, off)
            }
            None => panic!(),
        }
    }

    pub fn get_image_refbit(&self, off: usize) -> Option<bool> {
        self.image_info(off).map(|info| info.mark())
    }

    pub fn sweep(&mut self) {
        let mut off: usize = 8;

        while let Some(info) = self.image_info(off) {
            if !info.mark() {
                let uvec = &mut self.free[info.image_type() as usize];
                match uvec.clone().into_iter().find(|&toff| toff == off) {
                    Some(_) => (),
                    None => uvec.push(off),
                }
            }
            off += info.len() as usize
        }
    }

    pub fn gc_stats(&self) -> Vec<(u8, usize)> {
        let mut nfree = Vec::<(u8, usize)>::new();

        for (index, unmarked) in self.free.iter().enumerate() {
            if !unmarked.is_empty() {
                nfree.push((index as u8, unmarked.len()))
            }
        }

        nfree
    }

    pub fn alloc_free(&mut self, id: u8) -> Option<usize> {
        self.free[id as usize].pop()
    }

    // image header info from heap tag
    pub fn image_info(&self, off: usize) -> Option<Info> {
        if off == 0 || off > self.write_barrier {
            None
        } else {
            let data = &self.mmap;
            let mut info = 0u64.to_le_bytes();

            info.copy_from_slice(&data[(off - 8)..off]);
            Some(Info::from_bytes(info))
        }
    }

    pub fn of_length(&self, off: usize, len: usize) -> Option<&[u8]> {
        if off == 0 || off > self.write_barrier {
            None
        } else {
            let data = &self.mmap;
            Some(&data[off..off + len])
        }
    }

    pub fn image_reloc(&self, off: usize) -> Option<u32> {
        self.image_info(off).map(|info| info.reloc())
    }

    pub fn image_length(&self, off: usize) -> Option<usize> {
        self.image_info(off).map(|info| info.len() as usize)
    }

    pub fn image_refbit(&self, off: usize) -> Option<bool> {
        self.image_info(off).map(|info| info.mark())
    }

    pub fn image_tag_type(&self, off: usize) -> Option<u8> {
        self.image_info(off).map(|info| info.image_type())
    }
}

// iterators
pub struct BumpHeapIterator<'a> {
    pub heap: &'a BumpHeap,
    pub offset: usize,
}

impl<'a> BumpHeapIterator<'a> {
    pub fn new(heap: &'a BumpHeap) -> Self {
        Self { heap, offset: 8 }
    }
}

impl<'a> Iterator for BumpHeapIterator<'a> {
    type Item = (Info, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.heap.image_info(self.offset) {
            Some(info) => {
                let id = self.offset;
                self.offset += info.len() as usize;
                Some((info, id))
            }
            None => None,
        }
    }
}
