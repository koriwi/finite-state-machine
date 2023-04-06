#[macro_export]
macro_rules! state_machine {
    (
        $name:ident$(<$($lt:lifetime),*>)?($data:ident$(<$($lt_data:lifetime),*>)?);
        $(
            $state_name:ident {
                $($event:ident => $possible_target_state:ident),*
            }
        ),*
    ) => {
        paste::paste!{
        mod [<$name:snake>] {
            use super::*;
            use std::error::Error;
            #[derive(Debug, Default)]
            pub enum State {
                #[default]
            $(
                $state_name,
            )*
                Invalid(String),
                End
            }
            #[derive(Debug, Default)]
            pub struct $name$(<$($lt),*>)? {
                pub state: State,
                pub data: $data$(<$($lt_data),*>)?,
            }
            enum Events {
            $(
                $([<$state_name $event>],)*
            )*
            }

            $(
                pub enum [<$state_name Events>] {
                    $($event,)*
                    Illegal
                }
            )*
            pub trait Deciders {
            $(
                fn [<$state_name:snake>](&self) -> [<$state_name Events>];
            )*
            }
            $(
                pub trait [<$state_name Transitions>] {
                    $(fn [<$event:snake>](&mut self) -> Result<(),String>;)*
                    fn illegal(&mut self);
                }
            )*
            impl$(<$($lt),*>)? $name$(<$($lt),*>)? {
                pub fn run(&mut self) -> Result<(), String> {
                    loop {
                        match &self.state {
                            $(State::$state_name => match self.[<$state_name:snake>]() {
                                $([<$state_name Events>]::$event => {
                                    match [<$state_name Transitions>]::[<$event:snake>](self) {
                                        Ok(_) => {
                                            #[cfg(feature = "verbose")]
                                            println!("{} + {} -> {}", stringify!($state_name), stringify!($event), stringify!($possible_target_state));
                                            self.state = State::$possible_target_state
                                        },
                                        Err(message) => {
                                            #[cfg(feature = "verbose")]
                                            println!("{} + {} + error({}) -> {}", stringify!($state_name), stringify!($event), message, stringify!(Invalid));
                                            self.state = State::Invalid(message);
                                        }
                                    }

                                },)*
                                [<$state_name Events>]::Illegal => {
                                    [<$state_name Transitions>]::illegal(self);
                                    #[cfg(feature = "verbose")]
                                    println!("{} + illegal -> {}", stringify!($state_name), stringify!(Invalid));
                                    self.state = State::Invalid(Err("illegal")?);
                                }
                            } ,)*
                            State::End => return Ok(()),
                            State::Invalid(message) => Err(message)?,
                        };
                    };
                }
            }
        }
        }
    };
}
