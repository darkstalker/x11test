pub trait Array<T>
{
    fn map<F>(self, f: F) -> Self
        where T: Copy, F: Fn(T) -> T;
    fn len(&self) -> usize;
    fn as_ptr(&self) -> *const T;
}

impl<T> Array<T> for [T; 1]
{
    fn map<F>(self, f: F) -> Self
        where T: Copy, F: Fn(T) -> T
    {
        [f(self[0])]
    }

    fn len(&self) -> usize { 1 }
    fn as_ptr(&self) -> *const T { &self[0] }
}

impl<T> Array<T> for [T; 2]
{
    fn map<F>(self, f: F) -> Self
        where T: Copy, F: Fn(T) -> T
    {
        [f(self[0]), f(self[1])]
    }

    fn len(&self) -> usize { 2 }
    fn as_ptr(&self) -> *const T { &self[0] }
}

impl<T> Array<T> for [T; 3]
{
    fn map<F>(self, f: F) -> Self
        where T: Copy, F: Fn(T) -> T
    {
        [f(self[0]), f(self[1]), f(self[2])]
    }

    fn len(&self) -> usize { 3 }
    fn as_ptr(&self) -> *const T { &self[0] }
}

impl<T> Array<T> for [T; 4]
{
    fn map<F>(self, f: F) -> Self
        where T: Copy, F: Fn(T) -> T
    {
        [f(self[0]), f(self[1]), f(self[2]), f(self[3])]
    }

    fn len(&self) -> usize { 4 }
    fn as_ptr(&self) -> *const T { &self[0] }
}

impl<T> Array<T> for [T; 5]
{
    fn map<F>(self, f: F) -> Self
        where T: Copy, F: Fn(T) -> T
    {
        [f(self[0]), f(self[1]), f(self[2]), f(self[3]), f(self[4])]
    }

    fn len(&self) -> usize { 5 }
    fn as_ptr(&self) -> *const T { &self[0] }
}

impl<T> Array<T> for [T; 6]
{
    fn map<F>(self, f: F) -> Self
        where T: Copy, F: Fn(T) -> T
    {
        [f(self[0]), f(self[1]), f(self[2]), f(self[3]), f(self[4]), f(self[5])]
    }

    fn len(&self) -> usize { 6 }
    fn as_ptr(&self) -> *const T { &self[0] }
}
