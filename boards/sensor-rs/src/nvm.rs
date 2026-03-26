// This module should use non-volatile memory instead of constants

pub fn get_name() -> heapless::String<8> {
    heapless::String::try_from("SB101").expect("name is short")
}
