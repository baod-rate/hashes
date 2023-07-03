use core::fmt;
use data_encoding::BASE32;
use digest::{
    block_buffer::{BlockBuffer, Eager},
    core_api::{
        AlgorithmName, Block, BlockSizeUser, Buffer, BufferKindUser, FixedOutputCore,
        OutputSizeUser, Reset, UpdateCore,
    },
    typenum::Unsigned,
    HashMarker, Output,
};

use crate::compress::compress;
use crate::{State, TigerCore, S0};
use alloc::vec::Vec;
use digest::core_api::CoreWrapper;
use digest::Digest;

/// Core Tiger hasher state.
#[derive(Clone)]
pub struct TigerTreeCore {
    leaves: Vec<[u8; 24]>,
    block_len: usize,
    state: State,
}

impl TigerTreeCore {
    #[inline]
    fn update_data(&mut self, data: &[u8]) {}

    #[inline]
    fn update_block(&mut self, block: &Block<Self>) {
        self.update_data(block.as_slice());
        self.block_len += 1;
        compress(&mut self.state, block.as_ref());

        match self.block_len * <Self as BlockSizeUser>::BlockSize::USIZE {
            0 => {
                todo!("prefix w/ 0x00 and add")
            }
            1..=1023 => {
                todo!("add data")
            }
            1024 => {
                todo!("compress data block")
            }
            _ => unreachable!(),
        }

        // data blocks are 1024B in size
        // if let 128 = self.block_len {
        if let 1024 = self.block_len * <Self as BlockSizeUser>::BlockSize::USIZE {
            // leaf node content is hashed data block prefixed with 0x00
            // TODO:
            // let mut content = [LEAF_SIG; 25];

            // leaf node content is hash of data block prefixed with 0x00
            let mut buffer: Buffer<TigerCore> = BlockBuffer::<
                <TigerCore as BlockSizeUser>::BlockSize,
                <TigerCore as BufferKindUser>::BufferKind,
            >::new(&[LEAF_SIG]);

            self.state = S0;
            let bs = <TigerTreeCore as BlockSizeUser>::BlockSize::U64 as u64;
            let pos = buffer.get_pos() as u64;
            let bit_len = 8 * (pos + bs * self.block_len as u64);
            buffer.len64_padding_le(bit_len, |b| compress(&mut self.state, b.as_ref()));

            // store hash of content as a leaf
            let hash = {
                let mut hash = [0; 24];
                for (chunk, v) in hash[..].chunks_exact_mut(8).zip(self.state.iter()) {
                    chunk.copy_from_slice(&v.to_le_bytes());
                }
                hash
            };
            self.leaves.push(hash);

            // reset hasher
            self.state = S0;
            self.block_len = 0;
        }

        // TODO:
        // println!("update_block(): {}", BASE32.encode(block));
    }
}

const LEAF_SIG: u8 = 0u8;
const NODE_SIG: u8 = 1u8;

impl HashMarker for TigerTreeCore {}

impl BlockSizeUser for TigerTreeCore {
    type BlockSize = <TigerCore as BlockSizeUser>::BlockSize;
}

impl BufferKindUser for TigerTreeCore {
    type BufferKind = Eager;
}

impl OutputSizeUser for TigerTreeCore {
    type OutputSize = <TigerCore as OutputSizeUser>::OutputSize;
}

impl UpdateCore for TigerTreeCore {
    #[inline]
    fn update_blocks(&mut self, blocks: &[Block<Self>]) {
        for block in blocks {
            self.update_block(block)
        }
    }
}

impl FixedOutputCore for TigerTreeCore {
    #[inline]
    fn finalize_fixed_core(&mut self, buffer: &mut Buffer<Self>, out: &mut Output<Self>) {
        match buffer.get_pos() {
            0 => {}
            _ => {
                let hash = {
                    let mut hash = [0; 24];

                    let bs = <TigerTreeCore as BlockSizeUser>::BlockSize::U64 as u64;
                    let pos = buffer.get_pos() as u64;
                    let bit_len = 8 * (pos + bs * self.block_len as u64);
                    buffer.len64_padding_le(bit_len, |b| compress(&mut self.state, b.as_ref()));
                    for (chunk, v) in hash[..].chunks_exact_mut(8).zip(self.state.iter()) {
                        chunk.copy_from_slice(&v.to_le_bytes());
                    }
                    hash
                };

                self.state = S0;
                self.block_len = 0;

                let bs = Self::BlockSize::U64 as u64;
                let pos = buffer.get_pos() as u64;
                let bit_len = 8 * (pos + bs * self.block_len as u64);

                buffer.len64_padding_le(bit_len, |b| compress(&mut self.state, b.as_ref()));
                self.leaves.push(hash);

                self.leaves.push([0; 24]
                    // TODO:
                    // Tiger::new()
                    // .chain_update(&[0u8])
                    // .chain_update(buffer.get_data())
                    // .finalize()
                    // .try_into()
                    // .expect("wrong size")
                );
            }
        }

        let result = hash_nodes(self.leaves.as_slice());

        for (chunk, v) in out.chunks_exact_mut(1).zip(result.iter()) {
            chunk.copy_from_slice(&v.to_le_bytes());
        }
    }
}

fn hash_nodes(hashes: &[[u8; 24]]) -> [u8; 24] {
    println!("hashes");
    for hash in hashes {
        println!("{}", BASE32.encode(hash));
    }

    match hashes.len() {
        1 => hashes[0],
        // TODO:
        // 0 => Tiger::digest(&[0u8]).try_into().expect("wrong size"),
        0 => [0; 24],
        _ => {
            let left_hashes = hashes.into_iter().step_by(2);

            let right_hashes = hashes
                .into_iter()
                .map(|x| Some(x))
                .chain([None])
                .skip(1)
                .step_by(2);

            let foo: Vec<[u8; 24]> = left_hashes
                .zip(right_hashes)
                .map(|(left, right)| match right {
                        Some(right) => {
                            // [0; 24]

                            // let content: Vec<u8> = Vec::from_iter(
                            //     [LEAF_SIG]
                            //     .into_iter()
                            //     .copied()
                            //     .chain(
                            //         self.state
                            //         .iter()
                            //         .flat_map(|x| x.to_le_bytes())
                            //     )
                            // );

                            let content: [u8; 49] = {
                                let mut content: [u8; 49] = [LEAF_SIG; 49];
                                content[1..25].copy_from_slice(left);
                                content[25..].copy_from_slice(right);
                                content
                            };
                            // let hash: [u8; 24] = CoreWrapper::<TigerCore>::digest(content)
                            let hash: [u8; 24] = CoreWrapper::<TigerCore>::digest(content).into();
                                // .try_into()
                                // .expect("wrong length");
                            hash
                        }
                        ,
                        _ => {
                            let foo: [u8; 24] = [0; 24];
                            foo
                        }
                        // TODO:
                        // _ => Output::<TigerCore>::from(left)
                        // _ => *left,
                    })
                .collect();

            return hash_nodes(foo.as_slice());
        }
    }
}

impl Default for TigerTreeCore {
    fn default() -> Self {
        Self {
            leaves: Vec::new(),
            block_len: 0,
            state: S0,
        }
    }
}

impl Reset for TigerTreeCore {
    #[inline]
    fn reset(&mut self) {
        *self = Default::default();
    }
}

impl AlgorithmName for TigerTreeCore {
    fn write_alg_name(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TigerTree")
    }
}

impl fmt::Debug for TigerTreeCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TigerTreeCore { ... }")
    }
}
