#![no_std]
pub use paste::paste;
#[macro_export]
macro_rules! state_machine {
    (
        $name:ident($data:ident$(<$($lt:lifetime),*>)?);
        $(
            $state_name:ident {
                $($event:ident => $possible_target_state:ident),*
            }
        ),*
    ) => {
        $crate::paste!{
        mod [<$name:snake>] {
            use super::*;
            #[derive(Debug, Default)]
            pub enum State {
                #[default]
            $(
                $state_name,
            )*
                Invalid(&'static str),
                End
            }
            #[cfg_attr(feature = "verbose", derive(Debug))]
            #[cfg_attr(feature = "derive_default", derive(Default))]
            pub struct $name$(<$($lt),*>)? {
                pub state: State,
                pub data: $data$(<$($lt),*>)?,
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
            pub trait Deciders {
            $(
                fn [<$state_name:snake>](&self) -> [<$state_name Events>];
            )*
            }
            $(
                pub trait [<$state_name Transitions>] {
                    $(fn [<$event:snake>](&mut self) -> Result<(),&'static str>;)*
                    fn illegal(&mut self);
                }
            )*
            impl$(<$($lt),*>)? $name$(<$($lt),*>)? {
                pub fn run_to_end(&mut self) -> Result<(), &'static str> {
                    loop {
                        #[cfg(feature = "verbose")]
                        println!("Debug: {:?}", self.data);
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
                                            self.state = State::Invalid(message)
                                        }
                                    }

                                },)*
                                [<$state_name Events>]::Illegal(message) => {
                                    [<$state_name Transitions>]::illegal(self);
                                    #[cfg(feature = "verbose")]
                                    println!("{} + illegal -> invalid({})", stringify!($state_name), stringify!(message));
                                    self.state = State::Invalid(message);
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
