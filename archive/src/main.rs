#![allow(clippy::all)] // remove all clippy warnings

// necessary for this kind of main function
#![no_main]
#[unsafe(no_mangle)]
fn main(argc: u64, argv: *mut *mut u8) -> u64 {
    let raw_str_ptr: *mut u8 = "Hello" as *const str as *mut u8;
    let mut s1: Str = str_allocate(raw_str_ptr, 5);
    str_print(&s1);
    println!();

    let raw_str_ptr: *mut u8 = ", World!" as *const str as *mut u8;
    let s2: Str = str_allocate(raw_str_ptr, 8);
    str_print(&s2);
    println!();

    str_push_str(&mut s1, &s2);
    str_print(&s1);
    println!();

    println!("-----");

    let mut list: StrList = str_list_new();
    str_list_push(&mut list, s1);
    str_list_push(&mut list, s2);
    str_list_print(&list);

    0
}

// String

enum Str {
    // pointer to start, length, capacity
    Str(*mut u64, u64, u64),
}

enum StrRefOption<'a> {
    Some(&'a Str),
    None,
}

enum StrList {
    // pointer to start, length, capacity
    Init(*mut Str, u64, u64),
}

fn str_list_new() -> StrList {
    let ptr: *mut Str = malloc_str(10);
    StrList::Init(ptr, 0, 10)
}

fn str_list_push(list: &mut StrList, string: Str) {
    if str_list_len(list) + 1 > str_list_capacity(list) {
        str_list_double_capacity(list);
    }

    let StrList::Init(ptr, len, _) = list;
    unsafe {
        *str_ptr_add(*ptr, *len) = string;
    }
    *len = *len + 1;
}

fn str_list_get<'a>(list: &'a StrList, i: u64) -> StrRefOption<'a> {
    if i >= str_list_len(list) {
        return StrRefOption::None;
    }

    let &StrList::Init(ptr, _, _) = list;

    unsafe {
        let str: &Str = &*str_ptr_add(ptr, i);
        StrRefOption::Some(str)
    }
}

fn str_list_print(list: &StrList) {
    let len = str_list_len(list);

    let mut i = 0;
    while i < len {
        match str_list_get(list, i) {
            StrRefOption::Some(str) => {
                str_print(str);
                println!();
            }
            _ => panic!(),
        }

        i = i + 1;
    }
}

fn str_list_len(list: &StrList) -> u64 {
    let &StrList::Init(_, len, _) = list;
    len
}

fn str_list_capacity(list: &StrList) -> u64 {
    let &StrList::Init(_, _, capacity) = list;
    capacity
}

fn str_list_double_capacity(list: &mut StrList) {
    let StrList::Init(ptr, len, capacity) = list;
    *capacity = *capacity * 2;

    let new_list: *mut Str = malloc_str(*capacity);

    let mut i: u64 = 0;
    while i < *len {
        unsafe {
            let str: Str = std::ptr::read(str_ptr_add(*ptr, i));
            *str_ptr_add(new_list, i) = str;
        }

        i = i + 1;
    }
}

fn str_print(string: &Str) {
    let len = str_len(string);

    let mut i = 0;
    while i < len {
        let character = ptr_add(str_ptr(string), i);
        unsafe {
            print!("{}", *character as u8 as char);
        }

        i = i + 1;
    }
}

fn str_push_str(dest: &mut Str, src: &Str) {
    let &Str::Str(src_ptr, src_len, _) = src;

    if src_len + str_len(dest) > str_capacity(dest) {
        str_double_capacity(dest);
    }

    let Str::Str(dest_ptr, dest_len, _) = dest;

    let mut i = 0;
    while i < src_len {
        unsafe {
            *ptr_add(*dest_ptr, *dest_len + i) = *ptr_add(src_ptr, i);
        }

        i = i + 1;
    }

    *dest_len = *dest_len + src_len;
}

fn str_capacity(string: &Str) -> u64 {
    let &Str::Str(_, _, capacity) = string;
    capacity
}

fn str_len(string: &Str) -> u64 {
    let &Str::Str(_, len, _) = string;
    len
}

fn str_ptr(string: &Str) -> *mut u64 {
    let &Str::Str(ptr, _, _) = string;
    ptr
}

fn str_double_capacity(string: &mut Str) {
    let Str::Str(ptr, len, capacity) = string;
    *capacity = *capacity * 2;

    let new_string = malloc_u64(*capacity);

    let mut i = 0;
    while i < *len {
        unsafe {
            *ptr_add(new_string, i) = *ptr_add(*ptr, i);
        }

        i = i + 1;
    }

    // should deallocate old string
    *ptr = new_string;
}

fn str_allocate(str: *mut u8, len: u64) -> Str {
    let ptr = malloc_u64(len);
    let mut i = 0;
    while i < len {
        unsafe {
            let character: *mut u64 = ptr_add(ptr, i);
            *character = *((str as u64 + i) as *mut u8) as u64;
        }

        i = i + 1;
    }
    Str::Str(ptr, len, len) // start with len == capacity
}

// Memory

fn ptr_add(ptr: *mut u64, offset: u64) -> *mut u64 {
    (ptr as u64 + offset * 8) as *mut u64
}

fn str_ptr_add(ptr: *mut Str, offset: u64) -> *mut Str {
    (ptr as u64 + offset * size_of::<Str>() as u64) as *mut Str
}

// My try on extracting bytes using only u64. While
// this does works, it causes alignment issues, when
// dereferencing loads invalid parts of memory.
fn get_byte(data: *mut u64, i: u64) -> u64 {
    let word_index: u64 = i / 8;
    let byte_index: u64 = i % 8;

    let word_ptr: *mut u64 = ptr_add(data, word_index);
    unsafe {
        let word: u64 = *word_ptr;
        let mask: u64 = shift_left(255, byte_index * 8); // 255 = 0xFF
        let filtered_byte: u64 = word & mask;
        shift_right(filtered_byte, byte_index * 8)
    }
}

// Math Operations used in get_byte()

fn shift_left(bits: u64, shift: u64) -> u64 {
    bits * pow(2, shift)
}

fn shift_right(bits: u64, shift: u64) -> u64 {
    bits / pow(2, shift)
}

fn pow(base: u64, exp: u64) -> u64 {
    if exp == 0 {
        1
    } else {
        base * pow(base, exp - 1)
    }
}

// Heap allocation
//
// Not well thought out at the moment
// Missing:
//  Checking for overflows
//  Checking for allocation failure
//  Initialising memory

fn malloc_u64(len: u64) -> *mut u64 {
    unsafe {
        std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
            len as usize * size_of::<Str>(),
            std::mem::align_of::<Str>(),
        )) as *mut u64
    }
}

fn malloc_str(len: u64) -> *mut Str {
    unsafe {
        std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
            len as usize * size_of::<Str>(),
            std::mem::align_of::<Str>(),
        )) as *mut Str
    }
}
