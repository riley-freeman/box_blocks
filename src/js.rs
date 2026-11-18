use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use crate::threads::Thread;

#[link(wasm_import_module = "box_blocks")]
unsafe extern "C" {
    pub fn external_get_our_username() -> *mut u8;
    pub fn external_start_fetch_display_name(username: *const u8, len: usize, id: u32);
    pub fn external_start_fetch_threads(
        username: *const u8,
        len: usize,
        offset: usize,
        size: usize,
        id: u32,
    );
    pub fn external_format_time(time: usize) -> *mut u8;
}

thread_local! {
    static PENDING: RefCell<HashMap<u32, (Option<Box<dyn Any>>, Option<Waker>)>> = RefCell::new(HashMap::new());
    static NEXT_ID: RefCell<u32> = RefCell::new(1);

    static ADDING_THREADS: RefCell<HashMap<u32, Vec<Thread>>> = RefCell::new(HashMap::new());
}

#[unsafe(no_mangle)]
pub extern "C" fn on_display_name_fetched(id: u32, ptr: *const u8, len: usize) {
    let result =
        unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned() };
    PENDING.with(|p| {
        if let Some(entry) = p.borrow_mut().get_mut(&id) {
            entry.0 = Some(Box::new(result));
            if let Some(waker) = entry.1.take() {
                waker.wake();
            }
        }
    });
}

fn copy_js_string(ptr: *const u8, len: usize) -> String {
    if len == 0 {
        return String::new();
    }

    assert!(
        !ptr.is_null(),
        "JS returned a null string pointer with nonzero length"
    );

    unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned() }
}

#[unsafe(no_mangle)]
pub extern "C" fn thread_fetched(
    id: u32,
    username_ptr: *mut u8,
    username_len: usize,
    title_ptr: *mut u8,
    title_len: usize,
    message_ptr: *mut u8,
    message_len: usize,
    time: usize,
) {
    ADDING_THREADS.with(|p| {
        if let Some(entry) = p.borrow_mut().get_mut(&id) {
            let username = copy_js_string(username_ptr, username_len);
            let title = copy_js_string(title_ptr, title_len);
            let message = copy_js_string(message_ptr, message_len);

            entry.push(Thread::new(&username, &title, &message, time));
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn on_threads_fetched(id: u32) {
    PENDING.with(|p| {
        ADDING_THREADS.with(|at| {
            let threads = at.borrow_mut().remove(&id);
            if let Some(threads) = threads {
                if let Some(entry) = p.borrow_mut().get_mut(&id) {
                    entry.0 = Some(Box::new(threads));
                    if let Some(waker) = entry.1.take() {
                        waker.wake();
                    }
                }
            }
        })
    });
}

pub struct BoxFuture<T> {
    id: u32,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Clone + 'static> Future for BoxFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        PENDING.with(|p| {
            let mut map = p.borrow_mut();
            let entry = map.get_mut(&self.id).unwrap();
            if let Some(result) = entry.0.take() {
                map.remove(&self.id);
                Poll::Ready(result.downcast_ref::<T>().unwrap().clone())
            } else {
                entry.1 = Some(cx.waker().clone());
                Poll::Pending
            }
        })
    }
}

pub fn get_our_username() -> String {
    unsafe {
        let ptr = external_get_our_username();
        let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
        c_str.to_string_lossy().into_owned()
    }
}

pub fn format_time(time: usize) -> String {
    unsafe {
        let ptr = external_format_time(time);
        let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
        c_str.to_string_lossy().into_owned()
    }
}

pub fn fetch_display_name(username: &str) -> BoxFuture<String> {
    let id = NEXT_ID.with(|n| {
        let id = *n.borrow();
        *n.borrow_mut() = id + 1;
        id
    });
    PENDING.with(|p| p.borrow_mut().insert(id, (None, None)));
    unsafe {
        external_start_fetch_display_name(username.as_ptr(), username.len(), id);
    }
    BoxFuture {
        id,
        _marker: std::marker::PhantomData::default(),
    }
}

pub fn fetch_threads(username: &str, offset: usize, size: usize) -> BoxFuture<Vec<Thread>> {
    let id = NEXT_ID.with(|n| {
        let id = *n.borrow();
        *n.borrow_mut() = id + 1;
        id
    });
    PENDING.with(|p| p.borrow_mut().insert(id, (None, None)));
    ADDING_THREADS.with(|p| p.borrow_mut().insert(id, Vec::new()));
    unsafe {
        external_start_fetch_threads(username.as_ptr(), username.len(), offset, size, id);
    }
    BoxFuture {
        id,
        _marker: std::marker::PhantomData::default(),
    }
}
