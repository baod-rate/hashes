use crate::{Digest, Tiger, TigerCore};
use alloc::vec::Vec;
use core::fmt;
use core::mem::swap;
use digest::{
    core_api::CoreWrapper,
    core_api::{
        AlgorithmName, Block, BlockSizeUser, Buffer, BufferKindUser, FixedOutputCore,
        OutputSizeUser, Reset, UpdateCore,
    },
    typenum::Unsigned,
    typenum::U1024,
    HashMarker, Output,
};

/// Core Tiger hasher state.
#[derive(Clone)]
pub struct TigerTreeCore {
    leaves: Vec<Output<TigerCore>>,
    hasher: Tiger,
    blocks_processed: usize,
}

impl Default for TigerTreeCore {
    fn default() -> Self {
        Self {
            leaves: Vec::default(),
            hasher: Tiger::new_with_prefix([LEAF_SIG]),
            blocks_processed: 0,
        }
    }
}

type DataBlockSize = U1024;
const LEAF_SIG: u8 = 0u8;
const NODE_SIG: u8 = 1u8;
const DATA_BLOCKS_PER_LEAF: usize =
    DataBlockSize::USIZE / <TigerCore as BlockSizeUser>::BlockSize::USIZE;

impl HashMarker for TigerTreeCore {}

impl BlockSizeUser for TigerTreeCore {
    type BlockSize = <TigerCore as BlockSizeUser>::BlockSize;
}

impl BufferKindUser for TigerTreeCore {
    type BufferKind = <TigerCore as BufferKindUser>::BufferKind;
}

impl OutputSizeUser for TigerTreeCore {
    type OutputSize = <TigerCore as OutputSizeUser>::OutputSize;
}

impl UpdateCore for TigerTreeCore {
    #[inline]
    fn update_blocks(&mut self, blocks: &[Block<Self>]) {
        for block in blocks {
            self.hasher.update(block);
            self.blocks_processed += 1;
            if self.blocks_processed == DATA_BLOCKS_PER_LEAF {
                let mut hasher = Tiger::new_with_prefix([LEAF_SIG]);
                swap(&mut self.hasher, &mut hasher);
                let hash = hasher.finalize();
                self.leaves.push(hash);
                self.blocks_processed = 0;
            }
        }
    }
}

impl FixedOutputCore for TigerTreeCore {
    #[inline]
    fn finalize_fixed_core(&mut self, buffer: &mut Buffer<Self>, out: &mut Output<Self>) {
        match buffer.get_pos() {
            0 => {}
            _ => {
                self.hasher.update(buffer.get_data());
                self.blocks_processed += 1;
            }
        }

        match self.blocks_processed {
            0 => {}
            _ => {
                let mut hasher = Tiger::new_with_prefix([LEAF_SIG]);
                swap(&mut self.hasher, &mut hasher);
                let hash = hasher.finalize();
                self.leaves.push(hash);
                self.blocks_processed = 0;
            }
        }

        let result = hash_nodes(self.leaves.as_slice());
        out.copy_from_slice(&result);
    }
}

fn hash_nodes(hashes: &[Output<TigerCore>]) -> Output<TigerCore> {
    match hashes.len() {
        0 => hash_nodes(&[Tiger::digest([LEAF_SIG])]),
        1 => hashes[0],
        _ => {
            let left_hashes = hashes.iter().step_by(2);

            let right_hashes = hashes.iter().map(Some).skip(1).chain([None]).step_by(2);

            let next_level_hashes: Vec<Output<TigerCore>> = left_hashes
                .zip(right_hashes)
                .map(|(left, right)| match right {
                    Some(right) => {
                        let content = {
                            let mut leaf_content =
                                [0; <TigerCore as OutputSizeUser>::OutputSize::USIZE * 2 + 1];
                            let (sig, contents) = leaf_content.split_at_mut(1);
                            sig.copy_from_slice(&[NODE_SIG]);
                            let (left_split, right_split) = contents
                                .split_at_mut(<TigerCore as OutputSizeUser>::OutputSize::USIZE);
                            left_split.copy_from_slice(left);
                            right_split.copy_from_slice(right);
                            leaf_content
                        };
                        CoreWrapper::<TigerCore>::digest(content)
                    }
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
