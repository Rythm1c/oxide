use ash::vk;

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as usize]
        .iter()
        .enumerate()
        .find(|(index, mt)| {
            (1 << index) & memory_req.memory_type_bits != 0 && mt.property_flags.contains(flags)
        })
        .map(|(index, _)| index as u32)
}
