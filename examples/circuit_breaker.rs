use std::time::{Duration, SystemTime};

use circuit_breaker::*;
use finite_state_machine::state_machine;
#[derive(Debug, Default)]
pub struct Config {
    max_amperage: u8,
    max_attempts: u8,
    cool_down_time: u8,
}

#[derive(Debug, Default)]
pub struct Data {
    current_amperage: u8,
    tripped_at: Option<SystemTime>,
    reset_at: Option<SystemTime>,
    attempts: u8,
}
state_machine!(
    CircuitBreaker<Config, Data>; // The name of the state machine and the type of the data, you can also use live times here
    Closed { // the first state will automatically made the start state, no matter the name
        Ok => Closed, // ok Ok event go to Closed state
        AmperageTooHigh => Open // on AmperageTooHigh event go to open state
    },
    Open {
        AttemptReset => HalfOpen,
        Wait => Open,
        MaxAttemptsExceeded => End
    },
    HalfOpen {
        Success => Closed,
        AmperageTooHigh => Open
    }
);

impl Deciders for CircuitBreaker {
    fn closed(&self, data: &Data) -> circuit_breaker::ClosedEvents {
        if data.current_amperage > self.config.max_amperage {
            circuit_breaker::ClosedEvents::AmperageTooHigh
        } else {
            circuit_breaker::ClosedEvents::Ok
        }
    }
    fn half_open(&self, data: &Data) -> circuit_breaker::HalfOpenEvents {
        if data.reset_at.is_none() {
            return HalfOpenEvents::Illegal("reset time not set");
        }

        if data.current_amperage > self.config.max_amperage {
            HalfOpenEvents::AmperageTooHigh
        } else {
            HalfOpenEvents::Success
        }
    }
    fn open(&self, data: &Data) -> OpenEvents {
        if data.attempts == self.config.max_attempts {
            return OpenEvents::MaxAttemptsExceeded;
        }
        let now = SystemTime::now();
        let tripped_at = match data.tripped_at {
            Some(t) => t,
            None => return OpenEvents::Illegal("tripped_at not set"),
        };
        let diff = now.duration_since(tripped_at).unwrap();
        if diff.as_secs() < self.config.cool_down_time as u64 {
            OpenEvents::Wait
        } else {
            OpenEvents::AttemptReset
        }
    }
}

impl ClosedTransitions for CircuitBreaker {
    fn amperage_too_high(&mut self, mut data: Data) -> Result<Data, &'static str> {
        data.tripped_at = Some(SystemTime::now());
        Ok(data)
    }
    fn ok(&mut self, mut data: Data) -> Result<Data, &'static str> {
        data.current_amperage += 1;
        std::thread::sleep(Duration::from_millis(500));
        Ok(data)
    }
    fn illegal(&mut self) {}
}

impl HalfOpenTransitions for CircuitBreaker {
    fn success(&mut self, mut data: Data) -> Result<Data, &'static str> {
        data.reset_at = Some(SystemTime::now());
        data.attempts = 0;
        Ok(data)
    }
    fn amperage_too_high(&mut self, mut data: Data) -> Result<Data, &'static str> {
        data.tripped_at = Some(SystemTime::now());
        Ok(data)
    }
    fn illegal(&mut self) {}
}

impl OpenTransitions for CircuitBreaker {
    fn attempt_reset(&mut self, mut data: Data) -> Result<Data, &'static str> {
        data.reset_at = Some(SystemTime::now());
        data.attempts += 1;
        Ok(data)
    }
    fn max_attempts_exceeded(&mut self, mut data: Data) -> Result<Data, &'static str> {
        Ok(data)
    }
    fn wait(&mut self, mut data: Data) -> Result<Data, &'static str> {
        // sleep for a second or cooldown time
        std::thread::sleep(std::time::Duration::from_millis(1000));
        Ok(data)
    }
    fn illegal(&mut self) {}
}

// impl<C> CircuitBreaker<C> {
//     fn new(config: C) -> Self {
//         Self { config }
//     }
//     fn run(&mut self) -> Result<Data, &'static str> {
//         self.run_to_end(State::Closed(Data {
//             current_amperage: 0,
//             tripped_at: None,
//             reset_at: None,
//             attempts: 0,
//         }))
//     }
// }

fn main() {
    let mut circuit_breaker = CircuitBreaker {
        config: Config {
            max_amperage: 10,
            max_attempts: 3,
            cool_down_time: 10,
        },
    };
    circuit_breaker.run_to_end(State::Closed(Data {
        current_amperage: 0,
        tripped_at: None,
        reset_at: None,
        attempts: 0,
    }));
}
