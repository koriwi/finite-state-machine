use std::time::{Duration, SystemTime};

use circuit_breaker::*;
use finite_state_machine::state_machine;
#[derive(Debug, Default)]
pub struct Config {
    max_amperage: u8,
    max_attempts: u8,
    cool_down_time: Duration,
}

#[derive(Debug, Default)]
pub struct Data {
    current_amperage: u8,
    tripped_at: Option<SystemTime>,
    reset_at: Option<SystemTime>,
    attempts: u8,
}
state_machine!(
    CircuitBreaker(Config, Data); // The name of the state machine and the type of the (), you can also use live times here
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

impl Deciders<Data> for CircuitBreaker {
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
        if diff < self.config.cool_down_time {
            OpenEvents::Wait
        } else {
            OpenEvents::AttemptReset
        }
    }
}

impl ClosedTransitions<Data> for CircuitBreaker {
    fn amperage_too_high(&mut self, data: &mut Data) -> Result<(), &'static str> {
        data.tripped_at = Some(SystemTime::now());
        Ok(())
    }
    fn ok(&mut self, data: &mut Data) -> Result<(), &'static str> {
        data.current_amperage += 1;
        std::thread::sleep(Duration::from_millis(500));
        Ok(())
    }
    fn illegal(&mut self) {}
}

impl HalfOpenTransitions<Data> for CircuitBreaker {
    fn success(&mut self, data: &mut Data) -> Result<(), &'static str> {
        data.reset_at = Some(SystemTime::now());
        data.attempts = 0;
        Ok(())
    }
    fn amperage_too_high(&mut self, data: &mut Data) -> Result<(), &'static str> {
        data.tripped_at = Some(SystemTime::now());
        Ok(())
    }
    fn illegal(&mut self) {}
}

impl OpenTransitions<Data> for CircuitBreaker {
    fn attempt_reset(&mut self, data: &mut Data) -> Result<(), &'static str> {
        data.reset_at = Some(SystemTime::now());
        data.attempts += 1;
        Ok(())
    }
    fn max_attempts_exceeded(&mut self, _data: &mut Data) -> Result<(), &'static str> {
        Ok(())
    }
    fn wait(&mut self, _data: &mut Data) -> Result<(), &'static str> {
        std::thread::sleep(self.config.cool_down_time);
        Ok(())
    }
    fn illegal(&mut self) {}
}

fn main() {
    let mut circuit_breaker = CircuitBreaker {
        config: Config {
            max_amperage: 10,
            max_attempts: 3,
            cool_down_time: std::time::Duration::from_millis(2000),
        },
    };
    let mut data = Data::default();
    circuit_breaker.run_to_end(&mut data).unwrap();
    println!("data: {:?}", data)
}
