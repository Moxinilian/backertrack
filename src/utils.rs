use std::path::PathBuf;

pub fn path_exists_or_panic(input: &str) -> PathBuf {
    let path = PathBuf::from(input);

    if !&path.exists() {
        panic!("File at `{}` does not exist", input);
    }

    path
}

pub trait GetOrDefault<T: Default> {
    fn get_or_default(&'_ mut self, index: usize) -> &'_ T;
    fn get_mut_or_default(&'_ mut self, index: usize) -> &'_ mut T;
}

impl<T: Default + Clone> GetOrDefault<T> for Vec<T> {
    fn get_or_default(&'_ mut self, index: usize) -> &'_ T {
        if self.len() <= index {
            self.resize(index + 1, T::default());
        }

        unsafe { self.get_unchecked(index) }
    }

    fn get_mut_or_default(&'_ mut self, index: usize) -> &'_ mut T {
        if self.len() <= index {
            self.resize(index + 1, T::default());
        }

        unsafe { self.get_unchecked_mut(index) }
    }
}
