#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

mod bindings_impl {
    include!("macos_15_5.rs");
}

pub use bindings_impl::*;
