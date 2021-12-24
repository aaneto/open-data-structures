use crate::array::Array;

/// The Array Queue treating the backing Array as a Ring, controlling the first and last element
/// wrapping around the array end. If there is not enough capacity we resize the array.
///
/// If first_in is J and size is K, then the elements are at:
/// ARR[J], ARR[J + 1], ... , ARR[(J + K) % len(ARR)].
///
/// If size exceeds len(ARR) at insertion, we need to resize the backing array.
pub struct ArrayQueue<T> {
    arr: Array<T>,
    first_in: usize,
    size: usize,
}

impl<T: Sized> ArrayQueue<T> {
    pub fn with_capacity(capacity: usize) -> Option<ArrayQueue<T>> {
        Array::with_capacity(capacity).map(|arr| Self {
            arr,
            first_in: 0,
            size: 0,
        })
    }

    pub fn new() -> Option<ArrayQueue<T>> {
        Array::new().map(|arr| Self {
            arr,
            first_in: 0,
            size: 0,
        })
    }

    pub fn length(&self) -> usize {
        self.size
    }

    pub fn peek(&self, idx: usize) -> Option<T> {
        if idx >= self.size {
            None
        } else {
            unsafe {
                self.arr
                    .read_at((self.first_in + idx) % self.arr.capacity())
            }
        }
    }

    pub fn peek_back(&self) -> Option<T> {
        if self.size == 0 {
            None
        } else {
            self.peek(self.size - 1)
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.size == 0 {
            None
        } else {
            let v = unsafe { self.arr.read_at(self.first_in) };
            self.first_in = (self.first_in + 1) % self.arr.capacity();
            self.size -= 1;

            v
        }
    }

    /// Add at an arbitrary index of the deque.
    /// Effectively shifting every element to the right
    /// or left, which is less expensive.
    pub fn add(&mut self, idx: usize, val: T) {
        if self.size >= self.arr.capacity() {
            self.resize();
        }

        if idx > self.size / 2 {
            // Shift elements right
            for i in (idx..self.size).rev() {
                unsafe {
                    self.arr.write_at(
                        self.mod_index(i + 1),
                        self.arr.read_at(self.mod_index(i)).unwrap(),
                    );
                }
            }
        } else {
            if self.first_in == 0 {
                let mut j = self.arr.capacity() - 1;
                for i in 0..idx + 1 {
                    unsafe {
                        self.arr.write_at(
                            self.mod_index(j),
                            self.arr.read_at(self.mod_index(i)).unwrap(),
                        );
                    }
                    j = i;
                }
                self.first_in = self.arr.capacity() - 1;
            } else {
                let mut j = self.first_in - 1;
                for i in 0..idx + 1 {
                    unsafe {
                        self.arr.write_at(
                            self.mod_index(j),
                            self.arr.read_at(self.mod_index(i)).unwrap(),
                        );
                    }
                    j = i;
                }
                self.first_in -= 1;
            }
        }

        unsafe {
            self.arr.write_at(self.mod_index(idx), val);
        }

        self.size += 1;
    }

    /// Remove at an arbitrary index of the deque.
    /// Effectively shifting every element to the right
    /// or left, which is less expensive.
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx >= self.size {
            return None;
        }

        let el = self.peek(idx);
        if idx > self.size / 2 {
            // Remove and shift elements left
            // Starting from index, replace every element with the next value.

            for i in idx..self.size - 1 {
                unsafe {
                    self.arr.write_at(
                        self.mod_index(i),
                        self.arr.read_at(self.mod_index(i + 1)).unwrap(),
                    );
                }
            }
        } else {
            // Remove and shift elements right
            // Starting from index, replace every element with the previous value.

            for i in (0..idx).rev() {
                unsafe {
                    self.arr.write_at(
                        self.mod_index(i + 1),
                        self.arr.read_at(self.mod_index(i)).unwrap(),
                    );
                }
            }

            self.first_in += 1;
        }

        self.size -= 1;
        el
    }

    fn mod_index(&self, idx: usize) -> usize {
        (self.first_in + idx) % self.arr.capacity()
    }


