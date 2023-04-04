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
            enum State {
            $(
                $state_name,
            )*
                Invalid(String),
                End
            }
            struct $name {
                state: State,
                data: $data,
            }
            enum Events {
            $(
                $([<$state_name $event>],)*
            )*
            }

            $(
                enum [<$state_name Events>] {
                    $($event,)*
                    Impossible
                }
            )*
            trait StateActions {
            $(
                fn [<run _ $state_name:snake>](&self) -> [<$state_name Events>];
            )*
            }
            trait Transitions {
                $($(fn [<$state_name:snake _ $event:snake>](&mut self) -> Result<(),String>;)*)*
                $(fn [<$state_name:snake _ impossible>](&mut self);)*
            }
            impl $name {
                fn run(&mut self) -> Result<Data, String> {
                    loop {
                        match &self.state {
                            $(State::$state_name => match self.[<run _ $state_name:snake>]() {
                                $([<$state_name Events>]::$event => {
                                    match self.[<$state_name:snake _ $event:snake>]() {
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
                                    self.[<$state_name:snake _ impossible>]();
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
    };
}
