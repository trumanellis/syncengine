//! Timer and event scheduler for scenario execution.
//!
//! Provides `after()` and `every()` functionality for Lua scenarios.

#![allow(dead_code)]

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use mlua::{Function, Lua, RegistryKey};

/// A scheduled task
#[derive(Debug)]
pub struct ScheduledTask {
    /// When the task should run
    pub run_at: Instant,
    /// Unique task ID
    pub id: u64,
    /// The Lua callback registry key
    pub callback_key: RegistryKey,
    /// If Some, this is a repeating task with the given interval
    pub repeat_interval: Option<Duration>,
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior (earliest first)
        other.run_at.cmp(&self.run_at)
    }
}

/// Command sent to the scheduler
#[derive(Debug)]
pub enum SchedulerCommand {
    /// Schedule a one-shot task
    After {
        delay: Duration,
        callback_key: RegistryKey,
    },
    /// Schedule a repeating task
    Every {
        interval: Duration,
        callback_key: RegistryKey,
    },
    /// Stop the scheduler
    Stop,
}

/// The scheduler that runs timed callbacks
pub struct Scheduler {
    tasks: BinaryHeap<ScheduledTask>,
    next_id: u64,
    running: bool,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: BinaryHeap::new(),
            next_id: 0,
            running: true,
        }
    }

    /// Add a one-shot task
    pub fn schedule_after(&mut self, delay: Duration, callback_key: RegistryKey) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        self.tasks.push(ScheduledTask {
            run_at: Instant::now() + delay,
            id,
            callback_key,
            repeat_interval: None,
        });

        id
    }

    /// Add a repeating task
    pub fn schedule_every(&mut self, interval: Duration, callback_key: RegistryKey) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        self.tasks.push(ScheduledTask {
            run_at: Instant::now() + interval,
            id,
            callback_key,
            repeat_interval: Some(interval),
        });

        id
    }

    /// Check if there are pending tasks
    pub fn has_pending(&self) -> bool {
        !self.tasks.is_empty() && self.running
    }

    /// Get the next task if it's ready
    pub fn pop_ready(&mut self) -> Option<ScheduledTask> {
        if let Some(task) = self.tasks.peek() {
            if task.run_at <= Instant::now() {
                return self.tasks.pop();
            }
        }
        None
    }

    /// Get duration until next task (for sleep)
    pub fn time_until_next(&self) -> Option<Duration> {
        self.tasks.peek().map(|task| {
            let now = Instant::now();
            if task.run_at > now {
                task.run_at - now
            } else {
                Duration::ZERO
            }
        })
    }

    /// Re-schedule a repeating task
    pub fn reschedule(&mut self, task: ScheduledTask) {
        if let Some(interval) = task.repeat_interval {
            self.tasks.push(ScheduledTask {
                run_at: Instant::now() + interval,
                id: task.id,
                callback_key: task.callback_key,
                repeat_interval: Some(interval),
            });
        }
    }

    /// Stop the scheduler
    pub fn stop(&mut self) {
        self.running = false;
    }
}

/// Thread-safe scheduler wrapper
pub type SharedScheduler = Arc<RwLock<Scheduler>>;

pub fn create_shared_scheduler() -> SharedScheduler {
    Arc::new(RwLock::new(Scheduler::new()))
}

/// Run the scheduler loop
pub async fn run_scheduler_loop(
    lua: &Lua,
    scheduler: SharedScheduler,
) -> anyhow::Result<()> {
    loop {
        let sleep_duration;
        let ready_task;

        // Check for ready tasks
        {
            let mut sched = scheduler.write().await;

            if !sched.has_pending() {
                break;
            }

            ready_task = sched.pop_ready();
            sleep_duration = sched.time_until_next();
        }

        // Execute ready task
        if let Some(task) = ready_task {
            // Get the callback from registry
            let callback: Function = lua.registry_value(&task.callback_key)?;

            // Execute callback
            if let Err(e) = callback.call::<()>(()) {
                tracing::warn!("Scheduled callback error: {}", e);
            }

            // Reschedule if repeating
            if task.repeat_interval.is_some() {
                let mut sched = scheduler.write().await;
                sched.reschedule(task);
            }

            continue; // Check for more ready tasks immediately
        }

        // Sleep until next task
        if let Some(duration) = sleep_duration {
            if duration > Duration::ZERO {
                tokio::time::sleep(duration.min(Duration::from_millis(100))).await;
            }
        } else {
            // No more tasks
            break;
        }
    }

    Ok(())
}
