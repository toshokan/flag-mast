# flag-mast

Ergonomic Rust bit flags

## Usage
This `flag-mast` crate provides a derive macro to help create ergonomic C-compatible bit flags.

### Example
```rust
use flag_mast::*;

#[derive(Flags, Default)]
#[flag(name = "BARKS", method_name = "can_bark", value = 0x1)]
#[flag(name = "SITS", method_name = "can_sit", value = 0x2)]
#[flag(name = "BROWN", method_name = "is_brown", value = 0x4)]
struct Dog(#[flag_backing_field] u32);

fn foo() {
  let mut dog = Dog::default();
  dog.set_can_bark(true)
     .set_can_sit(false)
	 .set_is_brown(true);
	 
  if dog.is_brown() {
    println!("Cute doggo!");
  } else {
    println!("Doggo is not brown, but is cute anyway");
  }
}
```

The derive macro does not change the underlying layout of the struct, it can even be `repr(C)`!.

The `name` argument is the canonical name of the flag, which need not be a valid Rust identifier or follow idiomatic Rust naming conventions.
If the `name` argument _is_ appropriate as a Rust identifier (and you don't want to customize or prefix it), the `method_name` argument can be ommitted.
The `value` argument can either be an integer literal or a string containing an expression which resolves to the value for the flag.
The `value` needn't have to have the same exact type as the backing field, it only has to be castable to that type.
The `flag_backing_field` attribute specifies which field of the struct is used to hold the bit flags.

This means we can also have
```
use flag_mast::*;

const BLUE: u8 = 0x1;
const RED: u8 = 0x2;

mod secondary_colours {
  pub const YELLOW: u16 = 0x4;
}


#[derive(Flags, Default)]
#[flag(name = "blue", value = "BLUE")]
#[flag(name = "red", value = "RED")]
#[flag(name = "yellow", value = "secondary_colours::YELLOW")]
#[flag(name = "purple", value = "BLUE & RED")]
#[flag(name = "black", value = 0x8)]
#[repr(C)]
struct Colour{
  is_useful: bool,
  #[flag_backing_field] 
  flags: u32
}

fn bar() {
  let colour = Colour::default();
  colour.set_blue(true);
  colour.set_red(true);
  
  if colour.purple() {
    println("That's red AND blue!");
  }
}
```

### Automatic Debug Implementation

The derive macro can also automaticall generate a `Debug` implementation for you in a way that makes sense for your flags.
This behaviour is controlled by an additional attribute.

```rust
use flag_mast::*;

#[derive(Flags, Default)]
#[flag_debug]
#[flag(name = "one", value = 4)]
#[flag(name = "second", method_name = "two", value = 8)]
#[flag(name = "three", value = 16)]
struct Buttons(#[flag_backing_field] u16)

fn baz() {
  let mut buttons = Buttons::default();
  buttons.set_one(true);
  
  println!("{:?}", buttons);
  println!("---");
  println!("{:#?}", buttons);
}
```

This will print
```
Buttons { one: true, two: false, three: true }
---
Buttons {
    one: true,
	two: false,
	three: false
}
```

You can also choose a (potentially) more compact debug format by specifying the `compact` argument to the `flag_debug` attribute.
This format only displays the flags that are set.

```rust
use flag_mast::*;

#[derive(Flags, Default)]
#[flag_debug]
#[flag(name = "one", value = 4)]
#[flag(name = "second", method_name = "two", value = 8)]
#[flag(name = "three", value = 16)]
struct Buttons(#[flag_backing_field] u16)

fn baz() {
  let mut buttons = Buttons::default();
  buttons.set_one(true);
  buttons.set_three(true);
  
  println!("{:?}", buttons);
  println!("---");
  println!("{:#?}", buttons);
}
```

This will print
```
Buttons { "one", "three" }
---
Buttons {
    "one",
    "three",
}
```

## License
`flag-mast` is dual licensed under the MIT license and the Apache 2.0 license. You may choose whichever one you prefer.
