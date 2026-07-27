/// Task scheduler with continuation-based control flow

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Task {
    pub id: u32,
    pub status: TaskStatus,
    pub bytecode: Vec<u8>,
    pub ip: u32,
    pub registers: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
    Completed(i32),
}

pub struct Scheduler {
    ready_queue: VecDeque<Task>,
    blocked_queue: Vec<Task>,
    next_task_id: u32,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            blocked_queue: Vec::new(),
            next_task_id: 0,
        }
    }

    pub fn spawn_task(&mut self, bytecode: Vec<u8>) -> u32 {
        let task_id = self.next_task_id;
        self.next_task_id += 1;

        let task = Task {
            id: task_id,
            status: TaskStatus::Ready,
            bytecode,
            ip: 0,
            registers: vec![0; 256],
        };

        self.ready_queue.push_back(task);
        task_id
    }

    pub fn schedule(&mut self) -> Option<Task> {
        self.ready_queue.pop_front()
    }

    pub fn resume_task(&mut self, task: Task) {
        if task.status == TaskStatus::Ready {
            self.ready_queue.push_back(task);
        }
    }

    pub fn block_task(&mut self, task: Task) {
        let mut blocked_task = task;
        blocked_task.status = TaskStatus::Blocked;
        self.blocked_queue.push(blocked_task);
    }

    pub fn complete_task(&mut self, _task: Task, exit_code: i32) {
        // Task completed, exit_code is return value
    }

    pub fn queue_length(&self) -> usize {
        self.ready_queue.len()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_task() {
        let mut scheduler = Scheduler::new();
        let bytecode = vec![0; 100];
        let task_id = scheduler.spawn_task(bytecode);
        assert_eq!(task_id, 0);
        assert_eq!(scheduler.queue_length(), 1);
    }

    #[test]
    fn test_schedule_task() {
        let mut scheduler = Scheduler::new();
        scheduler.spawn_task(vec![1, 2, 3]);
        let task = scheduler.schedule();
        assert!(task.is_some());
        assert_eq!(task.unwrap().id, 0);
    }
}
