//! Process management syscalls

use core::convert::TryInto;

use crate::config::{MAX_SYSCALL_NUM, PAGE_SIZE};
use crate::task::{exit_current_and_run_next, get_current_task, suspend_current_and_run_next, TaskStatus, current_user_token};
use crate::timer::get_time_us;
use crate::mm::{PTEFlags, copyout, PageTable};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let _us = get_time_us();
    let ts = TimeVal {
        sec: _us / 1_000_000,
        usec: _us % 1_000_000,
    };
    copyout(current_user_token(), _ts as usize, &ts as *const TimeVal as *const u8, core::mem::size_of::<TimeVal>());
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    // 使用 FRAME_ALLOCATOR 获取一个 FrameTracker
    // 将 FrameTracker 物理页面与 _start (虚拟地址) 映射至一块
    if _port & !0x7 != 0 || _port & 0x7 == 0 || _start % PAGE_SIZE != 0 {
        return -1;
    }

    let flags = PTEFlags::V | PTEFlags::U | PTEFlags::from_bits_truncate((_port << 1) as u8);

    PageTable::from_token(current_user_token()).kmap(_start, _len, flags)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    PageTable::from_token(current_user_token()).kunmap(_start, _len)
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let task = get_current_task();
    copyout(current_user_token(), ti as usize, &task as *const TaskInfo as *const u8, core::mem::size_of::<TaskInfo>());
    0
}
