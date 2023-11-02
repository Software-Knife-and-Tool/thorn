//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu heap
use {
    crate::core::heap::{AllocImageInfo, AllocTypeInfo /* HeapAllocator */},
    memmap,
    std::{
        fs::{remove_file, OpenOptions},
        io::{Seek, SeekFrom, Write},
    },
};

use {futures::executor::block_on, futures_locks::RwLock};

#[derive(Debug)]
pub struct BumpAllocator {
    pub mmap: Box<memmap::MmapMut>,
    pub alloc_map: RwLock<Vec<RwLock<AllocTypeInfo>>>,
    pub free_map: Vec<Vec<usize>>,
    pub page_size: usize,
    pub npages: usize,
    pub size: usize,
    pub write_barrier: usize,
}

impl BumpAllocator {
    /*
        pub fn allocator(npages: usize) -> HeapAllocator<'static> {

            let bump_allocator = Self::new(npages);

            HeapAllocator {
                mmap: &bump_allocator.mmap,
                alloc: Self::alloc,
                valloc: Self::valloc,
                // begin_gc: fn(u8, u16) -> u32,

                /*
                info_iter: &'a dyn Iterator<Item = AllocImageInfo>,
                image_iter: &'a dyn Iterator<Item = AllocImageInfo>,
                 */

                freelist: bump_allocator.free_map,
                page_size: bump_allocator.page_size,
                npages,
            }
    }
        */

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

        let mut heap = BumpAllocator {
            mmap: Box::new(data),
            page_size: 4096,
            npages: pages,
            size: pages * 4096,
            alloc_map: RwLock::new(Vec::new()),
            free_map: Vec::new(),
            write_barrier: 0,
        };

        for _i in 0..16 {
            heap.free_map.push(Vec::<usize>::new())
        }

        {
            let mut alloc_ref = block_on(heap.alloc_map.write());

            for _ in 0..16 {
                alloc_ref.push(RwLock::new(AllocTypeInfo {
                    size: 0,
                    total: 0,
                    free: 0,
                }))
            }
        }

        heap
    }

    pub fn iter(&self) -> BumpAllocatorIterator {
        BumpAllocatorIterator {
            heap: self,
            offset: 8,
        }
    }

    // allocation metrics
    fn alloc_id(&self, id: u8) -> AllocTypeInfo {
        let alloc_ref = block_on(self.alloc_map.read());
        let alloc_type = block_on(alloc_ref[id as usize].read());

        *alloc_type
    }

    fn alloc_map(&self, id: u8, size: usize) {
        let alloc_ref = block_on(self.alloc_map.write());
        let mut alloc_type = block_on(alloc_ref[id as usize].write());

        alloc_type.size += size;
        alloc_type.total += 1;
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
            let hinfo = AllocImageInfo::new()
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
            panic!()
        }

        if let Some(image) = self.valloc_free(id, vdata.len()) {
            let data = &mut self.mmap;
            let mut off = image;

            for n in src {
                data[off..(off + 8)].copy_from_slice(n);
                off += 8;
            }

            data[off..(off + vdata.len())].copy_from_slice(vdata);

            image
        } else {
            let data = &mut self.mmap;
            let hinfo = AllocImageInfo::new()
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

    fn alloc_free(&mut self, id: u8) -> Option<usize> {
        match self.free_map[id as usize].pop() {
            Some(tag) => {
                let alloc_ref = block_on(self.alloc_map.read());
                let mut alloc_type = block_on(alloc_ref[id as usize].write());

                alloc_type.free -= 1;

                Some(tag)
            }
            None => None,
        }
    }

    // try first fit
    fn valloc_free(&mut self, id: u8, size: usize) -> Option<usize> {
        for (index, off) in self.free_map[id as usize].iter().enumerate() {
            match self.image_info(*off) {
                Some(info) => {
                    if info.len() >= size as u16 {
                        let alloc_ref = block_on(self.alloc_map.read());
                        let mut alloc_type = block_on(alloc_ref[id as usize].write());

                        alloc_type.free -= 1;

                        return Some(self.free_map[id as usize].remove(index));
                    }
                }
                None => panic!(),
            }
        }

        None
    }

    // rewrite info header
    pub fn write_info(&mut self, info: AllocImageInfo, off: usize) {
        self.mmap[(off - 8)..off].copy_from_slice(&(info.into_bytes()))
    }

    // info header from heap tag
    pub fn image_info(&self, off: usize) -> Option<AllocImageInfo> {
        if off == 0 || off > self.write_barrier {
            None
        } else {
            let data = &self.mmap;
            let mut info = 0u64.to_le_bytes();

            info.copy_from_slice(&data[(off - 8)..off]);
            Some(AllocImageInfo::from_bytes(info))
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

    // read and write image data
    pub fn write_image(&mut self, image: &[[u8; 8]], offset: usize) {
        let mut index = offset;

        for n in image {
            self.mmap[index..(index + 8)].copy_from_slice(n);
            index += 8;
        }
    }

    pub fn image_slice(&self, off: usize, len: usize) -> Option<&[u8]> {
        if off == 0 || off > self.write_barrier {
            None
        } else {
            let data = &self.mmap;
            Some(&data[off..off + len])
        }
    }

    // gc
    pub fn gc_clear(&mut self) {
        let mut off: usize = 8;
        let alloc_ref = block_on(self.alloc_map.read());

        while let Some(mut info) = self.image_info(off) {
            info.set_mark(false);
            self.write_info(info, off);
            off += info.len() as usize
        }

        for (id, _) in alloc_ref.iter().enumerate() {
            let mut alloc_type = block_on(alloc_ref[id].write());

            alloc_type.free = 0;
        }

        for free in self.free_map.iter_mut() {
            free.clear()
        }
    }

    pub fn gc_sweep(&mut self) {
        let mut off: usize = 8;
        let alloc_ref = block_on(self.alloc_map.write());

        while let Some(info) = self.image_info(off) {
            if !info.mark() {
                let id = info.image_type() as usize;
                let mut alloc_type = block_on(alloc_ref[id].write());

                alloc_type.free += 1;
                self.free_map[id].push(off);
            }
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
}

// iterators
pub struct BumpAllocatorIterator<'a> {
    pub heap: &'a BumpAllocator,
    pub offset: usize,
}

impl<'a> BumpAllocatorIterator<'a> {
    pub fn new(heap: &'a BumpAllocator) -> Self {
        Self { heap, offset: 8 }
    }
}

impl<'a> Iterator for BumpAllocatorIterator<'a> {
    type Item = (AllocImageInfo, usize);

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
