/// high level representation of the callee saved registers for thread context. The caller
/// saved registers will is pushed into the stack already upon the interrupt entry
/// fpstate is NOT saved here
#[repr(C)]
pub struct Context {
	rbx: u64,
	r12: u64,
	r13: u64,
	r14: u64,
	r15: u64,
	rbp: u64,
	rsp: u64,
}

/// prepare the thread (coroutine) for the first execution
pub unsafe fn settle() {
	todo!()
	// it will be something like this...
	// void **sp = (void**)tos;
	// *(--sp) = object;      // 7th parameter for kickoff
	// *(--sp) = (void*)0;    // return address
	// *(--sp) = kickoff;            // address
	// regs->rsp = sp;
}

pub unsafe fn switch(_ctx_curr: usize, _ctx_next: usize) {
	todo!()
}
