use crate::{Digest, Md4, Md4Core};
use core::fmt;
use digest::{
    core_api::{
        AlgorithmName, Block, BlockSizeUser, Buffer, BufferKindUser, FixedOutputCore,
        OutputSizeUser, Reset, UpdateCore,
    },
    typenum::Unsigned,
    HashMarker, Output,
};

/// Core eD2k hasher state.
#[derive(Clone, Default)]
pub struct Ed2kCore {
    inner_hasher: Md4,
    blocks_hashed: u64,
    root_hasher: Md4,
    chunks_hashed: usize,
}

/// The number of Md4Core blocks in a eD2k hash chunk
const BLOCKS_PER_CHUNK: u64 = 9_728_000_u64 / <Md4Core as BlockSizeUser>::BlockSize::U64;

impl HashMarker for Ed2kCore {}

impl BlockSizeUser for Ed2kCore {
    type BlockSize = <Md4Core as BlockSizeUser>::BlockSize;
}

impl BufferKindUser for Ed2kCore {
    type BufferKind = <Md4Core as BufferKindUser>::BufferKind;
}

impl OutputSizeUser for Ed2kCore {
    type OutputSize = <Md4Core as OutputSizeUser>::OutputSize;
}

impl Ed2kCore {
    #[inline]
    fn finalize_inner(&mut self) {
        self.finalize_inner_with_buffer(&mut Buffer::<Self>::default());
    }

    #[inline]
    fn finalize_inner_with_buffer(&mut self, buffer: &mut Buffer<Self>) {
        let hash = {
            let mut hash = Output::<Md4Core>::default();
            self.inner_hasher.update(buffer.get_data());
            self.inner_hasher.finalize_into_reset(&mut hash);
            self.blocks_hashed = 0;
            hash
        };
        self.root_hasher.update(hash);
        self.chunks_hashed += 1;
    }

    #[inline]
    fn update_block(&mut self, block: Block<Self>) {
        if self.blocks_hashed == BLOCKS_PER_CHUNK {
            self.finalize_inner();
        }
        self.inner_hasher.update(block);
        self.blocks_hashed = self.blocks_hashed.wrapping_add(1);
    }
}

impl UpdateCore for Ed2kCore {
    #[inline]
    fn update_blocks(&mut self, blocks: &[Block<Self>]) {
        for block in blocks {
            self.update_block(*block);
        }
    }
}

impl FixedOutputCore for Ed2kCore {
    #[inline]
    fn finalize_fixed_core(&mut self, buffer: &mut Buffer<Self>, out: &mut Output<Self>) {
        if self.blocks_hashed == BLOCKS_PER_CHUNK {
            if buffer.get_pos() > 0 {
                self.finalize_inner();
            }
        }

        if self.chunks_hashed == 0 {
            self.inner_hasher.update(buffer.get_data());
            self.inner_hasher.finalize_into_reset(out);
        } else {
            self.finalize_inner_with_buffer(buffer);
            self.root_hasher.finalize_into_reset(out);
        }
    }
}

impl Reset for Ed2kCore {
    #[inline]
    fn reset(&mut self) {
        *self = Default::default();
    }
}

impl AlgorithmName for Ed2kCore {
    fn write_alg_name(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Ed2k")
    }
}

impl fmt::Debug for Ed2kCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Ed2kCore { ... }")
    }
}
