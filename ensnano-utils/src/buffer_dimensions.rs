/// This struct handle the alignment of row in WGPU buffers.
pub struct BufferDimensions {
    pub width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
}

impl BufferDimensions {
    pub fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let block_size = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padding = (block_size - unpadded_bytes_per_row % block_size) % block_size;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
    pub fn buffer_size(&self) -> usize {
        self.padded_bytes_per_row * self.height
    }
}
