#[macro_export]
macro_rules! state_machine {
    (
        $name:ident($data:ident);
        $(
            $state_name:ident {
                $($event:ident => $possible_target_state:ident),*
            }
        ),*
    ) => {
        paste::paste!{
        mod [<$name:snake>] {
            use super::*;
            #[derive(Debug, Clone, PartialEq, Default)]
            pub enum State {
                #[default]
            $(
                $state_name,
            )*
                Invalid(String),
                End
            }
            #[derive(Debug, Clone, PartialEq, Default)]
            pub struct $name {
                pub state: State,
                pub data: $data,
            }
            enum Events {
            $(
                $([<$state_name $event>],)*
            )*
            }

            $(
                pub enum [<$state_name Events>] {
                    $($event,)*
                    Impossible
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
                    fn impossible(&mut self);
                }
            )*
            impl $name {
                pub fn run(&mut self) -> Result<$data, String> {
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
                                [<$state_name Events>]::Impossible => {
                                    [<$state_name Transitions>]::impossible(self);
                                    #[cfg(feature = "verbose")]
                                    println!("{} + impossible -> {}", stringify!($state_name), stringify!(Invalid));
                                    self.state = State::Invalid("impossible".to_owned());
                                }
                            } ,)*
                            State::End => return Ok(self.data.clone()),
                            State::Invalid(message) => return Err(message.to_owned())
                        };
                    };
                }
            }
        }
        }
    };
}
