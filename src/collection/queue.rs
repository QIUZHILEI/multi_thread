
pub struct Queue<T> {
    head: AtomicPtr<QueueNode<T>>,
    tail: AtomicPtr<QueueNode<T>>,
}

struct QueueNode<T> {
    data: Option<T>,
    next: *mut QueueNode<T>,
    _marker: PhantomData<T>,
}

impl<T> QueueNode<T> {
    fn new(data: T) -> Self {
        Self {
            data: Some(data),
            next: std::ptr::null_mut::<QueueNode<T>>(),
            _marker: PhantomData::default(),
        }
    }
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        let h = Box::new(QueueNode {
            data: None,
            next: std::ptr::null_mut(),
            _marker: PhantomData::default(),
        });
        let ptr = Box::into_raw(h);
        Self {
            head: AtomicPtr::new(ptr),
            tail: AtomicPtr::new(ptr),
        }
    }

    pub fn is_empty(&self) -> bool {
        let h = self.head.load(Ordering::Relaxed);
        unsafe { (*h).next.is_null() }
    }

    pub fn enqueue(&self, data: T) {
        let node = Box::into_raw(Box::new(QueueNode::new(data)));
        loop {
            let t = self.tail.load(Ordering::Acquire);
            if let Ok(pre) =
                self.tail
                    .compare_exchange(t, node, Ordering::SeqCst, Ordering::Relaxed)
            {
                unsafe {
                    (*pre).next = node;
                }
                break;
            }
        }
    }

    pub fn dequeue(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        loop {
            let h = self.head.load(Ordering::Acquire);
            let first = unsafe { (*h).next };
            if first.is_null() {
                return None;
            } else {
                if let Ok(_) =
                    self.head
                        .compare_exchange_weak(h, first, Ordering::SeqCst, Ordering::Relaxed)
                {
                    let data = unsafe { (*first).data.take() };
                    //TODO!() h不能直接回收，可能造成UB行为
                    // unsafe{Box::form_raw(h)};

                    return data;
                }
            }
        }
    }

    pub fn size(&self) -> u32 {
        let mut tmp = self.head.load(Ordering::Relaxed);
        let mut count = 0;
        loop {
            tmp = unsafe { (*tmp).next };
            if tmp.is_null() {
                break;
            }
            count += 1;
        }
        count
    }

    // pub fn foreach(func:F)
    // where F:Fn(){

    // }
}

impl<T> Queue<T>
where
    T: Eq,
{
    pub fn contain(&self, _other: &T) -> bool {
        true
    }
}

impl<T> Queue<T>
where
    T: Display,
{
    pub fn traverse(&self) {
        let mut tmp = self.head.load(Ordering::Relaxed);
        loop {
            tmp = unsafe { (*tmp).next };
            if tmp.is_null() {
                break;
            }
            let data = unsafe { (*tmp).data.as_ref().unwrap() };
            println!("{}", data);
        }
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        let mut tmp = self.head.load(Ordering::Relaxed);
        loop {
            let next = unsafe { (*tmp).next };
            unsafe { Box::from_raw(tmp) };
            if next.is_null() {
                break;
            }
            tmp = next;
        }
    }
}

unsafe impl<T> Send for Queue<T> {}
unsafe impl<T> Sync for Queue<T> {}