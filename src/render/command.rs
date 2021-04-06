use ash::vk;

pub struct CommandBuffer {
    inner: vk::CommandBuffer,
    pool: vk::CommandPool,
}
