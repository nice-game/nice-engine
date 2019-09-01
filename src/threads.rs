use futures::executor::ThreadPool;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
	pub static ref FILE_THREAD: Mutex<ThreadPool> =
		Mutex::new(ThreadPool::builder().pool_size(1).create().unwrap());
}
