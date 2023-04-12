use std::time::{Duration, SystemTime};

use circuit_breaker::*;
use finite_state_machine::state_machine;
#[derive(Debug, Default)]
pub struct Data {
    max_amperage: u8,
    current_amperage: u8,
    tripped_at: Option<SystemTime>,
    reset_at: Option<SystemTime>,
    attemps: u8,
    max_attemps: u8,
    cool_down_time: u8,
}
state_machine!(
    CircuitBreaker(Data); // The name of the state machine and the type of the data, you can also use live times here
    Closed { // the first state will automatically made the start state, no matter the name
        Ok => Closed, // ok Ok event go to Closed state
        AmperageTooHigh => Open // on AmperageTooHigh event go to open state
    },
    Open {
        AttemptReset => HalfOpen,
        Wait => Open
    },
    HalfOpen {
        Success => Closed,
        AmperageTooHigh => Open,
        MaxAttemps => End
    }
);

impl Deciders for CircuitBreaker {
    fn closed(&self) -> circuit_breaker::ClosedEvents {
        if self.data.current_amperage > self.data.max_amperage {
            circuit_breaker::ClosedEvents::AmperageTooHigh
        } else {
            circuit_breaker::ClosedEvents::Ok
        }
    }
    fn half_open(&self) -> circuit_breaker::HalfOpenEvents {
        if self.data.reset_at.is_none() {
            return HalfOpenEvents::Illegal("reset time not set");
        }

        if self.data.attemps > self.data.max_attemps {
            return HalfOpenEvents::MaxAttemps;
        }

        if self.data.current_amperage > self.data.max_amperage {
            HalfOpenEvents::AmperageTooHigh
        } else {
            HalfOpenEvents::Success
        }
    }
    fn open(&self) -> OpenEvents {
        let now = SystemTime::now();
        let tripped_at = match self.data.tripped_at {
            Some(t) => t,
            None => return OpenEvents::Illegal("tripped_at not set"),
        };
        let diff = now.duration_since(tripped_at).unwrap();
        if diff.as_secs() < self.data.cool_down_time as u64 {
            OpenEvents::Wait
        } else {
            OpenEvents::AttemptReset
        }
    }
}

impl ClosedTransitions for CircuitBreaker {
    fn amperage_too_high(&mut self) -> Result<(), &'static str> {
        self.data.tripped_at = Some(SystemTime::now());
        Ok(())
    }
    fn ok(&mut self) -> Result<(), &'static str> {
        self.data.current_amperage += 1;
        std::thread::sleep(Duration::from_millis(500));
        Ok(())
    }
    fn illegal(&mut self) {}
}

impl HalfOpenTransitions for CircuitBreaker {
    fn success(&mut self) -> Result<(), &'static str> {
        self.data.reset_at = Some(SystemTime::now());
        self.data.attemps = 0;
        Ok(())
    }
    fn amperage_too_high(&mut self) -> Result<(), &'static str> {
        self.data.tripped_at = Some(SystemTime::now());
        self.data.attemps += 1;
        Ok(())
    }
    fn max_attemps(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
    fn illegal(&mut self) {}
}

impl OpenTransitions for CircuitBreaker {
    fn attempt_reset(&mut self) -> Result<(), &'static str> {
        self.data.reset_at = Some(SystemTime::now());
        Ok(())
    }
    fn wait(&mut self) -> Result<(), &'static str> {
        // sleep for a second or cooldown time
        std::thread::sleep(std::time::Duration::from_millis(1000));
        Ok(())
    }
    fn illegal(&mut self) {}
}

impl CircuitBreaker {
    fn run(&mut self) -> Result<(), &'static str> {
        self.data.current_amperage = 5;
        self.data.max_amperage = 20;
        self.data.cool_down_time = 5;
        self.run_to_end()
    }
}

fn main() {
    let mut circuit_breaker = CircuitBreaker::default();
    circuit_breaker.run().unwrap();
}
