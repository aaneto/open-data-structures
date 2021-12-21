use std::ptr::{self, NonNull};
use std::marker::PhantomData;
use std::alloc::{realloc, alloc, Layout};

/// A dynamically sized array implementation.
pub struct Array<T: Sized> {
    ptr: NonNull<T>,
    capacity: usize,
    size: usize,
    _m: PhantomData<T>
}

impl<T: Sized> Array<T> {
    /// Create a new dynamic Array.
    pub fn new() -> Option<Array<T>> {
        Self::with_capacity(1)
    }

    /// Create an array with custom capacity.
    pub fn with_capacity(capacity: usize) -> Option<Array<T>> {
        let layout = Layout::array::<T>(capacity).ok()?;
        let ptr = unsafe { NonNull::new(alloc(layout) as *mut T)? };

        Some(Array {
            ptr,
            size: 0,
            capacity,
            _m: PhantomData::default(),
        })
    }

    /// Removes element at index
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx >= self.size {
            return None;
        }

        let v = unsafe {
            let v = self.read_at(idx);

            for i in idx..self.size - 1 {
                self.write_at(i, self.read_at(i + 1).unwrap())
            }
            v
        };
        self.size -= 1;
        self.adjust_size();

        v
    }

    // Removes element at the end of array.
    pub fn pop_back(&mut self) -> Option<T> {
        self.remove(self.size - 1)
    }

    // Removes element at the start of array.
    pub fn pop_front(&mut self) -> Option<T> {
        self.remove(0)
    }

    /// Gets the value at an index.
    pub fn get(&self, idx: usize) -> Option<T> {
        if idx >= self.size {
            return None;
        }
        unsafe {
            self.read_at(idx)
        }
    }

    /// Set value at an index, returning the former value.
    pub fn set(&mut self, idx: usize, val: T) -> Option<T> {
        if idx >= self.size {
            return None;
        }

        unsafe {
            let v = self.read_at(idx);
            self.write_at(idx, val);

            v
        }
    }

    /// Gets how many elements are in the array.
    pub fn length(&self) -> usize {
        self.size
    }

    /// Gets how many elements can fit in the array.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Add element at index 'index' in the array, pushing all elements with index > 'index' to the right.
    pub fn push(&mut self, index: usize, val: T) {
        self.adjust_size();

        unsafe {
            if index == self.size {
                self.write_at(self.size, val);
            } else {
                for i in (index..self.size).rev() {
                    self.write_at(i + 1, self.read_at(i).unwrap());
                }
                self.write_at(index, val);
            }
        }
        self.size += 1;
    }

    /// Add an element at the end of the array.
    pub fn push_back(&mut self, val: T) {
        self.push(self.size, val);
    }

    /// Add an element to the start of the array and push right all elements.
    pub fn push_front(&mut self, val: T) {
        self.push(0, val);
    }

    /// Write at location using inner pointer.
    pub unsafe fn write_at(&mut self, idx: usize, val: T) {
        if idx > self.capacity {
            panic!("Index error");
        }
        ptr::write(self.ptr.as_ptr().add(idx), val);
    }

    /// Read at location using inner pointer.
    pub unsafe fn read_at(&self, idx: usize) -> Option<T> {
        if idx > self.capacity {
            return None;
        }
        Some(ptr::read(self.ptr.as_ptr().add(idx)))
    }

    pub unsafe fn reallocate(&mut self, new_capacity: usize) {
        let old_layout = Layout::array::<T>(self.capacity).unwrap();
        self.capacity = new_capacity;
        let layout = Layout::array::<T>(new_capacity).unwrap();
        let new_ptr = unsafe { realloc(self.ptr.as_ptr() as *mut u8, old_layout, layout.size()) as *mut T };
        self.ptr = NonNull::new(new_ptr).unwrap();
    }

    /// Grow and shrink array if needed by operation.
    fn adjust_size(&mut self) {
        let new_capacity = if self.size >= self.capacity {
            self.capacity * 2
        } else if self.size < self.capacity / 2 {
            self.capacity / 2
        } else {
            self.capacity
        };

        if new_capacity != self.capacity {
            unsafe { self.reallocate(new_capacity) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Array;

    #[test]
    fn test_create() {
        let arr = Array::<u8>::new().unwrap();
        assert_eq!(arr.length(), 0);
    }

    #[test]
    fn test_shrink() {
        let mut arr = Array::<u8>::new().unwrap();

        for _ in 0..9 {
            arr.push_back(0);
        }
        let cap1 = arr.capacity;

        arr.pop_back();
        arr.pop_back();

        assert_eq!(arr.capacity, 8);
        assert_eq!(cap1, 16);
    }

    #[test]
    fn test_push_back() {
        let mut arr = Array::<u8>::new().unwrap();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);

        assert_eq!(arr.length(), 5);
        assert_eq!(arr.get(0), Some(1));
        assert_eq!(arr.get(4), Some(5));
    }

    #[test]
    fn test_push_front() {
        let mut arr = Array::<u8>::new().unwrap();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_front(10);
        arr.push_front(11);


        assert_eq!(arr.get(0), Some(11));
        assert_eq!(arr.get(1), Some(10));
    }

    #[test]
    fn test_grow() {
        let mut arr = Array::<u8>::with_capacity(4).unwrap();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);


        assert_eq!(arr.length(), 5);
        assert_eq!(arr.capacity, 8);
    }

    #[test]
    fn test_set() {
        let mut arr = Array::<u8>::with_capacity(4).unwrap();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);
        arr.set(2, 10);

        assert_eq!(arr.get(2), Some(10));
        assert_eq!(arr.get(4), Some(5));
    }

    #[test]
    fn test_remove_middle() {
        let mut arr = Array::<u8>::with_capacity(4).unwrap();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);
        let v = arr.remove(2);

        assert_eq!(arr.length(), 4);
        assert_eq!(arr.get(arr.length() - 1), Some(5));
        assert_eq!(arr.get(2), Some(4));
        assert_eq!(v, Some(3));
    }

    #[test]
    fn test_remove_start() {
        let mut arr = Array::<u8>::with_capacity(4).unwrap();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);
        let v = arr.remove(0);

        assert_eq!(arr.length(), 4);
        assert_eq!(arr.get(arr.length() - 1), Some(5));
        assert_eq!(arr.get(0), Some(2));
        assert_eq!(v, Some(1));
    }

    #[test]
    fn test_remove_end() {
        let mut arr = Array::<u8>::with_capacity(4).unwrap();
        arr.push_back(1);
        arr.push_back(2);
        arr.push_back(3);
        arr.push_back(4);
        arr.push_back(5);
        let v = arr.remove(arr.length() - 1);

        assert_eq!(arr.length(), 4);
        assert_eq!(arr.get(arr.length() - 1), Some(4));
        assert_eq!(v, Some(5));
    }
}
