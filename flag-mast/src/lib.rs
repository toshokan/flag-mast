pub use flag_mast_derive::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
	#[derive(Flags, Default)]
	#[flag_debug()]
	#[flag(name = "BARKS", method_name = "can_bark", value = 0x1)]
	#[flag(name = "sits", value = 0x2)]
	#[flag(name = "OBEDIENT", method_name = "is_obedient", value = 0x4)]
	struct Dog(#[flag_backing_field] u32);

	let mut dog = Dog::default();
	dog.set_can_bark(true);
	dog.set_sits(true);
	assert!(dog.can_bark() == true);
    }
}
