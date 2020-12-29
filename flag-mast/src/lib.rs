pub use flag_mast_derive::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
	#[derive(Flags, Default)]
	#[flag_debug()]
	#[flag(name = "BARKS", method_name = "can_bark", doc = "is a borky boi", value = 0x1)]
	#[flag(name = "sits", value = 0x2)]
	#[flag(name = "OBEDIENT", method_name = "is_obedient", value = 0x4)]
	struct Dog(#[flag_backing_field] u32);

	let mut dog = Dog::default();
	dog.set_can_bark(true);
	dog.set_sits(true);
	assert!(dog.can_bark() == true);
    }

    const BLUE: u8 = 0x1;
    const RED: u8 = 0x2;

    mod secondary_colours {
	pub const YELLOW: u16 = 0x4;
    }

    #[test]
    fn it_works_two() {
	#[derive(Flags, Default)]
	#[flag_debug]
	#[flag(name = "blue", value = "BLUE")]
	#[flag(name = "red", value = "RED")]
	#[flag(name = "yellow", value = "secondary_colours::YELLOW")]
	#[flag(name = "purple", value = "BLUE | RED")]
	#[flag(name = "black", value = 16)]
	#[repr(C)]
	struct Colour{
	    is_useful: bool,
	    #[flag_backing_field] 
	    flags: u16
	}

	
	let mut colour = Colour::default();
	colour.set_blue(true);
	colour.set_red(true);
	
	dbg!(&colour);
	assert!(colour.purple());
    }

    #[test]
    fn it_works_three() {
	#[derive(Flags, Default)]
	#[flag_debug(compact)]
	#[flag(name = "one", value = 4)]
	#[flag(name = "second", method_name = "two", value = 8)]
	#[flag(name = "three", value = 16)]
	struct Buttons(#[flag_backing_field] u16);

	let mut buttons = Buttons::default();
	buttons.set_one(true);
	buttons.set_three(true);
	
	println!("{:?}", buttons);
	println!("---");
	println!("{:#?}", buttons);
    }

}