    pub fn enqueue(&mut self, val: T) {
        if self.size >= self.arr.capacity() {
            self.resize();
        }

        let dest_idx = (self.first_in + self.size) % self.arr.capacity();
        // Because we check that size <= capacity, we know dest_idx is within the array.
        unsafe {
            self.arr.write_at(dest_idx, val);
        }
        self.size += 1;
    }

    pub fn resize(&mut self) {
        let old_capacity = self.arr.capacity();
        unsafe { self.arr.reallocate(2 * old_capacity) };

        for i in 0..self.size {
            let former_idx = (self.first_in + i) % old_capacity;
            let new_idx = (self.first_in + i) % self.arr.capacity();
            if former_idx != new_idx {
                unsafe {
                    self.arr
                        .write_at(new_idx, self.arr.read_at(former_idx).unwrap());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ArrayQueue;

    #[test]
    fn test_create() {
        let queue = ArrayQueue::<u8>::new().unwrap();
        assert_eq!(queue.length(), 0);
    }

    #[test]
    fn test_resize() {
        let mut queue = ArrayQueue::<u8>::new().unwrap();

        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);
        queue.enqueue(5);

        let v1 = queue.dequeue();
        let v2 = queue.dequeue();
        let v3 = queue.dequeue();
        let v4 = queue.dequeue();
        let v5 = queue.dequeue();

        assert_eq!(queue.length(), 0);
        assert_eq!(v1, Some(1));
        assert_eq!(v2, Some(2));
        assert_eq!(v3, Some(3));
        assert_eq!(v4, Some(4));
        assert_eq!(v5, Some(5));
    }

    #[test]
    fn test_add_shift_right() {
        let mut queue = ArrayQueue::<u8>::new().unwrap();

        for i in 0..10 {
            queue.enqueue(i);
        }

        queue.add(7, 100);
        queue.add(9, 101);
        queue.add(7, 102);

        assert_eq!(queue.length(), 13);
        assert_eq!(queue.peek(7), Some(102));
        assert_eq!(queue.peek(10), Some(101));
        assert_eq!(queue.peek(8), Some(100));
        assert_eq!(queue.peek(0), Some(0));
    }

    #[test]
    fn test_add_shift_left() {
        let mut queue = ArrayQueue::<u8>::new().unwrap();

        for i in 0..10 {
            queue.enqueue(i);
        }

        queue.add(2, 100);
        queue.add(3, 102);

        assert_eq!(queue.length(), 12);
        assert_eq!(queue.peek(3), Some(102));
        assert_eq!(queue.peek(2), Some(100));
        assert_eq!(queue.peek(0), Some(0));
        assert_eq!(queue.peek_back(), Some(9));
    }

    #[test]
    fn test_peek() {
        let queue = ArrayQueue::<u8>::new().unwrap();
        assert_eq!(queue.length(), 0);
        assert_eq!(queue.peek_back(), None);
    }

    #[test]
    fn test_remove_at_middle() {
        let mut queue = ArrayQueue::<u8>::with_capacity(10).unwrap();

        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);
        queue.enqueue(5);

        let v1 = queue.remove(1);
        let v2 = queue.remove(1);
        let v3 = queue.remove(1);

        assert_eq!(queue.length(), 2);
        assert_eq!(queue.peek(0), Some(1));
        assert_eq!(queue.peek_back(), Some(5));

        assert_eq!(v1, Some(2));
        assert_eq!(v2, Some(3));
        assert_eq!(v3, Some(4));
    }

    #[test]
    fn test_remove_dequeue() {
        let mut queue = ArrayQueue::<u8>::with_capacity(10).unwrap();

        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);
        queue.enqueue(5);

        let v1 = queue.dequeue();
        let v2 = queue.dequeue();
        let v3 = queue.dequeue();

        assert_eq!(queue.length(), 2);
        assert_eq!(queue.peek(0), Some(4));
        assert_eq!(queue.peek_back(), Some(5));
        assert_eq!(v1, Some(1));
        assert_eq!(v2, Some(2));
        assert_eq!(v3, Some(3));
    }

    #[test]
    fn test_add() {
        let mut queue = ArrayQueue::<u8>::with_capacity(10).unwrap();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);
        queue.enqueue(5);

        assert_eq!(queue.length(), 5);
        assert_eq!(queue.peek(0), Some(1));
        assert_eq!(queue.peek(4), Some(5));
    }
}
