#[macro_export]
macro_rules! state_machine {
    (
        $name:ident{
            $($var_name:tt : $var_type:ty),*
        };
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
                End
            }
            #[derive(Clone)]
            struct Data {
                $($var_name:$var_type,)*
            }
            struct $name {
                state: State,
                data: Data
            }
            enum Events {
            $(
                $([<$state_name $event>],)*
            )*
            }

            $(
                enum [<$state_name Events>] {
                    $($event),*
                }
            )*
            trait StateActions {
            $(
                fn [<run _ $state_name:snake>](&self) -> [<$state_name Events>];
            )*
            }
            trait Transitions {
                $($(fn [<$state_name:snake _ $event:snake>](&mut self);)*)*
            }
            impl $name {
                fn run(&mut self) -> Data {
                    loop {
                        match &self.state {
                            $(State::$state_name => match self.[<run _ $state_name:snake>]() {
                                $([<$state_name Events>]::$event => {
                                    self.[<$state_name:snake _ $event:snake>]();
                                    self.state = State::$possible_target_state;
                                },)*
                            } ,)*
                            State::End => {break}
                        };
                    }
                    self.data.clone()
                }
            }
        }
    };
}
