use crate::proc::task::*;
use alloc::collections::linked_list::LinkedList;
// TODO the lifetime here is pretty much broken. Fix this later
pub struct Scheduler<'a> {
	run_list: LinkedList<&'a Task>,
}

impl<'a> Scheduler<'a> {
	#[inline]
	pub fn pop_front(&mut self) -> Option<&Task> {
		self.run_list.pop_front()
	}
	#[inline]
	pub fn push_back(&mut self, t: &'a Task) {
		self.run_list.push_back(t);
	}
}
