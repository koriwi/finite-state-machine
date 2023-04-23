#![no_std]
pub use paste::paste;
#[macro_export]
macro_rules! state_machine {
    (
        $name:ident($config:ident, $data:ident$(<$($lt:lifetime),*>)?);
        $(
            $state_name:ident {
                $($event:ident => $possible_target_state:ident),*
            }
        ),*
    ) => {
        $crate::paste!{
        mod [<$name:snake>] {
            use super::*;

            #[cfg_attr(feature = "verbose", derive(Debug))]
            #[derive(Default)]
            pub enum State$(<$($lt),*>)? {
                #[default]
            $(
                $state_name,
            )*
                Invalid(&'static str),
                End
            }
            pub struct $name {
                pub config: $config,
            }
            enum Events {
            $(
                $([<$state_name $event>],)*
            )*
            }

            $(
                pub enum [<$state_name Events>] {
                    $($event,)*
                    Illegal(&'static str)
                }
            )*
            pub trait Deciders<D> {
            $(
                fn [<$state_name:snake>](&self, data: &D) -> [<$state_name Events>];
            )*
            }
            $(
                pub trait [<$state_name Transitions>]<D> {
                    $(fn [<$event:snake>](&mut self, data: &mut D) -> Result<(),&'static str>;)*
                    fn illegal(&mut self);
                }
            )*
            impl $name{
                pub fn run_to_end$(<$($lt),*>)?(&mut self, state_data: &mut $data$(<$($lt),*>)?)  -> Result<(), &'static str> {
                    let mut state = State::default();
                    loop {
                        match state {
                            $(State::$state_name => match self.[<$state_name:snake>](&state_data) {
                                $([<$state_name Events>]::$event => {
                                    match [<$state_name Transitions>]::[<$event:snake>](self, state_data) {
                                        Ok(data) => {
                                            #[cfg(feature = "verbose")]
                                            println!("{} + {} -> {}", stringify!($state_name), stringify!($event), stringify!($possible_target_state));
                                            state = State::$possible_target_state;
                                        },
                                        Err(message) => {
                                            #[cfg(feature = "verbose")]
                                            println!("{} + {} + error({}) -> {}", stringify!($state_name), stringify!($event), message, stringify!(Invalid));
                                            state = State::Invalid(message)
                                        }
                                    }

                                },)*
                                [<$state_name Events>]::Illegal(message) => {
                                    [<$state_name Transitions>]::illegal(self);
                                    #[cfg(feature = "verbose")]
                                    println!("{} + illegal -> invalid({})", stringify!($state_name), stringify!(message));
                                    state = State::Invalid(message);
                                }
                            } ,)*
                            State::End => return Ok(()),
                            State::Invalid(message) => return Err(message),
                        };
                    };
                }
            }
        }
        }
    };
}
