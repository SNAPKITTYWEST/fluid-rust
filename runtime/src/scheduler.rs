//! Task Scheduler: Manage continuations and async execution
//!
//! Implements a cooperative multitasking scheduler that:
//! - Maintains a queue of runnable tasks
//! - Saves/restores continuation frames
//! - Handles yield/resume for async effects

use crate::effect_handler::RuntimeState;
use std::collections::VecDeque;

/// A runnable task with its current state and continuation.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub state: RuntimeState,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Suspended,
    Completed,
}

/// The task scheduler: manages all active tasks.
pub struct Scheduler {
    tasks: VecDeque<Task>,
    current_task: Option<u32>,
    task_counter: u32,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: VecDeque::new(),
            current_task: None,
            task_counter: 0,
        }
    }

    /// Spawn a new task.
    pub fn spawn(&mut self, initial_state: RuntimeState) -> u32 {
        let id = self.task_counter;
        self.task_counter += 1;

        self.tasks.push_back(Task {
            id,
            state: initial_state,
            status: TaskStatus::Ready,
        });

        id
    }

    /// Schedule the next ready task.
    pub fn schedule_next(&mut self) -> Option<u32> {
        while let Some(mut task) = self.tasks.pop_front() {
            if task.status == TaskStatus::Ready {
                let task_id = task.id;
                task.status = TaskStatus::Running;
                self.current_task = Some(task_id);
                self.tasks.push_back(task); // Put back at end
                return Some(task_id);
            }
        }
        None
    }

    /// Yield control (task suspends).
    pub fn yield_task(&mut self) {
        if let Some(current_id) = self.current_task {
            if let Some(pos) = self.tasks.iter().position(|t| t.id == current_id) {
                self.tasks[pos].status = TaskStatus::Suspended;
            }
        }
    }

    /// Resume a suspended task.
    pub fn resume_task(&mut self, task_id: u32) -> Result<(), String> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == task_id) {
            if self.tasks[pos].status == TaskStatus::Suspended {
                self.tasks[pos].status = TaskStatus::Ready;
                return Ok(());
            }
        }
        Err(format!("Task {} not found or not suspended", task_id))
    }

    /// Mark a task as completed.
    pub fn complete_task(&mut self, task_id: u32) {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == task_id) {
            self.tasks[pos].status = TaskStatus::Completed;
        }
    }

    /// Get the current task's state (mutable).
    pub fn get_current_state_mut(&mut self) -> Option<&mut RuntimeState> {
        if let Some(current_id) = self.current_task {
            self.tasks
                .iter_mut()
                .find(|t| t.id == current_id)
                .map(|t| &mut t.state)
        } else {
            None
        }
    }

    /// Number of ready tasks.
    pub fn ready_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.status == TaskStatus::Ready).count()
    }

    /// Check if any tasks remain.
    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }
}

// TODO: Implement work-stealing scheduler for parallelism
// TODO: Implement priority queue for task scheduling
// TODO: Implement continuation frame management
// TODO: Implement task cleanup and resource deallocation
// TODO: Implement deadlock detection

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_task() {
        let mut scheduler = Scheduler::new();
        let state = RuntimeState::new();
        let task_id = scheduler.spawn(state);
        assert_eq!(task_id, 0);
        assert_eq!(scheduler.ready_count(), 1);
    }

    #[test]
    fn test_schedule_next() {
        let mut scheduler = Scheduler::new();
        let state = RuntimeState::new();
        scheduler.spawn(state);
        let next = scheduler.schedule_next();
        assert_eq!(next, Some(0));
    }

    #[test]
    fn test_yield_and_resume() {
        let mut scheduler = Scheduler::new();
        let state = RuntimeState::new();
        scheduler.spawn(state);
        scheduler.schedule_next();

        scheduler.yield_task();
        assert_eq!(scheduler.ready_count(), 0);

        scheduler.resume_task(0).unwrap();
        assert_eq!(scheduler.ready_count(), 1);
    }
}
