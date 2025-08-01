pub mod prelude {
    pub struct Renderer {}

    #[derive(Default, Clone)]
    pub enum Reserve {
        #[default]
        Top,
        Bottom,
        Left,
        Right,
        Full,
    }

    #[derive(Default, Clone)]
    pub struct Layout {
        pub x: u32,
        pub y: u32,

        pub width: u32,
        pub height: u32,

        pub reserve: Option<Reserve>,
    }

    pub struct Canvas {}

    impl Canvas {}
}
