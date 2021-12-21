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

    pub fn peek(&self) -> Option<T> {
        if self.size == 0 {
            None
        } else {
            unsafe { self.arr.read_at(self.first_in) }
        }
    }

    pub fn remove(&mut self) -> Option<T> {
        if self.size == 0 {
            None
        } else {
            let v = unsafe { self.arr.read_at(self.first_in) };
            self.first_in = (self.first_in + 1) % self.arr.capacity();
            self.size -= 1;

            v
        }
    }

    pub fn add(&mut self, val: T) {
        if self.size >= self.arr.capacity() {
            let old_capacity = self.arr.capacity();
            unsafe { self.arr.reallocate(2 * old_capacity) };

            for i in 0..self.size {
                let former_idx = (self.first_in + i) % old_capacity;
                let new_idx = (self.first_in + i) % self.arr.capacity();
                if former_idx != new_idx {
                    unsafe {
                        self.arr.write_at(new_idx, self.arr.read_at(former_idx).unwrap());
                    }
                }
            }
        }

        let dest_idx = (self.first_in + self.size) % self.arr.capacity();
        // Because we check that size <= capacity, we know dest_idx is within the array.
        unsafe {
            self.arr.write_at(dest_idx, val);
        }
        self.size += 1;
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

        queue.add(1);
        queue.add(2);
        queue.add(3);
        queue.add(4);
        queue.add(5);

        let v1 = queue.remove();
        let v2 = queue.remove();
        let v3 = queue.remove();
        let v4 = queue.remove();
        let v5 = queue.remove();

        assert_eq!(queue.length(), 0);
        assert_eq!(v1, Some(1));
        assert_eq!(v2, Some(2));
        assert_eq!(v3, Some(3));
        assert_eq!(v4, Some(4));
        assert_eq!(v5, Some(5));
    }

    #[test]
    fn test_peek() {
        let queue = ArrayQueue::<u8>::new().unwrap();
        assert_eq!(queue.length(), 0);
        assert_eq!(queue.peek(), None);
    }

    #[test]
    fn test_remove_01() {
        let mut queue = ArrayQueue::<u8>::with_capacity(10).unwrap();

        queue.add(1);
        queue.add(2);
        queue.add(3);
        queue.add(4);
        queue.add(5);

        let v1 = queue.remove();
        let v2 = queue.remove();
        let v3 = queue.remove();

        assert_eq!(queue.length(), 2);
        assert_eq!(queue.peek(), Some(4));
        assert_eq!(v1, Some(1));
        assert_eq!(v2, Some(2));
        assert_eq!(v3, Some(3));
    }

    #[test]
    fn test_add() {
        let mut queue = ArrayQueue::<u8>::with_capacity(10).unwrap();
        queue.add(1);
        queue.add(2);
        queue.add(3);
        queue.add(4);
        queue.add(5);

        assert_eq!(queue.length(), 5);
        assert_eq!(queue.peek(), Some(1));
    }
}
