use std::ptr::{self, NonNull};
use std::marker::PhantomData;
use std::alloc::{realloc, alloc, Layout};

static STARTING_CAPACITY: usize = 64;

struct Array<T: Sized> {
    ptr: NonNull<T>,
    capacity: usize,
    size: usize,
    _m: PhantomData<T>
}

impl<T: Sized> Array<T> {
    pub fn with_capacity(cap: usize) -> Array<T> {
        let layout = Layout::array::<T>(cap).unwrap();
        let ptr = unsafe { NonNull::new(alloc(layout) as *mut T).unwrap() };

        Array {
            ptr,
            size: 0,
            capacity: cap,
            _m: PhantomData::default(),
        }
    }

    pub fn new() -> Array<T> {
        let layout = Layout::array::<T>(STARTING_CAPACITY).unwrap();
        let ptr = unsafe { NonNull::new(alloc(layout) as *mut T).unwrap() };

        Array {
            ptr,
            size: 0,
            capacity: STARTING_CAPACITY,
            _m: PhantomData::default(),
        }
    }

    pub fn get(&self, idx: usize) -> T {
        self.read_at(idx)
    }

    pub fn set(&mut self, idx: usize, val: T) -> T {
        if idx == self.size {
            panic!("Index error");
        }
        let v = self.read_at(idx);
        self.write_at(idx, val);

        v
    }

    pub fn length(&self) -> usize {
        self.size
    }

    pub fn push(&mut self, index: usize, val: T) {
        self.grow_if_needed();

        if index == self.size {
            self.write_at(self.size, val);
        } else {
            for i in (index..self.size).rev() {
                self.write_at(i + 1, self.read_at(i));
            }
            self.write_at(index, val);
        }
        self.size += 1;
    }

    pub fn push_back(&mut self, val: T) {
        self.push(self.size, val);
    }

    pub fn push_front(&mut self, val: T) {
        self.push(0, val);
    }

    fn swap(&mut self, a_idx: usize, b_idx: usize) {
        let v_a = self.read_at(a_idx);
        let v_b = self.read_at(b_idx);

        self.write_at(a_idx, v_b);
        self.write_at(b_idx, v_a);
    }

    fn write_at(&mut self, idx: usize, val: T) {
        if idx > self.size || idx >= self.capacity {
            panic!("Index error");
        }
        unsafe {
            ptr::write(self.ptr.as_ptr().add(idx), val);
        }
    }

    fn read_at(&self, idx: usize) -> T {
        if idx >= self.size || idx >= self.capacity {
            panic!("Index error");
        }
        unsafe {
            ptr::read(self.ptr.as_ptr().add(idx))
        }
    }

    fn grow_if_needed(&mut self) {
        if self.size >= self.capacity {
            let old_layout = Layout::array::<T>(self.capacity).unwrap();
            self.capacity *= 2;
            let layout = Layout::array::<T>(self.capacity).unwrap();
            let new_ptr = unsafe { realloc(self.ptr.as_ptr() as *mut u8, old_layout, layout.size()) as *mut T };
            self.ptr = NonNull::new(new_ptr).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Array;

    #[test]
    fn test_create() {
        let arr = Array::<u8>::new();
        assert_eq!(arr.length(), 0);
    }

    #[test]
    fn test_push_back() {
        let mut arr = Array::<u8>::new();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);

        assert_eq!(arr.length(), 5);
        assert_eq!(arr.get(0), 1);
        assert_eq!(arr.get(1), 2);
        assert_eq!(arr.get(2), 3);
        assert_eq!(arr.get(3), 4);
        assert_eq!(arr.get(4), 5);
    }

    #[test]
    fn test_push_front() {
        let mut arr = Array::<u8>::new();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);
        arr.push_front(10);
        arr.push_front(11);


        assert_eq!(arr.length(), 7);
        assert_eq!(arr.get(0), 11);
        assert_eq!(arr.get(1), 10);
        assert_eq!(arr.get(2), 1);
        assert_eq!(arr.get(3), 2);
        assert_eq!(arr.get(4), 3);
    }

    #[test]
    fn test_grow() {
        let mut arr = Array::<u8>::with_capacity(4);
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);


        assert_eq!(arr.length(), 5);
        assert_eq!(arr.capacity, 8);
        assert_eq!(arr.get(0), 1);
        assert_eq!(arr.get(1), 2);
        assert_eq!(arr.get(2), 3);
        assert_eq!(arr.get(3), 4);
        assert_eq!(arr.get(4), 5);
    }

}
