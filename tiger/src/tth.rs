use core::fmt;
// use data_encoding::BASE32;
use digest::{
    core_api::{
        AlgorithmName, Block, BlockSizeUser, Buffer, BufferKindUser, FixedOutputCore,
        OutputSizeUser, Reset, UpdateCore,
    },
    typenum::Unsigned,
    typenum::U1024,
    HashMarker, Output,
};

use crate::{TigerCore, Tiger, Digest};
use alloc::vec::Vec;
use data_encoding::BASE32;
use digest::core_api::CoreWrapper;

/// Core Tiger hasher state.
#[derive(Clone, Default)]
pub struct TigerTreeCore {
    leaves: Vec<Output<TigerCore>>,
}

const LEAF_SIG: u8 = 0u8;
const NODE_SIG: u8 = 1u8;

impl HashMarker for TigerTreeCore {}

impl BlockSizeUser for TigerTreeCore {
    type BlockSize = U1024;
}

impl BufferKindUser for TigerTreeCore {
    type BufferKind = <TigerCore as BufferKindUser>::BufferKind;
}

impl OutputSizeUser for TigerTreeCore {
    type OutputSize = <TigerCore as OutputSizeUser>::OutputSize;
}

impl TigerTreeCore {
    #[inline]
    fn update_block(&mut self, block: &Block<Self>) {
        let mut hasher = Tiger::new();
        let data: Vec<u8> = [&[LEAF_SIG], block.as_slice()].concat();
        hasher.update(data);
        let hash = hasher.finalize();
        // TODO:
        println!("update_block(): {}", BASE32.encode(&hash));
        self.leaves.push(hash);
    }
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
                let data: Vec<u8> = [&[LEAF_SIG], buffer.get_data()].concat();
                let hash = Tiger::digest(data);
                self.leaves.push(hash);
            }
        }

        let result = hash_nodes(self.leaves.as_slice());
        out.copy_from_slice(&result);
    }
}

fn hash_nodes(hashes: &[Output<TigerCore>]) -> Output<TigerCore> {
    println!("hashes: {} nodes", hashes.len());
    for hash in hashes {
        println!("{}", BASE32.encode(hash));
    }

    match hashes.len() {
        // 0 => Tiger::digest(&[0u8]).try_into().expect("wrong size"),
        0 => {
            let hash = CoreWrapper::<TigerCore>::digest([LEAF_SIG]);
            hash_nodes(&[hash])
        }
        1 => hashes[0],
        _ => {
            let left_hashes = hashes.iter().step_by(2);

            let right_hashes = hashes
                .iter()
                .map(Some)
                .chain([None])
                .skip(1)
                .step_by(2);

            let next_level_hashes: Vec<Output<TigerCore>> = left_hashes
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

                            let content = {
                                const LEAF_CONTENT_SIZE: usize = <TigerCore as OutputSizeUser>::OutputSize::USIZE * 2 + 1;
                                let mut leaf_content = [0; LEAF_CONTENT_SIZE];
                                let (sig, contents) = leaf_content.split_at_mut(1);
                                sig.copy_from_slice(&[NODE_SIG]);
                                let (left_split, right_split) = contents.split_at_mut(<TigerCore as OutputSizeUser>::OutputSize::USIZE);
                                left_split.copy_from_slice(left);
                                right_split.copy_from_slice(right);
                                leaf_content
                            };
                            CoreWrapper::<TigerCore>::digest(content)
                        },
                        None => *left,
                    })
                .collect();

            return hash_nodes(next_level_hashes.as_slice());
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
