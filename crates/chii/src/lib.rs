pub mod prelude {
    pub struct Renderer {}

    #[derive(Default, Clone)]
    pub enum Placement {
        #[default]
        None,
        Top,
        Bottom,
        Left,
        Right,
        Windowed,
    }

    #[derive(Default, Clone)]
    pub enum KeyboardSensitivity {
        #[default]
        None,
        OnClick,
        Exclusive,
    }

    #[derive(Default, Clone)]
    pub struct Layout {
        pub x: u32,
        pub y: u32,

        pub width: u32,
        pub height: u32,

        pub placement: Placement,
        pub keyboard_sensitivity: KeyboardSensitivity,
    }

    pub struct Canvas {}

    impl Canvas {}
}
