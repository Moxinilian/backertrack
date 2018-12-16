pub fn increase_modular(value: &mut usize, step: usize, max: usize) {
    if max != 0 {
        *value = (*value + step) % max;
    }
}

pub fn decrease_modular(value: &mut usize, step: usize, max: usize) {
    if max != 0 {
        let x = (*value as isize - step as isize) % max as isize;
        *value = if x >= 0 { x } else { x + max as isize } as usize; // It's 5 AM okay
    }
}