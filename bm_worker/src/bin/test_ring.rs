#[derive(Debug)]
pub struct RingBuf<T: Clone>
{
    pub cap: usize,
    pub len: usize,
    pub buf: Vec<T>,
}

impl<T> RingBuf<T>
    where T: Clone
{
    pub fn new(cap: usize) -> Self {
        RingBuf {
            cap,
            len: 0,
            buf: Vec::new(),
        }
    }

    pub fn add(&mut self, item: T) {
        if self.len < self.cap {
            self.buf.push(item);
            self.len += 1;
        } else {
            self.buf.remove(0);
            self.buf.push(item);
        }
    }

    pub fn append(&mut self, list: Vec<T>) {
        for v in list {
            self.add(v);
        }
    }
}

fn main() {
    let mut list: RingBuf<String> = RingBuf::new(5);

    for i in 0..10 {
        list.add(format!("{}", i + 1));
        println!("->{:?}", list);
    }

    let items = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string()];
    list.append(items);

    println!("aaaa");
    println!("->{:?}", list);
}